//! TDD contract tests for the grading_report → grading_report_events extraction.
//!
//! These tests verify:
//! 1. Module structure: EventsGradingInput and grade_events_and_collision exist
//!    in crate::grading_report_events
//! 2. Shared helpers: build_preconditions, cascade_blocked, no_program_chain,
//!    ast_check_step are pub(crate) in crate::grading_report
//! 3. Quality gates: both files ≤ 500 lines
//! 4. Behavioral contracts: all grading behavior preserved after extraction

use crate::grading_report::StepStatus;
use crate::grading_report_events::{EventsGradingInput, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};

// ── Fixtures ──────────────────────────────────────────────────

fn complete_events_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            body: vec![
                Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Hello world!\"".into()],
                    }],
                },
                Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Ouch!\"".into()],
                    }],
                },
            ],
        }],
    }
}

fn all_ready_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

// ── Quality-gate tests ────────────────────────────────────────

#[test]
fn grading_report_is_under_500_lines() {
    let source = include_str!("grading_report.rs");
    let line_count = source.lines().count();
    assert!(
        line_count <= 500,
        "grading_report.rs is {line_count} lines, exceeds 500-line quality gate"
    );
}

#[test]
fn grading_report_events_is_under_500_lines() {
    let source = include_str!("grading_report_events.rs");
    let line_count = source.lines().count();
    assert!(
        line_count <= 500,
        "grading_report_events.rs is {line_count} lines, exceeds 500-line quality gate"
    );
}

// ── Pub(crate) helper accessibility tests ─────────────────────

#[test]
fn build_preconditions_is_accessible_as_pub_crate() {
    let (steps, blocked) = crate::grading_report::build_preconditions(
        true,
        "All valid".into(),
        true,
        "All available".into(),
    );
    assert_eq!(steps.len(), 3);
    assert!(!blocked);
}

#[test]
fn build_preconditions_reports_blocked_when_assets_fail() {
    let (steps, blocked) = crate::grading_report::build_preconditions(
        false,
        "Assets failed".into(),
        true,
        "All available".into(),
    );
    assert!(blocked);
    assert_eq!(steps[0].status, StepStatus::Blocked);
    assert_eq!(steps[1].status, StepStatus::Ready);
    assert_eq!(steps[2].status, StepStatus::Blocked);
}

#[test]
fn cascade_blocked_is_accessible_as_pub_crate() {
    let step = crate::grading_report::cascade_blocked("test-step", &["dep-a"]);
    assert_eq!(step.name, "test-step");
    assert_eq!(step.status, StepStatus::Blocked);
    assert_eq!(step.depends_on, vec!["dep-a"]);
    assert!(step.reason.contains("Blocked by: dep-a"));
}

#[test]
fn no_program_chain_is_accessible_as_pub_crate() {
    let chain =
        crate::grading_report::no_program_chain(&[("step-a", "dep-a"), ("step-b", "step-a")]);
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].name, "step-a");
    assert_eq!(chain[0].status, StepStatus::Blocked);
    assert!(chain[0].reason.contains("No student program"));
    assert_eq!(chain[1].depends_on, vec!["step-a"]);
}

#[test]
fn ast_check_step_found_is_accessible_as_pub_crate() {
    let step = crate::grading_report::ast_check_step("check", "dep", true, "SomeConstruct");
    assert_eq!(step.status, StepStatus::Ready);
    assert!(step.reason.contains("SomeConstruct found"));
    assert_eq!(step.depends_on, vec!["dep"]);
}

#[test]
fn ast_check_step_not_found_is_accessible_as_pub_crate() {
    let step = crate::grading_report::ast_check_step("check", "dep", false, "SomeConstruct");
    assert_eq!(step.status, StepStatus::Blocked);
    assert!(step.reason.contains("No SomeConstruct found"));
}

// ── Module structure tests ────────────────────────────────────

#[test]
fn events_grading_input_is_constructible_from_events_module() {
    let input = EventsGradingInput {
        assets_valid: true,
        asset_reason: "test".into(),
        deps_available: true,
        deps_reason: "test".into(),
        student_program: None,
    };
    assert!(input.assets_valid);
    assert!(input.deps_available);
}

// ── Behavioral contract: schema and structure ─────────────────

#[test]
fn schema_version_is_grading_v1() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_events_collision_proximity_game() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.lesson, "events-collision-proximity-game");
}

#[test]
fn always_produces_seven_steps() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps.len(), 7);
}

#[test]
fn step_names_in_expected_order() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "add-event-listener",
            "add-collision-listener",
            "run-world",
            "save-project",
        ]
    );
}

// ── Behavioral contract: dependency chain ─────────────────────

#[test]
fn root_steps_have_no_dependencies() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert!(report.steps[0].depends_on.is_empty());
    assert!(report.steps[1].depends_on.is_empty());
}

#[test]
fn launch_smoke_depends_on_preconditions() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn add_event_listener_depends_on_launch_smoke() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn add_collision_depends_on_event_listener() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[4].depends_on, vec!["add-event-listener"]);
}

#[test]
fn run_world_depends_on_collision_listener() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[5].depends_on, vec!["add-collision-listener"]);
}

#[test]
fn save_project_depends_on_run_world() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[6].depends_on, vec!["run-world"]);
}

// ── Behavioral contract: complete program ─────────────────────

#[test]
fn complete_program_preconditions_ready() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
}

#[test]
fn complete_program_event_listener_ready() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert!(report.steps[3].reason.contains("EventListener found"));
}

#[test]
fn complete_program_collision_listener_ready() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert!(report.steps[4].reason.contains("CollisionListener found"));
}

#[test]
fn complete_program_run_world_not_yet_tested() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
}

#[test]
fn complete_program_save_project_round_trip_ready() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert!(report.steps[6].reason.contains("round-trip"));
}

#[test]
fn complete_program_does_not_pass_because_run_world_not_tested() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert!(!report.passed);
}
