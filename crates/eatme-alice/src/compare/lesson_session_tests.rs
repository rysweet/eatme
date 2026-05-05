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
