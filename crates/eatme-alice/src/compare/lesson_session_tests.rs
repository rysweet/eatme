use super::*;
use std::path::{Path, PathBuf};

#[test]
fn first_lesson_comparison_records_lesson_session_contract() {
    let root = unique_test_dir("first-lesson-comparison-contract");
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Existing Alice checkout used as the reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Modernized Alice checkout used as the comparison target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();

    let manifest = run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
        run_id: "first-lesson-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
    })
    .unwrap();

    assert_eq!(
        manifest.lesson_session_contract.session_kind,
        "first_lesson_action_contract"
    );
    assert_eq!(
        manifest.lesson_session_contract.automation_status,
        "action_contract_blocked_until_ui_automation"
    );
    assert_contract_contains(
        &manifest.lesson_session_contract.required_session_steps,
        "student runs the world",
    );
    assert_contract_contains(
        &manifest.lesson_session_contract.executable_evidence,
        "ui-action-contract.json",
    );
    assert_contract_contains(
        &manifest.lesson_session_contract.boundaries,
        "does not grade student worlds",
    );
}

#[test]
fn lesson_session_contract_check_passes_first_lesson_manifest() {
    let root = unique_test_dir("first-lesson-contract-check");
    let manifest = write_first_lesson_manifest(&root);

    let report =
        check_lesson_session_contract(Path::new(&manifest.comparison_manifest_path)).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(
        report.session_kind.as_deref(),
        Some("first_lesson_action_contract")
    );
}

#[test]
fn lesson_session_contract_check_fails_when_contract_is_missing() {
    let root = unique_test_dir("missing-lesson-contract-check");
    fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-comparison/v1","scenario_id":"first-lessons-real-ui-actions"}"#,
    )
    .unwrap();

    let report = check_lesson_session_contract(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(&report.issues, "missing lesson_session_contract");
}

