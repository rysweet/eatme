use eatme_alice::{
    AliceComparisonOptions, FIRST_LESSON_SCENARIO_ID, LaunchSmokeScenario,
    run_launch_smoke_comparison,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct DesktopFixture {
    pub(super) run_frame_present: bool,
    pub(super) vm_statement_execution_present: bool,
    pub(super) visible_desktop_screenshot_present: bool,
    pub(super) pixel_boundary_present: bool,
    pub(super) pixel_observation: PixelObservationFixture,
    pub(super) first_lesson_next_action: FirstLessonNextActionFixture,
}

#[derive(Clone, Copy)]
pub(super) enum PixelObservationFixture {
    Missing,
    Blocked,
    BlockedWithNextAction,
    Observed,
}

#[derive(Clone, Copy)]
pub(super) enum FirstLessonNextActionFixture {
    Missing,
    Blocked,
}

pub(super) fn write_manifest(fixture: DesktopFixture) -> PathBuf {
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
        if fixture.pixel_boundary_present {
            fs::write(
                evidence_dir.join("desktop-run-pixel-boundary.json"),
                r#"{"schema_version":"eatme.alice-desktop-run-pixel-boundary/v1","status":"not_observed","reason":"Run view attachment was observed, but this Alice-side signal does not inspect screenshots or pixel output."}"#,
            )
            .unwrap();
        }
        match fixture.pixel_observation {
            PixelObservationFixture::Missing => {}
            PixelObservationFixture::Blocked => {
                fs::write(
                    evidence_dir.join("desktop-run-pixel-observation.json"),
                    r#"{"schema_version":"eatme.alice-desktop-run-pixel-observation/v1","status":"blocked","source":"desktop_run_render_target_attachment","claim":"No desktop pixel was sampled.","component_state":{"graphicsEnvironmentHeadless":true,"renderTargetDisplayable":false,"renderTargetShowing":false,"renderTargetWidth":0,"renderTargetHeight":0},"blocker":{"reason":"A desktop screenshot requires a non-headless graphics environment, a showing Run render target, positive component size, and screen-capture access.","codes":["java_awt_headless","render_target_not_displayable","render_target_not_showing","render_target_has_no_positive_size"],"exceptionType":""}}"#,
                )
                .unwrap();
            }
            PixelObservationFixture::BlockedWithNextAction => {
                fs::write(
                    evidence_dir.join("desktop-run-pixel-observation.json"),
                    r#"{"schema_version":"eatme.alice-desktop-run-pixel-observation/v1","status":"blocked","source":"desktop_run_render_target_attachment","claim":"No desktop pixel was sampled.","next_action":{"summary":"rerun RabbitHole with DISPLAY backed by a visible desktop and capture desktop-run-render-target.png"},"component_state":{"graphicsEnvironmentHeadless":true,"renderTargetDisplayable":false,"renderTargetShowing":false,"renderTargetWidth":0,"renderTargetHeight":0},"blocker":{"reason":"A desktop screenshot requires a non-headless graphics environment, a showing Run render target, positive component size, and screen-capture access.","codes":["java_awt_headless","render_target_not_displayable"],"exceptionType":""}}"#,
                )
                .unwrap();
            }
            PixelObservationFixture::Observed => {
                fs::write(evidence_dir.join("desktop-run-render-target.png"), "png").unwrap();
                fs::write(
                    evidence_dir.join("desktop-run-pixel-observation.json"),
                    r#"{"schema_version":"eatme.alice-desktop-run-pixel-observation/v1","status":"observed","source":"desktop_run_render_target_attachment","claim":"A desktop screenshot of the Run render target area was captured and its center pixel was sampled.","component_state":{"graphicsEnvironmentHeadless":false,"renderTargetDisplayable":true,"renderTargetShowing":true,"renderTargetWidth":24,"renderTargetHeight":24},"screenshot":{"file":"desktop-run-render-target.png","width":24,"height":24},"captureArea":{"coordinateSystem":"screen","x":10,"y":20,"width":24,"height":24},"sample":{"coordinateSystem":"screenshot","x":12,"y":12,"argb":"0xFF336699"}}"#,
                )
                .unwrap();
            }
        }
        if matches!(
            fixture.first_lesson_next_action,
            FirstLessonNextActionFixture::Blocked
        ) {
            fs::write(
                evidence_dir.join("desktop-first-lesson-next-action.json"),
                r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","evaluated_after":"desktop-run-pixel-observation.json","candidate_actions":["desktop_save_menu_action","desktop_code_editor_or_procedure_action"],"blocker":{"reason":"The Run render attachment seam does not receive or invoke a stable desktop Save menu or code editor/procedure action target.","codes":["desktop_save_menu_action_not_bound","procedure_editor_action_not_bound","no_ui_action_invoker_at_run_render_attachment"],"details":[{"observed":"recordRenderTargetAttached receives render target, render panel, Run view, and control-panel attachment state only","required":"stable desktop Save command/menu target plus invocation result"},{"observed":"no code editor or procedure operation target is exposed at this seam","required":"stable code editor/procedure action target plus invocation result"}]},"requiresNextEvidence":["desktop Save menu readiness or invocation artifact from the menu/action owner","code editor/procedure action readiness or invocation artifact from the editor/action owner"],"doesNotClaim":["full Alice UI automation","desktop save-menu completion","code editor/procedure action completion","first-lesson completion","grading","creative assessment"]}"#,
            )
            .unwrap();
        }
    }
}

pub(super) fn overwrite_modernized_pixel_boundary(manifest_path: &Path, content: &str) {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let contract_path = PathBuf::from(
        value["targets"]["modernized"]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap(),
    );
    fs::write(
        contract_path
            .parent()
            .unwrap()
            .join("run-window-evidence")
            .join("desktop-run-pixel-boundary.json"),
        content,
    )
    .unwrap();
}

pub(super) fn overwrite_modernized_pixel_observation(manifest_path: &Path, content: &str) {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let contract_path = PathBuf::from(
        value["targets"]["modernized"]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap(),
    );
    fs::write(
        contract_path
            .parent()
            .unwrap()
            .join("run-window-evidence")
            .join("desktop-run-pixel-observation.json"),
        content,
    )
    .unwrap();
}

pub(super) fn overwrite_modernized_first_lesson_next_action(manifest_path: &Path, content: &str) {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let contract_path = PathBuf::from(
        value["targets"]["modernized"]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap(),
    );
    fs::write(
        contract_path
            .parent()
            .unwrap()
            .join("run-window-evidence")
            .join("desktop-first-lesson-next-action.json"),
        content,
    )
    .unwrap();
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

pub(super) fn assert_contains(entries: &[String], expected: &str) {
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
