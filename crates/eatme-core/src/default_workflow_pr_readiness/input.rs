use super::CheckRunEvidence;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GhPrMetadataInput {
    #[serde(rename = "number")]
    pub(crate) pr_number: Option<u64>,
    #[serde(rename = "headRefName")]
    pub(crate) head_ref_name: String,
    #[serde(rename = "headRefOid")]
    pub(crate) head_ref_oid: String,
    pub(crate) state: String,
    #[serde(rename = "isDraft")]
    pub(crate) is_draft: bool,
}

#[derive(Deserialize)]
pub(crate) struct OfflineEvidenceInput {
    pub(crate) repository: String,
    pub(crate) pr_number: u64,
    pub(crate) head_ref_name: String,
    pub(crate) pr_head_sha: String,
    pub(crate) state: Option<String>,
    #[serde(alias = "isDraft", alias = "is_draft")]
    pub(crate) draft: Option<bool>,
    pub(crate) local_branch: String,
    pub(crate) local_head_sha: String,
    pub(crate) final_pr_head_sha: String,
    pub(crate) worktree_clean: bool,
    pub(crate) merge_state_status: String,
    pub(crate) mergeable: String,
    pub(crate) checks: Vec<CheckRunEvidence>,
    pub(crate) validated_gates: Vec<String>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) quality_audit_cycles: Vec<OfflineAuditCycleInput>,
}

#[derive(Deserialize)]
pub(crate) struct OfflineAuditCycleInput {
    pub(crate) seek: String,
    pub(crate) validate: String,
    pub(crate) fix: String,
}
