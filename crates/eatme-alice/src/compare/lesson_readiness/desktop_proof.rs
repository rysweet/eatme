use super::LessonTargetEvidence;
use crate::compare::desktop_evidence::{
    DesktopFirstLessonNextActionEvidence, DesktopRunPixelBoundaryEvidence,
    DesktopRunPixelObservationEvidence, FirstLessonEvidenceBoundary,
    check_first_lesson_next_action_evidence, check_pixel_boundary_evidence,
    check_pixel_observation_evidence, check_visible_desktop_evidence, comparison_evidence_root,
    first_lesson_evidence_boundaries,
};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub struct DesktopProofContract {
    pub status: String,
    pub reason_code: String,
    pub detail: String,
    pub target_role: String,
    pub artifact: Option<String>,
}

pub(super) struct DesktopProofEvidence {
    pub run_pixel_boundary: DesktopRunPixelBoundaryEvidence,
    pub run_pixel_observation: DesktopRunPixelObservationEvidence,
    pub first_lesson_next_action: DesktopFirstLessonNextActionEvidence,
}

pub(super) fn inspect_desktop_proof_evidence(
    manifest_path: &Path,
    ui_action_contract_path: &Path,
    issues: &mut Vec<String>,
) -> DesktopProofEvidence {
    let evidence_root = comparison_evidence_root(manifest_path);

    let run_pixel_boundary = check_pixel_boundary_evidence(&evidence_root, ui_action_contract_path);
    issues.extend(run_pixel_boundary.issue_when_missing_or_invalid());

    let run_pixel_observation =
        check_pixel_observation_evidence(&evidence_root, ui_action_contract_path);
    issues.extend(run_pixel_observation.issue_when_missing_or_invalid());

    let first_lesson_next_action =
        check_first_lesson_next_action_evidence(&evidence_root, ui_action_contract_path);
    issues.extend(first_lesson_next_action.issue_when_invalid());
    issues.extend(first_lesson_next_action.boundary_issues());
    issues.extend(first_lesson_next_action.proof_artifact_issues());

    issues.extend(
        check_visible_desktop_evidence(&evidence_root, ui_action_contract_path)
            .issue_when_missing(),
    );

    DesktopProofEvidence {
        run_pixel_boundary,
        run_pixel_observation,
        first_lesson_next_action,
    }
}

