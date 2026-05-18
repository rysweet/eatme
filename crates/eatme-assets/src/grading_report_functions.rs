//! Functions grading — covers the "Using Functions" curriculum lesson.
//!
//! Validates that a student has created a function with a return type,
//! added a return statement, called the function from a procedure, run
//! the world, and saved the project.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

pub struct FunctionsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_functions(input: FunctionsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-function", &["launch-smoke"]),
            cascade_blocked("add-return-statement", &["create-function"]),
            cascade_blocked("call-function-from-procedure", &["add-return-statement"]),
            cascade_blocked("run-world", &["call-function-from-procedure"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_functions_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "using-functions-mini-challenge".into(),
        passed,
        steps,
    }
}

fn evaluate_functions_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("create-function", "launch-smoke"),
            ("add-return-statement", "create-function"),
            ("call-function-from-procedure", "add-return-statement"),
            ("run-world", "call-function-from-procedure"),
            ("save-project", "run-world"),
        ]);
    };

    let has_function = !program.functions.is_empty();
    let has_return = has_function && program.functions.iter().any(|f| contains_return(&f.body));
    let has_function_call = program
        .procedures
        .iter()
        .any(|p| contains_function_call(&p.body));

    vec![
        ast_check_step(
            "create-function",
            "launch-smoke",
            has_function,
            "function with a return type",
        ),
        ast_check_step(
            "add-return-statement",
            "create-function",
            has_return,
            "return statement in a function",
        ),
        ast_check_step(
            "call-function-from-procedure",
            "add-return-statement",
            has_function_call,
            "function call from a procedure",
        ),
        ast_check_step(
            "run-world",
            "call-function-from-procedure",
            has_function && has_return && has_function_call,
            "world run with function",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            has_function && has_return && has_function_call,
            "project save with function",
        ),
    ]
}

fn contains_return(statements: &[Statement]) -> bool {
    statements.iter().any(|s| match s {
        Statement::ReturnStatement { .. } => true,
        Statement::CountLoop { body, .. } => contains_return(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_return(if_body) || contains_return(else_body),
        Statement::EventListener { body, .. } => contains_return(body),
        Statement::CollisionListener { body, .. } => contains_return(body),
        _ => false,
    })
}

fn contains_function_call(statements: &[Statement]) -> bool {
    statements.iter().any(|s| match s {
        Statement::FunctionCall { .. } => true,
        Statement::CountLoop { body, .. } => contains_function_call(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_function_call(if_body) || contains_function_call(else_body),
        Statement::EventListener { body, .. } => contains_function_call(body),
        Statement::CollisionListener { body, .. } => contains_function_call(body),
        _ => false,
    })
}
