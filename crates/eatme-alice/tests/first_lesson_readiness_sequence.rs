use eatme_alice::{FirstLessonReadinessOptions, run_first_lesson_readiness_sequence};
use std::fs;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

#[test]
fn sequence_executes_fake_targets_until_ui_action_blocker() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-sequence".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    for role in ["baseline", "modernized"] {
        let target = report.target_statuses.get(role).unwrap();
        assert_eq!(
            target.failure_category.as_deref(),
            Some("ui_action_automation_unimplemented")
        );
        assert!(target.launch_manifest_present);
        assert!(target.ui_action_contract_path.is_some());
    }
}

#[test]
fn sequence_reports_action_progress_when_earlier_window_detection_fails() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_unrelated_window_tool();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-action-progress".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    let modernized = report
        .readiness_report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .unwrap();
    assert_eq!(
        modernized.failure_category.as_deref(),
        Some("alice_window_not_detected")
    );
    assert_action(modernized, "verify-specific-alice-window", false);
    assert_action(modernized, "activate-specific-alice-window", false);
    assert_action(modernized, "place-object", true);
    assert_action(modernized, "edit-procedure-or-code-block", false);
    assert_action(modernized, "run-world", false);
    assert_action(modernized, "save-project", false);
}

fn assert_action(
    target: &eatme_alice::compare::LessonTargetEvidence,
    action_id: &str,
    expected_passed: bool,
) {
    let action = target
        .action_assertions
        .iter()
        .find(|action| action.action_id == action_id)
        .unwrap_or_else(|| panic!("missing action evidence for {action_id}"));
    assert_eq!(action.passed, expected_passed, "{action:?}");
}
