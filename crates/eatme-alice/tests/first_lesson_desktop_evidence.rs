use eatme_alice::{
    AliceComparisonOptions, FIRST_LESSON_SCENARIO_ID, LaunchSmokeScenario,
    check_lesson_session_readiness, run_launch_smoke_comparison,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MISSING_DESKTOP_PROOF: &str = "missing visible desktop rendering evidence after Run-frame and VM statement execution; expected screenshots/run-window-after-dispatch.png under the comparison evidence root";

#[test]
fn readiness_passes_with_visible_run_window_screenshot() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
}

#[test]
fn vm_execution_sentinel_alone_is_not_visible_desktop_proof() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: false,
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(&report.issues, MISSING_DESKTOP_PROOF);
}

#[test]
fn run_frame_prerequisite_is_preserved_when_screenshot_exists() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: false,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"run_world_desktop_toolbar_window_observed\" must pass before first-lesson readiness is evidence-ready",
    );
}

#[test]
fn vm_statement_prerequisite_is_preserved_when_screenshot_exists() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: false,
        visible_desktop_screenshot_present: true,
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"run_world_desktop_execution_observed\" must pass before first-lesson readiness is evidence-ready",
    );
}

struct DesktopFixture {
    run_frame_present: bool,
    vm_statement_execution_present: bool,
    visible_desktop_screenshot_present: bool,
}

fn write_manifest(fixture: DesktopFixture) -> PathBuf {
    let root = unique_test_dir();
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: RabbitHole target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();
    let manifest = run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new(FIRST_LESSON_SCENARIO_ID),
        run_id: "first-lesson-desktop-evidence".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
    })
    .unwrap();
    let manifest_path = PathBuf::from(&manifest.comparison_manifest_path);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    value["execute_requested"] = serde_json::json!(true);
    for role in ["baseline", "modernized"] {
        let run_dir = root
            .join("runs")
            .join(FIRST_LESSON_SCENARIO_ID)
            .join(format!("{role}-first-lesson-desktop-evidence"));
        fs::create_dir_all(&run_dir).unwrap();
        let contract_path = run_dir.join("ui-action-contract.json");
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&ui_action_contract_json()).unwrap(),
        )
        .unwrap();
        if role == "modernized" {
            write_modernized_desktop_artifacts(&run_dir, &fixture);
        }
        value["targets"][role]["status"] = serde_json::json!("failed");
        value["targets"][role]["failure_category"] =
            serde_json::json!("ui_action_remaining_steps_unimplemented");
        value["targets"][role]["launch_manifest"] =
            launch_manifest_json(&contract_path, role, &fixture);
    }
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&value).unwrap(),
    )
    .unwrap();
    manifest_path
}

fn write_modernized_desktop_artifacts(run_dir: &Path, fixture: &DesktopFixture) {
    if fixture.visible_desktop_screenshot_present {
        let screenshot_dir = run_dir.join("screenshots");
        fs::create_dir_all(&screenshot_dir).unwrap();
        fs::write(screenshot_dir.join("run-window-after-dispatch.png"), "png").unwrap();
    }
    if fixture.vm_statement_execution_present {
        let evidence_dir = run_dir.join("run-window-evidence");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(
            evidence_dir.join("desktop-run-runtime.log"),
            "executing:Comment\n",
        )
        .unwrap();
        fs::write(
            evidence_dir.join("desktop-run-execution.json"),
            r#"{"schema_version":"eatme.alice-desktop-run-execution/v1","status":"statement_execution_observed","active_scene_invoke_started":true,"executing_statement_count":1,"runtime_log":"desktop-run-runtime.log"}"#,
        )
        .unwrap();
    }
}