pub(super) fn desktop_proof_contract(
    execute_requested: Option<bool>,
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> DesktopProofContract {
    if execute_requested != Some(true) {
        return DesktopProofContract {
            status: "skipped".into(),
            reason_code: "execute_not_requested".into(),
            detail: "execution was not requested; rerun with --execute on a machine with Alice desktop access to collect real desktop proof".into(),
            target_role: "modernized".into(),
            artifact: None,
        };
    }

    let Some(modernized) = target_evidence
        .iter()
        .find(|target| target.role == "modernized")
    else {
        return DesktopProofContract {
            status: "skipped".into(),
            reason_code: "modernized_target_missing".into(),
            detail: "modernized target evidence is missing; no desktop proof can be evaluated"
                .into(),
            target_role: "modernized".into(),
            artifact: None,
        };
    };

    if !modernized.launch_manifest_present {
        let reason_code = modernized
            .failure_category
            .clone()
            .unwrap_or_else(|| "target_not_launched".into());
        return DesktopProofContract {
            status: "unsupported_environment".into(),
            reason_code: reason_code.clone(),
            detail: format!(
                "modernized target did not launch desktop Alice proof collection ({reason_code})"
            ),
            target_role: "modernized".into(),
            artifact: None,
        };
    }

    if desktop_proof_verified(modernized, issues) {
        return DesktopProofContract {
            status: "verified".into(),
            reason_code: "desktop_pixel_observation_verified".into(),
            detail: "modernized desktop proof has Run-window dispatch, desktop execution, visible screenshot, and observed pixel evidence; this does not prove full lesson automation".into(),
            target_role: "modernized".into(),
            artifact: modernized
                .desktop_run_pixel_observation
                .as_ref()
                .and_then(|evidence| evidence.artifact.clone()),
        };
    }

    let (reason_code, detail, artifact) = desktop_proof_gap(modernized, issues);
    DesktopProofContract {
        status: "launched_but_unverified".into(),
        reason_code,
        detail,
        target_role: "modernized".into(),
        artifact,
    }
}

pub(super) fn readiness_evidence_boundaries(
    manifest_path: &Path,
    target_evidence: &[LessonTargetEvidence],
    issues: &mut Vec<String>,
) -> Vec<FirstLessonEvidenceBoundary> {
    if let Some(boundaries) = target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .and_then(|target| target.desktop_first_lesson_next_action.as_ref())
        .map(|next_action| next_action.evidence_boundaries.clone())
    {
        return boundaries;
    }

    let evidence_root = comparison_evidence_root(manifest_path);
    let canonical_root = match evidence_root.canonicalize() {
        Ok(canonical_root) => canonical_root,
        Err(error) => {
            issues.push(format!(
                "comparison evidence root could not be canonicalized at {}: {error}",
                evidence_root.display()
            ));
            evidence_root.clone()
        }
    };
    first_lesson_evidence_boundaries(&serde_json::Value::Null, &canonical_root, &evidence_root)
}

fn desktop_proof_verified(target: &LessonTargetEvidence, issues: &[String]) -> bool {
    !visible_desktop_evidence_missing(issues)
        && action_passed(target, "observe-run-window-after-toolbar-button")
        && action_passed(target, "observe-desktop-run-execution-after-toolbar-button")
        && target
            .desktop_run_pixel_observation
            .as_ref()
            .is_some_and(|evidence| evidence.status == "observed")
}

fn desktop_proof_gap(
    target: &LessonTargetEvidence,
    issues: &[String],
) -> (String, String, Option<String>) {
    if !action_passed(target, "observe-run-window-after-toolbar-button") {
        return (
            "desktop_run_window_unverified".into(),
            "desktop Run window dispatch lacks modernized-target proof".into(),
            None,
        );
    }
    if !action_passed(target, "observe-desktop-run-execution-after-toolbar-button") {
        return (
            "desktop_run_execution_unverified".into(),
            "desktop Run execution lacks modernized-target proof".into(),
            None,
        );
    }
    if visible_desktop_evidence_missing(issues) {
        return (
            "visible_desktop_evidence_missing".into(),
            "visible desktop rendering evidence is missing after Run-frame and VM statement execution".into(),
            None,
        );
    }
    if let Some(observation) = &target.desktop_run_pixel_observation {
        if observation.status != "observed" {
            let reason_code = format!("desktop_pixel_observation_{}", observation.status);
            return (
                reason_code,
                format!(
                    "desktop Run pixel-observation evidence is {}: {}",
                    observation.status, observation.detail
                ),
                observation.artifact.clone(),
            );
        }
    } else {
        return (
            "desktop_pixel_observation_missing".into(),
            "desktop Run pixel-observation evidence was not read".into(),
            None,
        );
    }
    (
        "desktop_evidence_unverified".into(),
        "desktop evidence was launched but does not meet the verified desktop proof contract"
            .into(),
        None,
    )
}

fn visible_desktop_evidence_missing(issues: &[String]) -> bool {
    issues
        .iter()
        .any(|issue| issue.contains("missing visible desktop rendering evidence after Run-frame"))
}

fn action_passed(target: &LessonTargetEvidence, action_id: &str) -> bool {
    target
        .action_assertions
        .iter()
        .any(|action| action.action_id == action_id && action.passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_comparison_evidence_root_is_a_readiness_issue() {
        let manifest_path = std::env::temp_dir()
            .join(format!(
                "eatme-missing-evidence-root-{}",
                std::process::id()
            ))
            .join("comparisons")
            .join("run")
            .join("comparison-manifest.json");
        let mut issues = Vec::new();

        let boundaries = readiness_evidence_boundaries(&manifest_path, &[], &mut issues);

        assert!(!boundaries.is_empty());
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("comparison evidence root could not be canonicalized")),
            "missing comparison evidence root must fail closed as a readiness issue: {issues:?}"
        );
    }
}
