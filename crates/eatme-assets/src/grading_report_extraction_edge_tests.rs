//! Edge-case and boundary contract tests for grading_report_events extraction.
//!
//! These tests cover:
//! 1. Boundary conditions: no program, missing individual listeners
//! 2. Cascade behavior: blocked assets, blocked deps, both blocked
//! 3. Edge-case AST structures: nested constructs, empty program,
//!    multi-procedure, loops, if/else
//! 4. JSON serialization round-trip

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
        asset_reason: "All 101 scenario assets passed validation".into(),
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
        asset_reason: "All 101 scenario assets passed validation".into(),
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
