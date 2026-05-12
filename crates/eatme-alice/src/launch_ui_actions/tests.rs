use super::*;
use crate::launch_window_targeting::alice_window_id;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;

#[test]
fn finds_alice_window_id_from_wmctrl_output() {
    let window_list = "0x001  0 host org.alice.stageide.EntryPoint Alice 3";

    assert_eq!(alice_window_id(window_list).as_deref(), Some("0x001"));
}

#[test]
fn finds_main_alice_window_id_from_xwininfo_tree() {
    let window_list = r#"
     0x60002a "License Agreement (Part 1 of 2): Alice 3": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")  488x432+256+154  +256+154
     0x600007 "Alice 3 ": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")  1000x740+0+0  +0+0
"#;

    assert_eq!(alice_window_id(window_list).as_deref(), Some("0x600007"));
}

#[test]
fn activation_probe_runs_wmctrl_against_detected_window() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "wmctrl -ia 0x001".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });

    let probe = probe_alice_window_activation(
        &runner,
        ":99",
        "0x001  0 host org.alice.stageide.EntryPoint Alice 3",
    );

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.window_id.as_deref(), Some("0x001"));
    assert_eq!(runner.commands(), vec!["wmctrl -ia 0x001"]);
}

#[test]
fn activation_probe_blocks_without_specific_alice_window() {
    let runner = FakeCommandRunner::default();

    let probe =
        probe_alice_window_activation(&runner, ":99", "0x002  0 host firefox.Firefox Firefox");

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
}

#[test]
fn place_object_precondition_probe_records_no_go_after_window_activation() {
    let activation_probe = UiActionProbe {
        id: "activate-specific-alice-window".into(),
        status: "passed".into(),
        detail: "wmctrl activated Alice window 0x001".into(),
        window_id: Some("0x001".into()),
        command: Some("wmctrl -ia 0x001".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    };

    let probe = probe_place_object_preconditions(true, true, true, Some(&activation_probe));

    assert_eq!(probe.id, "place-object-precondition");
    assert_eq!(probe.action_id, "place-object");
    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.decision, "no_go");
    assert_eq!(
        probe.missing_affordance.id,
        "deterministic-alice-object-gallery-placement-affordance"
    );
    assert!(
        probe
            .missing_affordance
            .required_capability
            .contains("named object identifier")
    );
    assert!(
        probe
            .missing_affordance
            .next_implementation
            .contains("named gallery selector")
    );
    assert!(probe.preconditions.iter().any(|precondition| {
        precondition.id == "deterministic-alice-object-gallery-placement-affordance"
            && !precondition.passed
    }));
}

#[test]
fn ui_action_failure_category_advances_after_object_placement_proof() {
    let placed = object_placement_probe_with_status("passed");
    let blocked = object_placement_probe_with_status("blocked");

    assert_eq!(
        ui_action_failure_category(&placed),
        "ui_action_remaining_steps_unimplemented"
    );
    assert_eq!(
        ui_action_failure_category(&blocked),
        "ui_action_automation_unimplemented"
    );
}

#[test]
fn edit_procedure_precondition_probe_records_no_go_after_object_placement() {
    let placed = object_placement_probe_with_status("passed");

    let probe = probe_edit_procedure_preconditions(&placed);

    assert_eq!(probe.id, "edit-procedure-precondition");
    assert_eq!(probe.action_id, "edit-procedure-or-code-block");
    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.decision, "no_go");
    assert_eq!(
        probe.missing_affordance.id,
        "deterministic-alice-procedure-edit-affordance"
    );
    assert!(
        probe
            .missing_affordance
            .missing_contract
            .contains("tools/eatme-edit-procedure")
    );
    assert!(
        probe
            .preconditions
            .iter()
            .any(|precondition| { precondition.id == "place-object" && precondition.passed })
    );
    assert!(probe.preconditions.iter().any(|precondition| {
        precondition.id == "deterministic-alice-procedure-edit-affordance" && !precondition.passed
    }));
}

#[test]
fn run_world_precondition_probe_records_no_go_after_procedure_edit() {
    let edited = edit_procedure_probe_with_status("passed");

    let probe = probe_run_world_preconditions(&edited);

    assert_eq!(probe.id, "run-world-precondition");
    assert_eq!(probe.action_id, "run-world");
    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.decision, "no_go");
    assert_eq!(
        probe.missing_affordance.id,
        "deterministic-alice-world-run-affordance"
    );
    assert!(
        probe
            .missing_affordance
            .missing_contract
            .contains("tools/eatme-run-world")
    );
    assert!(probe.preconditions.iter().any(|precondition| {
        precondition.id == "edit-procedure-or-code-block" && precondition.passed
    }));
    assert!(probe.preconditions.iter().any(|precondition| {
        precondition.id == "deterministic-alice-world-run-affordance" && !precondition.passed
    }));
}

fn object_placement_probe_with_status(status: &str) -> UiActionObjectPlacementProbe {
    UiActionObjectPlacementProbe {
        id: "alice-side-object-placement-command-hook".into(),
        action_id: "place-object".into(),
        status: status.into(),
        detail: "probe detail".into(),
        object_identifier: "alice-gallery://animals/bunny".into(),
        candidate_hook_path: "tools/eatme-place-object".into(),
        command: Some("tools/eatme-place-object --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        placement_artifact: artifact_if_passed(status, "object-placement/placement.json"),
        scene_or_project_diff: artifact_if_passed(status, "object-placement/scene.diff.json"),
        validation_errors: Vec::new(),
        missing_affordance: None,
    }
}

fn edit_procedure_probe_with_status(status: &str) -> UiActionEditProcedureProbe {
    UiActionEditProcedureProbe {
        id: "alice-side-procedure-edit-command-hook".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: status.into(),
        detail: "edit probe detail".into(),
        procedure_selector: "scene.eatmeFirstLesson".into(),
        edit_spec: "append-comment:eatme first lesson edit proof".into(),
        candidate_hook_path: "tools/eatme-edit-procedure".into(),
        command: Some("tools/eatme-edit-procedure --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        edited_project_artifact: artifact_if_passed(status, "procedure-edit/edited-project.a3p"),
        procedure_or_code_diff: artifact_if_passed(status, "procedure-edit/procedure.diff.json"),
        validation_errors: Vec::new(),
        missing_affordance: None,
    }
}

fn artifact_if_passed(status: &str, path: &str) -> Option<ArtifactInfo> {
    (status == "passed").then(|| ArtifactInfo {
        path: path.into(),
        size_bytes: 2,
        sha256: format!("{path}-sha"),
    })
}

#[test]
fn preflight_blockers_include_post_focus_screenshot_captured() {
    let activation_probe = UiActionProbe {
        id: "activate-specific-alice-window".into(),
        status: "blocked".into(),
        detail: "blocked: no Alice window".into(),
        window_id: None,
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
    };
    let place_object_probe =
        probe_place_object_preconditions(false, false, false, Some(&activation_probe));
    let mut assertions = std::collections::BTreeMap::new();
    record_preflight_ui_action_blockers(&mut assertions, &place_object_probe);

    let post_focus = assertions
        .get("post_focus_screenshot_captured")
        .expect("preflight blockers must include post_focus_screenshot_captured assertion");
    assert!(
        !post_focus.passed,
        "post_focus_screenshot_captured must be failed in preflight blockers"
    );
    assert!(
        post_focus.detail.contains("preflight blocked"),
        "detail should explain preflight blocking: {}",
        post_focus.detail
    );
}
