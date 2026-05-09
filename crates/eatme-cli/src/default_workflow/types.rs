use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ReadinessEvidence {
    pub(crate) schema_version: String,
    pub(crate) pr: PrEvidence,
    pub(crate) local: LocalEvidence,
    pub(crate) checks: Vec<CheckEvidence>,
    pub(crate) commands: Vec<CommandEvidence>,
    pub(crate) audit_cycles: Vec<AuditCycleEvidence>,
    pub(crate) diff: DiffEvidence,
    pub(crate) docs: DocsEvidence,
    pub(crate) pr_description_evidence: PrDescriptionEvidence,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PrEvidence {
    pub(crate) number: u64,
    pub(crate) state: String,
    pub(crate) is_draft: bool,
    pub(crate) mergeable: String,
    pub(crate) merge_state_status: String,
    pub(crate) head_ref_oid: String,
    pub(crate) head_ref_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct LocalEvidence {
    pub(crate) head: String,
    pub(crate) checkout_mode: String,
    pub(crate) manual_merge_performed: bool,
    pub(crate) repository_changes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CheckEvidence {
    pub(crate) name: String,
    pub(crate) head_sha: String,
    pub(crate) status: String,
    pub(crate) conclusion: Option<String>,
    pub(crate) required: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CommandEvidence {
    pub(crate) id: String,
    pub(crate) command: String,
    pub(crate) exit_status: i32,
    pub(crate) used_timeout_wrapper: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AuditCycleEvidence {
    pub(crate) name: String,
    pub(crate) seek: String,
    pub(crate) validate: String,
    pub(crate) fix: String,
    pub(crate) clean: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DiffEvidence {
    pub(crate) files: Vec<String>,
    pub(crate) focused: bool,
    pub(crate) unrelated_churn: Vec<String>,
    pub(crate) generated_artifacts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DocsEvidence {
    pub(crate) impact_reviewed: bool,
    pub(crate) updated_or_ruled_out: bool,
    pub(crate) strict_build_passed: bool,
    pub(crate) strict_build_command: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PrDescriptionEvidence {
    pub(crate) head_ref_oid: String,
    pub(crate) mentions_green_actions: bool,
    pub(crate) mentions_runnable_qa: bool,
    pub(crate) mentions_docs_impact: bool,
    pub(crate) mentions_quality_audit_cycles: bool,
    pub(crate) unsupported_claims: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReadinessReport {
    pub(crate) decision: Decision,
    pub(crate) pr_number: Option<u64>,
    pub(crate) head_ref_name: Option<String>,
    pub(crate) head_ref_oid: Option<String>,
    pub(crate) local_head: Option<String>,
    pub(crate) files_modified: Vec<String>,
    pub(crate) no_op_justification: Option<String>,
    pub(crate) blockers: Vec<Blocker>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Decision {
    MergeReady,
    NotMergeReady,
}

#[derive(Debug, Serialize)]
pub(crate) struct Blocker {
    gate: String,
    code: String,
    message: String,
}

impl Blocker {
    pub(crate) fn new(gate: &str, code: &str, message: impl Into<String>) -> Self {
        Self {
            gate: gate.to_string(),
            code: code.to_string(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct GithubEvidenceReport {
    pub(crate) schema_version: String,
    pub(crate) pr: PrEvidence,
    pub(crate) local: LocalEvidence,
    pub(crate) checks: Vec<CheckEvidence>,
    pub(crate) diff_files: Vec<String>,
    pub(crate) pr_body: String,
    pub(crate) service_calls: Vec<ExternalServiceCall>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExternalServiceCall {
    pub(crate) service: String,
    pub(crate) command: String,
    pub(crate) attempts: usize,
    pub(crate) exit_status: Option<i32>,
}
