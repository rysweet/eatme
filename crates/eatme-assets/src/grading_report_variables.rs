//! Variables grading — covers the "Using Variables" curriculum lesson.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};
use crate::quality_scoring::score_variable_quality;

pub struct VariablesGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_variables(input: VariablesGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("declare-variable", &["launch-smoke"]),
            cascade_blocked("use-variable-in-method", &["declare-variable"]),
            cascade_blocked("modify-variable", &["use-variable-in-method"]),
            cascade_blocked("run-world", &["modify-variable"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_variables_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "using-variables-mini-challenge",
        passed,
        steps,
    )
    .with_quality_scores(score_variable_quality(input.student_program.as_ref()))
}

fn evaluate_variables_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("declare-variable", "launch-smoke"),
            ("use-variable-in-method", "declare-variable"),
            ("modify-variable", "use-variable-in-method"),
            ("run-world", "modify-variable"),
            ("save-project", "run-world"),
        ]);
    };

    let has_declaration = program
        .procedures
        .iter()
        .any(|p| contains_var_declaration(&p.body));
    let has_usage = has_declaration
        && program
            .procedures
            .iter()
            .any(|p| contains_var_in_method_call(&p.body));
    let has_assignment = has_declaration
        && program
            .procedures
            .iter()
            .any(|p| contains_var_assignment(&p.body));

    vec![
        ast_check_step(
            "declare-variable",
            "launch-smoke",
            has_declaration,
            "variable declaration",
        ),
        ast_check_step(
            "use-variable-in-method",
            "declare-variable",
            has_usage,
            "variable used in a method call",
        ),
        ast_check_step(
            "modify-variable",
            "use-variable-in-method",
            has_assignment,
            "variable assignment",
        ),
        ast_check_step(
            "run-world",
            "modify-variable",
            has_declaration && has_usage && has_assignment,
            "world run with variable",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            has_declaration && has_usage && has_assignment,
            "project save with variable",
        ),
    ]
}

fn contains_var_declaration(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| match s {
        Statement::VariableDeclaration { .. } => true,
        Statement::CountLoop { body, .. } | Statement::DoInOrder { body } => {
            contains_var_declaration(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_var_declaration(if_body) || contains_var_declaration(else_body),
        _ => false,
    })
}

fn contains_var_in_method_call(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| match s {
        Statement::MethodCall { arguments, .. } => arguments.iter().any(|a| !a.starts_with('"')),
        Statement::CountLoop { body, .. } | Statement::DoInOrder { body } => {
            contains_var_in_method_call(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_var_in_method_call(if_body) || contains_var_in_method_call(else_body),
        _ => false,
    })
}

fn contains_var_assignment(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| match s {
        Statement::VariableAssignment { .. } => true,
        Statement::CountLoop { body, .. } | Statement::DoInOrder { body } => {
            contains_var_assignment(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_var_assignment(if_body) || contains_var_assignment(else_body),
        _ => false,
    })
}
