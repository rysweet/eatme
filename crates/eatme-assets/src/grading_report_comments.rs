//! Comment grading — covers lessons that ask students to explain their work.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

pub struct CommentsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_comments(input: CommentsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("add-comment", &["launch-smoke"]),
            cascade_blocked("write-meaningful-comment", &["add-comment"]),
            cascade_blocked("save-project", &["write-meaningful-comment"]),
        ]
    } else {
        evaluate_comments_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "comments-mini-challenge",
        passed,
        steps,
    )
}

fn evaluate_comments_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("add-comment", "launch-smoke"),
            ("write-meaningful-comment", "add-comment"),
            ("save-project", "write-meaningful-comment"),
        ]);
    };

    let mut comments = Vec::new();
    for procedure in &program.procedures {
        collect_comments(&procedure.body, &mut comments);
    }

    let has_comment = !comments.is_empty();
    let has_meaningful_comment = comments.into_iter().any(is_meaningful_comment);
    let has_non_comment_work = program
        .procedures
        .iter()
        .any(|procedure| contains_non_comment_work(&procedure.body))
        || program
            .functions
            .iter()
            .any(|function| contains_non_comment_work(&function.body));

    vec![
        ast_check_step("add-comment", "launch-smoke", has_comment, "comment"),
        ast_check_step(
            "write-meaningful-comment",
            "add-comment",
            has_meaningful_comment,
            "meaningful comment",
        ),
        ast_check_step(
            "save-project",
            "write-meaningful-comment",
            has_meaningful_comment && has_non_comment_work,
            "project save with comments on executable work",
        ),
    ]
}

fn collect_comments<'a>(statements: &'a [Statement], comments: &mut Vec<&'a str>) {
    for statement in statements {
        match statement {
            Statement::Comment { text } => comments.push(text.as_str()),
            Statement::CountLoop { body, .. }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => collect_comments(body, comments),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_comments(if_body, comments);
                collect_comments(else_body, comments);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_comments(&method.body, comments);
                }
            }
            _ => {}
        }
    }
}

fn contains_non_comment_work(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::Comment { .. } => false,
        Statement::CountLoop { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body } => contains_non_comment_work(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_non_comment_work(if_body) || contains_non_comment_work(else_body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_non_comment_work(&method.body)),
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. } => true,
    })
}

fn is_meaningful_comment(comment: &str) -> bool {
    let trimmed = comment.trim();
    let lowercase = trimmed.to_ascii_lowercase();
    trimmed.len() >= 15
        && trimmed.split_whitespace().count() >= 3
        && lowercase != "todo"
        && lowercase != "comment"
        && lowercase != "fix later"
}
