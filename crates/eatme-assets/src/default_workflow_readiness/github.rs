use std::collections::HashSet;
use std::fmt;
use std::process::Command;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use super::{
    CheckConclusion, CheckRunEvidence, CheckStatus, CommandEvidence, DocsImpactReview,
    QualityAuditCycle, ReadinessInput,
};
use evidence::{contains_ascii_case_insensitive, pr_evidence_from_texts};

mod evidence;

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
    pub required: bool,
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
                required: check.required,
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

    fn gh_json(&self, args: &[&str]) -> Result<String, ExternalServiceError> {
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
        let pr_number = pr_number.to_string();
        let output = self.gh_json(&[
            "pr",
            "view",
            pr_number.as_str(),
            "--json",
            "headRefName,headRefOid,mergeStateStatus,mergeable,isCrossRepository,body,comments",
        ])?;
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
        let pr_number = pr_number.to_string();
        let output = self.gh_json(&[
            "pr",
            "checks",
            pr_number.as_str(),
            "--json",
            "name,state,bucket",
        ])?;
        let required_output = self.gh_json(&[
            "pr",
            "checks",
            pr_number.as_str(),
            "--required",
            "--json",
            "name,state,bucket",
        ])?;
        let response: Vec<GhCheckResponse> = serde_json::from_str(&output).map_err(|error| {
            ExternalServiceError::new(
                ExternalServiceErrorKind::ParseFailed,
                format!("failed to parse gh pr checks response: {error}"),
            )
        })?;
        let required_response: Vec<GhCheckResponse> = serde_json::from_str(&required_output)
            .map_err(|error| {
                ExternalServiceError::new(
                    ExternalServiceErrorKind::ParseFailed,
                    format!("failed to parse gh pr checks --required response: {error}"),
                )
            })?;
        let required_names = required_response
            .into_iter()
            .map(|check| check.name)
            .collect::<HashSet<_>>();

        Ok(response
            .into_iter()
            .map(|check| GitHubCheckRun {
                required: required_names.contains(&check.name),
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

fn map_check_status(state: Option<&str>) -> CheckStatus {
    if normalized_is_any(
        state,
        &[
            "COMPLETED",
            "SUCCESS",
            "FAILURE",
            "CANCELLED",
            "SKIPPED",
            "TIMED_OUT",
            "TIMEDOUT",
            "NEUTRAL",
        ],
    ) {
        CheckStatus::Completed
    } else if normalized_is_any(state, &["IN_PROGRESS", "ACTION_REQUIRED"]) {
        CheckStatus::InProgress
    } else if normalized_is_any(state, &["QUEUED", "REQUESTED", "WAITING"]) {
        CheckStatus::Queued
    } else {
        CheckStatus::Pending
    }
}

fn map_check_conclusion(state: Option<&str>, bucket: Option<&str>) -> CheckConclusion {
    if normalized_is_any(state, &["SUCCESS"]) {
        CheckConclusion::Success
    } else if normalized_is_any(state, &["FAILURE"]) {
        CheckConclusion::Failure
    } else if normalized_is_any(state, &["CANCELLED"]) {
        CheckConclusion::Cancelled
    } else if normalized_is_any(state, &["SKIPPED"]) {
        CheckConclusion::Skipped
    } else if normalized_is_any(state, &["TIMED_OUT", "TIMEDOUT"]) {
        CheckConclusion::TimedOut
    } else if normalized_is_any(state, &["NEUTRAL"]) {
        CheckConclusion::Neutral
    } else if normalized_is_any(bucket, &["PASS"]) {
        CheckConclusion::Success
    } else if normalized_is_any(bucket, &["FAIL"]) {
        CheckConclusion::Failure
    } else if normalized_is_any(bucket, &["CANCEL"]) {
        CheckConclusion::Cancelled
    } else if normalized_is_any(bucket, &["SKIPPING", "SKIP"]) {
        CheckConclusion::Skipped
    } else {
        CheckConclusion::Unknown
    }
}

fn is_trusted_comment_association(author_association: Option<&str>) -> bool {
    normalized_is_any(author_association, &["OWNER", "MEMBER", "COLLABORATOR"])
}

fn normalized_is_any(value: Option<&str>, expected_values: &[&str]) -> bool {
    let Some(value) = value else {
        return false;
    };
    expected_values
        .iter()
        .any(|expected| normalized_eq(value, expected))
}

fn normalized_eq(value: &str, expected: &str) -> bool {
    value
        .trim()
        .bytes()
        .map(normalized_ascii_byte)
        .eq(expected.bytes().map(normalized_ascii_byte))
}

fn normalized_ascii_byte(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => byte.to_ascii_uppercase(),
        b'-' => b'_',
        _ => byte,
    }
}

fn classify_gh_failure(stderr: &str) -> ExternalServiceError {
    let kind = if contains_ascii_case_insensitive(stderr, "rate limit") {
        ExternalServiceErrorKind::RateLimited
    } else if contains_ascii_case_insensitive(stderr, "timeout")
        || contains_ascii_case_insensitive(stderr, "timed out")
    {
        ExternalServiceErrorKind::Timeout
    } else if stderr.contains("502")
        || stderr.contains("503")
        || stderr.contains("504")
        || contains_ascii_case_insensitive(stderr, "temporarily unavailable")
    {
        ExternalServiceErrorKind::TemporarilyUnavailable
    } else {
        ExternalServiceErrorKind::CommandFailed
    };
    ExternalServiceError::new(kind, stderr)
}
