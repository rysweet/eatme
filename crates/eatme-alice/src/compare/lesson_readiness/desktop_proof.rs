use super::LessonTargetEvidence;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DesktopProofContract {
    pub status: String,
    pub reason_code: String,
    pub detail: String,
    pub target_role: String,
    pub artifact: Option<String>,
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
            "desktop Run window dispatch has not been verified by the modernized target".into(),
            None,
        );
    }
    if !action_passed(target, "observe-desktop-run-execution-after-toolbar-button") {
        return (
            "desktop_run_execution_unverified".into(),
            "desktop Run execution has not been verified by the modernized target".into(),
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
