use super::*;
use eatme_core::{ArtifactInfo, AssertionResult, LaunchSmokeManifest};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn archives_existing_run_dir_instead_of_deleting_it() {
    let root = unique_test_dir("archive-existing-run");
    let run_dir = root.join("runs/real-alice-launch-smoke/reused-run");
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(run_dir.join("manifest.json"), "old evidence").unwrap();

    prepare_run_dir(&run_dir).unwrap();

    assert!(run_dir.join("screenshots").is_dir());
    let archived_manifest = fs::read_dir(run_dir.parent().unwrap())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("reused-run.previous-"))
                .unwrap_or(false)
                && path.join("manifest.json").is_file()
        });
    assert!(
        archived_manifest.is_some(),
        "existing evidence should be archived next to the new run"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_non_kebab_case_scenario_names() {
    assert!(validate_scenario_name("../bad").is_err());
    assert!(validate_scenario_name("building-a-scene-first-world").is_ok());
}

#[test]
fn objects_first_full_path_preflight_reports_missing_required_hooks() {
    let root = unique_test_dir("objects-first-missing-hooks");
    let tools = root.join("alice/tools");
    fs::create_dir_all(&tools).unwrap();
    for hook in [
        DEFAULT_OBJECT_PLACEMENT_HOOK,
        DEFAULT_PROCEDURE_EDIT_HOOK,
        DEFAULT_WORLD_RUN_HOOK,
        DEFAULT_PROJECT_SAVE_HOOK,
        DEFAULT_PROJECT_REOPEN_HOOK,
    ] {
        fs::write(root.join("alice").join(hook), "#!/bin/sh\n").unwrap();
    }

    let missing = missing_objects_first_full_path_hooks(&root.join("alice"));

    assert_eq!(missing, vec![DEFAULT_OBJECT_TRANSFORM_HOOK]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn objects_first_full_path_preflight_accepts_complete_hook_set() {
    let root = unique_test_dir("objects-first-complete-hooks");
    let tools = root.join("alice/tools");
    fs::create_dir_all(&tools).unwrap();
    for hook in [
        DEFAULT_OBJECT_PLACEMENT_HOOK,
        DEFAULT_OBJECT_TRANSFORM_HOOK,
        DEFAULT_PROCEDURE_EDIT_HOOK,
        DEFAULT_WORLD_RUN_HOOK,
        DEFAULT_PROJECT_SAVE_HOOK,
        DEFAULT_PROJECT_REOPEN_HOOK,
    ] {
        fs::write(root.join("alice").join(hook), "#!/bin/sh\n").unwrap();
    }

    let missing = missing_objects_first_full_path_hooks(&root.join("alice"));

    assert!(missing.is_empty(), "unexpected missing hooks: {missing:?}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn relative_runs_dir_resolves_to_absolute_launch_evidence_path() {
    let run_dir = launch_run_dir(
        PathBuf::from("runs").as_path(),
        "first-lessons-real-ui-actions",
        "sample-run",
    )
    .unwrap();

    assert!(run_dir.is_absolute());
    assert!(run_dir.ends_with("runs/first-lessons-real-ui-actions/sample-run"));
}

#[test]
fn run_id_cannot_escape_launch_evidence_directory() {
    assert!(launch_run_dir(PathBuf::from("runs").as_path(), "scenario", "../bad").is_err());
}

#[test]
fn manifest_schema_round_trip() {
    let mut assertions = BTreeMap::new();
    assertions.insert(
        "dependencies_available".into(),
        AssertionResult::pass("all present"),
    );
    assertions.insert(
        "display_responsive".into(),
        AssertionResult::pass(":99 responds"),
    );
    assertions.insert(
        "process_started".into(),
        AssertionResult::pass("Alice stayed alive"),
    );
    assertions.insert(
        "startup_screenshot".into(),
        AssertionResult::pass("screenshot captured"),
    );
    assertions.insert(
        "no_fatal_logs".into(),
        AssertionResult::pass("no fatal patterns"),
    );
    assertions.insert(
        "real_alice_execution_evidence".into(),
        AssertionResult::pass("all evidence captured"),
    );

    let manifest = LaunchSmokeManifest {
        schema_version: "1".into(),
        scenario_id: "real-alice-launch-smoke".into(),
        run_id: "round-trip-test".into(),
        alice_home: "/fake/alice".into(),
        alice_git_commit: "abc123".into(),
        eatme_git_commit: "def456".into(),
        java_version: "openjdk 21".into(),
        maven_version: "Apache Maven 3.9.0".into(),
        dependency_checks: BTreeMap::from([("java".into(), true), ("mvn".into(), true)]),
        build_command: "mvn package -o".into(),
        build_exit_status: Some(0),
        launch_command: "java -cp ... org.alice.stageide.EntryPoint".into(),
        display: ":99".into(),
        xvfb_pid: Some(1234),
        alice_pid: Some(5678),
        timeout_seconds: 90,
        window_list: Some(ArtifactInfo {
            path: "window-list.txt".into(),
            size_bytes: 128,
            sha256: "aabbcc".into(),
        }),
        window_list_error: None,
        screenshot: Some(ArtifactInfo {
            path: "screenshots/startup.png".into(),
            size_bytes: 4096,
            sha256: "ddeeff".into(),
        }),
        screenshot_error: None,
        post_focus_screenshot: Some(ArtifactInfo {
            path: "screenshots/post_focus.png".into(),
            size_bytes: 8192,
            sha256: "aabb11".into(),
        }),
        post_focus_screenshot_error: None,
        ui_action_contract: None,
        log: Some(ArtifactInfo {
            path: "alice.log".into(),
            size_bytes: 256,
            sha256: "112233".into(),
        }),
        log_error: None,
        fatal_log_scan: vec![],
        assertions,
        command: None,
        scenario: None,
        evidence: None,
        persistence_assertions: None,
        failure_category: None,
    };

    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let round_tripped: LaunchSmokeManifest =
        serde_json::from_str(&json).expect("deserialize manifest");

    assert_eq!(round_tripped.schema_version, manifest.schema_version);
    assert_eq!(round_tripped.scenario_id, manifest.scenario_id);
    assert_eq!(round_tripped.run_id, manifest.run_id);
    assert_eq!(round_tripped.alice_home, manifest.alice_home);
    assert_eq!(round_tripped.alice_pid, manifest.alice_pid);
    assert_eq!(round_tripped.xvfb_pid, manifest.xvfb_pid);
    assert_eq!(round_tripped.timeout_seconds, manifest.timeout_seconds);
    assert_eq!(round_tripped.assertions.len(), manifest.assertions.len());
    assert!(round_tripped.failure_category.is_none());
    for (key, original) in &manifest.assertions {
        let restored = round_tripped
            .assertions
            .get(key)
            .unwrap_or_else(|| panic!("missing assertion key {key}"));
        assert_eq!(restored.passed, original.passed, "mismatch on {key}");
        assert_eq!(restored.detail, original.detail, "detail mismatch on {key}");
    }
    assert_eq!(
        round_tripped.screenshot.as_ref().map(|s| &s.sha256),
        manifest.screenshot.as_ref().map(|s| &s.sha256),
    );
    assert_eq!(
        round_tripped
            .post_focus_screenshot
            .as_ref()
            .map(|s| &s.sha256),
        manifest.post_focus_screenshot.as_ref().map(|s| &s.sha256),
    );
    assert!(
        round_tripped.post_focus_screenshot.is_some(),
        "post_focus_screenshot should survive round-trip serialization"
    );
    assert!(
        round_tripped.post_focus_screenshot_error.is_none(),
        "post_focus_screenshot_error should remain None after round-trip"
    );
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-tests")
        .join(format!("{prefix}-{nonce}"))
}

#[test]
fn manifest_deserializes_without_post_focus_fields() {
    let json = r#"{
        "schema_version": "eatme.launch-smoke/v1",
        "scenario_id": "test",
        "run_id": "old-run",
        "alice_home": "/opt/alice3",
        "alice_git_commit": "abc",
        "eatme_git_commit": "def",
        "java_version": "21",
        "maven_version": "3.9.0",
        "dependency_checks": {},
        "build_command": "mvn package",
        "build_exit_status": 0,
        "launch_command": "java ...",
        "display": ":99",
        "xvfb_pid": 1234,
        "alice_pid": 5678,
        "timeout_seconds": 90,
        "window_list": null,
        "window_list_error": null,
        "screenshot": null,
        "screenshot_error": null,
        "ui_action_contract": null,
        "log": null,
        "log_error": null,
        "fatal_log_scan": [],
        "assertions": {},
        "failure_category": null
    }"#;

    let manifest: LaunchSmokeManifest =
        serde_json::from_str(json).expect("older manifests without post_focus fields must parse");
    assert!(manifest.post_focus_screenshot.is_none());
    assert!(manifest.post_focus_screenshot_error.is_none());
}
