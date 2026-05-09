use super::MISSING_REAL_ACTION_EVIDENCE_CODE;
use super::output::LessonTargetEvidence;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginalAliceActionEvidenceStatus {
    Missing,
    Available,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct OriginalAliceActionEvidenceReport {
    pub status: OriginalAliceActionEvidenceStatus,
    pub summary: &'static str,
    pub detail: &'static str,
}

impl OriginalAliceActionEvidenceReport {
    pub const fn missing() -> Self {
        Self {
            status: OriginalAliceActionEvidenceStatus::Missing,
            summary: "Original Alice action evidence is missing.",
            detail: "Original Alice action evidence was not found in the comparison target evidence.",
        }
    }

    pub const fn available() -> Self {
        Self {
            status: OriginalAliceActionEvidenceStatus::Available,
            summary: "Original Alice action evidence is available.",
            detail: "The readiness report did not find a missing original Alice action evidence blocker.",
        }
    }
}

pub(super) fn original_alice_action_evidence(
    target_evidence: &[LessonTargetEvidence],
) -> OriginalAliceActionEvidenceReport {
    if target_evidence.iter().any(|target| {
        target
            .blockers
            .iter()
            .any(|blocker| blocker.code == MISSING_REAL_ACTION_EVIDENCE_CODE)
    }) {
        return OriginalAliceActionEvidenceReport::missing();
    }

    OriginalAliceActionEvidenceReport::available()
}

#[cfg(test)]
mod tests {
    use super::super::output::LessonTargetEvidenceBlocker;
    use super::*;

    #[test]
    fn missing_state_comes_from_blockers_even_when_action_contract_is_unreadable() {
        let target_evidence = vec![target_with_blocker(
            false,
            Some(LessonTargetEvidenceBlocker {
                code: MISSING_REAL_ACTION_EVIDENCE_CODE,
                action: "save-project".into(),
                reason:
                    "Required original Alice action evidence is missing from automation scenarios."
                        .into(),
            }),
        )];

        assert_eq!(
            original_alice_action_evidence(&target_evidence),
            OriginalAliceActionEvidenceReport::missing()
        );
    }

    #[test]
    fn available_state_requires_no_missing_original_action_evidence_blocker() {
        let target_evidence = vec![target_with_blocker(
            true,
            Some(LessonTargetEvidenceBlocker {
                code: "other_blocker",
                action: "save-project".into(),
                reason: "Other blocker.".into(),
            }),
        )];

        assert_eq!(
            original_alice_action_evidence(&target_evidence),
            OriginalAliceActionEvidenceReport::available()
        );
    }

    fn target_with_blocker(
        ui_action_contract_readable: bool,
        blocker: Option<LessonTargetEvidenceBlocker>,
    ) -> LessonTargetEvidence {
        LessonTargetEvidence {
            role: "baseline".into(),
            target_id: None,
            target_status: None,
            failure_category: None,
            launch_manifest_present: true,
            ui_action_contract_path: None,
            ui_action_contract_readable,
            desktop_run_pixel_boundary: None,
            desktop_run_pixel_observation: None,
            desktop_first_lesson_next_action: None,
            action_assertions: Vec::new(),
            required_actions: Vec::new(),
            missing_assertions: Vec::new(),
            missing_required_actions: Vec::new(),
            blockers: blocker.into_iter().collect(),
            no_go_contracts: Vec::new(),
        }
    }
}
