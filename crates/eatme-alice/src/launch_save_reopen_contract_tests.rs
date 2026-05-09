use crate::launch_reopen_project::{
    DEFAULT_PROJECT_REOPEN_HOOK, UiActionReopenProjectProbe, probe_project_reopen_hook,
    probe_project_reopen_preconditions,
};
use crate::launch_save_project::UiActionSaveProjectProbe;
use crate::launch_ui_actions::UiActionMissingAffordance;
use eatme_core::{ArtifactInfo, CommandOutput};
use eatme_test_support::FakeCommandRunner;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn reopen_probe_blocks_until_save_proof_exists() {
    let root = unique_test_dir("reopen-before-save");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    fs::create_dir_all(&alice_home).unwrap();
    let runner = FakeCommandRunner::default();

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("blocked"),
        ":99",
    );

    assert_eq!(probe.status, "blocked");
    assert_eq!(probe.action_id, "reopen-project");
    assert!(
        probe
            .missing_affordance
            .as_ref()
            .expect("blocked reopen must name the missing affordance")
            .required_capability
            .contains("reopen the saved .a3p")
    );
    assert!(runner.commands().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reopen_probe_passes_only_with_saved_artifact_reopened_and_state_verified() {
    let root = unique_test_dir("reopen-passed");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    let save_evidence_dir = run_dir.join("project-save");
    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::create_dir_all(&reopen_evidence_dir).unwrap();
    fs::write(alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK), "#!/bin/sh\n").unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved project").unwrap();
    fs::write(
        save_evidence_dir.join("project-save.json"),
        r#"{"saved":true}"#,
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-project.a3p"),
        "reopened project",
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("project-reopen.json"),
        r#"{"reopened":true}"#,
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-state.json"),
        r#"{"world":"expected learner-world state"}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-reopen-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-reopen-result/v1",
            "status": "reopened",
            "source_saved_project_artifact": "project-save/saved-project.a3p",
            "reopen_selector": "scene.eatmeFirstLessonStep",
            "reopened_project_artifact": "reopened-project.a3p",
            "reopen_artifact": "project-reopen.json",
            "reopened_state_artifact": "reopened-state.json",
            "state_verification": "passed"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("passed"),
        ":99",
    );

    assert_reopen_passed(&probe);
    assert_eq!(
        probe.source_saved_project_artifact,
        "project-save/saved-project.a3p"
    );
    assert!(probe.reopened_project_artifact.unwrap().size_bytes > 0);
    assert!(probe.reopen_artifact.unwrap().size_bytes > 0);
    assert!(probe.reopened_state_artifact.unwrap().size_bytes > 0);
    assert_eq!(runner.commands().len(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reopen_preconditions_require_reopening_saved_artifact_not_bundled_starter_project() {
    let no_go = probe_project_reopen_preconditions(&save_probe_with_status("passed"));

    assert_eq!(no_go.id, "project-reopen-precondition");
    assert_eq!(no_go.action_id, "reopen-project");
    assert_eq!(no_go.status, "blocked");
    assert_eq!(no_go.decision, "no_go");
    assert_contains_all(
        "reopen no-go required evidence",
        &no_go.required_evidence,
        &[
            "saved .a3p artifact",
            "reopened in a new or reset Alice session",
            "reopened state",
            "not the original bundled starter project",
        ],
    );
    assert_eq!(
        no_go.missing_affordance.id,
        "deterministic-alice-project-reopen-affordance"
    );
}

#[test]
fn reopen_probe_rejects_original_starter_project_as_source_artifact() {
    let root = unique_test_dir("reopen-original-starter");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(&reopen_evidence_dir).unwrap();
    fs::write(alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK), "#!/bin/sh\n").unwrap();
    fs::write(
        reopen_evidence_dir.join("project-reopen.json"),
        r#"{"reopened":true}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-reopen-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-reopen-result/v1",
            "status": "reopened",
            "source_saved_project_artifact": "assets/alice/starter-projects/africa.a3p",
            "reopen_selector": "scene.eatmeFirstLessonStep",
            "reopened_project_artifact": "reopened-project.a3p",
            "reopen_artifact": "project-reopen.json",
            "reopened_state_artifact": "reopened-state.json",
            "state_verification": "passed"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_reopen());
    assert!(
        probe.validation_errors.iter().any(|error: &String| error
            .contains("must reopen the saved artifact, not the bundled starter project")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reopen_probe_rejects_different_project_save_artifact_as_source() {
    let root = unique_test_dir("reopen-different-save-artifact");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    let save_evidence_dir = run_dir.join("project-save");
    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::create_dir_all(&reopen_evidence_dir).unwrap();
    fs::write(alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK), "#!/bin/sh\n").unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved project").unwrap();
    fs::write(save_evidence_dir.join("other-project.a3p"), "other project").unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-project.a3p"),
        "reopened project",
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("project-reopen.json"),
        r#"{"reopened":true}"#,
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-state.json"),
        r#"{"world":"expected learner-world state"}"#,
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-reopen-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-reopen-result/v1",
            "status": "reopened",
            "source_saved_project_artifact": "project-save/other-project.a3p",
            "reopen_selector": "scene.eatmeFirstLessonStep",
            "reopened_project_artifact": "reopened-project.a3p",
            "reopen_artifact": "project-reopen.json",
            "reopened_state_artifact": "reopened-state.json",
            "state_verification": "passed"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_reopen());
    assert!(
        probe.validation_errors.iter().any(|error| error.contains(
            "source_saved_project_artifact must match save-project saved_project_artifact"
        )),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn reopen_probe_rejects_symlink_escape_from_reopen_evidence_dir() {
    let root = unique_test_dir("reopen-symlink-escape");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    let save_evidence_dir = run_dir.join("project-save");
    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::create_dir_all(&reopen_evidence_dir).unwrap();
    fs::write(alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK), "#!/bin/sh\n").unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved project").unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-project.a3p"),
        "reopened project",
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("project-reopen.json"),
        r#"{"reopened":true}"#,
    )
    .unwrap();
    fs::write(
        root.join("outside-reopened-state.json"),
        r#"{"outside":true}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(
        root.join("outside-reopened-state.json"),
        reopen_evidence_dir.join("reopened-state.json"),
    )
    .unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-reopen-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-reopen-result/v1",
            "status": "reopened",
            "source_saved_project_artifact": "project-save/saved-project.a3p",
            "reopen_selector": "scene.eatmeFirstLessonStep",
            "reopened_project_artifact": "reopened-project.a3p",
            "reopen_artifact": "project-reopen.json",
            "reopened_state_artifact": "reopened-state.json",
            "state_verification": "passed"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_reopen());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error| error.contains("must stay under project-reopen evidence dir")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reopen_probe_fails_when_reopened_state_artifact_is_missing_or_empty() {
    let root = unique_test_dir("reopen-missing-state");
    let alice_home = root.join("alice");
    let run_dir = root.join("runs");
    let save_evidence_dir = run_dir.join("project-save");
    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(alice_home.join("tools")).unwrap();
    fs::create_dir_all(&save_evidence_dir).unwrap();
    fs::create_dir_all(&reopen_evidence_dir).unwrap();
    fs::write(alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK), "#!/bin/sh\n").unwrap();
    fs::write(save_evidence_dir.join("saved-project.a3p"), "saved project").unwrap();
    fs::write(
        save_evidence_dir.join("project-save.json"),
        r#"{"saved":true}"#,
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("reopened-project.a3p"),
        "reopened project",
    )
    .unwrap();
    fs::write(
        reopen_evidence_dir.join("project-reopen.json"),
        r#"{"reopened":true}"#,
    )
    .unwrap();
    fs::write(reopen_evidence_dir.join("reopened-state.json"), "").unwrap();
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "tools/eatme-reopen-project --json".into(),
        exit_status: Some(0),
        stdout: serde_json::json!({
            "schema_version": "eatme.alice-project-reopen-result/v1",
            "status": "reopened",
            "source_saved_project_artifact": "project-save/saved-project.a3p",
            "reopen_selector": "scene.eatmeFirstLessonStep",
            "reopened_project_artifact": "reopened-project.a3p",
            "reopen_artifact": "project-reopen.json",
            "reopened_state_artifact": "reopened-state.json",
            "state_verification": "passed"
        })
        .to_string(),
        stderr: String::new(),
    });

    let probe = probe_project_reopen_hook(
        &runner,
        &alice_home,
        &run_dir,
        &save_probe_with_status("passed"),
        ":99",
    );

    assert_eq!(probe.status, "failed");
    assert!(!probe.proves_reopen());
    assert!(
        probe
            .validation_errors
            .iter()
            .any(|error: &String| error.contains("reopened_state_artifact must be non-empty")),
        "{:?}",
        probe.validation_errors
    );
    let _ = fs::remove_dir_all(root);
}

