use super::*;
use eatme_core::AssertionResult;
use regex::Regex;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    assert_eq!(
        manifest.lesson_session_contract.schema_version,
        "eatme.alice-lesson-session-contract/v1"
    );
    assert_eq!(
        manifest.lesson_session_contract.automation_status,
        "launch_smoke_only"
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
fn validate_id_rejects_empty_uppercase_and_trailing_dash() {
    assert!(validate_id("target id", "").is_err());
    assert!(validate_id("target id", "Uppercase").is_err());
    assert!(validate_id("target id", "trailing-").is_err());
    assert!(validate_id("target id", "valid-id-1").is_ok());
}

#[test]
fn resolve_alice_home_prefers_override_then_registry_then_env() {
    let registry_path = PathBuf::from("/registry/home");
    let override_path = PathBuf::from("/override/home");
    let env_path =
        PathBuf::from(std::env::var_os("HOME").expect("HOME should be available during tests"));
    let target = AliceTargetDefinition {
        label: "Target".into(),
        description: "Comparison target".into(),
        alice_home: Some(registry_path.clone()),
        alice_home_env: Some("HOME".into()),
        required_paths: Vec::new(),
        metadata: std::collections::BTreeMap::new(),
        notes: Vec::new(),
    };

    assert_eq!(
        resolve_alice_home(&target, Some(&override_path)),
        Some((override_path, "cli_override".into()))
    );
    assert_eq!(
        resolve_alice_home(&target, None),
        Some((registry_path, "registry".into()))
    );

    let env_target = AliceTargetDefinition {
        alice_home: None,
        ..target.clone()
    };
    assert_eq!(
        resolve_alice_home(&env_target, None),
        Some((env_path, "env:HOME".into()))
    );

    let missing_env_target = AliceTargetDefinition {
        alice_home: None,
        alice_home_env: Some("EATME_TEST_UNSET_HOME".into()),
        ..target
    };
    assert_eq!(resolve_alice_home(&missing_env_target, None), None);
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

#[test]
fn identical_programs_keep_the_same_comparison_score() {
    let baseline =
        target_run_with_feature_score("baseline", &["place_object", "edit_code", "run_world"]);
    let modernized =
        target_run_with_feature_score("modernized", &["place_object", "edit_code", "run_world"]);

    assert_eq!(feature_score(&baseline), feature_score(&modernized));
}

#[test]
fn additional_modernized_features_improve_its_score_vs_baseline() {
    let baseline = target_run_with_feature_score("baseline", &["place_object", "edit_code"]);
    let modernized = target_run_with_feature_score(
        "modernized",
        &["place_object", "edit_code", "run_world", "save_project"],
    );

    assert!(feature_score(&modernized) > feature_score(&baseline));
}

#[test]
fn removing_features_from_modernized_shows_regression_vs_baseline() {
    let baseline = target_run_with_feature_score(
        "baseline",
        &["place_object", "edit_code", "run_world", "save_project"],
    );
    let modernized = target_run_with_feature_score("modernized", &["place_object"]);

    assert!(feature_score(&modernized) < feature_score(&baseline));
}

#[test]
fn real_a3p_fixture_same_version_produces_no_comparison_diff() {
    let fixture = real_fixture_path("amazonMinimum.a3p");
    let mut targets = BTreeMap::new();
    targets.insert(
        "baseline".into(),
        target_run_with_fixture_signature("baseline", &fixture),
    );
    targets.insert(
        "modernized".into(),
        target_run_with_fixture_signature("modernized", &fixture),
    );

    let diff = compare_status_and_assertions(&targets);

    assert!(!diff.status_changed);
    assert!(diff.assertion_diffs.is_empty());
}

#[test]
fn real_a3p_fixture_variant_registers_diff_for_same_project() {
    let baseline_fixture = real_fixture_path("amazonMinimum.a3p");
    let variant_fixture = write_fixture_variant_copy("amazonMinimum.a3p");
    let mut targets = BTreeMap::new();
    targets.insert(
        "baseline".into(),
        target_run_with_fixture_signature("baseline", &baseline_fixture),
    );
    targets.insert(
        "modernized".into(),
        target_run_with_fixture_signature("modernized", &variant_fixture),
    );

    let diff = compare_status_and_assertions(&targets);

    assert_eq!(diff.assertion_diffs.len(), 1);
    let assertion_diff = &diff.assertion_diffs[0];
    assert_eq!(assertion_diff.assertion, "project_fixture_signature");
    assert_eq!(
        assertion_diff
            .baseline
            .as_ref()
            .map(|snapshot| snapshot.passed),
        Some(true)
    );
    assert_eq!(
        assertion_diff
            .modernized
            .as_ref()
            .map(|snapshot| snapshot.passed),
        Some(true)
    );
    assert_ne!(
        assertion_diff.baseline.as_ref().unwrap().detail,
        assertion_diff.modernized.as_ref().unwrap().detail
    );
    assert!(
        assertion_diff
            .baseline
            .as_ref()
            .unwrap()
            .detail
            .contains("methods=")
    );
    assert!(
        assertion_diff
            .modernized
            .as_ref()
            .unwrap()
            .detail
            .contains("comparisonVariant")
    );
}

fn assert_contract_contains(entries: &[String], expected: &str) {
    assert!(
        entries.iter().any(|entry| entry.contains(expected)),
        "contract entries should contain {expected:?}: {entries:?}"
    );
}

fn feature_score(target: &ComparisonTargetRun) -> usize {
    target
        .launch_manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .assertions
                .values()
                .filter(|result| result.passed)
                .count()
        })
        .unwrap_or(0)
}

