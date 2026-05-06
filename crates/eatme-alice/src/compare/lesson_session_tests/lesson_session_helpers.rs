use super::super::{AliceComparisonManifest, AliceComparisonOptions, run_launch_smoke_comparison};
use crate::scenario::LaunchSmokeScenario;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn write_first_lesson_manifest(root: &Path) -> AliceComparisonManifest {
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Existing Alice checkout used as the reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Modernized Alice checkout used as the comparison target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();

    run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
        run_id: "first-lesson-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
    })
    .unwrap()
}

pub(super) fn write_executable_blocked_first_lesson_manifest(
    root: &Path,
    omit_save_action: bool,
) -> PathBuf {
    let manifest = write_first_lesson_manifest(root);
    let manifest_path = PathBuf::from(&manifest.comparison_manifest_path);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    value["execute_requested"] = serde_json::json!(true);
    for role in ["baseline", "modernized"] {
        let action_contract_path = root
            .join("runs")
            .join("first-lessons-real-ui-actions")
            .join(format!("{role}-first-lesson-run"))
            .join("ui-action-contract.json");
        let run_dir = action_contract_path.parent().unwrap();
        fs::create_dir_all(run_dir).unwrap();
        if role == "modernized" {
            fs::create_dir_all(run_dir.join("screenshots")).unwrap();
            fs::write(
                run_dir.join("screenshots/run-window-after-dispatch.png"),
                "png",
            )
            .unwrap();
        }
        fs::write(
            &action_contract_path,
            serde_json::to_vec_pretty(&ui_action_contract_json(omit_save_action)).unwrap(),
        )
        .unwrap();
        value["targets"][role]["status"] = serde_json::json!("failed");
        value["targets"][role]["failure_category"] =
            serde_json::json!("ui_action_remaining_steps_unimplemented");
        value["targets"][role]["launch_manifest"] = launch_manifest_json(&action_contract_path);
    }
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&value).unwrap(),
    )
    .unwrap();
    manifest_path
}

fn launch_manifest_json(action_contract_path: &Path) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1",
        "scenario_id": "first-lessons-real-ui-actions",
        "failure_category": "ui_action_remaining_steps_unimplemented",
        "ui_action_contract": {
            "path": action_contract_path.display().to_string(),
            "size_bytes": 1,
            "sha256": "test-sha"
        },
        "assertions": {
            "real_alice_execution_evidence": {
                "passed": true,
                "detail": "real Alice process, responsive virtual display, visual evidence, and launch log were captured"
            },
            "specific_alice_window_detected": {
                "passed": true,
                "detail": "wmctrl window list contains an Alice Stage IDE window"
            },
            "activate_alice_window_ui_action": {
                "passed": true,
                "detail": "wmctrl activated Alice window 0x001"
            },
            "save_project_desktop_shortcut_dispatch": {"passed": true, "detail": "input dispatch only: xdotool sent Ctrl+S to Alice window 0x001; this does not prove saved project content"},
            "run_world_desktop_toolbar_window_observed": {"passed": true, "detail": "observed RabbitHole Run-window-created sentinel after Run toolbar click; this records Alice preparing the desktop Run frame, not world completion"},
            "run_world_desktop_execution_observed": {"passed": true, "detail": "observed RabbitHole desktop Run execution artifact with VM statement events; this proves desktop execution started, not rendering correctness or lesson completion"},
            "place_object_precondition_no_go_probe": {
                "passed": true,
                "detail": "blocked: no supported deterministic Alice object placement backend is wired"
            },
            "place_object_candidate_hook_probe": {
                "passed": true,
                "detail": "blocked: Alice checkout does not expose tools/eatme-place-object; object placement remains unproven"
            },
            "place_object_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can add/place an object yet"
            },
            "edit_procedure_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can edit a procedure or code block yet"
            },
            "run_world_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can run the world yet"
            },
            "save_project_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can save the project yet"
            },
            "ui_action_artifact_captured": {
                "passed": true,
                "detail": "ui action contract artifact exists and is non-empty"
            }
        }
    })
}

