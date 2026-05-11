use crate::grading_report::{GradingInput, StepStatus, grade_first_lesson_readiness};
use std::path::Path;

fn repository_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn grade_committed_assets_produces_valid_report() {
    let root = repository_root();
    let asset_report = crate::validate_assets(&root).unwrap();

    let input = GradingInput {
        assets_valid: asset_report.passed,
        asset_reason: if asset_report.passed {
            format!(
                "All {} scenario assets passed validation",
                asset_report.scenario_asset_count
            )
        } else {
            format!("{} errors found", asset_report.errors.len())
        },
        deps_available: false,
        deps_reason: "Dependencies not checked in this test".into(),
    };

    let report = grade_first_lesson_readiness(input);

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "building-a-scene-first-world");
    assert_eq!(report.steps.len(), 6);
    // Committed assets should pass validation
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[0].name, "validate-assets");
    // Deps are explicitly blocked in this test
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].name, "check-dependencies");
    // Launch smoke should be blocked because deps are blocked
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert_eq!(report.steps[2].name, "launch-smoke");
    // Interaction steps should be blocked (upstream is blocked)
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert_eq!(report.steps[3].name, "place-object");
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert_eq!(report.steps[4].name, "edit-code");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[5].name, "run-world");
    assert!(!report.passed);
}

#[test]
fn grade_committed_assets_all_ready_path() {
    let root = repository_root();
    let asset_report = crate::validate_assets(&root).unwrap();
    assert!(
        asset_report.passed,
        "committed assets must pass: {:?}",
        asset_report.errors
    );

    let input = GradingInput {
        assets_valid: true,
        asset_reason: format!(
            "All {} scenario assets passed validation",
            asset_report.scenario_asset_count
        ),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    };

    let report = grade_first_lesson_readiness(input);

    // Precondition steps should be ready
    for step in &report.steps[..3] {
        assert_eq!(
            step.status,
            StepStatus::Ready,
            "precondition step {} should be ready",
            step.name
        );
    }
    // Interaction steps should be not-yet-tested
    for step in &report.steps[3..] {
        assert_eq!(
            step.status,
            StepStatus::NotYetTested,
            "interaction step {} should be not-yet-tested",
            step.name
        );
    }
    // Report should not pass because interaction steps are not yet tested
    assert!(!report.passed);
}

#[test]
fn grading_report_json_round_trips_cleanly() {
    let input = GradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    };
    let report = grade_first_lesson_readiness(input);
    let json = serde_json::to_string_pretty(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(parsed["lesson"], "building-a-scene-first-world");
    assert!(parsed["passed"].is_boolean());
    assert!(parsed["steps"].is_array());

    for step in parsed["steps"].as_array().unwrap() {
        assert!(step["name"].is_string());
        assert!(step["status"].is_string());
        assert!(step["reason"].is_string());
        assert!(step["depends_on"].is_array(), "depends_on must be an array");
        let status = step["status"].as_str().unwrap();
        assert!(
            ["ready", "blocked", "not-yet-tested"].contains(&status),
            "unexpected status: {status}"
        );
    }
}

#[test]
fn grading_report_schema_version_follows_eatme_pattern() {
    let report = grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
    });
    assert!(
        report.schema_version.starts_with("eatme.assets/"),
        "schema_version should start with eatme.assets/"
    );
    assert!(
        report.schema_version.ends_with("/v1"),
        "schema_version should end with /v1"
    );
}
