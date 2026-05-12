use super::*;
use crate::launch_run_world::UiActionRunWorldProbe;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn project_save_hook_blocks_until_run_world_is_proven() {
    let root = unique_test_dir("save-hook-before-run");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("blocked"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.action_id, "save-project");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_save_hook_blocks_when_alice_side_command_is_absent() {
    let root = unique_test_dir("save-hook-absent");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(run_dir.join("procedure-edit")).unwrap();
    fs::write(
        run_dir.join("procedure-edit").join("edited-project.a3p"),
        "edited",
    )
    .unwrap();
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_save_hook_passes_only_with_saved_project_and_save_artifact() {
    let root = unique_test_dir("save-hook-passed");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    let save_evidence_dir = run_dir.join("project-save");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::write(tools.join("eatme-save-project"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved").unwrap();
    fs::write(
        save_evidence_dir.join("project-save.json"),
        r#"{"saved":true}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-save-result/v1",
            "status": "saved",
            "save_selector": DEFAULT_SAVE_SELECTOR,
            "saved_project_artifact": "saved-project.a3p",
            "save_artifact": "project-save.json"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "passed");
    assert!(probe.proves_save());
    assert!(probe.saved_project_artifact.unwrap().size_bytes > 0);
    assert!(probe.save_artifact.unwrap().size_bytes > 0);
    assert!(probe.validation_errors.is_empty());
    assert_eq!(runner.commands().len(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn proves_save_requires_passed_status_required_artifacts_and_empty_validation_errors() {
    let probe = save_project_probe_for_proof("passed");
    assert!(probe.proves_save());

    let mut failed_status = save_project_probe_for_proof("failed");
    failed_status.saved_project_artifact = Some(artifact("project-save/saved-project.a3p"));
    failed_status.save_artifact = Some(artifact("project-save/project-save.json"));
    assert!(!failed_status.proves_save());

    let mut missing_saved_project = save_project_probe_for_proof("passed");
    missing_saved_project.saved_project_artifact = None;
    assert!(!missing_saved_project.proves_save());

    let mut missing_save_artifact = save_project_probe_for_proof("passed");
    missing_save_artifact.save_artifact = None;
    assert!(!missing_save_artifact.proves_save());

    let mut validation_error = save_project_probe_for_proof("passed");
    validation_error
        .validation_errors
        .push("save artifact did not validate".into());
    assert!(!validation_error.proves_save());
}

#[test]
fn project_save_hook_fails_before_running_when_edited_project_artifact_is_missing() {
    let root = unique_test_dir("save-hook-missing-edited-project");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    fs::create_dir_all(&tools).unwrap();
    fs::write(tools.join("eatme-save-project"), "#!/bin/sh\n").unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_save());
    assert!(runner.commands().is_empty());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("procedure edit did not leave an edited project")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_save_hook_rejects_paths_outside_evidence_dir() {
    let root = unique_test_dir("save-hook-bad-path");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::write(tools.join("eatme-save-project"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-save-result/v1",
            "status": "saved",
            "save_selector": DEFAULT_SAVE_SELECTOR,
            "saved_project_artifact": "../saved-project.a3p",
            "save_artifact": "project-save.json"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_save());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("simple relative path"))
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn project_save_hook_rejects_symlink_escape_from_evidence_dir() {
    let root = unique_test_dir("save-hook-symlink-escape");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    let save_evidence_dir = run_dir.join("project-save");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::write(tools.join("eatme-save-project"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    fs::write(root.join("outside-saved-project.a3p"), "outside").unwrap();
    std::os::unix::fs::symlink(
        root.join("outside-saved-project.a3p"),
        save_evidence_dir.join("saved-project.a3p"),
    )
    .unwrap();
    fs::write(
        save_evidence_dir.join("project-save.json"),
        r#"{"saved":true}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-save-result/v1",
            "status": "saved",
            "save_selector": DEFAULT_SAVE_SELECTOR,
            "saved_project_artifact": "saved-project.a3p",
            "save_artifact": "project-save.json"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_save());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("must stay under project-save evidence dir")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

fn run_world_probe_with_status(status: &str) -> UiActionRunWorldProbe {
    UiActionRunWorldProbe {
        id: "alice-side-world-run-command-hook".into(),
        action_id: "run-world".into(),
        status: status.into(),
        detail: "run probe detail".into(),
        run_selector: "scene.myFirstMethod".into(),
        candidate_hook_path: "tools/eatme-run-world".into(),
        command: Some("tools/eatme-run-world --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        run_artifact: artifact_if_passed(status, "world-run/world-run.json"),
        runtime_or_log_evidence: artifact_if_passed(status, "world-run/runtime.log"),
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

fn save_project_probe_for_proof(status: &str) -> UiActionSaveProjectProbe {
    UiActionSaveProjectProbe {
        id: "alice-side-project-save-command-hook".into(),
        action_id: "save-project".into(),
        status: status.into(),
        detail: "save probe detail".into(),
        save_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: "tools/eatme-save-project".into(),
        command: Some("tools/eatme-save-project --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        saved_project_artifact: Some(artifact("project-save/saved-project.a3p")),
        save_artifact: Some(artifact("project-save/project-save.json")),
        validation_errors: Vec::new(),
        missing_affordance: None,
    }
}

fn artifact(path: &str) -> ArtifactInfo {
    ArtifactInfo {
        path: path.into(),
        size_bytes: 2,
        sha256: format!("{path}-sha"),
    }
}

#[test]
fn project_save_preconditions_reflect_run_world_state_and_missing_save_affordance() {
    let blocked = probe_project_save_preconditions(&run_world_probe_with_status("blocked"));
    assert_eq!(blocked.id, "project-save-precondition");
    assert_eq!(blocked.status, "blocked");
    assert!(
        blocked
            .blocking_reason
            .contains("run-world proof is required")
    );
    assert!(
        !blocked
            .preconditions
            .iter()
            .find(|p| p.id == "run-world")
            .unwrap()
            .passed
    );

    let ready = probe_project_save_preconditions(&run_world_probe_with_status("passed"));
    assert!(
        ready
            .blocking_reason
            .contains("missing deterministic-alice-project-save-affordance")
    );
    assert!(
        ready
            .preconditions
            .iter()
            .find(|p| p.id == "run-world")
            .unwrap()
            .passed
    );
    assert!(
        !ready
            .preconditions
            .iter()
            .find(|p| p.id == "deterministic-alice-project-save-affordance")
            .unwrap()
            .passed
    );
}

#[test]
fn project_save_hook_fails_on_non_zero_exit_status() {
    let (root, alice_home, run_dir, runner) = save_hook_test_scaffold("save-exit-fail");
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "hook crashed".into(),
    });
    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );
    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_save());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|e| e.contains("exited unsuccessfully")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_save_hook_fails_on_malformed_json_stdout() {
    let (root, alice_home, run_dir, runner) = save_hook_test_scaffold("save-bad-json");
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(0),
        stdout: "not valid json".into(),
        stderr: String::new(),
    });
    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );
    assert_eq!(probe.status, "failed");
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|e| e.contains("not valid save JSON")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_save_hook_fails_on_wrong_schema_version() {
    let (root, alice_home, run_dir, runner) = save_hook_test_scaffold("save-bad-schema");
    let save_evidence_dir = run_dir.join("project-save");
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved").unwrap();
    fs::write(
        save_evidence_dir.join("project-save.json"),
        r#"{"saved":true}"#,
    )
    .unwrap();
    runner.push_output(CommandOutput {
        command: "tools/eatme-save-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "wrong/v1",
            "status": "saved",
            "save_selector": DEFAULT_SAVE_SELECTOR,
            "saved_project_artifact": "saved-project.a3p",
            "save_artifact": "project-save.json"
        })
        .to_string(),
        stderr: String::new(),
    });
    let probe = probe_project_save_hook(
        &runner,
        &alice_home,
        &run_dir,
        &run_world_probe_with_status("passed"),
        ":99",
    );
    assert_eq!(probe.status, "failed");
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|e| e.contains("schema_version must be")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

fn save_hook_test_scaffold(name: &str) -> (PathBuf, PathBuf, PathBuf, FakeCommandRunner) {
    let root = unique_test_dir(name);
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(run_dir.join("procedure-edit")).unwrap();
    fs::write(alice_home.join("tools/eatme-save-project"), "#!/bin/sh\n").unwrap();
    fs::write(run_dir.join("procedure-edit/edited-project.a3p"), "edited").unwrap();
    (root, alice_home, run_dir, FakeCommandRunner::default())
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-save-project-tests")
        .join(format!("{prefix}-{nonce}"))
}
