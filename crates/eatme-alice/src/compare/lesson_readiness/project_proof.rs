use super::LessonTargetEvidence;
use super::progress::{LessonReadinessEvidenceProgressItem, progress_item};
use crate::compare::desktop_evidence::ProjectProofArtifactEvidence;

pub(super) fn save_project_proof_progress_item(
    evidence: &str,
    target: Option<&LessonTargetEvidence>,
) -> LessonReadinessEvidenceProgressItem {
    project_proof_progress_item(
        evidence,
        "Save Project proof artifact",
        target
            .and_then(|target| target.desktop_first_lesson_next_action.as_ref())
            .map(|next_action| &next_action.save_project_proof_artifact),
    )
}

pub(super) fn select_project_proof_progress_item(
    evidence: &str,
    target: Option<&LessonTargetEvidence>,
) -> LessonReadinessEvidenceProgressItem {
    project_proof_progress_item(
        evidence,
        "Select Project proof artifact",
        target
            .and_then(|target| target.desktop_first_lesson_next_action.as_ref())
            .map(|next_action| &next_action.select_project_proof_artifact),
    )
}

fn project_proof_progress_item(
    evidence: &str,
    label: &str,
    artifact: Option<&ProjectProofArtifactEvidence>,
) -> LessonReadinessEvidenceProgressItem {
    let Some(artifact) = artifact else {
        return progress_item(
            evidence,
            "missing",
            format!("{label} is missing; no next-action proof-artifact declaration was read."),
        );
    };

    progress_item(evidence, artifact.state(), artifact.detail.clone())
}
