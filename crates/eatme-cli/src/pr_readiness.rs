use std::fmt;

use serde::{Deserialize, Serialize};

mod github;
mod recovery;
mod recovery_model;
mod recovery_safety;
mod report;

pub use github::{GitHubPrSnapshotRequest, fetch_github_pr_snapshot};
pub use recovery::evaluate_recovery_readiness;
pub use recovery_model::{
    ChangeOutcome, DiffScopeEvidence, DocsImpactEvidence, PrDescriptionEvidence, QualityAuditCycle,
    QualityAuditOutcome, QualityAuditPhase, RecoveryReadinessInput, RecoveryReadinessReport,
    RecoveryReadinessStatus, RecoveryValidationEvidence,
};
pub use report::{render_final_report, render_review_note};

const SHA_LEN: usize = 40;
const PR_204_BRANCH: &str = "wave7-eatme-nonclaim-audit-1778303500";

const FORBIDDEN_CLAIMS: &[&str] = &[
    "full Alice UI automation",
    "grading",
    "creative assessment",
    "visible rendering correctness",
    "Save completion",
    "first-lesson completion",
];
const NONCLAIM_MARKERS: &[&str] = &[
    "nonclaims",
    "does not validate",
    "do not claim",
    "must not imply",
    "cannot convert",
    "unsupported claim",
    "not validate",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessError {
    message: String,
}

impl ReadinessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ReadinessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ReadinessError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrReadinessSnapshot {
    pub pr_number: u64,
    pub branch: String,
    pub local_head_sha: String,
    pub pr_head_sha: String,
    pub merge_state_status: String,
    pub mergeable: String,
    pub checks: Vec<CheckSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckSummary {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: CheckConclusion,
    pub required: bool,
    pub head_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Completed,
    InProgress,
    Queued,
    Requested,
    Waiting,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckConclusion {
    Success,
    Skipped,
    Failure,
    Pending,
    Cancelled,
    TimedOut,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalEvidence {
    pub asset_validation: bool,
    pub generated_gadugi_freshness: bool,
    pub quality_gates: bool,
    pub documentation_build: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewNoteInput {
    pub snapshot: PrReadinessSnapshot,
    pub local_evidence: LocalEvidence,
    pub stale_evidence_handled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaleEvidencePolicy {
    Label,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalGateInput {
    pub evidence_sha: String,
    pub latest_pr_head_sha: String,
    pub latest_review_note_body: String,
}

pub fn validate_target_branch(
    snapshot: &PrReadinessSnapshot,
    expected_branch: &str,
) -> Result<(), ReadinessError> {
    validate_sha(&snapshot.local_head_sha)?;
    validate_sha(&snapshot.pr_head_sha)?;

    if snapshot.branch != expected_branch {
        return Err(ReadinessError::new(format!(
            "PR #{} must be on branch {expected_branch}, but local branch is {}",
            snapshot.pr_number, snapshot.branch
        )));
    }

    if snapshot.local_head_sha != snapshot.pr_head_sha {
        return Err(ReadinessError::new(format!(
            "local HEAD {} does not match PR head {} for branch {expected_branch}",
            snapshot.local_head_sha, snapshot.pr_head_sha
        )));
    }

    Ok(())
}

pub fn validate_exact_head_evidence(
    evidence_sha: &str,
    evidence_items: &[String],
) -> Result<(), ReadinessError> {
    validate_sha(evidence_sha)?;

    if evidence_items.is_empty() {
        return Err(ReadinessError::new(format!(
            "readiness evidence for {evidence_sha} must include at least one item"
        )));
    }

    for (index, item) in evidence_items.iter().enumerate() {
        if !item.contains(evidence_sha) {
            return Err(ReadinessError::new(format!(
                "evidence item {} must name the full 40-character evidence SHA {evidence_sha}",
                index + 1
            )));
        }
    }

    Ok(())
}

pub fn scrub_stale_evidence(
    current_sha: &str,
    evidence_items: Vec<String>,
    policy: StaleEvidencePolicy,
) -> Result<Vec<String>, ReadinessError> {
    validate_sha(current_sha)?;

    Ok(evidence_items
        .into_iter()
        .filter_map(|item| {
            if !contains_stale_sha(&item, current_sha) {
                return Some(item);
            }

            match policy {
                StaleEvidencePolicy::Remove => None,
                StaleEvidencePolicy::Label => Some(label_stale_evidence(&item, current_sha)),
            }
        })
        .collect())
}

pub fn validate_pr_204_documentation(evidence_sha: &str, docs: &str) -> Result<(), ReadinessError> {
    validate_sha(evidence_sha)?;
    validate_forbidden_claims_are_nonclaims(docs)?;

    for required in [
        evidence_sha,
        PR_204_BRANCH,
        "asset validation",
        "generated Gadugi freshness",
        "repository quality gates",
        "documentation build",
        "required GitHub checks",
        "optional checks are skipped",
        "mergeStateStatus=CLEAN",
        "mergeable=MERGEABLE",
        "stale/non-current",
        "not current validation",
    ] {
        if !docs.contains(required) {
            return Err(ReadinessError::new(format!(
                "PR #204 documentation for {evidence_sha} is missing required wording: {required}"
            )));
        }
    }

    Ok(())
}

pub fn verify_final_gate(input: &FinalGateInput) -> Result<(), ReadinessError> {
    validate_sha(&input.evidence_sha)?;
    validate_sha(&input.latest_pr_head_sha)?;

    if input.evidence_sha != input.latest_pr_head_sha {
        return Err(ReadinessError::new(format!(
            "PR head changed after evidence collection; rerun readiness for {} because current evidence names {}",
            input.latest_pr_head_sha, input.evidence_sha
        )));
    }

    if !input.latest_review_note_body.contains(&input.evidence_sha) {
        return Err(ReadinessError::new(format!(
            "latest review note must name exact SHA {}",
            input.evidence_sha
        )));
    }

    if !input.latest_review_note_body.contains("stale/non-current") {
        return Err(ReadinessError::new(
            "latest review note must label older tested-head evidence stale/non-current",
        ));
    }

    Ok(())
}

fn validate_sha(sha: &str) -> Result<(), ReadinessError> {
    if sha.len() != SHA_LEN || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ReadinessError::new(format!(
            "expected full 40-character SHA, got {sha}"
        )));
    }
    Ok(())
}

fn contains_stale_sha(text: &str, current_sha: &str) -> bool {
    text.split(|character: char| !character.is_ascii_hexdigit())
        .any(|candidate| candidate.len() == SHA_LEN && candidate != current_sha)
}

fn label_stale_evidence(item: &str, current_sha: &str) -> String {
    let sanitized = item
        .replace("current validation", "non-current evidence")
        .replace("Current validation", "Non-current evidence");

    if sanitized.contains("stale/non-current") {
        sanitized
    } else {
        format!("stale/non-current: {sanitized} (superseded by exact SHA {current_sha})")
    }
}

fn validate_forbidden_claims_are_nonclaims(docs: &str) -> Result<(), ReadinessError> {
    for claim in FORBIDDEN_CLAIMS {
        for (position, _) in docs.match_indices(claim) {
            let context = paragraph_around(docs, position);
            if !is_nonclaim_context(context) {
                return Err(ReadinessError::new(format!(
                    "forbidden claim must be listed as an explicit nonclaim: {claim}"
                )));
            }
        }
    }
    Ok(())
}

fn paragraph_around(text: &str, position: usize) -> &str {
    let start = text[..position].rfind("\n\n").map_or(0, |index| index + 2);
    let end = text[position..]
        .find("\n\n")
        .map_or(text.len(), |index| position + index);
    &text[start..end]
}

fn is_nonclaim_context(context: &str) -> bool {
    NONCLAIM_MARKERS
        .iter()
        .any(|marker| contains_ascii_case_insensitive(context, marker))
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();

    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

impl fmt::Display for CheckConclusion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            CheckConclusion::Success => "success",
            CheckConclusion::Skipped => "skipped",
            CheckConclusion::Failure => "failure",
            CheckConclusion::Pending => "pending",
            CheckConclusion::Cancelled => "cancelled",
            CheckConclusion::TimedOut => "timed out",
            CheckConclusion::Missing => "missing",
        };
        formatter.write_str(label)
    }
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            CheckStatus::Completed => "completed",
            CheckStatus::InProgress => "in progress",
            CheckStatus::Queued => "queued",
            CheckStatus::Requested => "requested",
            CheckStatus::Waiting => "waiting",
            CheckStatus::Missing => "missing",
        };
        formatter.write_str(label)
    }
}