#[test]
fn lesson_session_contract_check_rejects_placeholder_first_lesson_steps() {
    let root = unique_test_dir("placeholder-lesson-contract-check");
    let manifest = write_first_lesson_manifest(&root);
    let manifest_path = Path::new(&manifest.comparison_manifest_path);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    value["lesson_session_contract"]["required_session_steps"] = serde_json::json!([
        "student opens x",
        "student places x",
        "student edits x",
        "student runs x",
        "student saves x"
    ]);
    fs::write(manifest_path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    let report = check_lesson_session_contract(manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(
        &report.issues,
        "student opens the configured starter project in Alice",
    );
}

#[test]
fn lesson_session_readiness_requires_executable_target_evidence() {
    let root = unique_test_dir("manifest-only-readiness-check");
    let manifest = write_first_lesson_manifest(&root);

    let report =
        check_lesson_session_readiness(Path::new(&manifest.comparison_manifest_path)).unwrap();

    assert!(!report.passed);
    assert_eq!(report.readiness_status, "incomplete");
    assert_contract_contains(&report.issues, "must be produced with --execute");
    assert_contract_contains(&report.issues, "missing embedded launch_manifest");
}

#[test]
fn lesson_session_readiness_consumes_ui_action_contract_artifacts() {
    let root = unique_test_dir("executable-readiness-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    assert_eq!(report.execute_requested, Some(true));
    assert_eq!(report.target_evidence.len(), 2);
    for target in &report.target_evidence {
        assert!(target.launch_manifest_present);
        assert!(target.ui_action_contract_readable);
        assert!(target.missing_assertions.is_empty());
        assert!(target.missing_required_actions.is_empty());
        assert!(
            target
                .required_actions
                .iter()
                .any(|id| id == "save-project")
        );
    }
    assert_contract_contains(&report.limitations, "does not grade student worlds");
}

#[test]
fn lesson_session_readiness_rejects_incomplete_ui_action_contract() {
    let root = unique_test_dir("incomplete-action-contract-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, true);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(&report.issues, "missing required action \"save-project\"");
}

#[test]
fn lesson_session_readiness_rejects_unsafe_ui_action_contract_path() {
    let root = unique_test_dir("unsafe-action-contract-path-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    value["targets"]["baseline"]["launch_manifest"]["ui_action_contract"]["path"] =
        serde_json::json!("../../outside-ui-action-contract.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&value).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(&report.issues, "ui-action-contract.path is unsafe");
    assert_contract_contains(&report.issues, "must not contain parent");
}

#[cfg(unix)]
#[test]
fn lesson_session_readiness_rejects_symlinked_ui_action_contract_escape() {
    let root = unique_test_dir("symlink-action-contract-path-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let outside = root.join("outside-ui-action-contract.json");
    fs::write(
        &outside,
        serde_json::to_vec_pretty(&ui_action_contract_json(false)).unwrap(),
    )
    .unwrap();
    let evidence_dir = manifest_path.parent().unwrap();
    let link = evidence_dir.join("linked-ui-action-contract.json");
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    value["targets"]["baseline"]["launch_manifest"]["ui_action_contract"]["path"] =
        serde_json::json!("linked-ui-action-contract.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&value).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(&report.issues, "ui-action-contract.path is unsafe");
    assert_contract_contains(&report.issues, "must stay under comparison evidence root");
}

#[test]
fn first_lesson_readiness_sequence_reports_manifest_only_gap() {
    let root = unique_test_dir("first-lesson-sequence-manifest-only");
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "first-lesson-sequence".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    assert_eq!(report.scenario_id, FIRST_LESSON_SCENARIO_ID);
    assert_eq!(report.readiness_status, "incomplete");
    assert!(Path::new(&report.comparison_manifest_path).is_file());
    assert_contract_contains(&report.issues, "must be produced with --execute");
    assert_contract_contains(&report.limitations, "does not grade student worlds");
}

fn write_first_lesson_manifest(root: &Path) -> AliceComparisonManifest {
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Existing Alice checkout used as the reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Modernized Alice checkout used as the comparison target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();

    run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
        run_id: "first-lesson-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
    })
    .unwrap()
}

fn write_executable_blocked_first_lesson_manifest(root: &Path, omit_save_action: bool) -> PathBuf {
    let manifest = write_first_lesson_manifest(root);
    let manifest_path = PathBuf::from(&manifest.comparison_manifest_path);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    value["execute_requested"] = serde_json::json!(true);
    for role in ["baseline", "modernized"] {
        let action_contract_path = root
            .join("runs")
            .join("first-lessons-real-ui-actions")
            .join(format!("{role}-first-lesson-run"))
            .join("ui-action-contract.json");
        fs::create_dir_all(action_contract_path.parent().unwrap()).unwrap();
        fs::write(
            &action_contract_path,
            serde_json::to_vec_pretty(&ui_action_contract_json(omit_save_action)).unwrap(),
        )
        .unwrap();
        value["targets"][role]["status"] = serde_json::json!("failed");
        value["targets"][role]["failure_category"] =
            serde_json::json!("ui_action_automation_unimplemented");
        value["targets"][role]["launch_manifest"] = launch_manifest_json(&action_contract_path);
    }
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&value).unwrap(),
    )
    .unwrap();
    manifest_path
}

fn launch_manifest_json(action_contract_path: &Path) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.launch-smoke/v1",
        "scenario_id": "first-lessons-real-ui-actions",
        "failure_category": "ui_action_automation_unimplemented",
        "ui_action_contract": {
            "path": action_contract_path.display().to_string(),
            "size_bytes": 1,
            "sha256": "test-sha"
        },
        "assertions": {
            "real_alice_execution_evidence": {
                "passed": true,
                "detail": "real Alice process, responsive virtual display, visual evidence, and launch log were captured"
            },
            "specific_alice_window_detected": {
                "passed": true,
                "detail": "wmctrl window list contains an Alice Stage IDE window"
            },
            "activate_alice_window_ui_action": {
                "passed": true,
                "detail": "wmctrl activated Alice window 0x001"
            },
            "place_object_precondition_no_go_probe": {
                "passed": true,
                "detail": "blocked: no supported deterministic Alice object placement backend is wired"
            },
            "place_object_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can add/place an object yet"
            },
            "edit_procedure_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can edit a procedure or code block yet"
            },
            "run_world_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can run the world yet"
            },
            "save_project_ui_action": {
                "passed": false,
                "detail": "blocked: no supported Alice desktop automation can save the project yet"
            },
            "ui_action_artifact_captured": {
                "passed": true,
                "detail": "ui action contract artifact exists and is non-empty"
            }
        }
    })
}

