//! Creative/Design project grading — capstone curriculum lesson.
//!
//! Validates that a student combined multiple concepts: scene objects,
//! procedures, events, functions/variables, and storytelling flow.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

pub struct CreativeProjectGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_creative_project(input: CreativeProjectGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("build-scene-with-objects", &["launch-smoke"]),
            cascade_blocked("create-custom-procedure", &["build-scene-with-objects"]),
            cascade_blocked("add-control-structure", &["create-custom-procedure"]),
            cascade_blocked("add-event-or-interaction", &["add-control-structure"]),
            cascade_blocked("run-world", &["add-event-or-interaction"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_creative_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "creative-design-project",
        passed,
        steps,
    )
}

fn evaluate_creative_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("build-scene-with-objects", "launch-smoke"),
            ("create-custom-procedure", "build-scene-with-objects"),
            ("add-control-structure", "create-custom-procedure"),
            ("add-event-or-interaction", "add-control-structure"),
            ("run-world", "add-event-or-interaction"),
            ("save-project", "run-world"),
        ]);
    };

    let has_multiple_objects = program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .filter(|s| matches!(s, Statement::MethodCall { .. }))
        .count()
        >= 2;

    let has_custom_procedure = program.procedures.len() >= 2
        || program.procedures.iter().any(|p| !p.parameters.is_empty());

    let has_control = program
        .procedures
        .iter()
        .any(|p| has_control_structure(&p.body));

    let has_event = program
        .procedures
        .iter()
        .any(|p| has_event_or_interaction(&p.body));

    let all_met = has_multiple_objects && has_custom_procedure && has_control && has_event;

    vec![
        ast_check_step(
            "build-scene-with-objects",
            "launch-smoke",
            has_multiple_objects,
            "multiple method calls (scene building evidence)",
        ),
        ast_check_step(
            "create-custom-procedure",
            "build-scene-with-objects",
            has_custom_procedure,
            "custom procedure or parameterized method",
        ),
        ast_check_step(
            "add-control-structure",
            "create-custom-procedure",
            has_control,
            "loop or conditional control structure",
        ),
        ast_check_step(
            "add-event-or-interaction",
            "add-control-structure",
            has_event,
            "event listener or collision handler",
        ),
        ast_check_step(
            "run-world",
            "add-event-or-interaction",
            all_met,
            "world run with creative project",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            all_met,
            "project save with creative project",
        ),
    ]
}

fn has_control_structure(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| match s {
        Statement::CountLoop { .. } | Statement::IfElse { .. } => true,
        Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body } => has_control_structure(body),
        _ => false,
    })
}

fn has_event_or_interaction(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| {
        matches!(
            s,
            Statement::EventListener { .. } | Statement::CollisionListener { .. }
        )
    })
}
