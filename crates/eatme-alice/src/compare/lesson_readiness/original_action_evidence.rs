use super::output::LessonTargetEvidence;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginalAliceActionEvidenceStatus {
    Missing,
    Available,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OriginalAliceActionEvidenceReport {
    pub status: OriginalAliceActionEvidenceStatus,
    pub summary: String,
    pub detail: String,
}

pub(super) fn original_alice_action_evidence(
    target_evidence: &[LessonTargetEvidence],
) -> OriginalAliceActionEvidenceReport {
    if target_evidence.iter().any(|target| {
        target.ui_action_contract_readable
            && target
                .blockers
                .iter()
                .any(|blocker| blocker.code == "missing_real_action_evidence")
    }) {
        return OriginalAliceActionEvidenceReport {
            status: OriginalAliceActionEvidenceStatus::Missing,
            summary: "Original Alice action evidence is missing.".into(),
            detail:
                "Original Alice action evidence was not found in the comparison target evidence."
                    .into(),
        };
    }

    OriginalAliceActionEvidenceReport {
        status: OriginalAliceActionEvidenceStatus::Available,
        summary: "Original Alice action evidence is available.".into(),
        detail:
            "The readiness report did not find a missing original Alice action evidence blocker."
                .into(),
    }
}
