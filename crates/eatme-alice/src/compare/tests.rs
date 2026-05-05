use super::*;
use eatme_core::AssertionResult;
use std::path::Path;

#[test]
fn manifest_only_comparison_writes_bounded_manifest() {
    let root = unique_test_dir("manifest-only-comparison");
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
        registry_path: registry_path.clone(),
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("real-alice-launch-smoke"),
        run_id: "comparison-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: false,
    })
    .unwrap();

    assert_eq!(manifest.schema_version, "eatme.alice-comparison/v1");
    assert_eq!(
        manifest.comparison_contract.schema_version,
        "eatme.alice-comparison-contract/v1"
    );
    assert_contract_contains(
        &manifest.comparison_contract.inputs,
        "baseline and modernized Alice targets",
    );
    assert_contract_contains(
        &manifest.comparison_contract.outputs,
        "comparison manifest is written",
    );
    assert_contract_contains(
        &manifest.comparison_contract.functionality_rules,
        "matched means target statuses",
    );
    assert_contract_contains(
        &manifest.comparison_contract.functionality_rules,
        "failed display_responsive assertions",
    );
    assert_contract_contains(
        &manifest.comparison_contract.timing_rules,
        "repeated same-machine samples",
    );
    assert_contract_contains(
        &manifest.comparison_contract.non_claims,
        "does not automate full Alice lesson creation and consumption",
    );
    assert_contract_contains(
        &manifest.comparison_contract.next_capabilities,
        "instructor creates an assignment",
    );
    assert!(!manifest.execute_requested);
    assert_eq!(manifest.diff.baseline_status, "not_run");
    assert_eq!(manifest.diff.modernized_status, "not_run");
    assert_eq!(manifest.scorecard.execution_mode, "manifest_only");
    assert_eq!(manifest.scorecard.functionality_result, "not_measured");
    assert_eq!(manifest.scorecard.timing_result, "not_measured");
    assert_eq!(manifest.scorecard.modernized_minus_baseline_ms, None);
    assert!(Path::new(&manifest.comparison_manifest_path).is_file());
}

#[test]
fn execute_records_blocked_targets_when_homes_are_missing() {
    let root = unique_test_dir("missing-home-comparison");
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
    alice_home_env: EATME_TEST_BASELINE_HOME_NOT_SET
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home_env: EATME_TEST_MODERNIZED_HOME_NOT_SET
"#,
    )
    .unwrap();

    let manifest = run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("real-alice-launch-smoke"),
        run_id: "missing-home-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
    })
    .unwrap();

    for role in ["baseline", "modernized"] {
        let target = manifest.targets.get(role).unwrap();
        assert_eq!(target.status, "blocked");
        assert_eq!(
            target.failure_category.as_deref(),
            Some("alice_home_unresolved")
        );
        assert!(target.launch_manifest.is_none());
    }
    assert_eq!(manifest.scorecard.functionality_result, "incomplete");
    assert_eq!(manifest.scorecard.timing_result, "incomplete");
}

#[test]
fn execute_blocks_before_launch_when_required_target_paths_are_missing() {
    let root = unique_test_dir("missing-required-path-comparison");
    let registry_path = root.join("targets.yaml");
    let baseline_home = root.join("baseline-home");
    let modernized_home = root.join("modernized-home");
    fs::create_dir_all(&baseline_home).unwrap();
    fs::create_dir_all(&modernized_home).unwrap();
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
    required_paths:
      - tweedle-lang/Grammar/TweedleLexer.g4
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
    required_paths:
      - tweedle-lang/Grammar/TweedleLexer.g4
"#,
            baseline_home.display(),
            modernized_home.display()
        ),
    )
    .unwrap();

    let manifest = run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        scenario: LaunchSmokeScenario::new("real-alice-launch-smoke"),
        run_id: "missing-required-path-run".into(),
        runs_dir: root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
    })
    .unwrap();

    for role in ["baseline", "modernized"] {
        let target = manifest.targets.get(role).unwrap();
        assert_eq!(target.status, "blocked");
        assert_eq!(
            target.failure_category.as_deref(),
            Some("target_required_path_missing")
        );
        assert!(
            target
                .detail
                .contains("tweedle-lang/Grammar/TweedleLexer.g4")
        );
        assert!(target.launch_manifest.is_none());
    }
    assert_eq!(manifest.scorecard.functionality_result, "incomplete");
    assert_eq!(manifest.scorecard.timing_result, "incomplete");
}

#[test]
fn registry_rejects_required_paths_outside_alice_home() {
    let root = unique_test_dir("bad-required-path-comparison");
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: ./alice-modernized
    required_paths:
      - ../outside
"#,
    )
    .unwrap();

    let error = read_target_registry(&registry_path).unwrap_err();

    assert!(error.to_string().contains("required_paths"));
}

#[test]
fn registry_rejects_unknown_schema_version() {
    let root = unique_test_dir("bad-schema-comparison");
    let registry_path = root.join("targets.yaml");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v0
targets: {}
"#,
    )
    .unwrap();

    let error = read_target_registry(&registry_path).unwrap_err();

    assert!(error.to_string().contains("unsupported"));
}

#[test]
fn status_diff_records_changed_assertions() {
    let mut targets = BTreeMap::new();
    targets.insert(
        "baseline".into(),
        target_run_with_assertion("baseline", "passed", None, true),
    );
    targets.insert(
        "modernized".into(),
        target_run_with_assertion("modernized", "failed", Some("fatal_log"), false),
    );

    let diff = compare_status_and_assertions(&targets);

    assert!(diff.status_changed);
    assert!(diff.failure_category_changed);
    assert_eq!(diff.assertion_diffs.len(), 1);
    assert_eq!(diff.assertion_diffs[0].assertion, "no_fatal_logs");
    assert!(diff.assertion_diffs[0].baseline.as_ref().unwrap().passed);
    assert!(!diff.assertion_diffs[0].modernized.as_ref().unwrap().passed);
}

