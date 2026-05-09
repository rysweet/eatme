use eatme_alice::check_lesson_session_readiness;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SCENARIO_ID: &str = "real-alice-launch-smoke";
static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn single_baseline_target_without_modernized_is_not_ready() {
    let manifest_path = write_manifest(serde_json::json!({"baseline": ready_target("baseline")}));
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    assert!(!report.passed);
    assert_eq!(report.status, "not_ready");
    assert_eq!(report.readiness_status, "incomplete");
    assert_contains(&report.issues, "missing modernized target evidence");
}

#[test]
fn single_modernized_target_without_baseline_is_not_ready() {
    let manifest_path =
        write_manifest(serde_json::json!({"modernized": ready_target("modernized")}));
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    assert!(!report.passed);
    assert_eq!(report.status, "not_ready");
    assert_contains(&report.issues, "missing baseline target evidence");
}

#[test]
fn empty_targets_object_reports_both_targets_missing() {
    let manifest_path = write_manifest(serde_json::json!({}));
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    assert!(!report.passed);
    assert_contains(&report.issues, "missing baseline target evidence");
    assert_contains(&report.issues, "missing modernized target evidence");
}

#[test]
fn execute_not_requested_adds_issue_even_with_ready_targets() {
    let manifest_path = write_manifest_with_execute(both_ready(), false);
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    assert!(!report.passed);
    assert_contains(&report.issues, "must be produced with --execute");
}

#[test]
fn launch_smoke_never_populates_desktop_next_action() {
    for targets in [both_ready(), both_failed()] {
        let report = check_lesson_session_readiness(&write_manifest(targets)).unwrap();
        assert!(report.desktop_next_action.is_none());
    }
}

#[test]
fn launch_smoke_evidence_boundaries_are_always_empty() {
    let report = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert!(report.evidence_boundaries.is_empty());
}

// --- helpers ---

fn both_ready() -> serde_json::Value {
    serde_json::json!({"baseline": ready_target("baseline"), "modernized": ready_target("modernized")})
}

fn both_failed() -> serde_json::Value {
    serde_json::json!({
        "baseline": failed_target("baseline", "screenshot_missing"),
        "modernized": missing_launch_manifest_target("modernized"),
    })
}

fn write_manifest(targets: serde_json::Value) -> PathBuf {
    write_manifest_with_execute(targets, true)
}

fn write_manifest_with_execute(targets: serde_json::Value, execute: bool) -> PathBuf {
    let root = unique_test_dir("edge");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("comparison-manifest.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&manifest(&path, targets, execute)).unwrap(),
    )
    .unwrap();
    path
}

fn manifest(path: &Path, targets: serde_json::Value, execute: bool) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-comparison/v1",
        "comparison_contract": contract(),
        "lesson_session_contract": session_contract(),
        "scenario_id": SCENARIO_ID,
        "run_id": "edge-test",
        "execute_requested": execute,
        "created_at_unix_ms": 1, "started_at_unix_ms": 1, "finished_at_unix_ms": 2, "duration_ms": 1,
        "comparison_manifest_path": path.display().to_string(),
        "targets": targets, "scorecard": {}, "diff": {}
    })
}

fn contract() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-comparison-contract/v1",
        "inputs": [], "outputs": [], "functionality_rules": [],
        "timing_rules": [], "non_claims": [], "next_capabilities": []
    })
}

fn session_contract() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-lesson-session-contract/v1",
        "scenario_id": SCENARIO_ID,
        "session_kind": "launch_readiness",
        "automation_status": "launch_smoke_only",
        "actor_roles": ["target readiness evidence reviewer", "target launch evidence owner"],
        "required_session_steps": [
            "resolve and prepare both Alice targets",
            "package each target when execution is requested",
            "launch each target under an isolated virtual display",
            "capture manifest, window, screenshot, log, assertion, and timing evidence"
        ],
        "executable_evidence": [
            "comparison manifest records target metadata, status, scorecard, timing, and differences",
            "target launch manifests are attached when execution is requested and reaches launch smoke"
        ],
        "boundaries": [
            "does not automate complete instructor assignment creation",
            "does not automate complete student lesson consumption",
            "does not perform creative assessment",
            "does not grade student worlds",
            "does not prove broad Alice compatibility beyond the selected scenario"
        ]
    })
}

