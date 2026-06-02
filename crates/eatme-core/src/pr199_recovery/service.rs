use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::{CommandOutput, CommandRunner, CommandSpec};

use super::state::{CheckConclusion, CheckRun, PrState, PrStateCollector, PrStateInput};
use super::{RecoveryError, required_text};

const PR199: u32 = 199;
const PR199_JSON_FIELDS: &str = "number,headRefName,headRefOid,files,statusCheckRollup";
const PENDING_STATUSES: [&str; 6] = [
    "EXPECTED",
    "PENDING",
    "QUEUED",
    "REQUESTED",
    "WAITING",
    "IN_PROGRESS",
];
const FAILURE_CONCLUSIONS: [&str; 7] = [
    "FAILURE",
    "ERROR",
    "ACTION_REQUIRED",
    "TIMED_OUT",
    "STARTUP_FAILURE",
    "STALE",
    "NEUTRAL",
];

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
    let value = value.trim();
    if value.eq_ignore_ascii_case("SUCCESS") {
        Ok(CheckConclusion::Success)
    } else if value.eq_ignore_ascii_case("CANCELLED") {
        Ok(CheckConclusion::Cancelled)
    } else if value.eq_ignore_ascii_case("SKIPPED") {
        Ok(CheckConclusion::Skipped)
    } else if FAILURE_CONCLUSIONS
        .iter()
        .any(|failure| value.eq_ignore_ascii_case(failure))
    {
        Ok(CheckConclusion::Failure)
    } else {
        Err(RecoveryError::new(
            "github_pr_state_unknown_check_conclusion",
            format!("unknown GitHub check conclusion: {value}"),
        ))
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
    let status = status.trim();
    status.is_empty()
        || PENDING_STATUSES
            .iter()
            .any(|pending| status.eq_ignore_ascii_case(pending))
}

fn external_error_excerpt(output: &CommandOutput) -> String {
    let detail = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };
    detail.chars().take(500).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::Mutex;

    struct StubRunner {
        output: CommandOutput,
        specs: Mutex<Vec<CommandSpec>>,
    }

    impl StubRunner {
        fn new(output: CommandOutput) -> Self {
            Self {
                output,
                specs: Mutex::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for StubRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            self.specs.lock().unwrap().push(spec.clone());
            Ok(self.output.clone())
        }
    }

    #[test]
    fn fetch_state_uses_expected_gh_command_and_collects_rollup() {
        let runner = StubRunner::new(CommandOutput {
            command: "gh pr view 199".into(),
            exit_status: Some(0),
            stdout: r#"{
              "number": 199,
              "headRefName": "feat/pr-199",
              "headRefOid": "abc123",
              "files": [{"path": "crates/eatme-core/src/lib.rs"}],
              "statusCheckRollup": [
                {"name": "workspace", "conclusion": "SUCCESS"},
                {"context": "linux-headless", "status": "IN_PROGRESS"},
                {"workflowName": "preview", "state": "SKIPPED"}
              ]
            }"#
            .into(),
            stderr: String::new(),
        });
        let client = GitHubPrStateClient::with_config(
            &runner,
            GitHubPrStateClientConfig {
                attempts: 5,
                retry_delay: Duration::from_millis(25),
                timeout: Duration::from_secs(9),
            },
        );

        let state = client.fetch_state().unwrap();
        let specs = runner.specs.lock().unwrap();

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].program, "gh");
        assert_eq!(
            specs[0].args,
            vec![
                "pr".to_string(),
                "view".to_string(),
                "199".to_string(),
                "--json".to_string(),
                PR199_JSON_FIELDS.to_string(),
            ]
        );
        assert_eq!(specs[0].attempts, 5);
        assert_eq!(specs[0].retry_delay, Duration::from_millis(25));
        assert_eq!(specs[0].timeout, Some(Duration::from_secs(9)));
        assert_eq!(state.branch, "feat/pr-199");
        assert_eq!(state.head_sha, "abc123");
        assert_eq!(
            state.changed_files,
            vec![PathBuf::from("crates/eatme-core/src/lib.rs")]
        );
        assert_eq!(state.check_rollup.success, vec!["workspace".to_string()]);
        assert_eq!(
            state.check_rollup.pending,
            vec!["linux-headless".to_string()]
        );
        assert_eq!(state.check_rollup.skipped, vec!["preview".to_string()]);
    }

    #[test]
    fn fetch_state_reports_nonzero_exit_with_trimmed_excerpt() {
        let runner = StubRunner::new(CommandOutput {
            command: "gh pr view 199".into(),
            exit_status: Some(1),
            stdout: String::new(),
            stderr: format!("{} trailing detail", "x".repeat(600)),
        });
        let client = GitHubPrStateClient::new(&runner);

        let error = client.fetch_state().unwrap_err();

        assert_eq!(error.code(), "github_pr_state_fetch_failed");
        let rendered = error.to_string();
        assert!(rendered.contains("gh pr view exited Some(1)"));
        assert!(rendered.contains(&"x".repeat(500)));
        assert!(!rendered.contains(&"x".repeat(501)));
    }
}
