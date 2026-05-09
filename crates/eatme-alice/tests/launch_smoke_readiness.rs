use eatme_alice::check_lesson_session_readiness;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SCENARIO_ID: &str = "real-alice-launch-smoke";
static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn ready_launch_smoke_manifest_maps_to_bounded_readiness_only() {
    let manifest_path = write_comparison_manifest(serde_json::json!({
        "baseline": ready_target("baseline"),
        "modernized": ready_target("modernized"),
    }));

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        report.passed,
        "ready launch-smoke evidence should pass: {report_json}"
    );
    assert_eq!(report.status, "ready", "{report_json}");
    assert_eq!(report.readiness_status, "ready", "{report_json}");
    assert_eq!(report.scenario_id.as_deref(), Some(SCENARIO_ID));
    assert!(
        report.issues.is_empty(),
        "ready launch-smoke mapping should not inherit first-lesson issues: {:?}",
        report.issues
    );
    assert_eq!(
        report.human_summary,
        "real-alice-launch-smoke launch-smoke readiness is ready from existing target launch-smoke manifest evidence only."
    );

    assert_required_launch_smoke_evidence(&report.required_evidence);
    assert_role_status(&report_json, "baseline", "ready");
    assert_role_status(&report_json, "modernized", "ready");
    assert_launch_smoke_non_claims(&report_json);
    assert_no_lesson_or_assessment_claims(&report_json);
    assert_no_first_lesson_readiness_requirements(&report_json);
}

#[test]
fn missing_or_failed_launch_smoke_evidence_maps_to_non_ready() {
    let manifest_path = write_comparison_manifest(serde_json::json!({
        "baseline": failed_target("baseline", "screenshot_missing"),
        "modernized": missing_launch_manifest_target("modernized"),
    }));

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        !report.passed,
        "failed evidence must not pass: {report_json}"
    );
    assert_eq!(report.status, "not_ready", "{report_json}");
    assert_eq!(report.readiness_status, "incomplete", "{report_json}");
    assert_contains(&report.issues, "baseline target status must be passed");
    assert_contains(
        &report.issues,
        "baseline launch_manifest failure_category must be null",
    );
    assert_contains(
        &report.issues,
        "modernized target is missing embedded launch_manifest",
    );
    assert_role_status(&report_json, "baseline", "not_ready");
    assert_role_status(&report_json, "modernized", "not_ready");
    assert_launch_smoke_non_claims(&report_json);
    assert_no_lesson_or_assessment_claims(&report_json);
    assert_no_first_lesson_readiness_requirements(&report_json);
}

#[test]
fn partial_launch_smoke_manifest_never_reports_ready() {
    let mut partial = ready_target("modernized");
    partial["launch_manifest"]["assertions"]["startup_screenshot"]["passed"] =
        serde_json::json!(false);
    partial["launch_manifest"]["screenshot"] = serde_json::Value::Null;

    let manifest_path = write_comparison_manifest(serde_json::json!({
        "baseline": ready_target("baseline"),
        "modernized": partial,
    }));

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        !report.passed,
        "partial launch-smoke manifest evidence must not pass: {report_json}"
    );
    assert_eq!(report.status, "not_ready", "{report_json}");
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"startup_screenshot\" must pass",
    );
    assert_contains(
        &report.issues,
        "modernized launch_manifest screenshot metadata must be present",
    );
    assert_role_status(&report_json, "baseline", "ready");
    assert_role_status(&report_json, "modernized", "not_ready");
    assert_launch_smoke_non_claims(&report_json);
    assert_no_lesson_or_assessment_claims(&report_json);
    assert_no_first_lesson_readiness_requirements(&report_json);
}

#[test]
fn non_null_non_string_failure_categories_are_not_ready() {
    let mut baseline = ready_target("baseline");
    baseline["failure_category"] = serde_json::json!({"category": "hidden"});
    let mut modernized = ready_target("modernized");
    modernized["launch_manifest"]["failure_category"] = serde_json::json!(["hidden"]);

    let manifest_path = write_comparison_manifest(serde_json::json!({
        "baseline": baseline,
        "modernized": modernized,
    }));

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        !report.passed,
        "malformed non-null failure categories must not pass: {report_json}"
    );
    assert_eq!(report.status, "not_ready", "{report_json}");
    assert_eq!(report.readiness_status, "incomplete", "{report_json}");
    assert_contains(
        &report.issues,
        "baseline target failure_category must be null",
    );
    assert_contains(
        &report.issues,
        "modernized launch_manifest failure_category must be null",
    );
    assert_role_status(&report_json, "baseline", "not_ready");
    assert_role_status(&report_json, "modernized", "not_ready");
    assert_launch_smoke_non_claims(&report_json);
    assert_no_lesson_or_assessment_claims(&report_json);
    assert_no_first_lesson_readiness_requirements(&report_json);
}

