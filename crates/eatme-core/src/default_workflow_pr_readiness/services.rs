use super::error::{ReadinessError, ReadinessErrorKind};
use super::head::{
    CheckConclusion, CheckStatus, PrHeadEvidence, PrMetadata, PrReviewDecision, PrReviewState,
    ReviewEvidence, StatusCheck,
};
use crate::{CommandOutput, CommandRunner, CommandSpec};
use serde::Deserialize;
use std::time::Duration;

const PR_VIEW_FIELDS: &str = concat!(
    "number,title,body,headRefName,headRefOid,",
    "mergeStateStatus,mergeable,isDraft,labels,reviewDecision,",
    "latestReviews,statusCheckRollup,files"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalServiceRetryPolicy {
    attempts: usize,
    retry_delay: Duration,
}

impl ExternalServiceRetryPolicy {
    pub fn new(attempts: usize, retry_delay: Duration) -> Self {
        Self {
            attempts: attempts.max(1),
            retry_delay,
        }
    }

    fn apply_to(self, spec: CommandSpec) -> CommandSpec {
        spec.retries(self.attempts, self.retry_delay)
    }
}

impl Default for ExternalServiceRetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(500))
    }
}

pub trait GitHubPrService {
    fn pr_metadata(&self, pr_number: u64) -> Result<PrMetadata, ReadinessError>;
}

pub struct GitHubCliPrService<'a, R: CommandRunner> {
    runner: &'a R,
    retry_policy: ExternalServiceRetryPolicy,
}

impl<'a, R: CommandRunner> GitHubCliPrService<'a, R> {
    pub fn new(runner: &'a R) -> Self {
        Self {
            runner,
            retry_policy: ExternalServiceRetryPolicy::default(),
        }
    }

    pub fn with_retry_policy(mut self, retry_policy: ExternalServiceRetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }
}

impl<R: CommandRunner> GitHubPrService for GitHubCliPrService<'_, R> {
    fn pr_metadata(&self, pr_number: u64) -> Result<PrMetadata, ReadinessError> {
        let spec = self.retry_policy.apply_to(CommandSpec::new("gh").args([
            "pr".to_string(),
            "view".to_string(),
            pr_number.to_string(),
            "--json".to_string(),
            PR_VIEW_FIELDS.to_string(),
        ]));
        let output = run_external(self.runner, spec, "GitHub")?;
        let view = parse_pr_view(&output.stdout)?;
        Ok(view.into_metadata())
    }
}

pub trait PrHeadService {
    fn pr_head_evidence(
        &self,
        branch: &str,
        pr_head_ref_oid: &str,
    ) -> Result<PrHeadEvidence, ReadinessError>;
}

pub struct GitCliPrHeadAdapter<'a, R: CommandRunner> {
    runner: &'a R,
    retry_policy: ExternalServiceRetryPolicy,
}

impl<'a, R: CommandRunner> GitCliPrHeadAdapter<'a, R> {
    pub fn new(runner: &'a R) -> Self {
        Self {
            runner,
            retry_policy: ExternalServiceRetryPolicy::default(),
        }
    }

    pub fn with_retry_policy(mut self, retry_policy: ExternalServiceRetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }
}

impl<R: CommandRunner> PrHeadService for GitCliPrHeadAdapter<'_, R> {
    fn pr_head_evidence(
        &self,
        branch: &str,
        pr_head_ref_oid: &str,
    ) -> Result<PrHeadEvidence, ReadinessError> {
        run_external(
            self.runner,
            self.retry_policy.apply_to(CommandSpec::new("git").args([
                "fetch".to_string(),
                "origin".to_string(),
                branch.to_string(),
            ])),
            "Git",
        )?;

        let local_head = run_external(
            self.runner,
            self.retry_policy
                .apply_to(CommandSpec::new("git").args(["rev-parse", "HEAD"])),
            "Git",
        )?;
        let remote_head = run_external(
            self.runner,
            self.retry_policy
                .apply_to(CommandSpec::new("git").args(["rev-parse", &origin_ref(branch)])),
            "Git",
        )?;

        Ok(PrHeadEvidence {
            branch: branch.to_string(),
            local_head: single_line_stdout(&local_head),
            remote_head: single_line_stdout(&remote_head),
            pr_head_ref_oid: pr_head_ref_oid.to_string(),
            manually_merged: false,
            rebased_or_rewritten: false,
        })
    }
}

