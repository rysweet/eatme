use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::{CommandOutput, CommandRunner, CommandSpec};

use super::state::{CheckConclusion, CheckRun, PrState, PrStateCollector, PrStateInput};
use super::{RecoveryError, required_text};

const PR199: u32 = 199;
const PR199_JSON_FIELDS: &str = "number,headRefName,headRefOid,files,statusCheckRollup";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitHubPrStateClientConfig {
    pub attempts: usize,
    pub retry_delay: Duration,
    pub timeout: Duration,
}

impl Default for GitHubPrStateClientConfig {
    fn default() -> Self {
        Self {
            attempts: 3,
            retry_delay: Duration::from_secs(1),
            timeout: Duration::from_secs(30),
        }
    }
}

pub struct GitHubPrStateClient<'a> {
    runner: &'a dyn CommandRunner,
    config: GitHubPrStateClientConfig,
}

impl<'a> GitHubPrStateClient<'a> {
    pub fn new(runner: &'a dyn CommandRunner) -> Self {
        Self::with_config(runner, GitHubPrStateClientConfig::default())
    }

    pub fn with_config(runner: &'a dyn CommandRunner, config: GitHubPrStateClientConfig) -> Self {
        Self { runner, config }
    }

    pub fn fetch_state(&self) -> Result<PrState, RecoveryError> {
        let output = self.run_gh_pr_view()?;
        if output.exit_status != Some(0) {
            return Err(RecoveryError::new(
                "github_pr_state_fetch_failed",
                format!(
                    "gh pr view exited {:?}: {}",
                    output.exit_status,
                    external_error_excerpt(&output)
                ),
            ));
        }

        let response = serde_json::from_str::<GhPrView>(&output.stdout).map_err(|error| {
            RecoveryError::new(
                "github_pr_state_json_invalid",
                format!("gh pr view returned invalid JSON: {error}"),
            )
        })?;
        PrStateCollector::collect(response.into_input()?)
    }

    fn run_gh_pr_view(&self) -> Result<CommandOutput, RecoveryError> {
        let args = vec![
            "pr".to_owned(),
            "view".to_owned(),
            PR199.to_string(),
            "--json".to_owned(),
            PR199_JSON_FIELDS.to_owned(),
        ];

        self.runner
            .run(
                &CommandSpec::new("gh")
                    .args(args)
                    .timeout(self.config.timeout)
                    .retries(self.config.attempts, self.config.retry_delay),
            )
            .map_err(|error| {
                RecoveryError::new(
                    "github_pr_state_fetch_failed",
                    format!("failed to run gh pr view: {error}"),
                )
            })
    }
}

#[derive(Debug, Deserialize)]
struct GhPrView {
    number: u32,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "headRefOid")]
    head_ref_oid: String,
    #[serde(default)]
    files: Vec<GhPrFile>,
    #[serde(default, rename = "statusCheckRollup")]
    status_check_rollup: Vec<Value>,
}

impl GhPrView {
    fn into_input(self) -> Result<PrStateInput, RecoveryError> {
        let branch = required_text(Some(self.head_ref_name.as_str()), "pr_state_missing_branch")?;
        let head_sha = required_text(Some(self.head_ref_oid.as_str()), "pr_state_missing_head")?;
        let changed_files = self
            .files
            .into_iter()
            .map(|file| PathBuf::from(file.path))
            .collect();
        let check_runs = self
            .status_check_rollup
            .iter()
            .map(check_run_from_rollup_entry)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PrStateInput {
            pr_number: self.number,
            branch,
            head_sha,
            changed_files,
            check_runs,
        })
    }
}

#[derive(Debug, Deserialize)]
struct GhPrFile {
    path: String,
}

fn check_run_from_rollup_entry(entry: &Value) -> Result<CheckRun, RecoveryError> {
    let name = first_string_field(entry, &["name", "context", "workflowName"])
        .unwrap_or("unnamed GitHub check")
        .to_owned();
    let status = first_string_field(entry, &["status"]).unwrap_or_default();
    let conclusion = match first_string_field(entry, &["conclusion", "state"]) {
        Some(value) if is_pending_status(value) => None,
        Some(value) => Some(github_conclusion(value)?),
        None => None,
    };

    if conclusion.is_none() && is_pending_status(status) {
        return Ok(CheckRun::in_progress(name));
    }

    match conclusion {
        Some(conclusion) => Ok(CheckRun::completed(name, conclusion)),
        None => Ok(CheckRun::in_progress(name)),
    }
}

fn github_conclusion(value: &str) -> Result<CheckConclusion, RecoveryError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "SUCCESS" => Ok(CheckConclusion::Success),
        "CANCELLED" => Ok(CheckConclusion::Cancelled),
        "SKIPPED" => Ok(CheckConclusion::Skipped),
        "FAILURE" | "ERROR" | "ACTION_REQUIRED" | "TIMED_OUT" | "STARTUP_FAILURE" | "STALE"
        | "NEUTRAL" => Ok(CheckConclusion::Failure),
        other => Err(RecoveryError::new(
            "github_pr_state_unknown_check_conclusion",
            format!("unknown GitHub check conclusion: {other}"),
        )),
    }
}

fn first_string_field<'a>(entry: &'a Value, names: &[&str]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| entry.get(name).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_pending_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_uppercase().as_str(),
        "" | "EXPECTED" | "PENDING" | "QUEUED" | "REQUESTED" | "WAITING" | "IN_PROGRESS"
    )
}

fn external_error_excerpt(output: &CommandOutput) -> String {
    let detail = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };
    detail.chars().take(500).collect()
}
