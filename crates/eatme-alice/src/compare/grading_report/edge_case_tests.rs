use super::tests::{
    empty_readiness_report, find_step, readiness_with_all_evidence, target_evidence_with_actions,
    test_boundary,
};
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

// ── Display trait contract ───────────────────────────────────────────

#[test]
fn display_ready_is_lowercase() {
    assert_eq!(GradingStepStatus::Ready.to_string(), "ready");
}

#[test]
fn display_blocked_is_lowercase() {
    assert_eq!(GradingStepStatus::Blocked.to_string(), "blocked");
}

#[test]
fn display_not_yet_tested_is_snake_case() {
    assert_eq!(
        GradingStepStatus::NotYetTested.to_string(),
        "not_yet_tested"
    );
}

// ── Scenario ID fallback contract ────────────────────────────────────

#[test]
fn scenario_id_falls_back_to_default_when_none() {
    let mut readiness = empty_readiness_report();
    readiness.scenario_id = None;
    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        report.scenario_id, FIRST_LESSON_SCENARIO_ID,
        "must fall back to FIRST_LESSON_SCENARIO_ID when readiness has None"
    );
}

#[test]
fn custom_scenario_id_is_preserved() {
    let mut readiness = empty_readiness_report();
    readiness.scenario_id = Some("custom-scenario-42".into());
    let report = first_lesson_grading_report(&readiness);

    assert_eq!(report.scenario_id, "custom-scenario-42");
}

// ── Multi-target evidence resolution ─────────────────────────────────

#[test]
fn action_found_in_second_target_maps_to_ready() {
    let mut readiness = empty_readiness_report();
    readiness.target_evidence = vec![
        target_evidence_with_actions(&[]),
        target_evidence_with_actions(&[("verify-specific-alice-window", true)]),
    ];

    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        find_step(&report, "verify-specific-alice-window").status,
        GradingStepStatus::Ready,
        "action must be found across all target evidence entries"
    );
}

#[test]
fn action_failed_in_all_targets_maps_to_blocked() {
    let mut readiness = empty_readiness_report();
    readiness.target_evidence = vec![
        target_evidence_with_actions(&[("place-object", false)]),
        target_evidence_with_actions(&[("place-object", false)]),
    ];

    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        find_step(&report, "place-object").status,
        GradingStepStatus::Blocked,
        "action that fails in all targets must be blocked"
    );
}