fn save_probe_with_status(status: &str) -> UiActionSaveProjectProbe {
    let save_passed = status == "passed";
    UiActionSaveProjectProbe {
        id: "alice-side-project-save-command-hook".into(),
        action_id: "save-project".into(),
        status: status.into(),
        detail: format!("save probe {status}"),
        save_selector: "scene.eatmeFirstLessonStep".into(),
        candidate_hook_path: "tools/eatme-save-project".into(),
        command: save_passed.then(|| "tools/eatme-save-project --json".into()),
        exit_status: save_passed.then_some(0),
        stdout: String::new(),
        stderr: String::new(),
        saved_project_artifact: save_passed.then(|| artifact("project-save/saved-project.a3p", 13)),
        save_artifact: save_passed.then(|| artifact("project-save/project-save.json", 14)),
        validation_errors: Vec::new(),
        missing_affordance: (!save_passed).then(|| UiActionMissingAffordance {
            id: "deterministic-alice-project-save-affordance".into(),
            kind: "backend_or_ui_affordance".into(),
            required_capability: "save edited project".into(),
            missing_contract: "save proof missing".into(),
            next_implementation: "add deterministic save proof".into(),
        }),
    }
}

fn assert_reopen_passed(probe: &UiActionReopenProjectProbe) {
    assert_eq!(probe.status, "passed");
    assert_eq!(probe.action_id, "reopen-project");
    assert!(probe.proves_reopen());
    assert!(probe.validation_errors.is_empty());
}

fn artifact(path: &str, size_bytes: u64) -> ArtifactInfo {
    ArtifactInfo {
        path: path.into(),
        size_bytes,
        sha256: format!("{path}-sha"),
    }
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize_whitespace(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required evidence language: {missing:?}"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-save-reopen-contract-tests")
        .join(format!("{prefix}-{nonce}"))
}

#[test]
fn reopen_preconditions_blocked_when_save_proof_absent() {
    let no_go = probe_project_reopen_preconditions(&save_probe_with_status("blocked"));

    assert_eq!(no_go.status, "blocked");
    assert!(
        no_go
            .blocking_reason
            .contains("save-project proof is required")
    );
    let save_precondition = no_go
        .preconditions
        .iter()
        .find(|p| p.id == "save-project")
        .expect("save-project precondition must exist");
    assert!(!save_precondition.passed);
}

#[allow(dead_code)]
fn _assert_relative_to_project_reopen_dir(path: &Path) {
    assert!(
        path.components()
            .all(|component| matches!(component, std::path::Component::Normal(_))),
        "reopen artifacts must stay under the project-reopen evidence dir"
    );
}