#[test]
fn real_alice_launch_smoke_assets_keep_bounded_non_claim_scope() {
    let root = workspace_root();
    let canonical =
        fs::read_to_string(root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml"))
            .unwrap();
    let gadugi =
        fs::read_to_string(root.join("assets/scenarios/gadugi/real-alice-launch-smoke.yaml"))
            .unwrap();
    let combined = format!("{canonical}\n{gadugi}");
    let lower = combined.to_ascii_lowercase();

    for expected in [
        "manifest/log/window/screenshot evidence",
        "manifest-level evidence only",
        "not full ui automation",
        "not creative assessment",
        "not learner-world grading",
    ] {
        assert!(
            lower.contains(expected),
            "launch-smoke scenario assets should state bounded scope {expected:?}: {combined}"
        );
    }
    for forbidden in [
        "prove that the harness",
        "proves a scenario-labeled launch path",
        "proves launch smoke only",
        "lesson completion",
        "grading passed",
        "creative assessment passed",
        "full ui automation succeeds",
        "visible correctness",
        "visible rendering correctness",
    ] {
        assert!(
            !lower.contains(forbidden),
            "launch-smoke scenario assets must avoid overclaim {forbidden:?}: {combined}"
        );
    }
}

fn write_comparison_manifest(targets: serde_json::Value) -> PathBuf {
    let root = unique_test_dir("launch-smoke-readiness");
    fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&comparison_manifest(&manifest_path, targets)).unwrap(),
    )
    .unwrap();
    manifest_path
}

fn comparison_manifest(manifest_path: &Path, targets: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-comparison/v1",
        "comparison_contract": comparison_contract(),
        "lesson_session_contract": lesson_session_contract(),
        "scenario_id": SCENARIO_ID,
        "run_id": "launch-smoke-readiness-test",
        "execute_requested": true,
        "created_at_unix_ms": 1,
        "started_at_unix_ms": 1,
        "finished_at_unix_ms": 2,
        "duration_ms": 1,
        "comparison_manifest_path": manifest_path.display().to_string(),
        "targets": targets,
        "scorecard": {},
        "diff": {}
    })
}

fn comparison_contract() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-comparison-contract/v1",
        "inputs": [],
        "outputs": [],
        "functionality_rules": [],
        "timing_rules": [],
        "non_claims": [],
        "next_capabilities": []
    })
}

fn lesson_session_contract() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-lesson-session-contract/v1",
        "scenario_id": SCENARIO_ID,
        "session_kind": "launch_readiness",
        "automation_status": "launch_smoke_only",
        "actor_roles": [
            "target readiness evidence reviewer",
            "target launch evidence owner"
        ],
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
        "role": role,
        "target_id": role,
        "label": role,
        "description": "test target",
        "metadata": {},
        "notes": [],
        "alice_home_env": null,
        "required_paths": [],
        "resolved_alice_home": format!("/tmp/{role}-alice"),
        "alice_home_source": "test",
        "run_id": format!("{role}-run"),
        "started_at_unix_ms": 1,
        "finished_at_unix_ms": 2,
        "duration_ms": 1,
        "status": "passed",
        "detail": "launch smoke passed",
        "failure_category": null,
        "launch_manifest": launch_manifest(role, None, true, true),
        "launch_manifest_artifact": artifact(&format!("runs/{role}/manifest.json")),
    })
}

fn failed_target(role: &str, failure_category: &str) -> serde_json::Value {
    let mut target = ready_target(role);
    target["status"] = serde_json::json!("failed");
    target["detail"] = serde_json::json!("launch smoke failed");
    target["failure_category"] = serde_json::json!(failure_category);
    target["launch_manifest"] = launch_manifest(role, Some(failure_category), false, true);
    target
}

fn missing_launch_manifest_target(role: &str) -> serde_json::Value {
    let mut target = ready_target(role);
    target["status"] = serde_json::json!("not_executed");
    target["detail"] = serde_json::json!("no launch manifest was produced");
    target["launch_manifest"] = serde_json::Value::Null;
    target["launch_manifest_artifact"] = serde_json::Value::Null;
    target
}

