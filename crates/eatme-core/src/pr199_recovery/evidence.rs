use std::path::PathBuf;

use super::state::CheckRollup;
use super::{RecoveryError, summarize_names, summarize_paths};

const MISSING_REAL_ACTION_EVIDENCE: &str = "missing_real_action_evidence";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuredBlocker {
    pub code: String,
    pub status: String,
    pub subject: String,
    pub reason: String,
    pub resolution: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OriginalAliceActionEvidence {
    pub status: String,
    pub synthetic_sources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceSnapshot {
    pub branch: String,
    pub changed_files: Vec<PathBuf>,
    pub head_sha: String,
    pub check_rollup: CheckRollup,
    pub qa_summary: String,
    pub blockers: Vec<StructuredBlocker>,
    pub blocker_codes: Vec<String>,
    pub default_workflow_run_id: String,
    pub original_alice_action_evidence: OriginalAliceActionEvidence,
}

impl EvidenceSnapshot {
    pub fn with_blockers(blockers: Vec<StructuredBlocker>) -> Self {
        let mut snapshot = Self::for_pr199_recovery();
        snapshot.blocker_codes = blockers
            .iter()
            .map(|blocker| blocker.code.clone())
            .collect();
        snapshot.blockers = blockers;
        snapshot.refresh_original_alice_status();
        snapshot
    }

    pub fn for_pr199_recovery() -> Self {
        Self {
            branch: String::new(),
            changed_files: Vec::new(),
            head_sha: String::new(),
            check_rollup: CheckRollup::default(),
            qa_summary: String::new(),
            blockers: Vec::new(),
            blocker_codes: Vec::new(),
            default_workflow_run_id: String::new(),
            original_alice_action_evidence: OriginalAliceActionEvidence {
                status: "available".into(),
                synthetic_sources: Vec::new(),
            },
        }
    }

    pub fn from_existing(existing: &ExistingEvidenceFile) -> Self {
        let blockers = existing
            .blocker_codes
            .iter()
            .map(|code| StructuredBlocker {
                code: code.clone(),
                status: "blocked".into(),
                subject: "original_alice_action_evidence".into(),
                reason: "Preserved from existing PR #199 merge-ready evidence.".into(),
                resolution: "Preserve as explicit blocker until real evidence is provided.".into(),
            })
            .collect();
        let mut snapshot = Self {
            branch: existing.branch.clone(),
            changed_files: vec![existing.path.clone()],
            head_sha: existing.head_sha.clone(),
            check_rollup: existing.check_rollup.clone(),
            qa_summary: existing.qa_summary.clone(),
            blockers,
            blocker_codes: existing.blocker_codes.clone(),
            default_workflow_run_id: existing.default_workflow_run_id.clone(),
            original_alice_action_evidence: OriginalAliceActionEvidence {
                status: "available".into(),
                synthetic_sources: Vec::new(),
            },
        };
        snapshot.refresh_original_alice_status();
        snapshot
    }

    pub fn with_head_sha(mut self, head_sha: impl Into<String>) -> Self {
        self.head_sha = head_sha.into();
        self
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    pub fn with_existing_blocker_code(mut self, code: impl Into<String>) -> Self {
        let code = code.into();
        if !self.blocker_codes.contains(&code) {
            self.blocker_codes.push(code.clone());
        }
        if !self.blockers.iter().any(|blocker| blocker.code == code) {
            self.blockers.push(StructuredBlocker {
                code,
                status: "blocked".into(),
                subject: "original_alice_action_evidence".into(),
                reason: "Original Alice action evidence is unavailable.".into(),
                resolution: "Preserve as explicit blocker until real evidence is provided.".into(),
            });
        }
        self.refresh_original_alice_status();
        self
    }

    pub fn with_default_workflow_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.default_workflow_run_id = run_id.into();
        self
    }

    pub fn has_blocker_code(&self, code: &str) -> bool {
        self.blocker_codes.iter().any(|existing| existing == code)
            || self.blockers.iter().any(|blocker| blocker.code == code)
    }

    fn refresh_original_alice_status(&mut self) {
        self.original_alice_action_evidence.status =
            if self.has_blocker_code(MISSING_REAL_ACTION_EVIDENCE) {
                "missing".into()
            } else {
                "available".into()
            };
        self.original_alice_action_evidence
            .synthetic_sources
            .clear();
    }
}

pub struct AliceEvidenceBlockerPreserver;

impl AliceEvidenceBlockerPreserver {
    pub fn preserve(mut snapshot: EvidenceSnapshot) -> Result<EvidenceSnapshot, RecoveryError> {
        if snapshot
            .original_alice_action_evidence
            .synthetic_sources
            .iter()
            .any(|source| !source.trim().is_empty())
        {
            return Err(RecoveryError::new(
                "synthetic_alice_evidence_forbidden",
                "PR #199 recovery must not synthesize missing Alice action evidence",
            ));
        }
        snapshot.refresh_original_alice_status();
        Ok(snapshot)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistingEvidenceFile {
    pub path: PathBuf,
    pub head_sha: String,
    pub branch: String,
    pub check_rollup: CheckRollup,
    pub qa_summary: String,
    pub blocker_codes: Vec<String>,
    pub default_workflow_run_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceUpdate {
    pub path: PathBuf,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceDelta {
    existing: ExistingEvidenceFile,
    current: EvidenceSnapshot,
    changed: bool,
}

impl EvidenceDelta {
    pub fn from_existing_and_current(
        existing: &ExistingEvidenceFile,
        current: &EvidenceSnapshot,
    ) -> Result<Self, RecoveryError> {
        if existing.path.as_os_str().is_empty() {
            return Err(RecoveryError::new(
                "evidence_path_missing",
                "existing merge-ready evidence path is required",
            ));
        }
        let changed = existing.head_sha != current.head_sha
            || existing.branch != current.branch
            || existing.check_rollup != current.check_rollup
            || existing.qa_summary != current.qa_summary
            || existing.blocker_codes != current.blocker_codes
            || existing.default_workflow_run_id != current.default_workflow_run_id;

        Ok(Self {
            existing: existing.clone(),
            current: current.clone(),
            changed,
        })
    }

    pub fn required_update(&self) -> Option<EvidenceUpdate> {
        self.changed.then(|| EvidenceUpdate {
            path: self.existing.path.clone(),
            body: self.render_update_body(),
        })
    }

    fn render_update_body(&self) -> String {
        format!(
            "# PR #199 merge-readiness evidence\n\n\
             Scope: PR #199 recovery evidence only.\n\
             Current PR branch: {}\n\
             Current changed files: {}\n\
             Current PR head: {}\n\
             Current checks: {}\n\
             Scoped QA rerun: {}\n\
             Default-workflow proof: {}\n\
             Blockers preserved: {}\n",
            self.current.branch,
            summarize_paths(&self.current.changed_files),
            self.current.head_sha,
            self.current.check_rollup.summary(),
            if self.current.qa_summary.is_empty() {
                "not recorded"
            } else {
                self.current.qa_summary.as_str()
            },
            self.current.default_workflow_run_id,
            summarize_names(&self.current.blocker_codes)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryDecision {
    Push {
        files: Vec<PathBuf>,
        message: String,
    },
    NoOp {
        justification: String,
    },
}

pub struct PushOrNoopDecisionGate;

impl PushOrNoopDecisionGate {
    pub fn decide(delta: EvidenceDelta) -> Result<RecoveryDecision, RecoveryError> {
        if let Some(update) = delta.required_update() {
            return Ok(RecoveryDecision::Push {
                files: vec![update.path],
                message: format!(
                    "PR #199 recovery evidence update preserving {} blockers",
                    summarize_names(&delta.current.blocker_codes)
                ),
            });
        }

        Ok(RecoveryDecision::NoOp {
            justification: format!(
                "No-op: PR #199 recovery required no repository modification.\n\n\
                 Current PR branch: {}\n\
                 Current changed files: {}\n\
                 Current PR head: {}\n\
                 Current checks: {}\n\
                 Default-workflow proof: {}\n\
                 Scoped QA rerun: {}\n\
                 Blockers preserved: missing_real_action_evidence remains explicit\n\
                 Scope decision: existing PR #199 merge-ready evidence already matches current branch/files/head/checks/QA/default-workflow/blocker state",
                delta.current.branch,
                summarize_paths(&delta.current.changed_files),
                delta.current.head_sha,
                delta.current.check_rollup.summary(),
                delta.current.default_workflow_run_id,
                if delta.current.qa_summary.is_empty() {
                    "not recorded"
                } else {
                    delta.current.qa_summary.as_str()
                }
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pr199_recovery::{CheckConclusion, CheckRollup, CheckRun};

    fn existing_file() -> ExistingEvidenceFile {
        ExistingEvidenceFile {
            path: PathBuf::from("evidence/pr199.md"),
            head_sha: "abc123".into(),
            branch: "feat/pr-199".into(),
            check_rollup: CheckRollup::from_runs(vec![CheckRun::completed(
                "workspace",
                CheckConclusion::Success,
            )]),
            qa_summary: "cargo test".into(),
            blocker_codes: vec![MISSING_REAL_ACTION_EVIDENCE.into()],
            default_workflow_run_id: "run-1".into(),
        }
    }

    #[test]
    fn with_existing_blocker_code_deduplicates_and_marks_missing_status() {
        let snapshot = EvidenceSnapshot::for_pr199_recovery()
            .with_existing_blocker_code(MISSING_REAL_ACTION_EVIDENCE)
            .with_existing_blocker_code(MISSING_REAL_ACTION_EVIDENCE);

        assert_eq!(
            snapshot.blocker_codes,
            vec![MISSING_REAL_ACTION_EVIDENCE.to_string()]
        );
        assert_eq!(snapshot.blockers.len(), 1);
        assert_eq!(snapshot.original_alice_action_evidence.status, "missing");
        assert!(snapshot.has_blocker_code(MISSING_REAL_ACTION_EVIDENCE));
    }

    #[test]
    fn preserve_rejects_synthetic_original_alice_sources() {
        let mut snapshot = EvidenceSnapshot::for_pr199_recovery();
        snapshot
            .original_alice_action_evidence
            .synthetic_sources
            .push("reconstructed-from-summary".into());

        let error = AliceEvidenceBlockerPreserver::preserve(snapshot).unwrap_err();

        assert_eq!(error.code(), "synthetic_alice_evidence_forbidden");
    }

    #[test]
    fn decision_gate_pushes_for_changed_snapshot_and_noops_when_matching() {
        let existing = existing_file();
        let changed_snapshot = EvidenceSnapshot::from_existing(&existing)
            .with_head_sha("def456")
            .with_branch("feat/pr-199-refresh")
            .with_default_workflow_run_id("run-2");
        let changed_delta =
            EvidenceDelta::from_existing_and_current(&existing, &changed_snapshot).unwrap();

        let push = PushOrNoopDecisionGate::decide(changed_delta).unwrap();
        match push {
            RecoveryDecision::Push { files, message } => {
                assert_eq!(files, vec![PathBuf::from("evidence/pr199.md")]);
                assert!(message.contains(MISSING_REAL_ACTION_EVIDENCE));
            }
            other => panic!("expected push decision, got {other:?}"),
        }

        let matching_delta = EvidenceDelta::from_existing_and_current(
            &existing,
            &EvidenceSnapshot::from_existing(&existing),
        )
        .unwrap();
        let noop = PushOrNoopDecisionGate::decide(matching_delta).unwrap();
        match noop {
            RecoveryDecision::NoOp { justification } => {
                assert!(justification.contains("No-op"));
                assert!(justification.contains("missing_real_action_evidence remains explicit"));
            }
            other => panic!("expected noop decision, got {other:?}"),
        }
    }
}
