use super::tests::{empty_readiness_report, find_step, readiness_with_all_evidence, test_boundary};
use super::*;
use crate::compare::first_lesson::FIRST_LESSON_SCENARIO_ID;

#[test]
fn status_values_are_closed_set() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);

    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                GradingStepStatus::Ready
                    | GradingStepStatus::Blocked
                    | GradingStepStatus::NotYetTested
            ),
            "step {} has unexpected status {:?}",
            step.id,
            step.status
        );
    }
}

#[test]
fn json_serialization_produces_snake_case_statuses() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    let steps = json["steps"].as_array().unwrap();
    for step in steps {
        let status = step["status"].as_str().unwrap();
        assert!(
            matches!(status, "ready" | "blocked" | "not_yet_tested"),
            "JSON status must be snake_case: got {status}"
        );
    }
}

#[test]
fn json_round_trip_preserves_report() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);
    let json_string = serde_json::to_string_pretty(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    assert_eq!(
        parsed["schema_version"],
        "eatme.first-lesson-grading-report/v1"
    );
    assert_eq!(parsed["scenario_id"], FIRST_LESSON_SCENARIO_ID);
    assert_eq!(parsed["steps"].as_array().unwrap().len(), 11);
}

#[test]
fn boundary_invalid_maps_to_blocked() {
    let mut readiness = empty_readiness_report();
    readiness.evidence_boundaries = vec![
        test_boundary("select_project", "invalid"),
        test_boundary("visible_rendering", "invalid"),
        test_boundary("grading", "missing"),
        test_boundary("creative_assessment", "missing"),
        test_boundary("first_lesson_completion", "missing"),
    ];

    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        find_step(&report, "select-project").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "visible-rendering").status,
        GradingStepStatus::Blocked
    );
}

#[test]
fn boundary_blocked_maps_to_blocked() {
    let mut readiness = empty_readiness_report();
    readiness.evidence_boundaries = vec![
        test_boundary("select_project", "blocked"),
        test_boundary("visible_rendering", "blocked"),
        test_boundary("grading", "missing"),
        test_boundary("creative_assessment", "missing"),
        test_boundary("first_lesson_completion", "missing"),
    ];

    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        find_step(&report, "select-project").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "visible-rendering").status,
        GradingStepStatus::Blocked
    );
}
