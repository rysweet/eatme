use std::fmt;
use std::process::Command;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use super::{
    CheckConclusion, CheckRunEvidence, CheckStatus, CommandEvidence, DocsImpactReview,
    PREvidenceReview, QualityAuditCycle, ReadinessInput,
};

const DEFAULT_GH_BINARY: &str = "gh";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalServiceErrorKind {
    CommandFailed,
    InvalidResponse,
    ParseFailed,
    RateLimited,
    TemporarilyUnavailable,
    Timeout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalServiceError {
    kind: ExternalServiceErrorKind,
    message: String,
}

impl ExternalServiceError {
    pub fn new(kind: ExternalServiceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> &ExternalServiceErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ExternalServiceErrorKind::RateLimited
                | ExternalServiceErrorKind::TemporarilyUnavailable
                | ExternalServiceErrorKind::Timeout
        )
    }
}

impl fmt::Display for ExternalServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ExternalServiceError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    max_attempts: usize,
    backoff: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, backoff: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            backoff,
        }
    }

    pub fn no_retry() -> Self {
        Self::new(1, Duration::from_millis(0))
    }

    fn run<T>(
        &self,
        mut call: impl FnMut() -> Result<T, ExternalServiceError>,
    ) -> Result<T, ExternalServiceError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            match call() {
                Ok(value) => return Ok(value),
                Err(error) if attempts < self.max_attempts && error.is_retryable() => {
                    if !self.backoff.is_zero() {
                        thread::sleep(self.backoff);
                    }
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(250))
    }
}

