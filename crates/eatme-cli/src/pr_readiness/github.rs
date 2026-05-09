use std::collections::BTreeSet;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use eatme_core::{CommandRunner, CommandSpec};
use serde::Deserialize;

use super::{CheckConclusion, CheckStatus, CheckSummary, PrReadinessSnapshot, validate_sha};

const GH_TIMEOUT_SECONDS: u64 = 20;
const GH_ATTEMPTS: usize = 3;
const GH_RETRY_DELAY_MS: u64 = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubPrSnapshotRequest {
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub local_head_sha: String,
    pub required_checks: Vec<String>,
}

pub fn fetch_github_pr_snapshot(
    request: &GitHubPrSnapshotRequest,
    runner: &impl CommandRunner,
) -> Result<PrReadinessSnapshot> {
    validate_request(request)?;
    let repo_slug = format!("{}/{}", request.owner, request.repo);
    let output = runner
        .run(
            &CommandSpec::new("gh")
                .args([
                    "pr",
                    "view",
                    &request.pr_number.to_string(),
                    "--repo",
                    &repo_slug,
                    "--json",
                    "number,headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
                ])
                .timeout(Duration::from_secs(GH_TIMEOUT_SECONDS))
                .retries(GH_ATTEMPTS, Duration::from_millis(GH_RETRY_DELAY_MS)),
        )
        .context("querying GitHub PR readiness snapshot with gh")?;

    if output.exit_status != Some(0) {
        bail!(
            "GitHub PR snapshot fetch failed with exit status {:?}: {}",
            output.exit_status,
            summarize_external_error(&output.stderr)
        );
    }

    let response: GhPrViewResponse =
        serde_json::from_str(&output.stdout).context("parsing gh pr view JSON")?;
    response.into_snapshot(&request.local_head_sha, &request.required_checks)
}

fn validate_request(request: &GitHubPrSnapshotRequest) -> Result<()> {
    if request.owner.trim().is_empty() || request.repo.trim().is_empty() {
        bail!("GitHub owner and repo are required");
    }
    if request.pr_number == 0 {
        bail!("GitHub PR number must be greater than zero");
    }
    validate_sha(&request.local_head_sha).map_err(anyhow::Error::new)?;
    if request
        .required_checks
        .iter()
        .any(|check| check.trim().is_empty())
    {
        bail!("GitHub required check names must not be empty");
    }
    if request.required_checks.is_empty() {
        bail!("at least one required GitHub check must be named");
    }
    Ok(())
}

fn summarize_external_error(stderr: &str) -> String {
    let summary = stderr.lines().next().unwrap_or("").trim();
    if summary.is_empty() {
        "no stderr returned by gh".to_string()
    } else {
        summary.to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrViewResponse {
    number: u64,
    head_ref_name: String,
    head_ref_oid: String,
    merge_state_status: String,
    mergeable: String,
    #[serde(default)]
    status_check_rollup: Vec<GhStatusCheck>,
}

impl GhPrViewResponse {
    fn into_snapshot(
        self,
        local_head_sha: &str,
        required_checks: &[String],
    ) -> Result<PrReadinessSnapshot> {
        validate_sha(&self.head_ref_oid).map_err(anyhow::Error::new)?;
        let required: BTreeSet<&str> = required_checks.iter().map(String::as_str).collect();
        let mut seen = BTreeSet::new();
        let mut checks = Vec::new();

        for check in self.status_check_rollup {
            let name = check.display_name();
            if name.is_empty() {
                continue;
            }
            seen.insert(name.clone());
            checks.push(CheckSummary {
                required: required.contains(name.as_str()),
                name,
                status: parse_status(check.status.as_deref()),
                conclusion: parse_conclusion(check.conclusion.as_deref()),
                head_sha: self.head_ref_oid.clone(),
            });
        }

        for required_name in required_checks {
            if !seen.contains(required_name) {
                checks.push(CheckSummary {
                    name: required_name.clone(),
                    status: CheckStatus::Missing,
                    conclusion: CheckConclusion::Missing,
                    required: true,
                    head_sha: self.head_ref_oid.clone(),
                });
            }
        }

        Ok(PrReadinessSnapshot {
            pr_number: self.number,
            branch: self.head_ref_name,
            local_head_sha: local_head_sha.to_string(),
            pr_head_sha: self.head_ref_oid,
            merge_state_status: self.merge_state_status,
            mergeable: self.mergeable,
            checks,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhStatusCheck {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    workflow_name: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    conclusion: Option<String>,
}

impl GhStatusCheck {
    fn display_name(&self) -> String {
        [&self.name, &self.context, &self.workflow_name]
            .into_iter()
            .flatten()
            .map(|value| value.trim())
            .find(|value| !value.is_empty())
            .unwrap_or("")
            .to_string()
    }
}

fn parse_status(value: Option<&str>) -> CheckStatus {
    match normalized(value).as_str() {
        "COMPLETED" => CheckStatus::Completed,
        "IN_PROGRESS" | "PENDING" => CheckStatus::InProgress,
        "QUEUED" => CheckStatus::Queued,
        "REQUESTED" => CheckStatus::Requested,
        "WAITING" => CheckStatus::Waiting,
        _ => CheckStatus::Missing,
    }
}

fn parse_conclusion(value: Option<&str>) -> CheckConclusion {
    match normalized(value).as_str() {
        "SUCCESS" => CheckConclusion::Success,
        "SKIPPED" => CheckConclusion::Skipped,
        "CANCELLED" => CheckConclusion::Cancelled,
        "TIMED_OUT" | "TIMEDOUT" => CheckConclusion::TimedOut,
        "PENDING" | "" => CheckConclusion::Pending,
        _ => CheckConclusion::Failure,
    }
}

fn normalized(value: Option<&str>) -> String {
    value.unwrap_or("").trim().to_ascii_uppercase()
}
