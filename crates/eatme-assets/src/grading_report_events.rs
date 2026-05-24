//! Events-and-collision grading — extracted from grading_report.rs.
//!
//! Contains `EventsGradingInput`, `grade_events_and_collision`, and event-specific
//! AST helpers. Shared helpers are imported from `crate::grading_report`.

use eatme_core::ast::{Program, Statement};

// Re-export shared types so `use super::*` works in the test file
pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{build_preconditions, cascade_blocked, no_program_chain};

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

#[derive(Default)]
struct EventsEvidence {
    has_event_listener: bool,
    has_supported_event_type: bool,
    has_valid_event_listener: bool,
    has_collision_listener: bool,
    has_distinct_collision_targets: bool,
    has_valid_collision_listener: bool,
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

    let evidence = analyze_events(program);

    let add_event = if evidence.has_valid_event_listener {
        ready_step(
            "add-event-listener",
            &["launch-smoke"],
            "Key-press or mouse-click listener includes a guard condition",
        )
    } else {
        let reason = if !evidence.has_event_listener {
            "No EventListener found in student program"
        } else if !evidence.has_supported_event_type {
            "Event handler should use a key press or mouse click event"
        } else {
            "Event handler should include a guard condition to prevent infinite loops"
        };
        blocked_step("add-event-listener", &["launch-smoke"], reason)
    };

    let add_collision = if add_event.status == StepStatus::Blocked {
        cascade_blocked("add-collision-listener", &["add-event-listener"])
    } else if evidence.has_valid_collision_listener {
        ready_step(
            "add-collision-listener",
            &["add-event-listener"],
            "Collision handler references two different entities and includes a guard condition",
        )
    } else {
        let reason = if !evidence.has_collision_listener {
            "No CollisionListener found in student program"
        } else if !evidence.has_distinct_collision_targets {
            "Collision handler must reference two different entities"
        } else {
            "Collision handler should include a guard condition to prevent infinite loops"
        };
        blocked_step("add-collision-listener", &["add-event-listener"], reason)
    };

    let run_world = if add_collision.status == StepStatus::Blocked {
        cascade_blocked("run-world", &["add-collision-listener"])
    } else {
        ready_step(
            "run-world",
            &["add-collision-listener"],
            "Static grading found valid event and collision handlers for interactive play",
        )
    };

    let save_project = if run_world.status == StepStatus::Blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        let round_trip_ok = serde_json::to_vec(program)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Program>(&bytes).ok())
            .is_some_and(|restored| restored == *program);
        if round_trip_ok {
            ready_step(
                "save-project",
                &["run-world"],
                "Program round-trip (serialize → deserialize → compare) verified",
            )
        } else {
            blocked_step(
                "save-project",
                &["run-world"],
                "Program failed round-trip verification",
            )
        }
    };

    vec![add_event, add_collision, run_world, save_project]
}

fn analyze_events(program: &Program) -> EventsEvidence {
    let mut evidence = EventsEvidence::default();
    for procedure in &program.procedures {
        collect_event_evidence(&procedure.body, &mut evidence);
    }
    evidence
}

fn collect_event_evidence(statements: &[Statement], evidence: &mut EventsEvidence) {
    for statement in statements {
        match statement {
            Statement::EventListener { event, body } => {
                evidence.has_event_listener = true;
                if is_supported_event_type(event) {
                    evidence.has_supported_event_type = true;
                    if contains_guard_condition(body) {
                        evidence.has_valid_event_listener = true;
                    }
                }
                collect_event_evidence(body, evidence);
            }
            Statement::CollisionListener {
                object_a,
                object_b,
                body,
            } => {
                evidence.has_collision_listener = true;
                if object_a != object_b {
                    evidence.has_distinct_collision_targets = true;
                    if contains_guard_condition(body) {
                        evidence.has_valid_collision_listener = true;
                    }
                }
                collect_event_evidence(body, evidence);
            }
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => collect_event_evidence(body, evidence),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_event_evidence(if_body, evidence);
                collect_event_evidence(else_body, evidence);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_event_evidence(&method.body, evidence);
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
    }
}

fn contains_guard_condition(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::IfElse { .. } => true,
        Statement::CountLoop { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. } => contains_guard_condition(body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_guard_condition(&method.body)),
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. } => false,
    })
}

fn is_supported_event_type(event: &str) -> bool {
    let lower = event.to_ascii_lowercase();
    lower.contains("key") || lower.contains("mouse")
}

fn ready_step(name: &str, deps: &[&str], reason: &str) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Ready,
        reason: reason.into(),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

fn blocked_step(name: &str, deps: &[&str], reason: &str) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: reason.into(),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

#[cfg(test)]
#[path = "grading_report_events_tests.rs"]
mod events_tests;