fn run_external<R: CommandRunner>(
    runner: &R,
    spec: CommandSpec,
    service_name: &str,
) -> Result<CommandOutput, ReadinessError> {
    let output = runner.run(&spec).map_err(|error| {
        ReadinessError::new(
            ReadinessErrorKind::ExternalServiceUnavailable,
            format!("{service_name} service call could not run: {error}"),
        )
    })?;

    if output.exit_status == Some(0) {
        Ok(output)
    } else {
        Err(ReadinessError::new(
            ReadinessErrorKind::ExternalServiceFailed,
            format!(
                "{service_name} service command '{}' failed with status {:?}",
                output.command, output.exit_status
            ),
        ))
    }
}

fn parse_pr_view(stdout: &str) -> Result<GitHubPrView, ReadinessError> {
    serde_json::from_str(stdout).map_err(|error| {
        ReadinessError::new(
            ReadinessErrorKind::MalformedExternalResponse,
            format!("GitHub PR metadata response was not valid JSON: {error}"),
        )
    })
}

fn single_line_stdout(output: &CommandOutput) -> String {
    output
        .stdout
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn origin_ref(branch: &str) -> String {
    format!("origin/{branch}")
}

#[derive(Debug, Deserialize)]
struct GitHubPrView {
    number: u64,
    title: String,
    #[serde(default)]
    body: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "headRefOid")]
    head_ref_oid: String,
    #[serde(rename = "mergeStateStatus")]
    merge_state_status: String,
    mergeable: GitHubMergeable,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    labels: Vec<GitHubLabel>,
    #[serde(rename = "reviewDecision")]
    review_decision: GitHubReviewDecision,
    #[serde(rename = "latestReviews")]
    latest_reviews: Vec<GitHubReview>,
    #[serde(default, rename = "statusCheckRollup")]
    status_check_rollup: Vec<GitHubStatusCheck>,
    #[serde(default)]
    files: Vec<GitHubPrFile>,
}

