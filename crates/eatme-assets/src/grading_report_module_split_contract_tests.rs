//! Module-split contract tests for the grading_report → grading_report_events extraction.
//!
//! These tests validate invariants that would break if:
//! 1. The module split is reverted or re-exports are removed
//! 2. Shared precondition behavior diverges between graders
//! 3. Edge cases at module boundaries regress
//! 4. lib.rs re-export paths stop working

use crate::grading_report::{
    GradingInput, GradingReport, LoopsGradingInput, StepStatus, grade_first_lesson_readiness,
    grade_loops_and_conditionals,
};
use crate::grading_report_events::{EventsGradingInput, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};

// ── Helpers ───────────────────────────────────────────────────

fn grade_all_three(
    av: bool,
    ar: &str,
    dv: bool,
    dr: &str,
    program: Option<Program>,
) -> (GradingReport, GradingReport, GradingReport) {
    let fl = grade_first_lesson_readiness(GradingInput {
        assets_valid: av,
        asset_reason: ar.into(),
        deps_available: dv,
        deps_reason: dr.into(),
    });
    let lo = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: av,
        asset_reason: ar.into(),
        deps_available: dv,
        deps_reason: dr.into(),
        student_program: program.clone(),
    });
    let ev = grade_events_and_collision(EventsGradingInput {
        assets_valid: av,
        asset_reason: ar.into(),
        deps_available: dv,
        deps_reason: dr.into(),
        student_program: program,
    });
    (fl, lo, ev)
}

fn method_call_only_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myMethod".into(),
            body: vec![
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                },
                Statement::MethodCall {
                    object: "this.dog".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into(), "2.0".into()],
                },
            ],
        }],
    }
}

// ── Re-export path verification ───────────────────────────────

#[test]
fn lib_re_exports_events_types_and_function() {
    let input: crate::EventsGradingInput = crate::EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: None,
    };
    let report: crate::GradingReport = crate::grade_events_and_collision(input);
    assert_eq!(report.lesson, "events-collision-proximity-game");
}

#[test]
fn lib_re_exports_first_lesson_types_and_function() {
    let input: crate::GradingInput = crate::GradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
    };
    let _: crate::GradingReport = crate::grade_first_lesson_readiness(input);
}

#[test]
fn lib_re_exports_loops_types_and_function() {
    let input: crate::LoopsGradingInput = crate::LoopsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: None,
    };
    let _: crate::GradingReport = crate::grade_loops_and_conditionals(input);
}

#[test]
fn lib_re_exports_step_status_variants() {
    assert_eq!(crate::StepStatus::Ready, StepStatus::Ready);
    assert_eq!(crate::StepStatus::Blocked, StepStatus::Blocked);
    assert_eq!(crate::StepStatus::NotYetTested, StepStatus::NotYetTested);
}

// ── Cross-module consistency: schema_version ──────────────────

#[test]
fn all_graders_share_schema_version() {
    let (fl, lo, ev) = grade_all_three(true, "ok", true, "ok", None);
    assert_eq!(fl.schema_version, lo.schema_version);
    assert_eq!(lo.schema_version, ev.schema_version);
    assert_eq!(fl.schema_version, "eatme.assets/grading/v1");
}

// ── Cross-module consistency: precondition steps ──────────────

#[test]
fn all_graders_same_preconditions_when_all_ready() {
    let (fl, lo, ev) = grade_all_three(true, "All valid", true, "All available", None);
    for (label, report) in [("first_lesson", &fl), ("loops", &lo), ("events", &ev)] {
        assert!(report.steps.len() >= 3, "{label}");
        assert_eq!(report.steps[0].name, "validate-assets", "{label}");
        assert_eq!(report.steps[0].status, StepStatus::Ready, "{label}");
        assert_eq!(report.steps[1].name, "check-dependencies", "{label}");
        assert_eq!(report.steps[1].status, StepStatus::Ready, "{label}");
        assert_eq!(report.steps[2].name, "launch-smoke", "{label}");
        assert_eq!(report.steps[2].status, StepStatus::Ready, "{label}");
    }
}

#[test]
fn all_graders_same_preconditions_when_assets_blocked() {
    let (fl, lo, ev) = grade_all_three(false, "Bad", true, "ok", None);
    for (label, report) in [("first_lesson", &fl), ("loops", &lo), ("events", &ev)] {
        assert_eq!(report.steps[0].status, StepStatus::Blocked, "{label}");
        assert_eq!(report.steps[1].status, StepStatus::Ready, "{label}");
        assert_eq!(report.steps[2].status, StepStatus::Blocked, "{label}");
        assert!(
            report.steps[2].reason.contains("validate-assets"),
            "{label}"
        );
    }
}