#[test]
fn scorecard_summarizes_functionality_and_timing_evidence() {
    let mut targets = BTreeMap::new();
    let mut baseline = target_run_with_assertion("baseline", "passed", None, true);
    baseline.duration_ms = 12;
    let mut modernized = target_run_with_assertion("modernized", "passed", None, true);
    modernized.duration_ms = 7;
    targets.insert("baseline".into(), baseline);
    targets.insert("modernized".into(), modernized);
    let diff = compare_status_and_assertions(&targets);

    let scorecard = build_scorecard(true, &targets, &diff);

    assert_eq!(scorecard.execution_mode, "execute_requested");
    assert_eq!(scorecard.functionality_result, "matched");
    assert_eq!(scorecard.timing_result, "modernized_faster");
    assert_eq!(scorecard.baseline_duration_ms, Some(12));
    assert_eq!(scorecard.modernized_duration_ms, Some(7));
    assert_eq!(scorecard.modernized_minus_baseline_ms, Some(-5));
    assert_eq!(scorecard.faster_target.as_deref(), Some("modernized"));
}

#[test]
fn passing_display_responsive_details_do_not_create_functionality_difference() {
    let mut targets = BTreeMap::new();
    let mut baseline = target_run_with_assertion("baseline", "passed", None, true);
    baseline
        .launch_manifest
        .as_mut()
        .unwrap()
        .assertions
        .insert(
            "display_responsive".into(),
            AssertionResult::pass(":99 responds to xdpyinfo"),
        );
    let mut modernized = target_run_with_assertion("modernized", "passed", None, true);
    modernized
        .launch_manifest
        .as_mut()
        .unwrap()
        .assertions
        .insert(
            "display_responsive".into(),
            AssertionResult::pass(":100 responds to xdpyinfo"),
        );
    targets.insert("baseline".into(), baseline);
    targets.insert("modernized".into(), modernized);

    let diff = compare_status_and_assertions(&targets);
    let scorecard = build_scorecard(true, &targets, &diff);

    assert!(diff.assertion_diffs.is_empty());
    assert_eq!(scorecard.functionality_result, "matched");
}

#[test]
fn failed_display_responsive_assertions_still_create_functionality_difference() {
    let mut targets = BTreeMap::new();
    let mut baseline = target_run_with_assertion("baseline", "passed", None, true);
    baseline
        .launch_manifest
        .as_mut()
        .unwrap()
        .assertions
        .insert(
            "display_responsive".into(),
            AssertionResult::pass(":99 responds to xdpyinfo"),
        );
    let mut modernized = target_run_with_assertion("modernized", "passed", None, true);
    modernized
        .launch_manifest
        .as_mut()
        .unwrap()
        .assertions
        .insert(
            "display_responsive".into(),
            AssertionResult::fail(":100 did not respond to xdpyinfo"),
        );
    targets.insert("baseline".into(), baseline);
    targets.insert("modernized".into(), modernized);

    let diff = compare_status_and_assertions(&targets);
    let scorecard = build_scorecard(true, &targets, &diff);

    assert_eq!(diff.assertion_diffs.len(), 1);
    assert_eq!(diff.assertion_diffs[0].assertion, "display_responsive");
    assert_eq!(scorecard.functionality_result, "different");
}

fn assert_contract_contains(entries: &[String], expected: &str) {
    assert!(
        entries.iter().any(|entry| entry.contains(expected)),
        "contract entries should contain {expected:?}: {entries:?}"
    );
}

fn target_run_with_assertion(
    role: &str,
    status: &str,
    failure_category: Option<&str>,
    assertion_passed: bool,
) -> ComparisonTargetRun {
    let mut assertions = BTreeMap::new();
    assertions.insert(
        "no_fatal_logs".into(),
        if assertion_passed {
            AssertionResult::pass("clean")
        } else {
            AssertionResult::fail("fatal log detected")
        },
    );
    ComparisonTargetRun {
        role: role.into(),
        target_id: role.into(),
        label: role.into(),
        description: role.into(),
        metadata: BTreeMap::new(),
        notes: Vec::new(),
        alice_home_env: None,
        required_paths: Vec::new(),
        resolved_alice_home: None,
        alice_home_source: None,
        run_id: role.into(),
        started_at_unix_ms: 1,
        finished_at_unix_ms: 2,
        duration_ms: 1,
        status: status.into(),
        detail: String::new(),
        failure_category: failure_category.map(str::to_string),
        launch_manifest: Some(LaunchSmokeManifest {
            schema_version: "eatme.launch-smoke/v1".into(),
            scenario_id: "real-alice-launch-smoke".into(),
            run_id: role.into(),
            alice_home: String::new(),
            alice_git_commit: String::new(),
            eatme_git_commit: String::new(),
            java_version: String::new(),
            maven_version: String::new(),
            dependency_checks: BTreeMap::new(),
            build_command: String::new(),
            build_exit_status: None,
            launch_command: String::new(),
            display: String::new(),
            xvfb_pid: None,
            alice_pid: None,
            timeout_seconds: 1,
            window_list: None,
            window_list_error: None,
            screenshot: None,
            screenshot_error: None,
            ui_action_contract: None,
            log: None,
            log_error: None,
            fatal_log_scan: Vec::new(),
            assertions,
            failure_category: failure_category.map(str::to_string),
        }),
        launch_manifest_artifact: None,
    }
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-alice-comparison-tests")
        .join(format!("{prefix}-{}", now_ms()))
}