impl GitHubPrView {
    fn into_metadata(self) -> PrMetadata {
        let head_sha = self.head_ref_oid.clone();
        PrMetadata {
            number: self.number,
            title: self.title,
            body: self.body,
            head_ref_name: self.head_ref_name,
            head_ref_oid: self.head_ref_oid,
            merge_state_status: self.merge_state_status,
            mergeable: self.mergeable.is_mergeable(),
            is_draft: self.is_draft,
            labels: self
                .labels
                .into_iter()
                .map(GitHubLabel::into_name)
                .collect(),
            review_decision: self.review_decision.into_pr_review_decision(),
            latest_reviews: self
                .latest_reviews
                .into_iter()
                .map(GitHubReview::into_review_evidence)
                .collect(),
            status_checks: self
                .status_check_rollup
                .into_iter()
                .map(|check| check.into_status_check(&head_sha))
                .collect(),
            files: self
                .files
                .into_iter()
                .map(GitHubPrFile::into_path)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GitHubMergeable {
    Bool(bool),
    Text(String),
}

impl GitHubMergeable {
    fn is_mergeable(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Text(value) => value.eq_ignore_ascii_case("MERGEABLE"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GitHubLabel {
    Object { name: String },
    Name(String),
}

impl GitHubLabel {
    fn into_name(self) -> String {
        match self {
            Self::Object { name } | Self::Name(name) => name,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct GitHubReviewDecision(String);

impl GitHubReviewDecision {
    fn into_pr_review_decision(self) -> PrReviewDecision {
        match self.0.trim() {
            value if value.eq_ignore_ascii_case("APPROVED") => PrReviewDecision::Approved,
            value if value.eq_ignore_ascii_case("REVIEW_REQUIRED") => {
                PrReviewDecision::ReviewRequired
            }
            value if value.eq_ignore_ascii_case("CHANGES_REQUESTED") => {
                PrReviewDecision::ChangesRequested
            }
            _ => PrReviewDecision::Unknown,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitHubReview {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    commit: Option<GitHubCommit>,
    #[serde(default, rename = "commitOid")]
    commit_oid: Option<String>,
    #[serde(default)]
    author: Option<GitHubAuthor>,
}

impl GitHubReview {
    fn into_review_evidence(self) -> ReviewEvidence {
        let state = parse_review_state(self.state.as_deref());
        let commit_oid = self
            .commit
            .and_then(|commit| commit.oid)
            .or(self.commit_oid)
            .unwrap_or_default();
        let author_login = self
            .author
            .and_then(|author| author.login)
            .unwrap_or_default();

        ReviewEvidence {
            state,
            commit_oid,
            author_login,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitHubCommit {
    #[serde(default)]
    oid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubAuthor {
    #[serde(default)]
    login: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubStatusCheck {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default, rename = "workflowName")]
    workflow_name: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default, rename = "headSha")]
    head_sha: Option<String>,
}

impl GitHubStatusCheck {
    fn into_status_check(self, default_head_sha: &str) -> StatusCheck {
        let conclusion = parse_check_conclusion(self.conclusion.as_deref(), self.state.as_deref());
        StatusCheck {
            name: first_non_empty([self.name, self.context, self.workflow_name])
                .unwrap_or_else(|| "unnamed status check".to_string()),
            status: parse_check_status(self.status.as_deref(), self.state.as_deref()),
            conclusion,
            head_sha: self
                .head_sha
                .unwrap_or_else(|| default_head_sha.to_string()),
            required: conclusion != CheckConclusion::Skipped,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GitHubPrFile {
    Object { path: String },
    Path(String),
}

impl GitHubPrFile {
    fn into_path(self) -> String {
        match self {
            Self::Object { path } | Self::Path(path) => path,
        }
    }
}

fn first_non_empty(values: [Option<String>; 3]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn parse_check_status(status: Option<&str>, state: Option<&str>) -> CheckStatus {
    match first_non_empty_str(status).or_else(|| first_non_empty_str(state)) {
        Some(value) if matches_ignore_ascii_case(value, COMPLETED_STATUS_VALUES) => {
            CheckStatus::Completed
        }
        Some(value) if matches_ignore_ascii_case(value, MISSING_STATUS_VALUES) => {
            CheckStatus::Missing
        }
        Some(_) | None => CheckStatus::Pending,
    }
}

fn parse_check_conclusion(conclusion: Option<&str>, state: Option<&str>) -> CheckConclusion {
    match first_non_empty_str(conclusion).or_else(|| first_non_empty_str(state)) {
        Some(value) if value.eq_ignore_ascii_case("SUCCESS") => CheckConclusion::Success,
        Some(value) if value.eq_ignore_ascii_case("SKIPPED") => CheckConclusion::Skipped,
        Some(value) if matches_ignore_ascii_case(value, FAILURE_CONCLUSION_VALUES) => {
            CheckConclusion::Failure
        }
        Some(_) | None => CheckConclusion::Unknown,
    }
}

fn parse_review_state(state: Option<&str>) -> PrReviewState {
    match first_non_empty_str(state) {
        Some(value) if value.eq_ignore_ascii_case("APPROVED") => PrReviewState::Approved,
        Some(value) if value.eq_ignore_ascii_case("CHANGES_REQUESTED") => {
            PrReviewState::ChangesRequested
        }
        Some(value) if value.eq_ignore_ascii_case("COMMENTED") => PrReviewState::Commented,
        Some(value) if value.eq_ignore_ascii_case("DISMISSED") => PrReviewState::Dismissed,
        Some(_) | None => PrReviewState::Unknown,
    }
}

const COMPLETED_STATUS_VALUES: &[&str] = &["COMPLETED", "SUCCESS", "FAILURE", "ERROR"];
const MISSING_STATUS_VALUES: &[&str] = &["MISSING", "EXPECTED"];
const FAILURE_CONCLUSION_VALUES: &[&str] = &[
    "FAILURE",
    "ERROR",
    "CANCELLED",
    "TIMED_OUT",
    "ACTION_REQUIRED",
];

fn first_non_empty_str(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
