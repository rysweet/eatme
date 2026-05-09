use eatme_alice::check_lesson_session_readiness;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SCENARIO_ID: &str = "real-alice-launch-smoke";
static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn evidence_progress_contains_exactly_five_items() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.evidence_progress.items.len(), 5);
    assert_eq!(r.evidence_progress.total_required, 5);
}

#[test]
fn ready_evidence_progress_shows_all_present() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.evidence_progress.present, 5);
    assert_eq!(r.evidence_progress.missing, 0);
    assert_eq!(r.evidence_progress.invalid, 0);
    assert_eq!(r.evidence_progress.blocked, 0);
    assert_eq!(r.evidence_progress.not_observed, 0);
}

#[test]
fn failed_evidence_progress_tracks_invalid_or_missing() {
    let r = check_lesson_session_readiness(&write_manifest(both_failed())).unwrap();
    assert!(r.evidence_progress.missing > 0 || r.evidence_progress.invalid > 0);
    assert_eq!(r.evidence_progress.total_required, 5);
}

#[test]
fn unproven_claims_are_exactly_eight() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.unproven_claims.len(), 8, "{:?}", r.unproven_claims);
    for expected in [
        "First-lesson completion is not proven.",
        "Full world execution is not proven.",
        "Grading is not proven.",
        "Creative assessment is not proven.",
        "Full Alice UI automation is not proven.",
        "Visible rendering correctness is not proven.",
        "Save completion is not proven.",
        "Deployed sharing/platform success is not proven.",
    ] {
        assert!(
            r.unproven_claims.iter().any(|c| c == expected),
            "missing: {expected:?}"
        );
    }
}

#[test]
fn limitations_are_exactly_fourteen() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.limitations.len(), 14, "{:?}", r.limitations);
}

#[test]
fn ready_desktop_proof_contract_is_verified() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.desktop_proof_contract.status, "verified");
    assert_eq!(
        r.desktop_proof_contract.reason_code,
        "launch_smoke_manifest_ready"
    );
}

#[test]
fn missing_manifest_proof_contract_is_unsupported_environment() {
    let m = write_manifest(serde_json::json!({
        "baseline": ready_target("baseline"),
        "modernized": missing_manifest_target("modernized"),
    }));
    let r = check_lesson_session_readiness(&m).unwrap();
    assert_eq!(r.desktop_proof_contract.status, "unsupported_environment");
    assert_eq!(
        r.desktop_proof_contract.reason_code,
        "launch_smoke_manifest_missing"
    );
}

#[test]
fn failed_assertions_proof_contract_is_launched_but_unverified() {
    let mut t = ready_target("modernized");
    t["launch_manifest"]["assertions"]["startup_screenshot"]["passed"] = serde_json::json!(false);
    let m =
        write_manifest(serde_json::json!({"baseline": ready_target("baseline"), "modernized": t}));
    let r = check_lesson_session_readiness(&m).unwrap();
    assert_eq!(r.desktop_proof_contract.status, "launched_but_unverified");
    assert_eq!(
        r.desktop_proof_contract.reason_code,
        "launch_smoke_manifest_incomplete"
    );
}

#[test]
fn schema_version_is_stable() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    assert_eq!(r.schema_version, "eatme.alice-lesson-session-readiness/v1");
}

#[test]
fn required_evidence_text_is_stable() {
    let r = check_lesson_session_readiness(&write_manifest(both_ready())).unwrap();
    let expected = [
        "comparison manifest with baseline and modernized targets for real-alice-launch-smoke",
        "embedded launch-smoke manifest for each target",
        "each target status is passed with no launch failure category",
        "required launch-smoke assertions passed for each target",
        "launch-smoke artifact metadata for window list, screenshot, and log",
    ];
    assert_eq!(r.required_evidence.len(), expected.len());
    for (a, e) in r.required_evidence.iter().zip(expected.iter()) {
        assert_eq!(a, e);
    }
}

#[test]
fn mapping_doc_references_assertions() {
    let doc = read_doc("docs/launch-smoke-readiness-mapping.md");
    for name in [
        "display_responsive",
        "process_started",
        "startup_screenshot",
        "no_fatal_logs",
        "real_alice_execution_evidence",
    ] {
        assert!(doc.contains(name), "doc missing assertion {name:?}");
    }
}

