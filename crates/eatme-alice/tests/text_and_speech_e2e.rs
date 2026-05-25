// Text-and-speech E2E tests: validates Alice.org dialogue and narration lesson flows.

use eatme_assets::{GamesNarrativeGradingInput, StepStatus, grade_games_and_narrative};
use eatme_core::ast::{Procedure, Program, Statement};

fn dialogue_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::DoInOrder {
                body: vec![
                    Statement::MethodCall {
                        object: "this.guide".into(),
                        method: "say".into(),
                        arguments: vec!["\"Welcome to the tour.\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "think".into(),
                        arguments: vec!["\"I should pay attention.\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.textBubble".into(),
                        method: "setText".into(),
                        arguments: vec!["\"Follow the glowing path.\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.textBubble".into(),
                        method: "show".into(),
                        arguments: vec![],
                    },
                ],
            }],
        }],
        functions: vec![],
    }
}

fn narrative_input(program: Option<Program>) -> GamesNarrativeGradingInput {
    GamesNarrativeGradingInput {
        assets_valid: true,
        asset_reason: "dialogue fixture parsed".into(),
        deps_available: true,
        deps_reason: "dialogue scenario grading ready".into(),
        student_program: program,
    }
}

fn count_methods(program: &Program, methods: &[&str]) -> usize {
    program
        .procedures
        .iter()
        .map(|procedure| count_methods_in_statements(&procedure.body, methods))
        .sum()
}

fn count_methods_in_statements(statements: &[Statement], methods: &[&str]) -> usize {
    statements
        .iter()
        .map(|statement| match statement {
            Statement::MethodCall { method, .. }
                if methods.iter().any(|needle| method == needle) =>
            {
                1
            }
            Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => count_methods_in_statements(body, methods),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                count_methods_in_statements(if_body, methods)
                    + count_methods_in_statements(else_body, methods)
            }
            Statement::UserTypeDeclaration {
                methods: user_methods,
                ..
            } => user_methods
                .iter()
                .map(|method| count_methods_in_statements(&method.body, methods))
                .sum(),
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => 0,
        })
        .sum()
}

#[test]
fn dialogue_program_contains_say_think_and_text_bubble_actions() {
    let program = dialogue_program();

    assert_eq!(count_methods(&program, &["say"]), 1);
    assert_eq!(count_methods(&program, &["think"]), 1);
    assert_eq!(count_methods(&program, &["setText", "show"]), 2);
}

#[test]
fn dialogue_program_passes_narrative_grading() {
    let report = grade_games_and_narrative(narrative_input(Some(dialogue_program())));

    assert!(
        report.passed,
        "dialogue-heavy project should pass narrative grading"
    );
    assert_eq!(report.lesson, "games-and-interactive-narrative");
    assert_eq!(
        report
            .steps
            .iter()
            .find(|step| step.name == "detect-do-in-order")
            .unwrap()
            .status,
        StepStatus::Ready
    );
    assert_eq!(
        report
            .steps
            .iter()
            .find(|step| step.name == "detect-dialogue-sequence")
            .unwrap()
            .status,
        StepStatus::Ready
    );
    assert_eq!(
        report
            .steps
            .iter()
            .find(|step| step.name == "grade-narrative-project")
            .unwrap()
            .status,
        StepStatus::Ready
    );
}

#[test]
fn dialogue_program_survives_json_round_trip() {
    let program = dialogue_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert_eq!(count_methods(&restored, &["say", "think"]), 2);
}