fn ready_target(role: &str) -> serde_json::Value {
    serde_json::json!({
        "role": role, "target_id": role, "label": role, "description": "test target",
        "metadata": {}, "notes": [], "alice_home_env": null, "required_paths": [],
        "resolved_alice_home": format!("/tmp/{role}-alice"), "alice_home_source": "test",
        "run_id": format!("{role}-run"),
        "started_at_unix_ms": 1, "finished_at_unix_ms": 2, "duration_ms": 1,
        "status": "passed", "detail": "launch smoke passed", "failure_category": null,
        "launch_manifest": launch_manifest(role, None, true, true),
        "launch_manifest_artifact": artifact(&format!("runs/{role}/manifest.json")),
    })
}

fn failed_target(role: &str, failure_category: &str) -> serde_json::Value {
    let mut t = ready_target(role);
    t["status"] = serde_json::json!("failed");
    t["failure_category"] = serde_json::json!(failure_category);
    t["launch_manifest"] = launch_manifest(role, Some(failure_category), false, true);
    t
}

fn missing_launch_manifest_target(role: &str) -> serde_json::Value {
    let mut t = ready_target(role);
    t["status"] = serde_json::json!("not_executed");
    t["launch_manifest"] = serde_json::Value::Null;
    t["launch_manifest_artifact"] = serde_json::Value::Null;
    t
}

fn launch_manifest(
    role: &str,
    fc: Option<&str>,
    screenshot: bool,
    arts: bool,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1", "scenario_id": SCENARIO_ID,
        "run_id": format!("{role}-run"), "alice_home": format!("/tmp/{role}-alice"),
        "alice_git_commit": "c", "eatme_git_commit": "c",
        "java_version": "21", "maven_version": "3",
        "dependency_checks": {"java":true,"maven":true,"xvfb":true,"xdpyinfo":true,"screenshot-tool":true},
        "build_command": "mvn -q package", "build_exit_status": 0,
        "launch_command": "java Alice", "display": ":99",
        "xvfb_pid": 100, "alice_pid": 101, "timeout_seconds": 900,
        "window_list": arts.then(|| artifact(&format!("runs/{role}/window-list.txt"))),
        "window_list_error": null,
        "screenshot": arts.then(|| artifact(&format!("runs/{role}/screenshots/startup.png"))),
        "screenshot_error": null, "ui_action_contract": null,
        "log": arts.then(|| artifact(&format!("runs/{role}/alice.log"))),
        "log_error": null, "fatal_log_scan": [],
        "assertions": {
            "display_responsive": a(true, "ok"),
            "process_started": a(true, "ok"),
            "startup_screenshot": a(screenshot, "ok"),
            "no_fatal_logs": a(fc.is_none(), "ok"),
            "real_alice_execution_evidence": a(fc.is_none(), "ok")
        },
        "failure_category": fc,
    })
}

fn a(passed: bool, detail: &str) -> serde_json::Value {
    serde_json::json!({"passed": passed, "detail": detail})
}

fn artifact(path: &str) -> serde_json::Value {
    serde_json::json!({"path": path, "size_bytes": 1, "sha256": "test"})
}

fn assert_contains(values: &[String], expected: &str) {
    assert!(
        values.iter().any(|v| v.contains(expected)),
        "expected {values:?} to contain {expected:?}"
    );
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let c = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    workspace_root()
        .join("target/eatme-alice-readiness-tests")
        .join(format!("{prefix}-{}-{c}-{}", std::process::id(), now_ms()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
