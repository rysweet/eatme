use super::*;
use crate::compare::desktop_evidence::FirstLessonEvidenceBoundary;
use crate::compare::first_lesson::FIRST_LESSON_SCENARIO_ID;
use crate::compare::{
    DesktopProofContract, LessonActionAssertionEvidence, LessonReadinessEvidenceProgress,
    LessonSessionContractCheck, LessonSessionReadinessEnvelope, LessonSessionReadinessReport,
    LessonTargetEvidence, OriginalAliceActionEvidenceReport,
};

// ── Expected canonical step IDs in curriculum order ──────────────────
const EXPECTED_STEP_IDS: &[&str] = &[
    "verify-specific-alice-window",
    "activate-specific-alice-window",
    "select-project",
    "place-object",
    "edit-procedure-or-code-block",
    "run-world",
    "save-project",
    "visible-rendering",
    "grading",
    "creative-assessment",
    "first-lesson-completion",
];

const EXPECTED_STEP_NAMES: &[&str] = &[
    "Verify specific Alice window",
    "Activate specific Alice window",
    "Select project",
    "Place object",
    "Edit procedure or code block",
    "Run world",
    "Save project",
    "Visible rendering",
    "Grading",
    "Creative assessment",
    "First-lesson completion",
];

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn report_contains_exactly_eleven_steps() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        report.steps.len(),
        11,
        "grading report must have 11 canonical steps"
    );
}

#[test]
fn schema_version_is_correct() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        report.schema_version,
        "eatme.first-lesson-grading-report/v1"
    );
}

#[test]
fn scenario_id_is_preserved_from_readiness_report() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    assert_eq!(report.scenario_id, FIRST_LESSON_SCENARIO_ID);
}

#[test]
fn step_ids_match_expected_canonical_order() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    let ids: Vec<&str> = report.steps.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids, EXPECTED_STEP_IDS,
        "step IDs must match canonical order"
    );
}

#[test]
fn step_names_match_expected_values() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names, EXPECTED_STEP_NAMES,
        "step names must match expected values"
    );
}

#[test]
fn meta_boundaries_always_not_yet_tested_even_with_full_evidence() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);

    for meta_id in ["grading", "creative-assessment", "first-lesson-completion"] {
        let step = find_step(&report, meta_id);
        assert_eq!(
            step.status,
            GradingStepStatus::NotYetTested,
            "meta-boundary {meta_id} must always be not_yet_tested, got {:?}",
            step.status
        );
    }
}

#[test]
fn meta_boundaries_not_yet_tested_when_evidence_empty() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    for meta_id in ["grading", "creative-assessment", "first-lesson-completion"] {
        let step = find_step(&report, meta_id);
        assert_eq!(
            step.status,
            GradingStepStatus::NotYetTested,
            "meta-boundary {meta_id} must always be not_yet_tested"
        );
    }
}

#[test]
fn ui_action_passed_maps_to_ready() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);

    let step = find_step(&report, "verify-specific-alice-window");
    assert_eq!(
        step.status,
        GradingStepStatus::Ready,
        "UI action with passed assertion must map to ready"
    );
    let step = find_step(&report, "activate-specific-alice-window");
    assert_eq!(step.status, GradingStepStatus::Ready);
}

#[test]
fn ui_action_not_passed_maps_to_blocked() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    for action_id in [
        "verify-specific-alice-window",
        "activate-specific-alice-window",
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project",
    ] {
        let step = find_step(&report, action_id);
        assert_eq!(
            step.status,
            GradingStepStatus::Blocked,
            "UI action {action_id} without evidence must be blocked"
        );
    }
}

#[test]
fn boundary_present_maps_to_ready() {
    let readiness = readiness_with_all_evidence();
    let report = first_lesson_grading_report(&readiness);

    let step = find_step(&report, "select-project");
    assert_eq!(
        step.status,
        GradingStepStatus::Ready,
        "boundary with status present must map to ready"
    );
    let step = find_step(&report, "visible-rendering");
    assert_eq!(
        step.status,
        GradingStepStatus::Ready,
        "boundary with status present must map to ready"
    );
}

#[test]
fn boundary_missing_maps_to_blocked() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    let step = find_step(&report, "select-project");
    assert_eq!(
        step.status,
        GradingStepStatus::Blocked,
        "boundary with missing status must map to blocked"
    );
    let step = find_step(&report, "visible-rendering");
    assert_eq!(
        step.status,
        GradingStepStatus::Blocked,
        "boundary with missing status must map to blocked"
    );
}

#[test]
fn mixed_evidence_produces_correct_statuses() {
    let mut readiness = empty_readiness_report();
    readiness.target_evidence = vec![target_evidence_with_actions(&[
        ("verify-specific-alice-window", true),
        ("activate-specific-alice-window", true),
        ("place-object", false),
        ("edit-procedure-or-code-block", false),
        ("run-world", false),
        ("save-project", false),
    ])];
    readiness.evidence_boundaries = vec![
        test_boundary("select_project", "present"),
        test_boundary("visible_rendering", "missing"),
        test_boundary("grading", "missing"),
        test_boundary("creative_assessment", "missing"),
        test_boundary("first_lesson_completion", "missing"),
    ];

    let report = first_lesson_grading_report(&readiness);

    assert_eq!(
        find_step(&report, "verify-specific-alice-window").status,
        GradingStepStatus::Ready
    );
    assert_eq!(
        find_step(&report, "activate-specific-alice-window").status,
        GradingStepStatus::Ready
    );
    assert_eq!(
        find_step(&report, "select-project").status,
        GradingStepStatus::Ready
    );
    assert_eq!(
        find_step(&report, "place-object").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "edit-procedure-or-code-block").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "run-world").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "save-project").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "visible-rendering").status,
        GradingStepStatus::Blocked
    );
    assert_eq!(
        find_step(&report, "grading").status,
        GradingStepStatus::NotYetTested
    );
    assert_eq!(
        find_step(&report, "creative-assessment").status,
        GradingStepStatus::NotYetTested
    );
    assert_eq!(
        find_step(&report, "first-lesson-completion").status,
        GradingStepStatus::NotYetTested
    );
}