fn ui_action_contract_json(omit_save_action: bool) -> serde_json::Value {
    let mut actions = vec![
        serde_json::json!({
            "id": "verify-specific-alice-window",
            "required_evidence": "wmctrl output identifies an Alice Stage IDE window"
        }),
        serde_json::json!({
            "id": "activate-specific-alice-window",
            "required_evidence": "wmctrl -ia succeeds against the detected Alice window id"
        }),
        serde_json::json!({
            "id": "place-object",
            "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
            "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance"
        }),
        serde_json::json!({
            "id": "edit-procedure-or-code-block",
            "required_evidence": "artifact proves a procedure or code block was edited"
        }),
        serde_json::json!({
            "id": "run-world",
            "required_evidence": "artifact proves the world run control was invoked"
        }),
    ];
    if !omit_save_action {
        actions.push(serde_json::json!({
            "id": "save-project",
            "required_evidence": "saved .a3p project artifact exists and is non-empty"
        }));
    }
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet.",
        "preflight_evidence": {
            "specific_alice_window_detected": true,
            "visual_evidence_captured": true,
            "log_captured": true
        },
        "executed_action_probes": [{
            "id": "activate-specific-alice-window",
            "status": "passed",
            "detail": "wmctrl activated Alice window 0x001",
            "window_id": "0x001",
            "command": "wmctrl -ia 0x001",
            "exit_status": 0,
            "stdout": "",
            "stderr": ""
        }],
        "action_precondition_probes": [{
            "id": "place-object-precondition",
            "action_id": "place-object",
            "status": "blocked",
            "decision": "no_go",
            "blocking_reason": "blocked: missing deterministic-alice-object-gallery-placement-affordance",
            "required_evidence": "artifact proves a named object was added to the Alice scene and placed without coordinate guessing",
            "missing_affordance": {
                "id": "deterministic-alice-object-gallery-placement-affordance",
                "kind": "backend_or_ui_affordance",
                "required_capability": "Given an open Alice starter project and a named object identifier, deterministically add that object to the scene without coordinate guessing.",
                "missing_contract": "No Alice backend command, accessibility target, stable menu action, or scene-graph verification hook currently accepts a named object identifier and returns proof of placement.",
                "next_implementation": "Add one stable affordance: either an Alice-side object placement command/test hook, or a UI automation contract with a named gallery selector plus scene-graph or saved-project diff verification."
            },
            "preconditions": [
                {
                    "id": "specific-alice-window-detected",
                    "passed": true,
                    "detail": "wmctrl output identifies an Alice Stage IDE window"
                },
                {
                    "id": "activate-specific-alice-window",
                    "passed": true,
                    "detail": "wmctrl -ia succeeds against the detected Alice window id"
                },
                {
                    "id": "visual-evidence-captured",
                    "passed": true,
                    "detail": "startup screenshot or window evidence exists"
                },
                {
                    "id": "log-captured",
                    "passed": true,
                    "detail": "Alice launch log exists and is non-empty"
                },
                {
                    "id": "deterministic-alice-object-gallery-placement-affordance",
                    "passed": false,
                    "detail": "missing stable backend command, accessibility target, menu action, or scene-graph verification hook for named object placement"
                }
            ]
        }],
        "required_actions": actions
    })
}

fn assert_contract_contains(entries: &[String], expected: &str) {
    assert!(
        entries.iter().any(|entry| entry.contains(expected)),
        "contract entries should contain {expected:?}: {entries:?}"
    );
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-alice-comparison-tests")
        .join(format!("{prefix}-{}", now_ms()))
}