#[test]
fn mapping_doc_references_artifact_fields() {
    let doc = read_doc("docs/launch-smoke-readiness-mapping.md");
    for field in ["window list", "screenshot", "log"] {
        assert!(doc.contains(field), "doc missing artifact {field:?}");
    }
}

// --- helpers ---

fn both_ready() -> serde_json::Value {
    serde_json::json!({"baseline": ready_target("baseline"), "modernized": ready_target("modernized")})
}

fn both_failed() -> serde_json::Value {
    serde_json::json!({
        "baseline": failed_target("baseline", "screenshot_missing"),
        "modernized": missing_manifest_target("modernized"),
    })
}

fn read_doc(path: &str) -> String {
    fs::read_to_string(workspace_root().join(path))
        .unwrap()
        .to_ascii_lowercase()
}

fn write_manifest(targets: serde_json::Value) -> PathBuf {
    let root = unique_test_dir("contract");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("comparison-manifest.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&manifest(&path, targets)).unwrap(),
    )
    .unwrap();
    path
}

fn manifest(path: &Path, targets: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.alice-comparison/v1",
        "comparison_contract": contract(),
        "lesson_session_contract": session_contract(),
        "scenario_id": SCENARIO_ID, "run_id": "contract-test",
        "execute_requested": true,
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
        "scenario_id": SCENARIO_ID, "session_kind": "launch_readiness",
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
        "role": role, "target_id": role, "label": role, "description": "test",
        "metadata": {}, "notes": [], "alice_home_env": null, "required_paths": [],
        "resolved_alice_home": format!("/tmp/{role}-alice"), "alice_home_source": "test",
        "run_id": format!("{role}-run"),
        "started_at_unix_ms": 1, "finished_at_unix_ms": 2, "duration_ms": 1,
        "status": "passed", "detail": "ok", "failure_category": null,
        "launch_manifest": lm(role, None, true, true),
        "launch_manifest_artifact": art(&format!("runs/{role}/manifest.json")),
    })
}

fn failed_target(role: &str, fc: &str) -> serde_json::Value {
    let mut t = ready_target(role);
    t["status"] = serde_json::json!("failed");
    t["failure_category"] = serde_json::json!(fc);
    t["launch_manifest"] = lm(role, Some(fc), false, true);
    t
}

fn missing_manifest_target(role: &str) -> serde_json::Value {
    let mut t = ready_target(role);
    t["status"] = serde_json::json!("not_executed");
    t["launch_manifest"] = serde_json::Value::Null;
    t["launch_manifest_artifact"] = serde_json::Value::Null;
    t
}

fn lm(role: &str, fc: Option<&str>, ss: bool, arts: bool) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1", "scenario_id": SCENARIO_ID,
        "run_id": format!("{role}-run"), "alice_home": format!("/tmp/{role}-alice"),
        "alice_git_commit": "c", "eatme_git_commit": "c",
        "java_version": "21", "maven_version": "3",
        "dependency_checks": {"java":true,"maven":true,"xvfb":true,"xdpyinfo":true,"screenshot-tool":true},
        "build_command": "mvn -q package", "build_exit_status": 0,
        "launch_command": "java Alice", "display": ":99",
        "xvfb_pid": 100, "alice_pid": 101, "timeout_seconds": 900,
        "window_list": arts.then(|| art(&format!("runs/{role}/wl.txt"))),
        "window_list_error": null,
        "screenshot": arts.then(|| art(&format!("runs/{role}/ss.png"))),
        "screenshot_error": null, "ui_action_contract": null,
        "log": arts.then(|| art(&format!("runs/{role}/alice.log"))),
        "log_error": null, "fatal_log_scan": [],
        "assertions": {
            "display_responsive": a(true),
            "process_started": a(true),
            "startup_screenshot": a(ss),
            "no_fatal_logs": a(fc.is_none()),
            "real_alice_execution_evidence": a(fc.is_none())
        },
        "failure_category": fc,
    })
}

fn a(passed: bool) -> serde_json::Value {
    serde_json::json!({"passed": passed, "detail": "ok"})
}
fn art(path: &str) -> serde_json::Value {
    serde_json::json!({"path": path, "size_bytes": 1, "sha256": "t"})
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
