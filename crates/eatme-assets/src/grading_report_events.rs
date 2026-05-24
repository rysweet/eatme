//! Events-and-collision grading — extracted from grading_report.rs.
//!
//! Contains `EventsGradingInput`, `grade_events_and_collision`, and event-specific
//! AST helpers. Shared helpers are imported from `crate::grading_report`.

use eatme_core::ast::{Program, Statement};

// Re-export shared types so `use super::*` works in the test file
pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

/// Input struct for events-and-collision grading.
pub struct EventsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

/// Grade a student's events-and-collision lesson attempt.
pub fn grade_events_and_collision(input: EventsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("add-event-listener", &["launch-smoke"]),
            cascade_blocked("add-collision-listener", &["add-event-listener"]),
            cascade_blocked("run-world", &["add-collision-listener"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_events_steps(&input.student_program)
    };

    // Precondition steps are all Ready when !preconditions_blocked, so skip them.
    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "events-collision-proximity-game".into(),
        passed,
        steps,
    }
}

fn evaluate_events_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => {
            return no_program_chain(&[
                ("add-event-listener", "launch-smoke"),
                ("add-collision-listener", "add-event-listener"),
                ("run-world", "add-collision-listener"),
                ("save-project", "run-world"),
            ]);
        }
    };

    let (has_event, has_collision) = ast_find_event_constructs(program);

    let add_event = ast_check_step(
        "add-event-listener",
        "launch-smoke",
        has_event,
        "EventListener",
    );
    let event_blocked = add_event.status == StepStatus::Blocked;

    let add_collision = if event_blocked {
        cascade_blocked("add-collision-listener", &["add-event-listener"])
    } else {
        ast_check_step(
            "add-collision-listener",
            "add-event-listener",
            has_collision,
            "CollisionListener",
        )
    };

    let collision_blocked = add_collision.status == StepStatus::Blocked;

    let run_world = if collision_blocked {
        cascade_blocked("run-world", &["add-collision-listener"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["add-collision-listener".into()],
        }
    };

    let run_world_blocked = run_world.status == StepStatus::Blocked;

    let save_project = if run_world_blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        let round_trip_ok = serde_json::to_vec(program)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Program>(&bytes).ok())
            .is_some_and(|restored| restored == *program);
        let status = if round_trip_ok {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        };
        let reason = if round_trip_ok {
            "Program round-trip (serialize → deserialize → compare) verified"
        } else {
            "Program failed round-trip verification"
        };
        StepGrade {
            name: "save-project".into(),
            status,
            reason: reason.into(),
            depends_on: vec!["run-world".into()],
        }
    };

    vec![add_event, add_collision, run_world, save_project]
}

/// Single-pass AST scan: returns (has_event_listener, has_collision_listener).
fn ast_find_event_constructs(program: &Program) -> (bool, bool) {
    let (mut has_event, mut has_collision) = (false, false);
    for proc in &program.procedures {
        stmt_find_event_constructs(&proc.body, &mut has_event, &mut has_collision);
        if has_event && has_collision {
            return (true, true);
        }
    }
    (has_event, has_collision)
}

fn stmt_find_event_constructs(stmts: &[Statement], has_event: &mut bool, has_collision: &mut bool) {
    for stmt in stmts {
        match stmt {
            Statement::EventListener { body, .. } => {
                *has_event = true;
                if !*has_collision {
                    stmt_find_event_constructs(body, has_event, has_collision);
                }
            }
            Statement::CollisionListener { body, .. } => {
                *has_collision = true;
                if !*has_event {
                    stmt_find_event_constructs(body, has_event, has_collision);
                }
            }
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => {
                if !(*has_event && *has_collision) {
                    stmt_find_event_constructs(body, has_event, has_collision);
                }
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                if !(*has_event && *has_collision) {
                    stmt_find_event_constructs(if_body, has_event, has_collision);
                    if !(*has_event && *has_collision) {
                        stmt_find_event_constructs(else_body, has_event, has_collision);
                    }
                }
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                if !(*has_event && *has_collision) {
                    for method in methods {
                        stmt_find_event_constructs(&method.body, has_event, has_collision);
                        if *has_event && *has_collision {
                            break;
                        }
                    }
                }
            }
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => {}
        }
        if *has_event && *has_collision {
            return;
        }
    }
}

#[cfg(test)]
#[path = "grading_report_events_tests.rs"]
mod events_tests;
