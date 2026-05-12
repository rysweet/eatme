use super::*;
use crate::launch_object_placement::UiActionObjectPlacementProbe;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn edit_procedure_hook_blocks_until_object_placement_is_proven() {
    let root = unique_test_dir("edit-procedure-hook-before-placement");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_edit_procedure_hook(
        &runner,
        &alice_home,
        &run_dir,
        &object_placement_probe_with_status("blocked"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.action_id, "edit-procedure-or-code-block");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn edit_procedure_hook_blocks_when_alice_side_command_is_absent() {
    let root = unique_test_dir("edit-procedure-hook-absent");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(run_dir.join("object-placement")).unwrap();
    fs::write(
        run_dir.join("object-placement").join("placed-project.a3p"),
        "project",
    )
    .unwrap();
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_edit_procedure_hook(
        &runner,
        &alice_home,
        &run_dir,
        &object_placement_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn edit_procedure_hook_passes_only_with_edited_project_and_diff_proof() {
    let root = unique_test_dir("edit-procedure-hook-passed");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let object_evidence_dir = run_dir.join("object-placement");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&object_evidence_dir).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::write(tools.join("eatme-edit-procedure"), "#!/bin/sh\n").unwrap();
    fs::write(object_evidence_dir.join("placed-project.a3p"), "project").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    fs::write(
        edit_evidence_dir.join("procedure.diff.json"),
        r#"{"edited":["scene.eatmeFirstLesson"]}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-edit-procedure --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-procedure-edit-result/v1",
            "status": "edited",
            "procedure_selector": DEFAULT_PROCEDURE_SELECTOR,
            "edited_project_artifact": "edited-project.a3p",
            "procedure_or_code_diff": "procedure.diff.json"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_edit_procedure_hook(
        &runner,
        &alice_home,
        &run_dir,
        &object_placement_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "passed");
    assert!(probe.proves_edit());
    assert!(probe.edited_project_artifact.unwrap().size_bytes > 0);
    assert!(probe.procedure_or_code_diff.unwrap().size_bytes > 0);
    assert!(probe.validation_errors.is_empty());
    assert_eq!(runner.commands().len(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn edit_procedure_hook_rejects_paths_outside_evidence_dir() {
    let root = unique_test_dir("edit-procedure-hook-bad-path");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let object_evidence_dir = run_dir.join("object-placement");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&object_evidence_dir).unwrap();
    fs::write(tools.join("eatme-edit-procedure"), "#!/bin/sh\n").unwrap();
    fs::write(object_evidence_dir.join("placed-project.a3p"), "project").unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-edit-procedure --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-procedure-edit-result/v1",
            "status": "edited",
            "procedure_selector": DEFAULT_PROCEDURE_SELECTOR,
            "edited_project_artifact": "../edited-project.a3p",
            "procedure_or_code_diff": "procedure.diff.json"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_edit_procedure_hook(
        &runner,
        &alice_home,
        &run_dir,
        &object_placement_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_edit());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("simple relative path"))
    );
    let _ = fs::remove_dir_all(root);
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

fn artifact_if_passed(status: &str, path: &str) -> Option<ArtifactInfo> {
    (status == "passed").then(|| ArtifactInfo {
        path: path.into(),
        size_bytes: 2,
        sha256: format!("{path}-sha"),
    })
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-edit-procedure-tests")
        .join(format!("{prefix}-{nonce}"))
}

#[test]
fn default_procedure_selector_is_scene_eatme_first_lesson() {
    assert_eq!(
        DEFAULT_PROCEDURE_SELECTOR, "scene.eatmeFirstLesson",
        "Bug 1 (#252): EatmeEditProcedure hook requires scene.eatmeFirstLesson (no Step suffix)"
    );
}
