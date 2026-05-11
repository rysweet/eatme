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

fn event_only_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "eventOnly".into(),
            body: vec![Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                }],
            }],
        }],
    }
}

fn collision_only_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "collisionOnly".into(),
            body: vec![Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Ouch!\"".into()],
                }],
            }],
        }],
    }
}

fn all_ready_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn blocked_assets_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn blocked_deps_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn both_blocked_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
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

// ── Behavioral contract: no program ───────────────────────────

#[test]
fn no_program_preconditions_still_ready() {
    let report = grade_events_and_collision(all_ready_input(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
}

#[test]
fn no_program_all_interaction_steps_blocked() {
    let report = grade_events_and_collision(all_ready_input(None));
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// ── Behavioral contract: missing event listener ───────────────

#[test]
fn missing_event_listener_blocks_and_cascades() {
    let report = grade_events_and_collision(all_ready_input(Some(collision_only_program())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("No EventListener found"));
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[6].status, StepStatus::Blocked);
}

// ── Behavioral contract: missing collision listener ───────────

#[test]
fn missing_collision_listener_blocks_and_cascades() {
    let report = grade_events_and_collision(all_ready_input(Some(event_only_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No CollisionListener found")
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[6].status, StepStatus::Blocked);
}

// ── Behavioral contract: blocked assets ───────────────────────

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report = grade_events_and_collision(blocked_assets_input(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    for i in 3..=6 {
        assert_eq!(report.steps[i].status, StepStatus::Blocked);
    }
}

// ── Behavioral contract: blocked deps ─────────────────────────

#[test]
fn blocked_deps_cascades_all_downstream() {
    let report = grade_events_and_collision(blocked_deps_input(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    for i in 3..=6 {
        assert_eq!(report.steps[i].status, StepStatus::Blocked);
    }
}

// ── Behavioral contract: both blocked ─────────────────────────

#[test]
fn both_blocked_all_steps_blocked() {
    let report = grade_events_and_collision(both_blocked_input(Some(complete_events_program())));
    for step in &report.steps {
        assert_eq!(
            step.status,
            StepStatus::Blocked,
            "step {} should be blocked",
            step.name
        );
    }
}

#[test]
fn both_blocked_launch_smoke_mentions_both_blockers() {
    let report = grade_events_and_collision(both_blocked_input(Some(complete_events_program())));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both: {reason}"
    );
}

// ── Edge case: nested AST constructs ──────────────────────────

#[test]
fn nested_event_inside_collision_detected() {
    let program = Program {
        procedures: vec![Procedure {
            name: "nested".into(),
            body: vec![Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![],
                }],
            }],
        }],
    };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener nested in Collision"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener at top"
    );
}

#[test]
fn nested_collision_inside_event_detected() {
    let program = Program {
        procedures: vec![Procedure {
            name: "nested".into(),
            body: vec![Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![],
                }],
            }],
        }],
    };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener at top"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener nested in Event"
    );
}

#[test]
fn empty_program_blocks_all_ast_steps() {
    let program = Program { procedures: vec![] };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
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
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

#[test]
fn constructs_found_across_multiple_procedures() {
    let program = Program {
        procedures: vec![
            Procedure {
                name: "proc1".into(),
                body: vec![Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![],
                }],
            },
            Procedure {
                name: "proc2".into(),
                body: vec![Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![],
                }],
            },
        ],
    };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener in proc1"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener in proc2"
    );
}

#[test]
fn event_listener_inside_count_loop_detected() {
    let program = Program {
        procedures: vec![Procedure {
            name: "looped".into(),
            body: vec![Statement::CountLoop {
                count: 5,
                body: vec![
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![],
                    },
                    Statement::CollisionListener {
                        object_a: "this.cat".into(),
                        object_b: "this.dog".into(),
                        body: vec![],
                    },
                ],
            }],
        }],
    };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Ready);
}

#[test]
fn event_listener_inside_if_else_detected() {
    let program = Program {
        procedures: vec![Procedure {
            name: "conditional".into(),
            body: vec![Statement::IfElse {
                condition: "true".into(),
                if_body: vec![Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![],
                }],
                else_body: vec![Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![],
                }],
            }],
        }],
    };
    let report = grade_events_and_collision(all_ready_input(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener in if_body"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener in else_body"
    );
}

// ── JSON serialization ────────────────────────────────────────

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "events-collision-proximity-game");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 7);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "add-event-listener");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][5]["status"], "not-yet-tested");
    assert_eq!(json["steps"][6]["status"], "ready");
}
