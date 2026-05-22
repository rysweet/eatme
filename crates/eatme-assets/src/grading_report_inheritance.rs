//! Inheritance + OOP grading — covers custom type and class hierarchy lessons.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

pub struct InheritanceOopGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_inheritance_oop(input: InheritanceOopGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-custom-type", &["launch-smoke"]),
            cascade_blocked("set-extends-relationship", &["create-custom-type"]),
            cascade_blocked("define-custom-method", &["set-extends-relationship"]),
            cascade_blocked("run-world", &["define-custom-method"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_inheritance_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "inheritance-oop-mini-challenge".into(),
        passed,
        steps,
    }
}

fn evaluate_inheritance_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("create-custom-type", "launch-smoke"),
            ("set-extends-relationship", "create-custom-type"),
            ("define-custom-method", "set-extends-relationship"),
            ("run-world", "define-custom-method"),
            ("save-project", "run-world"),
        ]);
    };

    let (has_custom_type, has_inheritance, has_custom_method) = program
        .procedures
        .iter()
        .fold((false, false, false), |found, procedure| {
            merge_type_findings(found, scan_user_types(&procedure.body))
        });
    let all_complete = has_custom_type && has_inheritance && has_custom_method;

    vec![
        ast_check_step(
            "create-custom-type",
            "launch-smoke",
            has_custom_type,
            "custom user type",
        ),
        ast_check_step(
            "set-extends-relationship",
            "create-custom-type",
            has_inheritance,
            "extends relationship",
        ),
        ast_check_step(
            "define-custom-method",
            "set-extends-relationship",
            has_custom_method,
            "custom method on a user type",
        ),
        ast_check_step(
            "run-world",
            "define-custom-method",
            all_complete,
            "world run with custom type",
        ),
        ast_check_step(
            "save-project",
            "run-world",
            all_complete,
            "project save with custom type",
        ),
    ]
}

fn scan_user_types(statements: &[Statement]) -> (bool, bool, bool) {
    let mut found = (false, false, false);
    for statement in statements {
        match statement {
            Statement::UserTypeDeclaration {
                extends, methods, ..
            } => {
                found.0 = true;
                if extends
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    found.1 = true;
                }
                if methods.iter().any(|method| !method.name.trim().is_empty()) {
                    found.2 = true;
                }
                for method in methods {
                    found = merge_type_findings(found, scan_user_types(&method.body));
                }
            }
            Statement::CountLoop { body, .. }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => {
                found = merge_type_findings(found, scan_user_types(body));
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                found = merge_type_findings(found, scan_user_types(if_body));
                found = merge_type_findings(found, scan_user_types(else_body));
            }
            _ => {}
        }
    }
    found
}

fn merge_type_findings(left: (bool, bool, bool), right: (bool, bool, bool)) -> (bool, bool, bool) {
    (left.0 || right.0, left.1 || right.1, left.2 || right.2)
}