fn launch_manifest(
    role: &str,
    failure_category: Option<&str>,
    startup_screenshot_passed: bool,
    include_artifacts: bool,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1",
        "scenario_id": SCENARIO_ID,
        "run_id": format!("{role}-run"),
        "alice_home": format!("/tmp/{role}-alice"),
        "alice_git_commit": "test-alice-commit",
        "eatme_git_commit": "test-eatme-commit",
        "java_version": "21-test",
        "maven_version": "3-test",
        "dependency_checks": {
            "java": true,
            "maven": true,
            "xvfb": true,
            "xdpyinfo": true,
            "screenshot-tool": true
        },
        "build_command": "mvn -q package",
        "build_exit_status": 0,
        "launch_command": "java Alice",
        "display": ":99",
        "xvfb_pid": 100,
        "alice_pid": 101,
        "timeout_seconds": 900,
        "window_list": include_artifacts.then(|| artifact(&format!("runs/{role}/window-list.txt"))),
        "window_list_error": null,
        "screenshot": include_artifacts.then(|| artifact(&format!("runs/{role}/screenshots/startup.png"))),
        "screenshot_error": null,
        "ui_action_contract": null,
        "log": include_artifacts.then(|| artifact(&format!("runs/{role}/alice.log"))),
        "log_error": null,
        "fatal_log_scan": [],
        "assertions": {
            "display_responsive": assertion(true, "virtual display responded"),
            "process_started": assertion(true, "Alice process stayed alive"),
            "startup_screenshot": assertion(startup_screenshot_passed, "startup screenshot exists and is non-empty"),
            "no_fatal_logs": assertion(failure_category.is_none(), "0 fatal log lines found"),
            "real_alice_execution_evidence": assertion(failure_category.is_none(), "real Alice process, responsive virtual display, visual evidence, and launch log were captured")
        },
        "failure_category": failure_category,
    })
}

fn assertion(passed: bool, detail: &str) -> serde_json::Value {
    serde_json::json!({ "passed": passed, "detail": detail })
}

fn artifact(path: &str) -> serde_json::Value {
    serde_json::json!({
        "path": path,
        "size_bytes": 1,
        "sha256": "test-sha256"
    })
}

fn assert_required_launch_smoke_evidence(required_evidence: &[String]) {
    for expected in [
        "comparison manifest with baseline and modernized targets for real-alice-launch-smoke",
        "embedded launch-smoke manifest for each target",
        "each target status is passed with no launch failure category",
        "required launch-smoke assertions passed for each target",
        "launch-smoke artifact metadata for window list, screenshot, and log",
    ] {
        assert_contains(required_evidence, expected);
    }
}

fn assert_role_status(report_json: &serde_json::Value, role: &str, status: &str) {
    let readiness = report_json["role_readiness"]
        .as_array()
        .unwrap_or_else(|| panic!("role_readiness must be an array: {report_json}"))
        .iter()
        .find(|entry| entry["role"] == role)
        .unwrap_or_else(|| panic!("missing role_readiness for {role}: {report_json}"));
    assert_eq!(readiness["status"], status, "{readiness}");
}

fn assert_launch_smoke_non_claims(report_json: &serde_json::Value) {
    let text = serde_json::to_string(report_json)
        .unwrap()
        .to_ascii_lowercase();
    for required in [
        "lesson completion",
        "grading",
        "creative assessment",
        "full ui automation",
        "visible correctness",
    ] {
        assert!(
            text.contains(required),
            "launch-smoke readiness output should explicitly bound {required:?}: {text}"
        );
    }
    for required in [
        "lesson completion is not proven",
        "grading is not proven",
        "creative assessment is not proven",
        "full ui automation is not proven",
        "visible correctness is not proven",
    ] {
        assert!(
            text.contains(required),
            "launch-smoke readiness output should expose non-claim {required:?}: {text}"
        );
    }
}

fn assert_no_lesson_or_assessment_claims(report_json: &serde_json::Value) {
    let text = serde_json::to_string(report_json)
        .unwrap()
        .to_ascii_lowercase();
    for forbidden in [
        "lesson completion is proven",
        "lesson completed",
        "lesson is complete",
        "grading passed",
        "graded successfully",
        "creative assessment passed",
        "creative quality accepted",
        "full ui automation is proven",
        "full ui automation succeeded",
        "visible correctness is proven",
        "visible rendering correctness is proven",
    ] {
        assert!(
            !text.contains(forbidden),
            "launch-smoke readiness output must not claim {forbidden:?}: {text}"
        );
    }
}

fn assert_no_first_lesson_readiness_requirements(report_json: &serde_json::Value) {
    let text = serde_json::to_string(report_json)
        .unwrap()
        .to_ascii_lowercase();
    for forbidden in [
        "first-lesson",
        "automation scenario action evidence",
        "modernized run-window evidence",
        "modernized desktop-run-pixel",
        "save project proof artifact",
        "select project proof artifact",
        "ui-action-contract",
        "place-object",
        "run-world",
    ] {
        assert!(
            !text.contains(forbidden),
            "launch-smoke readiness must not reuse first-lesson readiness requirement {forbidden:?}: {text}"
        );
    }
}

fn assert_contains(values: &[String], expected: &str) {
    assert!(
        values.iter().any(|value| value.contains(expected)),
        "expected {values:?} to contain {expected:?}"
    );
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    workspace_root()
        .join("target/eatme-alice-readiness-tests")
        .join(format!(
            "{prefix}-{}-{counter}-{}",
            std::process::id(),
            now_ms()
        ))
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