pub(super) fn ui_action_contract_json(omit_save_action: bool) -> serde_json::Value {
    let mut actions = vec![
        serde_json::json!({
            "id": "verify-specific-alice-window",
            "required_evidence": "wmctrl or xwininfo output identifies the Alice main window"
        }),
        serde_json::json!({
            "id": "activate-specific-alice-window",
            "required_evidence": "wmctrl -ia or xdotool windowfocus succeeds against the detected Alice window id"
        }),
        serde_json::json!({
            "id": "place-object",
            "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
            "decision": "no_go",
            "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
            "contract_required": {
                "candidate_backend": "/alice/tools/eatme-place-object",
                "inputs": ["open_project", "object_identifier", "evidence_dir"],
                "outputs": ["placement_artifact", "scene_or_project_diff"],
                "unsafe_until_available": true
            }
        }),
        serde_json::json!({
            "id": "edit-procedure-or-code-block",
            "required_evidence": "artifact proves a procedure or code block was edited",
            "decision": "no_go",
            "missing_affordance_id": "deterministic-alice-procedure-edit-affordance",
            "contract_required": {
                "candidate_backend": "/alice/tools/eatme-edit-procedure",
                "inputs": ["project_after_object_placement", "procedure_selector", "edit_spec", "evidence_dir"],
                "outputs": ["edited_project_artifact", "procedure_or_code_diff"],
                "unsafe_until_available": true
            }
        }),
        serde_json::json!({
            "id": "run-world",
            "required_evidence": "artifact proves the world run control was invoked",
            "decision": "no_go",
            "missing_affordance_id": "deterministic-alice-world-run-affordance",
            "contract_required": {
                "candidate_backend": "/alice/tools/eatme-run-world",
                "inputs": ["edited_project", "run_selector", "evidence_dir"],
                "outputs": ["run_artifact", "runtime_or_log_evidence"],
                "unsafe_until_available": true
            }
        }),
    ];
    if !omit_save_action {
        actions.push(serde_json::json!({
            "id": "save-project",
            "required_evidence": "saved .a3p project artifact exists and is non-empty",
            "decision": "no_go",
            "missing_affordance_id": "deterministic-alice-project-save-affordance",
            "contract_required": {
                "candidate_backend": "/alice/tools/eatme-save-project",
                "inputs": ["edited_project", "save_selector", "evidence_dir"],
                "outputs": ["saved_project_artifact", "save_artifact"],
                "unsafe_until_available": true
            }
        }));
    }
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet.",
        "preflight_evidence": {
            "specific_alice_window_detected": true,
            "visual_evidence_captured": true,
            "log_captured": true
        },
        "executed_action_probes": [{
            "id": "activate-specific-alice-window",
            "status": "passed",
            "detail": "wmctrl activated Alice window 0x001",
            "window_id": "0x001",
            "command": "wmctrl -ia 0x001",
            "exit_status": 0,
            "stdout": "",
            "stderr": ""
        }, {
            "id": "dispatch-save-project-shortcut",
            "status": "passed",
            "detail": "input dispatch only: xdotool sent Ctrl+S to Alice window 0x001; this does not prove saved project content",
            "window_id": "0x001",
            "command": "xdotool key --window 0x001 --clearmodifiers ctrl+s",
            "exit_status": 0,
            "stdout": "",
            "stderr": ""
        }],
        "action_precondition_probes": [{
            "id": "place-object-precondition",
            "action_id": "place-object",
            "status": "blocked",
            "decision": "no_go",
            "blocking_reason": "blocked: missing deterministic-alice-object-gallery-placement-affordance",
            "required_evidence": "artifact proves a named object was added to the Alice scene and placed without coordinate guessing",
            "missing_affordance": {
                "id": "deterministic-alice-object-gallery-placement-affordance",
                "kind": "backend_or_ui_affordance",
                "required_capability": "Given an open Alice starter project and a named object identifier, deterministically add that object to the scene without coordinate guessing.",
                "missing_contract": "No Alice-side command at tools/eatme-place-object, accessibility target, stable menu action, or scene-graph verification hook currently accepts a named object identifier and returns proof of placement.",
                "next_implementation": "Add one stable affordance: either the Alice-side object placement command hook defined by this contract, or a UI automation contract with a named gallery selector plus scene-graph or saved-project diff verification."
            },
            "preconditions": [
                {"id": "specific-alice-window-detected", "passed": true, "detail": "wmctrl or xwininfo output identifies the Alice main window"},
                {"id": "activate-specific-alice-window", "passed": true, "detail": "wmctrl -ia or xdotool windowfocus succeeds against the detected Alice window id"},
                {"id": "dispatch-save-project-shortcut", "passed": true, "detail": "input dispatch only: xdotool sent Ctrl+S to the detected Alice window; this does not prove saved project content"},
                {"id": "visual-evidence-captured", "passed": true, "detail": "startup screenshot or window evidence exists"},
                {"id": "log-captured", "passed": true, "detail": "Alice launch log exists and is non-empty"},
                {"id": "deterministic-alice-object-gallery-placement-affordance", "passed": false, "detail": "missing stable backend command, accessibility target, menu action, or scene-graph verification hook for named object placement"}
            ]
        }],
        "candidate_affordance_probes": [{
            "id": "alice-side-object-placement-command-hook", "action_id": "place-object",
            "status": "blocked", "detail": "blocked: Alice checkout does not expose tools/eatme-place-object; object placement remains unproven",
            "object_identifier": "alice-gallery://animals/bunny", "candidate_hook_path": "/alice/tools/eatme-place-object",
            "command": null, "exit_status": null, "stdout": "", "stderr": "",
            "placement_artifact": null, "scene_or_project_diff": null, "validation_errors": [], "missing_affordance": null
        }],
        "required_actions": actions
    })
}

pub(super) fn assert_contract_contains(entries: &[String], expected: &str) {
    assert!(
        entries.iter().any(|entry| entry.contains(expected)),
        "contract entries should contain {expected:?}: {entries:?}"
    );
}

pub(super) fn unique_test_dir(prefix: &str) -> PathBuf {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-alice-comparison-tests")
        .join(format!("{prefix}-{now_ms}"))
}
