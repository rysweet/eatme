use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct LessonActionAssertionEvidence {
    pub assertion_id: String,
    pub action_id: String,
    pub passed: bool,
    pub detail: String,
}

pub(super) fn action_assertions(
    launch_manifest: &serde_json::Value,
) -> Vec<LessonActionAssertionEvidence> {
    #[rustfmt::skip]
    let action_ids = [
        ("specific_alice_window_detected", "verify-specific-alice-window"),
        ("activate_alice_window_ui_action", "activate-specific-alice-window"),
        ("save_project_desktop_shortcut_dispatch", "dispatch-save-project-shortcut"),
        ("run_world_desktop_shortcut_dispatch", "dispatch-run-world-shortcut"),
        ("run_world_desktop_window_observed", "observe-run-window-after-shortcut"),
        ("run_world_desktop_toolbar_dispatch", "dispatch-run-toolbar-button"),
        ("run_world_desktop_toolbar_window_observed", "observe-run-window-after-toolbar-button"),
        ("run_world_desktop_execution_observed", "observe-desktop-run-execution-after-toolbar-button"),
        ("place_object_ui_action", "place-object"),
        ("edit_procedure_ui_action", "edit-procedure-or-code-block"),
        ("run_world_ui_action", "run-world"),
        ("save_project_ui_action", "save-project"),
    ];
    action_ids
        .into_iter()
        .filter_map(|(assertion_id, action_id)| {
            let assertion = launch_manifest
                .get("assertions")
                .and_then(|assertions| assertions.get(assertion_id))?;
            Some(LessonActionAssertionEvidence {
                assertion_id: assertion_id.into(),
                action_id: action_id.into(),
                passed: assertion
                    .get("passed")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                detail: assertion
                    .get("detail")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .into(),
            })
        })
        .collect()
}

pub(super) fn missing_launch_assertions(launch_manifest: &serde_json::Value) -> Vec<String> {
    let assertions = launch_manifest
        .get("assertions")
        .and_then(serde_json::Value::as_object);
    super::REQUIRED_FIRST_LESSON_ASSERTIONS
        .iter()
        .filter(|assertion| {
            assertions
                .map(|entries| !entries.contains_key(**assertion))
                .unwrap_or(true)
        })
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn require_passed_assertion(
    issues: &mut Vec<String>,
    role: &str,
    launch_manifest: &serde_json::Value,
    assertion: &str,
) {
    let passed = launch_manifest
        .get("assertions")
        .and_then(|assertions| assertions.get(assertion))
        .and_then(|entry| entry.get("passed"))
        .and_then(serde_json::Value::as_bool);
    if passed != Some(true) {
        issues.push(format!(
            "{role} launch_manifest assertion {assertion:?} must pass before first-lesson readiness is evidence-ready"
        ));
    }
}

pub(super) fn assertion_passed(launch_manifest: &serde_json::Value, assertion: &str) -> bool {
    launch_manifest
        .get("assertions")
        .and_then(|assertions| assertions.get(assertion))
        .and_then(|entry| entry.get("passed"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}