#[test]
fn all_non_meta_steps_blocked_when_no_evidence() {
    let readiness = empty_readiness_report();
    let report = first_lesson_grading_report(&readiness);

    for step in &report.steps {
        match step.id.as_str() {
            "grading" | "creative-assessment" | "first-lesson-completion" => {
                assert_eq!(step.status, GradingStepStatus::NotYetTested);
            }
            _ => {
                assert_eq!(
                    step.status,
                    GradingStepStatus::Blocked,
                    "step {} must be blocked when no evidence is present",
                    step.id
                );
            }
        }
    }
}

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

// ── Helpers ──────────────────────────────────────────────────────────

fn find_step<'a>(report: &'a FirstLessonGradingReport, id: &str) -> &'a GradingStep {
    report
        .steps
        .iter()
        .find(|s| s.id == id)
        .unwrap_or_else(|| panic!("step {id} not found in report"))
}

fn empty_readiness_report() -> LessonSessionReadinessReport {
    let envelope = LessonSessionReadinessEnvelope {
        scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
        role: "student".into(),
        status: "not_ready".into(),
        blocked_reason: None,
        human_summary: "no evidence".into(),
        required_evidence: Vec::new(),
        no_go_contracts: Vec::new(),
    };
    LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: "test-manifest.json".into(),
        scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
        passed: false,
        status: "not_ready".into(),
        readiness_status: "incomplete".into(),
        blocked_reason: None,
        human_summary: "no evidence".into(),
        evidence_gap_message: None,
        desktop_proof_contract: DesktopProofContract {
            status: "not_launched".into(),
            reason_code: "no_evidence".into(),
            detail: "no evidence".into(),
            target_role: "modernized".into(),
            artifact: None,
        },
        shown_evidence: Vec::new(),
        not_yet_shown: Vec::new(),
        desktop_next_action: None,
        original_alice_action_evidence: OriginalAliceActionEvidenceReport::missing(),
        unproven_claims: Vec::new(),
        evidence_progress: LessonReadinessEvidenceProgress {
            total_required: 0,
            present: 0,
            missing: 0,
            invalid: 0,
            not_observed: 0,
            blocked: 0,
            summary: "0 of 0".into(),
            next_actionable_blocker: None,
            next_missing_real_desktop_proof: None,
            items: Vec::new(),
        },
        evidence_boundaries: Vec::new(),
        required_evidence: Vec::new(),
        no_go_contracts: Vec::new(),
        lesson_session_readiness: envelope.clone(),
        role_readiness: vec![envelope],
        contract_check: LessonSessionContractCheck {
            schema_version: "eatme.alice-lesson-session-check/v1".into(),
            manifest_path: "test-manifest.json".into(),
            scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
            session_kind: Some("first_lesson_action_contract".into()),
            automation_status: Some("blocked".into()),
            passed: false,
            issues: Vec::new(),
        },
        execute_requested: None,
        target_evidence: Vec::new(),
        issues: Vec::new(),
        limitations: Vec::new(),
    }
}

fn readiness_with_all_evidence() -> LessonSessionReadinessReport {
    let mut report = empty_readiness_report();
    report.target_evidence = vec![target_evidence_with_actions(&[
        ("verify-specific-alice-window", true),
        ("activate-specific-alice-window", true),
        ("place-object", true),
        ("edit-procedure-or-code-block", true),
        ("run-world", true),
        ("save-project", true),
    ])];
    report.evidence_boundaries = vec![
        test_boundary("select_project", "present"),
        test_boundary("visible_rendering", "present"),
        test_boundary("grading", "present"),
        test_boundary("creative_assessment", "present"),
        test_boundary("first_lesson_completion", "present"),
    ];
    report
}

fn target_evidence_with_actions(actions: &[(&str, bool)]) -> LessonTargetEvidence {
    let action_assertions = actions
        .iter()
        .map(|(action_id, passed)| LessonActionAssertionEvidence {
            assertion_id: format!("{action_id}_assertion"),
            action_id: (*action_id).into(),
            passed: *passed,
            detail: if *passed {
                "passed".into()
            } else {
                "blocked".into()
            },
        })
        .collect();

    LessonTargetEvidence {
        role: "baseline".into(),
        target_id: Some("baseline".into()),
        target_status: Some("passed".into()),
        failure_category: None,
        launch_manifest_present: true,
        ui_action_contract_path: None,
        ui_action_contract_readable: false,
        desktop_run_pixel_boundary: None,
        desktop_run_pixel_observation: None,
        desktop_first_lesson_next_action: None,
        action_assertions,
        required_actions: Vec::new(),
        missing_assertions: Vec::new(),
        missing_required_actions: Vec::new(),
        blockers: Vec::new(),
        no_go_contracts: Vec::new(),
    }
}

fn test_boundary(id: &str, status: &str) -> FirstLessonEvidenceBoundary {
    FirstLessonEvidenceBoundary {
        id: id.into(),
        label: format!("{id} boundary"),
        status: status.into(),
        source: "test".into(),
        metadata_state: status.into(),
        detail: format!("{id} is {status}"),
        claim: "test claim".into(),
        does_not_prove: Vec::new(),
        artifact: None,
    }
}