pub trait GitHubReadinessClient {
    fn pull_request(&self, pr_number: u64) -> Result<GitHubPullRequest, ExternalServiceError>;
    fn check_runs(&self, pr_number: u64) -> Result<Vec<GitHubCheckRun>, ExternalServiceError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubPullRequest {
    pub head_ref_name: String,
    pub head_ref_oid: String,
    pub merge_state_status: String,
    pub mergeable: String,
    pub evidence_texts: Vec<GitHubEvidenceText>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubEvidenceText {
    pub location: String,
    pub trusted: bool,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubCheckRun {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: CheckConclusion,
}

#[derive(Clone, Debug)]
pub struct ReadinessEvidenceDraft {
    pub pr_number: u64,
    pub local_branch: String,
    pub local_head_sha: String,
    pub command_evidence: Vec<CommandEvidence>,
    pub quality_audit_cycles: Vec<QualityAuditCycle>,
    pub changed_files: Vec<String>,
    pub docs_impact: DocsImpactReview,
    pub manual_merge_attempted: bool,
}

pub struct GitHubReadinessAdapter<C> {
    client: C,
    retry_policy: RetryPolicy,
}

impl<C> GitHubReadinessAdapter<C>
where
    C: GitHubReadinessClient,
{
    pub fn new(client: C) -> Self {
        Self {
            client,
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn with_retry_policy(client: C, retry_policy: RetryPolicy) -> Self {
        Self {
            client,
            retry_policy,
        }
    }

    pub fn build_input(
        &self,
        draft: ReadinessEvidenceDraft,
    ) -> Result<ReadinessInput, ExternalServiceError> {
        let pull_request = self
            .retry_policy
            .run(|| self.client.pull_request(draft.pr_number))?;
        let checks = self
            .retry_policy
            .run(|| self.client.check_runs(draft.pr_number))?;
        let confirmed_pull_request = self
            .retry_policy
            .run(|| self.client.pull_request(draft.pr_number))?;

        if confirmed_pull_request.head_ref_oid != pull_request.head_ref_oid {
            return Err(ExternalServiceError::new(
                ExternalServiceErrorKind::InvalidResponse,
                "PR head changed while collecting GitHub evidence; retry readiness collection",
            ));
        }

        let pr_evidence = pr_evidence_from_texts(&confirmed_pull_request);
        let check_runs = checks
            .into_iter()
            .map(|check| CheckRunEvidence {
                name: check.name,
                status: check.status,
                conclusion: check.conclusion,
                head_sha: confirmed_pull_request.head_ref_oid.clone(),
            })
            .collect();

        Ok(ReadinessInput {
            pr_number: draft.pr_number,
            head_ref_name: confirmed_pull_request.head_ref_name,
            head_ref_oid: confirmed_pull_request.head_ref_oid,
            local_branch: draft.local_branch,
            local_head_sha: draft.local_head_sha,
            merge_state_status: confirmed_pull_request.merge_state_status,
            mergeable: confirmed_pull_request.mergeable,
            command_evidence: draft.command_evidence,
            check_runs,
            quality_audit_cycles: draft.quality_audit_cycles,
            changed_files: draft.changed_files,
            docs_impact: draft.docs_impact,
            pr_evidence,
            manual_merge_attempted: draft.manual_merge_attempted,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GhCliReadinessClient {
    gh_binary: String,
}

impl GhCliReadinessClient {
    pub fn new() -> Self {
        Self {
            gh_binary: DEFAULT_GH_BINARY.into(),
        }
    }

    pub fn with_binary(gh_binary: impl Into<String>) -> Self {
        Self {
            gh_binary: gh_binary.into(),
        }
    }

    fn gh_json(&self, args: &[String]) -> Result<String, ExternalServiceError> {
        let output = Command::new(&self.gh_binary)
            .args(args)
            .output()
            .map_err(|error| {
                ExternalServiceError::new(
                    ExternalServiceErrorKind::CommandFailed,
                    format!("failed to execute '{}': {error}", self.gh_binary),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(classify_gh_failure(stderr.trim()));
        }

        String::from_utf8(output.stdout).map_err(|error| {
            ExternalServiceError::new(
                ExternalServiceErrorKind::InvalidResponse,
                format!("gh returned non-UTF-8 output: {error}"),
            )
        })
    }
}

impl Default for GhCliReadinessClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubReadinessClient for GhCliReadinessClient {
    fn pull_request(&self, pr_number: u64) -> Result<GitHubPullRequest, ExternalServiceError> {
        let args = vec![
            "pr".into(),
            "view".into(),
            pr_number.to_string(),
            "--json".into(),
            "headRefName,headRefOid,mergeStateStatus,mergeable,isCrossRepository,body,comments"
                .into(),
        ];
        let output = self.gh_json(&args)?;
        let response: GhPullRequestResponse = serde_json::from_str(&output).map_err(|error| {
            ExternalServiceError::new(
                ExternalServiceErrorKind::ParseFailed,
                format!("failed to parse gh pr view response: {error}"),
            )
        })?;

        let mut evidence_texts = vec![GitHubEvidenceText {
            location: "PR body".into(),
            trusted: !response.is_cross_repository,
            body: response.body,
        }];
        evidence_texts.extend(
            response
                .comments
                .into_iter()
                .enumerate()
                .map(|(index, comment)| GitHubEvidenceText {
                    location: format!(
                        "PR comment {} ({})",
                        index + 1,
                        comment.author_association.as_deref().unwrap_or("UNKNOWN")
                    ),
                    trusted: is_trusted_comment_association(comment.author_association.as_deref()),
                    body: comment.body,
                }),
        );

        Ok(GitHubPullRequest {
            head_ref_name: response.head_ref_name,
            head_ref_oid: response.head_ref_oid,
            merge_state_status: response.merge_state_status,
            mergeable: response.mergeable,
            evidence_texts,
        })
    }

    fn check_runs(&self, pr_number: u64) -> Result<Vec<GitHubCheckRun>, ExternalServiceError> {
        let args = vec![
            "pr".into(),
            "checks".into(),
            pr_number.to_string(),
            "--json".into(),
            "name,state,bucket".into(),
        ];
        let output = self.gh_json(&args)?;
        let response: Vec<GhCheckResponse> = serde_json::from_str(&output).map_err(|error| {
            ExternalServiceError::new(
                ExternalServiceErrorKind::ParseFailed,
                format!("failed to parse gh pr checks response: {error}"),
            )
        })?;

        Ok(response
            .into_iter()
            .map(|check| GitHubCheckRun {
                name: check.name,
                status: map_check_status(check.state.as_deref()),
                conclusion: map_check_conclusion(check.state.as_deref(), check.bucket.as_deref()),
            })
            .collect())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPullRequestResponse {
    head_ref_name: String,
    head_ref_oid: String,
    merge_state_status: String,
    mergeable: String,
    is_cross_repository: bool,
    #[serde(default)]
    body: String,
    #[serde(default)]
    comments: Vec<GhCommentResponse>,
}

#[derive(Deserialize)]
struct GhCommentResponse {
    #[serde(default)]
    body: String,
    #[serde(rename = "authorAssociation")]
    author_association: Option<String>,
}

#[derive(Deserialize)]
struct GhCheckResponse {
    name: String,
    state: Option<String>,
    bucket: Option<String>,
}

fn pr_evidence_from_texts(pull_request: &GitHubPullRequest) -> PREvidenceReview {
    let evidence_text = pull_request
        .evidence_texts
        .iter()
        .filter(|text| text.trusted)
        .find(|text| text.body.contains(&pull_request.head_ref_oid))
        .or_else(|| pull_request.evidence_texts.iter().find(|text| text.trusted));

    let Some(evidence_text) = evidence_text else {
        return empty_pr_evidence();
    };

    let lower = evidence_text.body.to_lowercase();
    PREvidenceReview {
        location: evidence_text.location.clone(),
        trusted_provenance: true,
        head_sha: if evidence_text.body.contains(&pull_request.head_ref_oid) {
            pull_request.head_ref_oid.clone()
        } else {
            String::new()
        },
        recorded_commands: super::REQUIRED_COMMANDS
            .iter()
            .filter(|command| evidence_text.body.contains(**command))
            .map(|command| (*command).into())
            .collect(),
        records_github_checks: lower.contains("github") && lower.contains("check"),
        records_diff_scope: lower.contains("diff") && lower.contains("scope"),
        records_docs_impact: lower.contains("docs") && lower.contains("impact"),
        records_quality_audit: lower.contains("quality audit"),
        records_no_manual_merge: lower.contains("no manual merge"),
        updated_during_review: false,
        reconfirmed_head_sha: None,
    }
}

fn empty_pr_evidence() -> PREvidenceReview {
    PREvidenceReview {
        location: "missing".into(),
        trusted_provenance: false,
        head_sha: String::new(),
        recorded_commands: Vec::new(),
        records_github_checks: false,
        records_diff_scope: false,
        records_docs_impact: false,
        records_quality_audit: false,
        records_no_manual_merge: false,
        updated_during_review: false,
        reconfirmed_head_sha: None,
    }
}

fn map_check_status(state: Option<&str>) -> CheckStatus {
    match normalize(state).as_deref() {
        Some(
            "COMPLETED" | "SUCCESS" | "FAILURE" | "CANCELLED" | "SKIPPED" | "TIMED_OUT"
            | "TIMEDOUT" | "NEUTRAL",
        ) => CheckStatus::Completed,
        Some("IN_PROGRESS") | Some("ACTION_REQUIRED") => CheckStatus::InProgress,
        Some("QUEUED") | Some("REQUESTED") | Some("WAITING") => CheckStatus::Queued,
        _ => CheckStatus::Pending,
    }
}

fn map_check_conclusion(state: Option<&str>, bucket: Option<&str>) -> CheckConclusion {
    match normalize(state).as_deref() {
        Some("SUCCESS") => CheckConclusion::Success,
        Some("FAILURE") => CheckConclusion::Failure,
        Some("CANCELLED") => CheckConclusion::Cancelled,
        Some("SKIPPED") => CheckConclusion::Skipped,
        Some("TIMED_OUT") | Some("TIMEDOUT") => CheckConclusion::TimedOut,
        Some("NEUTRAL") => CheckConclusion::Neutral,
        _ => match normalize(bucket).as_deref() {
            Some("PASS") => CheckConclusion::Success,
            Some("FAIL") => CheckConclusion::Failure,
            Some("CANCEL") => CheckConclusion::Cancelled,
            Some("SKIPPING") | Some("SKIP") => CheckConclusion::Skipped,
            _ => CheckConclusion::Unknown,
        },
    }
}

fn is_trusted_comment_association(author_association: Option<&str>) -> bool {
    matches!(
        normalize(author_association).as_deref(),
        Some("OWNER" | "MEMBER" | "COLLABORATOR")
    )
}

fn normalize(value: Option<&str>) -> Option<String> {
    value.map(|value| value.trim().replace('-', "_").to_uppercase())
}

fn classify_gh_failure(stderr: &str) -> ExternalServiceError {
    let lower = stderr.to_lowercase();
    let kind = if lower.contains("rate limit") || lower.contains("secondary rate limit") {
        ExternalServiceErrorKind::RateLimited
    } else if lower.contains("timeout") || lower.contains("timed out") {
        ExternalServiceErrorKind::Timeout
    } else if lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("temporarily unavailable")
    {
        ExternalServiceErrorKind::TemporarilyUnavailable
    } else {
        ExternalServiceErrorKind::CommandFailed
    };
    ExternalServiceError::new(kind, stderr)
}
