use super::*;

fn input_all_ready() -> GradingInput {
    GradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    }
}

fn input_blocked_assets() -> GradingInput {
    GradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    }
}

fn input_blocked_deps() -> GradingInput {
    GradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
    }
}

fn input_both_blocked() -> GradingInput {
    GradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
    }
}

// --- Schema and structure tests ---

#[test]
fn schema_version_is_grading_v1() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_building_a_scene_first_world() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.lesson, "building-a-scene-first-world");
}

#[test]
fn always_produces_three_steps() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.steps.len(), 3);
}

#[test]
fn step_names_in_order() {
    let report = grade_first_lesson_readiness(input_all_ready());
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["validate-assets", "check-dependencies", "launch-smoke"]
    );
}

// --- All ready scenario ---

#[test]
fn all_ready_report_passes() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert!(
        report.passed,
        "report should pass when all inputs are valid"
    );
}

#[test]
fn all_ready_validate_assets_is_ready() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.steps[0].status, StepStatus::Ready);
}

#[test]
fn all_ready_check_dependencies_is_ready() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.steps[1].status, StepStatus::Ready);
}

#[test]
fn all_ready_launch_smoke_is_ready() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(report.steps[2].status, StepStatus::Ready);
}

#[test]
fn all_ready_reasons_propagate() {
    let report = grade_first_lesson_readiness(input_all_ready());
    assert_eq!(
        report.steps[0].reason,
        "All 93 scenario assets passed validation"
    );
    assert_eq!(report.steps[1].reason, "All required tools available");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_report_fails() {
    let report = grade_first_lesson_readiness(input_blocked_assets());
    assert!(!report.passed, "report should fail when assets are invalid");
}

#[test]
fn blocked_assets_validate_assets_is_blocked() {
    let report = grade_first_lesson_readiness(input_blocked_assets());
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[0].reason,
        "3 scenario assets failed validation"
    );
}

#[test]
fn blocked_assets_check_dependencies_is_ready() {
    let report = grade_first_lesson_readiness(input_blocked_assets());
    assert_eq!(report.steps[1].status, StepStatus::Ready);
}

#[test]
fn blocked_assets_launch_smoke_is_blocked() {
    let report = grade_first_lesson_readiness(input_blocked_assets());
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert!(
        report.steps[2].reason.contains("validate-assets"),
        "launch-smoke reason should mention the blocking step: {}",
        report.steps[2].reason
    );
}

// --- Blocked dependencies scenario ---

#[test]
fn blocked_deps_report_fails() {
    let report = grade_first_lesson_readiness(input_blocked_deps());
    assert!(
        !report.passed,
        "report should fail when deps are unavailable"
    );
}

#[test]
fn blocked_deps_validate_assets_is_ready() {
    let report = grade_first_lesson_readiness(input_blocked_deps());
    assert_eq!(report.steps[0].status, StepStatus::Ready);
}

#[test]
fn blocked_deps_check_dependencies_is_blocked() {
    let report = grade_first_lesson_readiness(input_blocked_deps());
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[1].reason,
        "Missing required tools: Xvfb, wmctrl"
    );
}

#[test]
fn blocked_deps_launch_smoke_is_blocked() {
    let report = grade_first_lesson_readiness(input_blocked_deps());
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert!(
        report.steps[2].reason.contains("check-dependencies"),
        "launch-smoke reason should mention the blocking step: {}",
        report.steps[2].reason
    );
}

// --- Both blocked scenario ---

#[test]
fn both_blocked_report_fails() {
    let report = grade_first_lesson_readiness(input_both_blocked());
    assert!(!report.passed, "report should fail when both are blocked");
}

#[test]
fn both_blocked_validate_assets_is_blocked() {
    let report = grade_first_lesson_readiness(input_both_blocked());
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
}

#[test]
fn both_blocked_check_dependencies_is_blocked() {
    let report = grade_first_lesson_readiness(input_both_blocked());
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
}

#[test]
fn both_blocked_launch_smoke_is_blocked() {
    let report = grade_first_lesson_readiness(input_both_blocked());
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
}

#[test]
fn both_blocked_launch_smoke_mentions_both_blockers() {
    let report = grade_first_lesson_readiness(input_both_blocked());
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

// --- JSON serialization ---

#[test]
fn step_status_serializes_as_lowercase() {
    let json = serde_json::to_string(&StepStatus::Ready).unwrap();
    assert_eq!(json, "\"ready\"");

    let json = serde_json::to_string(&StepStatus::Blocked).unwrap();
    assert_eq!(json, "\"blocked\"");

    let json = serde_json::to_string(&StepStatus::NotYetTested).unwrap();
    assert_eq!(json, "\"not-yet-tested\"");
}

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = grade_first_lesson_readiness(input_all_ready());
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "building-a-scene-first-world");
    assert!(json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 3);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
}