#[test]
fn all_graders_same_preconditions_when_both_blocked() {
    let (fl, lo, ev) = grade_all_three(false, "Bad", false, "Missing", None);
    for (label, report) in [("first_lesson", &fl), ("loops", &lo), ("events", &ev)] {
        assert_eq!(report.steps[0].status, StepStatus::Blocked, "{label}");
        assert_eq!(report.steps[1].status, StepStatus::Blocked, "{label}");
        assert_eq!(report.steps[2].status, StepStatus::Blocked, "{label}");
        let r = &report.steps[2].reason;
        assert!(
            r.contains("validate-assets") && r.contains("check-dependencies"),
            "{label}: {r}"
        );
    }
}

#[test]
fn all_graders_not_passed_when_preconditions_blocked() {
    let (fl, lo, ev) =
        grade_all_three(false, "bad", false, "bad", Some(method_call_only_program()));
    assert!(!fl.passed, "first_lesson");
    assert!(!lo.passed, "loops");
    assert!(!ev.passed, "events");
}

// ── Edge case: method-call-only program ───────────────────────

#[test]
fn events_method_call_only_blocks_all_ast_steps() {
    let report = grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(method_call_only_program()),
    });
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "add-event-listener"
    );
    assert!(report.steps[3].reason.contains("No EventListener found"));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

#[test]
fn loops_method_call_only_blocks_all_ast_steps() {
    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(method_call_only_program()),
    });
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "build-counting-loop"
    );
    assert!(report.steps[3].reason.contains("No CountLoop found"));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-conditional-branch"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// ── Edge case: wrong-lesson constructs ────────────────────────

#[test]
fn events_with_loops_and_conditionals_only_blocks() {
    let program = Program {
        procedures: vec![Procedure {
            name: "wrongLesson".into(),
            body: vec![
                Statement::CountLoop {
                    count: 3,
                    body: vec![],
                },
                Statement::IfElse {
                    condition: "true".into(),
                    if_body: vec![],
                    else_body: vec![],
                },
            ],
        }],
    };
    let report = grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    });
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "add-event-listener"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener"
    );
}

#[test]
fn loops_with_listeners_only_blocks() {
    let program = Program {
        procedures: vec![Procedure {
            name: "wrongLesson".into(),
            body: vec![
                Statement::EventListener {
                    event: "X".into(),
                    body: vec![],
                },
                Statement::CollisionListener {
                    object_a: "a".into(),
                    object_b: "b".into(),
                    body: vec![],
                },
            ],
        }],
    };
    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    });
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "build-counting-loop"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-conditional-branch"
    );
}

// ── Integration: committed assets ─────────────────────────────

#[test]
fn grade_events_committed_assets_produces_valid_report() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ar = crate::validate_assets(&root).unwrap();
    let report = grade_events_and_collision(EventsGradingInput {
        assets_valid: ar.passed,
        asset_reason: format!("{} assets", ar.scenario_asset_count),
        deps_available: false,
        deps_reason: "Not checked".into(),
        student_program: None,
    });
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "events-collision-proximity-game");
    assert_eq!(report.steps.len(), 7);
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert!(!report.passed);
}

#[test]
fn grade_loops_committed_assets_produces_valid_report() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ar = crate::validate_assets(&root).unwrap();
    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: ar.passed,
        asset_reason: format!("{} assets", ar.scenario_asset_count),
        deps_available: false,
        deps_reason: "Not checked".into(),
        student_program: None,
    });
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps.len(), 7);
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert!(!report.passed);
}

// ── Distinct lesson names ─────────────────────────────────────

#[test]
fn each_grader_has_unique_lesson_name() {
    let (fl, lo, ev) = grade_all_three(true, "ok", true, "ok", None);
    assert_ne!(fl.lesson, lo.lesson);
    assert_ne!(lo.lesson, ev.lesson);
    assert_ne!(fl.lesson, ev.lesson);
}

// ── JSON: cross-module serialization consistency ──────────────

#[test]
fn all_graders_json_shares_same_top_level_keys() {
    let (fl, lo, ev) = grade_all_three(true, "ok", true, "ok", None);
    for (label, report) in [("first_lesson", fl), ("loops", lo), ("events", ev)] {
        let json: serde_json::Value = serde_json::to_value(&report).unwrap();
        assert!(json["schema_version"].is_string(), "{label}");
        assert!(json["lesson"].is_string(), "{label}");
        assert!(json["passed"].is_boolean(), "{label}");
        assert!(json["steps"].is_array(), "{label}");
        for step in json["steps"].as_array().unwrap() {
            let status = step["status"].as_str().unwrap();
            assert!(
                ["ready", "blocked", "not-yet-tested"].contains(&status),
                "{label}: unexpected status '{status}'"
            );
        }
    }
}
