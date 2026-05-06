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

fn run_world_probe_with_status(status: &str) -> UiActionRunWorldProbe {
    UiActionRunWorldProbe {
        id: "alice-side-world-run-command-hook".into(),
        action_id: "run-world".into(),
        status: status.into(),
        detail: "run probe detail".into(),
        run_selector: "scene.eatmeFirstLessonStep".into(),
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
