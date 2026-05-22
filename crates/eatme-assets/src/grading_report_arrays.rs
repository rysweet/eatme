//! Arrays + arithmetic grading — covers collection choreography lessons.

use eatme_core::ast::{ArithmeticOperator, Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

pub struct ArraysArithmeticGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_arrays_and_arithmetic(input: ArraysArithmeticGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-array", &["launch-smoke"]),
            cascade_blocked("access-array-element", &["create-array"]),
            cascade_blocked("iterate-array", &["access-array-element"]),
            cascade_blocked("use-arithmetic-operators", &["iterate-array"]),
            cascade_blocked("run-world", &["use-arithmetic-operators"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_arrays_arithmetic_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "arrays-collection-choreography".into(),
        passed,
        steps,
    }
}

fn evaluate_arrays_arithmetic_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("create-array", "launch-smoke"),
            ("access-array-element", "create-array"),
            ("iterate-array", "access-array-element"),
            ("use-arithmetic-operators", "iterate-array"),
            ("run-world", "use-arithmetic-operators"),
            ("save-project", "run-world"),
        ]);
    };

    let has_array = program
        .procedures
        .iter()
        .any(|procedure| contains_array_declaration(&procedure.body));
    let has_access = has_array
        && program
            .procedures
            .iter()
            .any(|procedure| contains_array_access(&procedure.body));
    let has_iteration = has_array
        && program
            .procedures
            .iter()
            .any(|procedure| contains_array_iteration(&procedure.body));

    let (has_add, has_subtract, has_multiply, has_divide) =
        program
            .procedures
            .iter()
            .fold((false, false, false, false), |mut found, procedure| {
                collect_arithmetic_operators(&procedure.body, &mut found);
                found
            });
    let has_all_arithmetic = has_add && has_subtract && has_multiply && has_divide;
    let all_complete = has_array && has_access && has_iteration && has_all_arithmetic;

    vec![
        ast_check_step(
            "create-array",
            "launch-smoke",
            has_array,
            "array declaration",
        ),
        ast_check_step(
            "access-array-element",
            "create-array",
            has_access,
            "array element access",
        ),
        ast_check_step(
            "iterate-array",
            "access-array-element",
            has_iteration,
            "array iteration",
        ),
        ast_check_step(
            "use-arithmetic-operators",
            "iterate-array",
            has_all_arithmetic,
            "all arithmetic operators",
        ),
        ast_check_step(
            "run-world",
            "use-arithmetic-operators",
            all_complete,
            "world run with arrays and arithmetic",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            all_complete,
            "project save with arrays and arithmetic",
        ),
    ]
}

fn contains_array_declaration(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::ArrayDeclaration { .. } => true,
        Statement::CountLoop { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. } => contains_array_declaration(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_array_declaration(if_body) || contains_array_declaration(else_body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_array_declaration(&method.body)),
        _ => false,
    })
}

fn contains_array_access(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::ArrayAccess { .. } => true,
        Statement::CountLoop { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. } => contains_array_access(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_array_access(if_body) || contains_array_access(else_body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_array_access(&method.body)),
        _ => false,
    })
}

fn contains_array_iteration(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::ForEachArray { .. } => true,
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. } => contains_array_iteration(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_array_iteration(if_body) || contains_array_iteration(else_body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_array_iteration(&method.body)),
        _ => false,
    })
}

fn collect_arithmetic_operators(statements: &[Statement], found: &mut (bool, bool, bool, bool)) {
    for statement in statements {
        match statement {
            Statement::ArithmeticExpression { operator, .. } => match operator {
                ArithmeticOperator::Add => found.0 = true,
                ArithmeticOperator::Subtract => found.1 = true,
                ArithmeticOperator::Multiply => found.2 = true,
                ArithmeticOperator::Divide => found.3 = true,
            },
            Statement::CountLoop { body, .. }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => {
                collect_arithmetic_operators(body, found);
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_arithmetic_operators(if_body, found);
                collect_arithmetic_operators(else_body, found);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_arithmetic_operators(&method.body, found);
                }
            }
            _ => {}
        }
    }
}
