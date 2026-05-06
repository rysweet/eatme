use super::*;
use std::path::Path;

mod lesson_session_helpers;
use lesson_session_helpers::{
    assert_contract_contains, ui_action_contract_json, unique_test_dir,
    write_executable_blocked_first_lesson_manifest, write_first_lesson_manifest,
};

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
    assert_eq!(
        (
            report.status.as_str(),
            report.readiness_status.as_str(),
            report.lesson_session_readiness.status.as_str()
        ),
        ("not_ready", "incomplete", "not_ready")
    );
    assert_contract_contains(&report.issues, "must be produced with --execute");
    assert_contract_contains(&report.issues, "missing embedded launch_manifest");
}

#[test]
fn lesson_session_readiness_consumes_ui_action_contract_artifacts() {
    let root = unique_test_dir("executable-readiness-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(
        (
            report.status.as_str(),
            report.readiness_status.as_str(),
            report.blocked_reason.as_deref(),
            report.lesson_session_readiness.status.as_str()
        ),
        (
            "blocked",
            "blocked_until_ui_automation",
            Some("blocked_until_ui_automation"),
            "blocked"
        )
    );
    assert_contract_contains(&report.required_evidence, "comparison-manifest.json");
    assert_contract_contains(&report.required_evidence, "ui-action-contract.json");
    for role in ["instructor", "student"] {
        let readiness = report
            .role_readiness
            .iter()
            .find(|readiness| readiness.role == role)
            .unwrap_or_else(|| panic!("missing {role} readiness: {:?}", report.role_readiness));
        assert_eq!(readiness.status, "blocked");
        assert_eq!(
            readiness.blocked_reason.as_deref(),
            Some("blocked_until_ui_automation")
        );
        assert_contract_contains(&readiness.required_evidence, "ui-action-contract.json");
    }
    for affordance in [
        "object_placement",
        "procedure_edit",
        "world_run",
        "project_save",
    ] {
        assert_no_go_affordance(&report.no_go_contracts, affordance);
    }
    assert_eq!(report.execute_requested, Some(true));
    assert_eq!(report.target_evidence.len(), 2);
    for target in &report.target_evidence {
        assert!(target.launch_manifest_present);
        assert!(target.ui_action_contract_readable);
        assert!(target.missing_assertions.is_empty());
        assert!(target.missing_required_actions.is_empty());
        for affordance in [
            "object_placement",
            "procedure_edit",
            "world_run",
            "project_save",
        ] {
            assert_no_go_affordance(&target.no_go_contracts, affordance);
        }
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
fn lesson_session_readiness_rejects_missing_required_action_no_go_contract() {
    let root = unique_test_dir("missing-action-no-go-contract-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let contract_path =
        value["targets"]["baseline"]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap()
            .to_string();
    let mut contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();
    let action = contract["required_actions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|action| {
            action.get("id").and_then(serde_json::Value::as_str)
                == Some("edit-procedure-or-code-block")
        })
        .unwrap();
    action.as_object_mut().unwrap().remove("decision");
    action.as_object_mut().unwrap().remove("contract_required");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contract_contains(
        &report.issues,
        "edit-procedure-or-code-block must carry a no-go contract",
    );
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
    assert_eq!(report.status, "not_ready");
    assert_eq!(report.readiness_status, "incomplete");
    for role in ["instructor", "student"] {
        let readiness = report
            .role_readiness
            .iter()
            .find(|readiness| readiness.role == role)
            .unwrap_or_else(|| panic!("missing {role} readiness: {:?}", report.role_readiness));
        assert_eq!(readiness.status, "not_ready");
    }
    assert!(Path::new(&report.comparison_manifest_path).is_file());
    assert_contract_contains(&report.issues, "must be produced with --execute");
    assert_contract_contains(&report.limitations, "does not grade student worlds");
}

fn assert_no_go_affordance(contracts: &[LessonSessionNoGoContract], affordance: &str) {
    assert!(
        contracts.iter().any(|contract| {
            contract.affordance == affordance
                && contract.decision == "no_go"
                && contract
                    .missing_affordance_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("deterministic-alice-"))
        }),
        "missing {affordance} no-go contract: {contracts:?}"
    );
}
