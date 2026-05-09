use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

mod checks;
mod input;
pub use checks::{
    CheckConclusion, CheckRollupEvidence, CheckRunEvidence, SupplementalValidation,
    required_supplemental_validations,
};
use input::{GhPrMetadataInput, OfflineEvidenceInput};

#[derive(Clone, Debug)]
pub struct FinalizationEvidence {
    pub repository: String,
    pub pr_number: u64,
    pub pr: PrHeadMetadata,
    pub local: LocalHeadEvidence,
    pub final_pr_head_sha: String,
    pub mergeability: Mergeability,
    pub checks: CheckRollupEvidence,
    pub supplemental_validations: Vec<SupplementalValidation>,
    pub scope_changes: Vec<ScopeChange>,
    pub preserved_patch: Option<PreservedPatchEvidence>,
    pub audit_cycles: Vec<AuditCycleEvidence>,
}

impl FinalizationEvidence {
    pub fn from_offline_json(input: &str) -> Result<Self> {
        let raw: OfflineEvidenceInput = serde_json::from_str(input)?;
        let Some(pr_state) = raw
            .state
            .map(|state| state.trim().to_string())
            .filter(|state| !state.is_empty())
        else {
            bail!("offline evidence missing required PR state field `state`");
        };
        let Some(pr_draft) = raw.draft else {
            bail!("offline evidence missing required PR draft field `draft`");
        };
        let checks = raw.checks;
        let scope_changes = raw
            .changed_files
            .into_iter()
            .map(|path| {
                let surface = ScopeSurface::from_path(&path);
                ScopeChange::new(path, surface)
            })
            .collect();
        let supplemental_validations = raw
            .validated_gates
            .into_iter()
            .map(SupplementalValidation::passed)
            .collect();
        let audit_cycles = raw
            .quality_audit_cycles
            .into_iter()
            .map(|cycle| AuditCycleEvidence {
                seek: cycle.seek,
                validate: cycle.validate,
                fix: cycle.fix,
            })
            .collect();
        Ok(Self {
            repository: raw.repository,
            pr_number: raw.pr_number,
            pr: PrHeadMetadata::with_pr_number(
                raw.pr_number,
                raw.head_ref_name,
                raw.pr_head_sha.clone(),
                &pr_state,
                pr_draft,
            ),
            local: LocalHeadEvidence {
                branch: raw.local_branch,
                head_sha: raw.local_head_sha,
                status_short_branch: String::new(),
                worktree_clean: raw.worktree_clean,
            },
            final_pr_head_sha: raw.final_pr_head_sha,
            mergeability: Mergeability {
                merge_state_status: raw.merge_state_status,
                mergeable: raw.mergeable,
            },
            checks: CheckRollupEvidence::for_head(raw.pr_head_sha, checks),
            supplemental_validations,
            scope_changes,
            preserved_patch: None,
            audit_cycles,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrHeadMetadata {
    pr_number: Option<u64>,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "headRefOid")]
    head_ref_oid: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
}

impl PrHeadMetadata {
    pub fn new(head_branch: &str, head_sha: &str, state: &str, is_draft: bool) -> Self {
        Self {
            pr_number: None,
            head_ref_name: head_branch.into(),
            head_ref_oid: head_sha.into(),
            state: state.into(),
            is_draft,
        }
    }

    pub fn with_pr_number(
        pr_number: u64,
        head_branch: String,
        head_sha: String,
        state: &str,
        is_draft: bool,
    ) -> Self {
        Self {
            pr_number: Some(pr_number),
            head_ref_name: head_branch,
            head_ref_oid: head_sha,
            state: state.into(),
            is_draft,
        }
    }

    pub fn from_gh_view_json(input: &str) -> Result<Self> {
        let raw: GhPrMetadataInput = serde_json::from_str(input)?;
        if raw.head_ref_oid.trim().is_empty() {
            bail!("gh pr view metadata did not include headRefOid");
        }
        Ok(Self {
            pr_number: raw.pr_number,
            head_ref_name: raw.head_ref_name,
            head_ref_oid: raw.head_ref_oid,
            state: raw.state,
            is_draft: raw.is_draft,
        })
    }

    pub fn pr_number(&self) -> Option<u64> {
        self.pr_number
    }

    pub fn head_branch(&self) -> &str {
        &self.head_ref_name
    }

    pub fn head_sha(&self) -> &str {
        &self.head_ref_oid
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn is_open(&self) -> bool {
        self.state == "OPEN"
    }

    pub fn is_draft(&self) -> bool {
        self.is_draft
    }
}

#[derive(Clone, Debug)]
pub struct LocalHeadEvidence {
    pub branch: String,
    pub head_sha: String,
    pub status_short_branch: String,
    pub worktree_clean: bool,
}

#[derive(Clone, Debug)]
pub struct Mergeability {
    pub merge_state_status: String,
    pub mergeable: String,
}

#[derive(Clone, Debug)]
pub struct ScopeChange {
    pub path: String,
    pub surface: ScopeSurface,
}

impl ScopeChange {
    pub fn new(path: impl Into<String>, surface: ScopeSurface) -> Self {
        Self {
            path: path.into(),
            surface,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeSurface {
    Documentation,
    ScenarioAsset,
    GeneratedGadugiAdapter,
    ReadinessGuardTest,
    Unrelated,
}

impl ScopeSurface {
    fn from_path(path: &str) -> Self {
        if path.starts_with("docs/") {
            Self::Documentation
        } else if path.contains("/scenarios/") || path.starts_with("assets/scenarios/") {
            Self::ScenarioAsset
        } else if path.contains("gadugi") {
            Self::GeneratedGadugiAdapter
        } else if path.contains("default_workflow_pr_readiness") {
            Self::ReadinessGuardTest
        } else {
            Self::Unrelated
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreservedPatchEvidence {
    pub path: String,
    pub readable: bool,
    pub error: Option<String>,
}

impl PreservedPatchEvidence {
    pub fn unreadable(path: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            readable: false,
            error: Some(error.into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuditCycleEvidence {
    pub seek: String,
    pub validate: String,
    pub fix: String,
}

impl AuditCycleEvidence {
    pub fn clean(name: impl Into<String>) -> Self {
        Self {
            seek: name.into(),
            validate: "clean".into(),
            fix: "no repository change required".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Decision {
    #[serde(rename = "MERGE_READY")]
    MergeReady,
    #[serde(rename = "NOT_MERGE_READY")]
    NotMergeReady,
    #[serde(rename = "BLOCKED")]
    Blocked,
}

#[derive(Clone, Debug)]
pub struct FinalizationDecision {
    pub decision: Decision,
    pub blockers: Vec<String>,
    pub no_op_justification: Option<String>,
}

pub fn evaluate_finalization(evidence: FinalizationEvidence) -> FinalizationDecision {
    let mut blockers = Vec::new();
    let mut blocked = false;
    if let Some(patch) = &evidence.preserved_patch
        && !patch.readable
    {
        blocked = true;
        blockers.push(format!(
            "required preserved patch {} is unreadable: {}",
            patch.path,
            patch.error.as_deref().unwrap_or("unknown error")
        ));
    }
    if !evidence.pr.is_open() {
        blockers.push(format!(
            "PR #{} state is {} instead of OPEN",
            evidence.pr_number,
            evidence.pr.state()
        ));
    }
    if evidence.pr.is_draft() {
        blockers.push(format!("PR #{} is still a draft", evidence.pr_number));
    }
    if evidence.local.head_sha != evidence.pr.head_sha() {
        blockers.push(format!(
            "local HEAD {} does not match live PR head {}",
            evidence.local.head_sha,
            evidence.pr.head_sha()
        ));
    }
    if evidence.final_pr_head_sha != evidence.pr.head_sha() {
        blockers.push(format!(
            "final PR head re-check saw {} but authoritative head is {}",
            evidence.final_pr_head_sha,
            evidence.pr.head_sha()
        ));
    }
    if !evidence.local.worktree_clean {
        blockers.push(format!(
            "worktree is dirty; status evidence: {}",
            evidence.local.status_short_branch.trim()
        ));
    }
    if evidence.mergeability.merge_state_status != "CLEAN" {
        blockers.push(format!(
            "mergeStateStatus is {} instead of CLEAN",
            evidence.mergeability.merge_state_status
        ));
    }
    if let Err(error) = evidence.checks.require_green_current_checks() {
        blockers.push(format!("checks are not green/current: {error}"));
    }
    for change in &evidence.scope_changes {
        if change.surface == ScopeSurface::Unrelated {
            blockers.push(format!(
                "unrelated scope change is outside recovery/finalization boundary: {}",
                change.path
            ));
        }
    }
    for required in required_supplemental_validations(&evidence.scope_changes, &evidence.checks) {
        if !evidence
            .supplemental_validations
            .iter()
            .any(|passed| passed.satisfies(&required))
        {
            blockers.push(format!("missing supplemental validation: {required:?}"));
        }
    }
    if !blockers.is_empty() {
        return FinalizationDecision {
            decision: if blocked {
                Decision::Blocked
            } else {
                Decision::NotMergeReady
            },
            blockers,
            no_op_justification: None,
        };
    }
    FinalizationDecision {
        decision: Decision::MergeReady,
        blockers,
        no_op_justification: Some(format!(
            "No-op justification: PR #{} at {} matches local HEAD, checks are green/current, scope is focused, and no repository edits or commits were required.",
            evidence.pr_number,
            evidence.pr.head_sha()
        )),
    }
}

#[derive(Clone, Debug)]
pub struct HandoffOptions {
    owner_free: bool,
}

impl HandoffOptions {
    pub fn owner_free() -> Self {
        Self { owner_free: true }
    }
}

pub fn render_handoff(
    evidence: &FinalizationEvidence,
    decision: &FinalizationDecision,
    options: HandoffOptions,
) -> Result<String> {
    let Some(justification) = &decision.no_op_justification else {
        bail!("cannot render no-op handoff without a no-op justification");
    };
    let audience = if options.owner_free {
        "owner-free reviewer/classroom"
    } else {
        "reviewer/classroom"
    };
    Ok(format!(
        "{justification}\n\n{audience} handoff: PR #{} ({}) is at live head {} on branch {}. Evidence supports classroom review handoff readiness only and does not claim deployed sharing, production readiness, merge completion, grading correctness, or broader feature completion.",
        evidence.pr_number,
        evidence.repository,
        evidence.pr.head_sha(),
        evidence.pr.head_branch()
    ))
}
