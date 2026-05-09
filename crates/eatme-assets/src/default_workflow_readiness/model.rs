#[derive(Clone, Debug)]
pub struct ReadinessInput {
    pub pr_number: u64,
    pub head_ref_name: String,
    pub head_ref_oid: String,
    pub local_branch: String,
    pub local_head_sha: String,
    pub merge_state_status: String,
    pub mergeable: String,
    pub command_evidence: Vec<CommandEvidence>,
    pub check_runs: Vec<CheckRunEvidence>,
    pub quality_audit_cycles: Vec<QualityAuditCycle>,
    pub changed_files: Vec<String>,
    pub docs_impact: DocsImpactReview,
    pub pr_evidence: PREvidenceReview,
    pub manual_merge_attempted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandEvidence {
    pub command: String,
    pub status: CommandStatus,
    pub head_sha: String,
    pub used_timeout_wrapper: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandStatus {
    Passed,
    Failed,
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckRunEvidence {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: CheckConclusion,
    pub head_sha: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckStatus {
    Completed,
    InProgress,
    Queued,
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    Neutral,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAuditCycle {
    pub seek: String,
    pub validate: String,
    pub fix: String,
    pub clean: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocsImpactReview {
    pub mkdocs_strict_passed: bool,
    pub bounded_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PREvidenceReview {
    pub location: String,
    pub trusted_provenance: bool,
    pub head_sha: String,
    pub recorded_commands: Vec<String>,
    pub records_github_checks: bool,
    pub records_diff_scope: bool,
    pub records_docs_impact: bool,
    pub records_quality_audit: bool,
    pub records_no_manual_merge: bool,
    pub updated_during_review: bool,
    pub reconfirmed_head_sha: Option<String>,
}
