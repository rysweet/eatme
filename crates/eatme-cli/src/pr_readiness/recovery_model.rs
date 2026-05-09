use serde::{Deserialize, Serialize};

use super::CheckSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOutcome {
    NoOp { justification: String },
    FilesModified(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryValidationEvidence {
    pub name: String,
    pub command: String,
    pub evidence_sha: String,
    pub exit_status: i32,
    pub summary: String,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityAuditCycle {
    pub cycle_number: u64,
    pub phases: Vec<QualityAuditPhase>,
    pub outcome: QualityAuditOutcome,
    pub head_sha: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityAuditPhase {
    Seek,
    Validate,
    Fix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityAuditOutcome {
    FixApplied,
    Clean,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffScopeEvidence {
    pub changed_files: Vec<String>,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsImpactEvidence {
    pub docs_changed: bool,
    pub strict_build_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrDescriptionEvidence {
    pub head_sha: String,
    pub contains_readiness_evidence: bool,
    pub contains_bounded_nonclaims: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryReadinessInput {
    pub schema_version: String,
    #[serde(default)]
    pub expected_remote_head_sha: Option<String>,
    pub snapshot: super::PrReadinessSnapshot,
    pub validation_sha: String,
    #[serde(default)]
    pub required_github_checks: Vec<String>,
    pub asset_validation: RecoveryValidationEvidence,
    pub generated_gadugi_check: RecoveryValidationEvidence,
    pub quality_gate: RecoveryValidationEvidence,
    pub documentation_build: RecoveryValidationEvidence,
    pub quality_audit_cycles: Vec<QualityAuditCycle>,
    pub diff_scope: DiffScopeEvidence,
    pub docs_impact: DocsImpactEvidence,
    pub pr_description_evidence: PrDescriptionEvidence,
    pub stale_evidence_handled: bool,
    pub wrapper_failures: Vec<String>,
    pub change_outcome: ChangeOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryReadinessStatus {
    MergeReady,
    NotMergeReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryReadinessReport {
    pub status: RecoveryReadinessStatus,
    pub branch: String,
    pub expected_remote_head_sha: Option<String>,
    pub final_head_sha: String,
    pub validation_status: String,
    pub change_outcome: ChangeOutcome,
    pub required_github_checks: Vec<String>,
    pub github_checks: Vec<CheckSummary>,
    pub qa_evidence: Vec<RecoveryValidationEvidence>,
    pub quality_audit_cycles: Vec<QualityAuditCycle>,
    pub diff_scope: DiffScopeEvidence,
    pub docs_impact: DocsImpactEvidence,
    pub pr_description_evidence: PrDescriptionEvidence,
    pub wrapper_failures: Vec<String>,
    pub blockers: Vec<String>,
}
