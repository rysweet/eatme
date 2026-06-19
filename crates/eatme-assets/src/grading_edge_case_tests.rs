use crate::{
    CommentsGradingInput, CreativeProjectGradingInput, EventsGradingInput, FunctionsGradingInput,
    GamesNarrativeGradingInput, StepStatus, grade_comments, grade_creative_project,
    grade_events_and_collision, grade_functions, grade_games_and_narrative,
};
use eatme_core::ast::{Function, Procedure, Program, Statement};

fn ready_comments_input(program: Program) -> CommentsGradingInput {
    CommentsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    }
}

fn ready_functions_input(program: Program) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    }
}

fn ready_events_input(program: Program) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    }
}

fn ready_games_input(program: Program) -> GamesNarrativeGradingInput {
    GamesNarrativeGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(program),
    }
}

fn comment_only_program() -> Program {
    Program::new(vec![Procedure {
        name: "explainOnly".into(),
        parameters: vec![],
        body: vec![Statement::Comment {
            text: "This scene should eventually tell a story".into(),
        }],
    }])
}

fn duplicate_method_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "updateScore".into(),
            parameters: vec![],
            body: vec![Statement::FunctionCall {
                object: "this".into(),
                function: "updateScore".into(),
                arguments: vec![],
            }],
        }],
        functions: vec![
            Function {
                name: "updateScore".into(),
                return_type: "Number".into(),
                body: vec![Statement::ReturnStatement {
                    expression: "1".into(),
                }],
            },
            Function {
                name: "updateScore".into(),
                return_type: "Number".into(),
                body: vec![Statement::ReturnStatement {
                    expression: "2".into(),
                }],
            },
        ],
    }
}

fn circular_event_program() -> Program {
    Program::new(vec![Procedure {
        name: "gameLoop".into(),
        parameters: vec![],
        body: vec![
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::EventListener {
                    event: "TimerTick".into(),
                    body: vec![Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![
                            Statement::CollisionListener {
                                object_a: "this.cat".into(),
                                object_b: "this.dog".into(),
                                body: vec![],
                            },
                            Statement::VariableAssignment {
                                name: "score".into(),
                                value: "score + 1".into(),
                            },
                            Statement::IfElse {
                                condition: "score > 0".into(),
                                if_body: vec![Statement::MethodCall {
                                    object: "this.cat".into(),
                                    method: "say".into(),
                                    arguments: vec!["\"win\"".into()],
                                }],
                                else_body: vec![],
                            },
                        ],
                    }],
                }],
            },
            Statement::DoInOrder {
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"hello\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.dog".into(),
                        method: "say".into(),
                        arguments: vec!["\"goodbye\"".into()],
                    },
                ],
            },
        ],
    }])
}

fn large_valid_function_program() -> Program {
    let mut body = Vec::new();
    for index in 0..110 {
        body.push(Statement::MethodCall {
            object: "this.hero".into(),
            method: "move".into(),
            arguments: vec![format!("step-{index}")],
        });
    }
    body.push(Statement::FunctionCall {
        object: "this".into(),
        function: "computeScore".into(),
        arguments: vec![],
    });

    Program {
        procedures: vec![Procedure {
            name: "playRound".into(),
            parameters: vec![],
            body,
        }],
        functions: vec![Function {
            name: "computeScore".into(),
            return_type: "Number".into(),
            body: vec![Statement::ReturnStatement {
                expression: "42".into(),
            }],
        }],
    }
}

#[test]
fn comment_only_program_does_not_pass_comment_grading() {
    let report = grade_comments(ready_comments_input(comment_only_program()));
    assert!(!report.passed);
    let save_step = report
        .steps
        .iter()
        .find(|step| step.name == "save-project")
        .unwrap();
    assert_eq!(save_step.status, StepStatus::Blocked);
}

#[test]
fn duplicate_method_names_block_function_grading() {
    let report = grade_functions(ready_functions_input(duplicate_method_program()));
    assert!(!report.passed);
    let create_step = report
        .steps
        .iter()
        .find(|step| step.name == "create-function")
        .unwrap();
    assert_eq!(create_step.status, StepStatus::Blocked);
    assert!(create_step.reason.contains("Duplicate method names"));
}

#[test]
fn circular_event_references_do_not_pass_event_driven_graders() {
    let events_report = grade_events_and_collision(ready_events_input(circular_event_program()));
    assert!(!events_report.passed);

    let games_report = grade_games_and_narrative(ready_games_input(circular_event_program()));
    assert!(!games_report.passed);
    for step_name in ["grade-game-project", "grade-narrative-project"] {
        let step = games_report
            .steps
            .iter()
            .find(|step| step.name == step_name)
            .unwrap();
        assert_eq!(step.status, StepStatus::Blocked, "{step_name}");
        assert!(
            step.reason.contains("Circular event references"),
            "{step_name}"
        );
    }
}

#[test]
fn very_large_program_still_grades_correctly() {
    let report = grade_functions(ready_functions_input(large_valid_function_program()));
    assert!(
        report.passed,
        "large valid program should still pass grading"
    );
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready),
        "all steps should be ready: {:?}",
        report
            .steps
            .iter()
            .map(|step| (&step.name, &step.status))
            .collect::<Vec<_>>()
    );
}

#[test]
fn comment_only_program_also_fails_capstone_grading() {
    let report = grade_creative_project(CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(comment_only_program()),
    });
    assert!(!report.passed);
}
