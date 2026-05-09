use super::REQUIRED_LOCAL_QA_COMMANDS;
use super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrHeadEvidence {
    pub branch: String,
    pub local_head: String,
    pub remote_head: String,
    pub pr_head_ref_oid: String,
    pub manually_merged: bool,
    pub rebased_or_rewritten: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrHead {
    branch: String,
    evaluated_head: String,
}

impl VerifiedPrHead {
    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub fn evaluated_head(&self) -> &str {
        &self.evaluated_head
    }
}

pub struct PrHeadSynchronizer;

impl PrHeadSynchronizer {
    pub fn verify(evidence: PrHeadEvidence) -> Result<VerifiedPrHead, ReadinessError> {
        if evidence.manually_merged || evidence.rebased_or_rewritten {
            return Err(ReadinessError::new(
                ReadinessErrorKind::ManualMergeOrHistoryRewrite,
                "manual merge, rebase, or history rewrite evidence blocks readiness",
            ));
        }

        if evidence.local_head != evidence.remote_head
            || evidence.local_head != evidence.pr_head_ref_oid
            || evidence.local_head.is_empty()
        {
            return Err(ReadinessError::new(
                ReadinessErrorKind::WrongHead,
                "local HEAD, remote branch HEAD, and PR headRefOid must match exactly",
            ));
        }

        Ok(VerifiedPrHead {
            branch: evidence.branch,
            evaluated_head: evidence.local_head,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckStatus {
    Missing,
    Pending,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckConclusion {
    Unknown,
    Success,
    Failure,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusCheck {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: CheckConclusion,
    pub head_sha: String,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrReviewDecision {
    Approved,
    ReviewRequired,
    ChangesRequested,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewEvidence {
    pub state: PrReviewState,
    pub commit_oid: String,
    pub author_login: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrMetadata {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub head_ref_name: String,
    pub head_ref_oid: String,
    pub merge_state_status: String,
    pub mergeable: bool,
    pub is_draft: bool,
    pub labels: Vec<String>,
    pub review_decision: PrReviewDecision,
    pub latest_reviews: Vec<ReviewEvidence>,
    pub status_checks: Vec<StatusCheck>,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectedEvidence {
    pr_number: u64,
    evaluated_head: String,
    github_actions_green: bool,
}

impl CollectedEvidence {
    pub fn pr_number(&self) -> u64 {
        self.pr_number
    }

    pub fn evaluated_head(&self) -> &str {
        &self.evaluated_head
    }

    pub fn github_actions_green(&self) -> bool {
        self.github_actions_green
    }
}

pub struct EvidenceCollector;

impl EvidenceCollector {
    pub fn from_pr_metadata(
        metadata: PrMetadata,
        evaluated_head: &str,
    ) -> Result<CollectedEvidence, ReadinessError> {
        if metadata.head_ref_oid != evaluated_head {
            return Err(ReadinessError::new(
                ReadinessErrorKind::WrongHead,
                "PR metadata headRefOid is stale for the evaluated head",
            ));
        }
        if !metadata.mergeable {
            return Err(ReadinessError::new(
                ReadinessErrorKind::MergeabilityBlocked,
                "PR metadata reports that the pull request is not mergeable",
            ));
        }
        if !is_acceptable_merge_state(&metadata.merge_state_status) {
            return Err(ReadinessError::new(
                ReadinessErrorKind::MergeabilityBlocked,
                format!(
                    "PR mergeStateStatus '{}' blocks readiness",
                    metadata.merge_state_status
                ),
            ));
        }
        if metadata.is_draft {
            return Err(ReadinessError::new(
                ReadinessErrorKind::DraftPullRequest,
                "draft pull requests are not merge-ready",
            ));
        }
        if let Some(label) = first_blocking_label(&metadata.labels) {
            return Err(ReadinessError::new(
                ReadinessErrorKind::BlockingPrLabel,
                format!("PR label '{label}' blocks readiness"),
            ));
        }
        if metadata.review_decision == PrReviewDecision::ChangesRequested
            || metadata
                .latest_reviews
                .iter()
                .any(|review| review.state == PrReviewState::ChangesRequested)
        {
            return Err(ReadinessError::new(
                ReadinessErrorKind::BlockingReviewState,
                "latest PR review state requests changes",
            ));
        }
        if has_stale_decisive_review(&metadata.latest_reviews, evaluated_head) {
            return Err(ReadinessError::new(
                ReadinessErrorKind::StaleReviewEvidence,
                "latest decisive PR review evidence is for a different commit",
            ));
        }

        let required_checks: Vec<&StatusCheck> = metadata
            .status_checks
            .iter()
            .filter(|check| check.required)
            .collect();
        if required_checks.is_empty() {
            return Err(ReadinessError::new(
                ReadinessErrorKind::MissingChecks,
                "no required GitHub Actions checks were reported",
            ));
        }

        for check in required_checks {
            if check.head_sha != evaluated_head {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::WrongHead,
                    format!("check '{}' is for a different commit", check.name),
                ));
            }
            if check.status == CheckStatus::Missing {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::MissingChecks,
                    format!("required check '{}' is missing", check.name),
                ));
            }
            if check.status != CheckStatus::Completed {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::IncompleteChecks,
                    format!("required check '{}' has not completed", check.name),
                ));
            }
            if check.conclusion != CheckConclusion::Success {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::FailingChecks,
                    format!("required check '{}' did not succeed", check.name),
                ));
            }
        }

        Ok(CollectedEvidence {
            pr_number: metadata.number,
            evaluated_head: evaluated_head.to_string(),
            github_actions_green: true,
        })
    }
}

fn is_acceptable_merge_state(status: &str) -> bool {
    matches!(status.trim(), value if value.eq_ignore_ascii_case("CLEAN") || value.eq_ignore_ascii_case("HAS_HOOKS"))
}

fn first_blocking_label(labels: &[String]) -> Option<&str> {
    labels
        .iter()
        .find(|label| is_blocking_label(label))
        .map(String::as_str)
}

fn is_blocking_label(label: &str) -> bool {
    let normalized = label.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "do-not-merge" | "do not merge" | "blocked" | "wip" | "hold"
    )
}

fn has_stale_decisive_review(reviews: &[ReviewEvidence], evaluated_head: &str) -> bool {
    reviews.iter().any(|review| {
        matches!(
            review.state,
            PrReviewState::Approved | PrReviewState::ChangesRequested
        ) && review.commit_oid != evaluated_head
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalQaCommandList {
    commands: Vec<String>,
}

impl LocalQaCommandList {
    pub fn shell_lines(&self) -> Vec<String> {
        self.commands.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalQaCommandOutput {
    pub command: String,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalQaReport {
    passed: bool,
    outputs: Vec<LocalQaCommandOutput>,
}

impl LocalQaReport {
    pub fn passed(&self) -> bool {
        self.passed
    }

    pub fn outputs(&self) -> &[LocalQaCommandOutput] {
        &self.outputs
    }
}

pub struct LocalQARunner;

impl LocalQARunner {
    pub fn required_commands() -> LocalQaCommandList {
        LocalQaCommandList {
            commands: REQUIRED_LOCAL_QA_COMMANDS
                .iter()
                .map(|command| (*command).to_string())
                .collect(),
        }
    }

    pub fn summarize(outputs: &[LocalQaCommandOutput]) -> Result<LocalQaReport, ReadinessError> {
        for output in outputs {
            if !REQUIRED_LOCAL_QA_COMMANDS.contains(&output.command.as_str())
                || output.command.starts_with("timeout ")
                || output.command.contains(" timeout ")
            {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::UnsupportedEvidenceSubstitution,
                    format!("unsupported local QA evidence command '{}'", output.command),
                ));
            }
        }

        for required in REQUIRED_LOCAL_QA_COMMANDS {
            let Some(output) = outputs.iter().find(|output| output.command == required) else {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::MissingLocalQa,
                    format!("missing required local QA evidence for '{required}'"),
                ));
            };
            if output.exit_status != Some(0) {
                return Err(ReadinessError::new(
                    ReadinessErrorKind::FailedLocalQa,
                    format!("local QA command '{required}' did not pass"),
                ));
            }
        }

        Ok(LocalQaReport {
            passed: true,
            outputs: outputs.to_vec(),
        })
    }
}
