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
