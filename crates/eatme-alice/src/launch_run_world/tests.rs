use super::*;
use crate::launch_edit_procedure::UiActionEditProcedureProbe;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn run_world_hook_blocks_until_procedure_edit_is_proven() {
    let root = unique_test_dir("run-world-hook-before-edit");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_run_world_hook(
        &runner,
        &alice_home,
        &run_dir,
        &edit_procedure_probe_with_status("blocked"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.action_id, "run-world");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_world_hook_blocks_when_alice_side_command_is_absent() {
    let root = unique_test_dir("run-world-hook-absent");
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

    let probe = probe_run_world_hook(
        &runner,
        &alice_home,
        &run_dir,
        &edit_procedure_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert!(probe.missing_affordance.is_some());
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_world_hook_passes_only_with_run_artifact_and_runtime_log() {
    let root = unique_test_dir("run-world-hook-passed");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    let run_evidence_dir = run_dir.join("world-run");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::create_dir_all(&run_evidence_dir).unwrap();
    fs::write(tools.join("eatme-run-world"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    fs::write(run_evidence_dir.join("world-run.json"), r#"{"ran":true}"#).unwrap();
    fs::write(run_evidence_dir.join("runtime.log"), "executed:Comment\n").unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-run-world --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-world-run-result/v1",
            "status": "ran",
            "run_selector": DEFAULT_RUN_SELECTOR,
            "run_artifact": "world-run.json",
            "runtime_or_log_evidence": "runtime.log"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_run_world_hook(
        &runner,
        &alice_home,
        &run_dir,
        &edit_procedure_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "passed");
    assert!(probe.proves_run());
    assert!(probe.run_artifact.unwrap().size_bytes > 0);
    assert!(probe.runtime_or_log_evidence.unwrap().size_bytes > 0);
    assert!(probe.validation_errors.is_empty());
    assert_eq!(runner.commands().len(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_world_hook_rejects_paths_outside_evidence_dir() {
    let root = unique_test_dir("run-world-hook-bad-path");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::write(tools.join("eatme-run-world"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-run-world --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-world-run-result/v1",
            "status": "ran",
            "run_selector": DEFAULT_RUN_SELECTOR,
            "run_artifact": "../world-run.json",
            "runtime_or_log_evidence": "runtime.log"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_run_world_hook(
        &runner,
        &alice_home,
        &run_dir,
        &edit_procedure_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_run());
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
fn run_world_hook_rejects_symlink_escape_from_evidence_dir() {
    let root = unique_test_dir("run-world-hook-symlink-escape");
    let alice_home = root.join("alice");
    let tools = alice_home.join("tools");
    let run_dir = root.join("runs");
    let edit_evidence_dir = run_dir.join("procedure-edit");
    let run_evidence_dir = run_dir.join("world-run");
    let outside_dir = root.join("outside");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&edit_evidence_dir).unwrap();
    fs::create_dir_all(&run_evidence_dir).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    fs::write(tools.join("eatme-run-world"), "#!/bin/sh\n").unwrap();
    fs::write(edit_evidence_dir.join("edited-project.a3p"), "edited").unwrap();
    fs::write(run_evidence_dir.join("world-run.json"), r#"{"ran":true}"#).unwrap();
    fs::write(outside_dir.join("runtime.log"), "executed bunny.move\n").unwrap();
    std::os::unix::fs::symlink(
        outside_dir.join("runtime.log"),
        run_evidence_dir.join("runtime.log"),
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-run-world --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-world-run-result/v1",
            "status": "ran",
            "run_selector": DEFAULT_RUN_SELECTOR,
            "run_artifact": "world-run.json",
            "runtime_or_log_evidence": "runtime.log"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_run_world_hook(
        &runner,
        &alice_home,
        &run_dir,
        &edit_procedure_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_run());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("must stay under"))
    );
    let _ = fs::remove_dir_all(root);
}

fn edit_procedure_probe_with_status(status: &str) -> UiActionEditProcedureProbe {
    UiActionEditProcedureProbe {
        id: "alice-side-procedure-edit-command-hook".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: status.into(),
        detail: "edit probe detail".into(),
        procedure_selector: "scene.myFirstMethod".into(),
        edit_spec: "append-movement:bunny.move(FORWARD,1.0)".into(),
        candidate_hook_path: "tools/eatme-edit-procedure".into(),
        command: Some("tools/eatme-edit-procedure --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        edited_project_artifact: artifact_if_passed(status, "procedure-edit/edited-project.a3p"),
        procedure_or_code_diff: artifact_if_passed(status, "procedure-edit/procedure.diff.json"),
        validation_errors: Vec::new(),
        missing_affordance: None,
        edit_procedure_verified: false,
        proof_detail: None,
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
        .join("eatme-alice-run-world-tests")
        .join(format!("{prefix}-{nonce}"))
}
