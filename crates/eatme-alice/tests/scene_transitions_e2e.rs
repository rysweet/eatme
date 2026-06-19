// Scene-transition E2E tests: validates Alice.org multi-scene lesson flows.

use eatme_assets::{
    CreativeProjectGradingInput, ParametersGradingInput, StepStatus, grade_creative_project,
    grade_parameters,
};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn multi_scene_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this".into(),
                        method: "enterHarborScene".into(),
                        arguments: vec!["0".into()],
                    },
                    Statement::MethodCall {
                        object: "this.seagull".into(),
                        method: "say".into(),
                        arguments: vec!["\"Set sail!\"".into()],
                    },
                ],
            },
            Procedure {
                name: "enterHarborScene".into(),
                parameters: vec![Parameter {
                    name: "score".into(),
                    param_type: "WholeNumber".into(),
                }],
                body: vec![
                    Statement::MethodCall {
                        object: "this.sceneManager".into(),
                        method: "showScene".into(),
                        arguments: vec!["\"harbor\"".into()],
                    },
                    Statement::IfElse {
                        condition: "score >= 3".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this".into(),
                            method: "enterCastleScene".into(),
                            arguments: vec!["score".into(), "\"north-key\"".into()],
                        }],
                        else_body: vec![Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "say".into(),
                            arguments: vec!["\"Need more stars first.\"".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.hud".into(),
                            method: "setText".into(),
                            arguments: vec!["score".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "enterCastleScene".into(),
                parameters: vec![
                    Parameter {
                        name: "score".into(),
                        param_type: "WholeNumber".into(),
                    },
                    Parameter {
                        name: "inventoryItem".into(),
                        param_type: "String".into(),
                    },
                ],
                body: vec![
                    Statement::MethodCall {
                        object: "this.sceneManager".into(),
                        method: "showScene".into(),
                        arguments: vec!["\"castle\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.hud".into(),
                        method: "setText".into(),
                        arguments: vec!["inventoryItem".into()],
                    },
                ],
            },
        ],
        functions: vec![],
    }
}

fn parameters_input(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "scene fixture parsed".into(),
        deps_available: true,
        deps_reason: "scene handoff grading ready".into(),
        student_program: program,
    }
}

fn creative_input(program: Option<Program>) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "scene fixture parsed".into(),
        deps_available: true,
        deps_reason: "scene project grading ready".into(),
        student_program: program,
    }
}

fn has_scene_transition(program: &Program, destination: &str) -> bool {
    program.procedures.iter().any(|procedure| {
        procedure.body.iter().any(|statement| {
            matches!(
                statement,
                Statement::MethodCall {
                    object,
                    method,
                    arguments,
                } if object == "this.sceneManager"
                    && method == "showScene"
                    && arguments.iter().any(|argument| argument == destination)
            )
        })
    })
}

fn has_scene_handoff_call(program: &Program, method_name: &str) -> bool {
    program
        .procedures
        .iter()
        .any(|procedure| has_scene_handoff_call_in_statements(&procedure.body, method_name))
}

fn has_scene_handoff_call_in_statements(statements: &[Statement], method_name: &str) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::MethodCall {
            object,
            method,
            arguments,
        } => object == "this" && method == method_name && arguments.len() >= 2,
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => {
            has_scene_handoff_call_in_statements(body, method_name)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            has_scene_handoff_call_in_statements(if_body, method_name)
                || has_scene_handoff_call_in_statements(else_body, method_name)
        }
        Statement::UserTypeDeclaration { methods, .. } => methods.iter().any(|method_decl| {
            has_scene_handoff_call_in_statements(&method_decl.body, method_name)
        }),
        Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. } => false,
    })
}

#[test]
fn multi_scene_program_contains_scene_changes_and_data_handoffs() {
    let program = multi_scene_program();

    assert!(has_scene_transition(&program, "\"harbor\""));
    assert!(has_scene_transition(&program, "\"castle\""));
    assert!(has_scene_handoff_call(&program, "enterCastleScene"));
}

#[test]
fn multi_scene_program_passes_scene_handoff_grading() {
    let program = multi_scene_program();
    let parameters_report = grade_parameters(parameters_input(Some(program.clone())));
    let creative_report = grade_creative_project(creative_input(Some(program)));

    assert!(
        parameters_report.passed,
        "scene handoff should satisfy parameter grading"
    );
    assert_eq!(parameters_report.lesson, "parameters-mini-challenge");
    assert_eq!(
        parameters_report
            .steps
            .iter()
            .find(|step| step.name == "create-parameterized-procedure")
            .unwrap()
            .status,
        StepStatus::Ready
    );
    assert_eq!(
        parameters_report
            .steps
            .iter()
            .find(|step| step.name == "call-with-argument")
            .unwrap()
            .status,
        StepStatus::Ready
    );
    assert!(
        creative_report.passed,
        "multi-scene project should satisfy creative grading"
    );
}

#[test]
fn multi_scene_program_survives_json_round_trip() {
    let program = multi_scene_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert!(has_scene_handoff_call(&restored, "enterCastleScene"));
}
