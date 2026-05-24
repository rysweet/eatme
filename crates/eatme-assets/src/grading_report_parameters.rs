//! Parameters grading — covers the "Parameters" curriculum lesson.

use eatme_core::ast::Program;

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};
use crate::quality_scoring::score_parameter_quality;

pub struct ParametersGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_parameters(input: ParametersGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-parameterized-procedure", &["launch-smoke"]),
            cascade_blocked("call-with-argument", &["create-parameterized-procedure"]),
            cascade_blocked("run-world", &["call-with-argument"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_parameters_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "parameters-mini-challenge",
        passed,
        steps,
    )
    .with_quality_scores(score_parameter_quality(input.student_program.as_ref()))
}

fn evaluate_parameters_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("create-parameterized-procedure", "launch-smoke"),
            ("call-with-argument", "create-parameterized-procedure"),
            ("run-world", "call-with-argument"),
            ("save-project", "run-world"),
        ]);
    };

    let has_parameterized = program.procedures.iter().any(|p| !p.parameters.is_empty());
    let has_call_with_args = program.procedures.iter().any(|p| {
        p.body.iter().any(|s| match s {
            eatme_core::ast::Statement::MethodCall { arguments, .. } => !arguments.is_empty(),
            _ => false,
        })
    });

    vec![
        ast_check_step(
            "create-parameterized-procedure",
            "launch-smoke",
            has_parameterized,
            "procedure with parameters",
        ),
        ast_check_step(
            "call-with-argument",
            "create-parameterized-procedure",
            has_call_with_args,
            "method call with arguments",
        ),
        ast_check_step(
            "run-world",
            "call-with-argument",
            has_parameterized && has_call_with_args,
            "world run with parameterized procedure",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            has_parameterized && has_call_with_args,
            "project save with parameterized procedure",
        ),
    ]
}