fn target_run_with_feature_score(role: &str, features: &[&str]) -> ComparisonTargetRun {
    let mut target = target_run_with_assertion(role, "passed", None, true);
    let assertions = &mut target.launch_manifest.as_mut().unwrap().assertions;
    assertions.clear();
    for feature in features {
        assertions.insert((*feature).into(), AssertionResult::pass("feature present"));
    }
    target
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
            post_focus_screenshot: None,
            post_focus_screenshot_error: None,
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

fn real_fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real")
        .join(name)
}

fn target_run_with_fixture_signature(role: &str, fixture: &Path) -> ComparisonTargetRun {
    let mut target = target_run_with_assertion(role, "passed", None, true);
    let assertions = &mut target.launch_manifest.as_mut().unwrap().assertions;
    assertions.clear();
    assertions.insert(
        "project_fixture_signature".into(),
        AssertionResult::pass(real_fixture_signature(fixture)),
    );
    target
}

fn real_fixture_signature(path: &Path) -> String {
    let xml = extract_fixture_xml(path);
    let method_names = user_method_name_regex()
        .captures_iter(&xml)
        .filter_map(|captures| {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|value| value.as_str().to_string())
        })
        .collect::<Vec<_>>();
    format!(
        "methods={};names={}",
        method_names.len(),
        method_names.join("|")
    )
}

fn extract_fixture_xml(path: &Path) -> String {
    let file =
        fs::File::open(path).unwrap_or_else(|err| panic!("open fixture {}: {err}", path.display()));
    let mut archive = zip::ZipArchive::new(file)
        .unwrap_or_else(|err| panic!("read fixture {} as zip: {err}", path.display()));
    let mut xml = String::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name().ends_with(".xml") {
            let mut content = String::new();
            entry.read_to_string(&mut content).unwrap();
            xml.push_str(&content);
            xml.push('\n');
        }
    }
    xml
}

fn write_fixture_variant_copy(name: &str) -> PathBuf {
    let source = real_fixture_path(name);
    let root = unique_test_dir("real-a3p-fixture-variant");
    fs::create_dir_all(&root).unwrap();
    let output = root.join(format!("variant-{name}"));
    let source_file = fs::File::open(&source)
        .unwrap_or_else(|err| panic!("open source fixture {}: {err}", source.display()));
    let mut archive = zip::ZipArchive::new(source_file)
        .unwrap_or_else(|err| panic!("read source fixture {} as zip: {err}", source.display()));
    let output_file = fs::File::create(&output)
        .unwrap_or_else(|err| panic!("create variant fixture {}: {err}", output.display()));
    let mut writer = zip::ZipWriter::new(output_file);
    let options = zip::write::SimpleFileOptions::default();
    let mut mutated = false;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let entry_name = entry.name().to_string();
        if entry_name.ends_with('/') {
            writer.add_directory(entry_name, options).unwrap();
            continue;
        }

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if !mutated && entry_name.ends_with(".xml") {
            let xml = String::from_utf8(bytes).unwrap();
            if let Some(next_xml) = rename_first_user_method(&xml) {
                bytes = next_xml.into_bytes();
                mutated = true;
            } else {
                bytes = xml.into_bytes();
            }
        }

        writer.start_file(entry_name, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
    assert!(mutated, "expected fixture variant to rename a user method");
    output
}

fn rename_first_user_method(xml: &str) -> Option<String> {
    let captures = user_method_name_regex().captures(xml)?;
    let name_match = captures.get(1).or_else(|| captures.get(2))?;
    let mut mutated = String::with_capacity(xml.len() + "comparisonVariant".len());
    mutated.push_str(&xml[..name_match.start()]);
    mutated.push_str(name_match.as_str());
    mutated.push_str("comparisonVariant");
    mutated.push_str(&xml[name_match.end()..]);
    Some(mutated)
}

fn user_method_name_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)type\s*=\s*"(?:[^"]+\.)?UserMethod"[^>]*?(?:name\s*=\s*"([^"]+)"|.*?<property\s+name\s*=\s*"name">\s*<value[^>]*>([^<]+)</value>)"#,
        )
        .unwrap()
    })
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-alice-comparison-tests")
        .join(format!("{prefix}-{}", now_ms()))
}