fn launch_manifest_json(
    contract_path: &Path,
    role: &str,
    fixture: &DesktopFixture,
) -> serde_json::Value {
    let mut assertions = serde_json::json!({
        "real_alice_execution_evidence": {"passed": true, "detail": "real Alice process, responsive virtual display, visual evidence, and launch log were captured"},
        "specific_alice_window_detected": {"passed": true, "detail": "wmctrl window list contains an Alice Stage IDE window"},
        "activate_alice_window_ui_action": {"passed": true, "detail": "wmctrl activated Alice window 0x001"},
        "save_project_desktop_shortcut_dispatch": {"passed": true, "detail": "input dispatch only: xdotool sent Ctrl+S to Alice window 0x001; this does not prove saved project content"},
        "place_object_candidate_hook_probe": {"passed": true, "detail": "blocked: Alice checkout does not expose tools/eatme-place-object; object placement remains unproven"},
        "place_object_precondition_no_go_probe": {"passed": true, "detail": "blocked: no supported deterministic Alice object placement backend is wired"},
        "place_object_ui_action": {"passed": false, "detail": "blocked: no supported Alice desktop automation can add/place an object yet"},
        "edit_procedure_ui_action": {"passed": false, "detail": "blocked: no supported Alice desktop automation can edit a procedure or code block yet"},
        "run_world_ui_action": {"passed": false, "detail": "blocked: no supported Alice desktop automation can run the world yet"},
        "save_project_ui_action": {"passed": false, "detail": "blocked: no supported Alice desktop automation can save the project yet"},
        "ui_action_artifact_captured": {"passed": true, "detail": "ui action contract artifact exists and is non-empty"}
    });
    if role == "modernized" {
        assertions["run_world_desktop_toolbar_window_observed"] = serde_json::json!({
            "passed": fixture.run_frame_present,
            "detail": "observed RabbitHole Run-window-created sentinel after Run toolbar click; this records Alice preparing the desktop Run frame, not world completion"
        });
        assertions["run_world_desktop_execution_observed"] = serde_json::json!({
            "passed": fixture.vm_statement_execution_present,
            "detail": "observed RabbitHole desktop Run execution artifact with VM statement events; this proves desktop execution started, not rendering correctness or lesson completion"
        });
    }
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1",
        "scenario_id": FIRST_LESSON_SCENARIO_ID,
        "failure_category": "ui_action_remaining_steps_unimplemented",
        "ui_action_contract": {"path": contract_path.display().to_string(), "size_bytes": 1, "sha256": "test-sha"},
        "assertions": assertions
    })
}

fn ui_action_contract_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "The harness can activate a detected Alice window, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet.",
        "preflight_evidence": {"specific_alice_window_detected": true, "visual_evidence_captured": true, "log_captured": true},
        "executed_action_probes": [
            {"id": "activate-specific-alice-window", "status": "passed", "detail": "wmctrl activated Alice window 0x001", "window_id": "0x001", "command": "wmctrl -ia 0x001", "exit_status": 0, "stdout": "", "stderr": ""},
            {"id": "dispatch-save-project-shortcut", "status": "passed", "detail": "input dispatch only: xdotool sent Ctrl+S to Alice window 0x001; this does not prove saved project content", "window_id": "0x001", "command": "xdotool key --window 0x001 --clearmodifiers ctrl+s", "exit_status": 0, "stdout": "", "stderr": ""}
        ],
        "action_precondition_probes": [{
            "id": "place-object-precondition",
            "action_id": "place-object",
            "status": "blocked",
            "decision": "no_go",
            "blocking_reason": "blocked: missing deterministic-alice-object-gallery-placement-affordance",
            "missing_affordance": {
                "id": "deterministic-alice-object-gallery-placement-affordance",
                "kind": "backend_or_ui_affordance",
                "required_capability": "Given an open Alice starter project and a named object identifier, deterministically add that object to the scene without coordinate guessing.",
                "missing_contract": "No Alice-side command at tools/eatme-place-object, accessibility target, stable menu action, or scene-graph verification hook currently accepts a named object identifier and returns proof of placement.",
                "next_implementation": "Add one stable affordance: either the Alice-side object placement command hook defined by this contract, or a UI automation contract with a named gallery selector plus scene-graph or saved-project diff verification."
            },
            "preconditions": [{"id": "deterministic-alice-object-gallery-placement-affordance", "passed": false}]
        }],
        "candidate_affordance_probes": [{
            "id": "alice-side-object-placement-command-hook",
            "action_id": "place-object",
            "status": "blocked",
            "object_identifier": "alice-gallery://animals/bunny",
            "candidate_hook_path": "/alice/tools/eatme-place-object"
        }],
        "required_actions": [
            {"id": "verify-specific-alice-window"},
            {"id": "activate-specific-alice-window"},
            {
                "id": "place-object",
                "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
                "decision": "no_go",
                "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
                "contract_required": {"unsafe_until_available": true}
            },
            {
                "id": "edit-procedure-or-code-block",
                "required_evidence": "artifact proves a procedure or code block was edited",
                "decision": "no_go",
                "missing_affordance_id": "deterministic-alice-procedure-edit-affordance",
                "contract_required": {"unsafe_until_available": true}
            },
            {
                "id": "run-world",
                "required_evidence": "artifact proves the world run control was invoked",
                "decision": "no_go",
                "missing_affordance_id": "deterministic-alice-world-run-affordance",
                "contract_required": {"unsafe_until_available": true}
            },
            {
                "id": "save-project",
                "required_evidence": "saved .a3p project artifact exists and is non-empty",
                "decision": "no_go",
                "missing_affordance_id": "deterministic-alice-project-save-affordance",
                "contract_required": {"unsafe_until_available": true}
            }
        ]
    })
}

fn assert_contains(entries: &[String], expected: &str) {
    assert!(
        entries.iter().any(|entry| entry.contains(expected)),
        "entries should contain {expected:?}: {entries:?}"
    );
}

fn unique_test_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("eatme-first-lesson-desktop-evidence-{nonce}"))
}
