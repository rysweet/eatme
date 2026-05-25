// Multi-object interaction E2E tests: validates Alice.org inter-object behavior lesson flows.

use eatme_assets::{CreativeProjectGradingInput, StepStatus, grade_creative_project};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn multi_object_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.rabbit".into(),
                        method: "moveToward".into(),
                        arguments: vec!["this.fox".into(), "0.5".into()],
                    },
                    Statement::MethodCall {
                        object: "this.fox".into(),
                        method: "moveToward".into(),
                        arguments: vec!["this.rabbit".into(), "0.5".into()],
                    },
                    Statement::IfElse {
                        condition: "this.robot getDistanceTo this.target < 2.0".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this.robot".into(),
                            method: "say".into(),
                            arguments: vec!["\"I found you!\"".into()],
                        }],
                        else_body: vec![Statement::MethodCall {
                            object: "this.robot".into(),
                            method: "moveToward".into(),
                            arguments: vec!["this.target".into(), "0.25".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.rider".into(),
                            method: "setVehicle".into(),
                            arguments: vec!["this.horse".into()],
                        }],
                    },
                    Statement::CollisionListener {
                        object_a: "this.rabbit".into(),
                        object_b: "this.fox".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.rabbit".into(),
                            method: "say".into(),
                            arguments: vec!["\"Tag!\"".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "rideTogether".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.horse".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "distance".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn creative_input(program: Option<Program>) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "interaction fixture parsed".into(),
        deps_available: true,
        deps_reason: "interaction grading ready".into(),
        student_program: program,
    }
}

fn has_method_call(program: &Program, object: &str, method: &str, argument: &str) -> bool {
    program
        .procedures
        .iter()
        .any(|procedure| has_method_call_in_statements(&procedure.body, object, method, argument))
}

fn has_method_call_in_statements(
    statements: &[Statement],
    object: &str,
    method: &str,
    argument: &str,
) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::MethodCall {
            object: call_object,
            method: call_method,
            arguments,
        } => {
            call_object == object
                && call_method == method
                && arguments.iter().any(|value| value == argument)
        }
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => {
            has_method_call_in_statements(body, object, method, argument)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            has_method_call_in_statements(if_body, object, method, argument)
                || has_method_call_in_statements(else_body, object, method, argument)
        }
        Statement::UserTypeDeclaration { methods, .. } => methods.iter().any(|method_decl| {
            has_method_call_in_statements(&method_decl.body, object, method, argument)
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

fn has_position_response(program: &Program) -> bool {
    program.procedures.iter().any(|procedure| {
        procedure.body.iter().any(|statement| {
            matches!(
                statement,
                Statement::IfElse { condition, .. }
                    if condition.contains("this.robot") && condition.contains("this.target")
            )
        })
    })
}

#[test]
fn interaction_program_contains_motion_response_and_vehicle_relationships() {
    let program = multi_object_program();

    assert!(has_method_call(
        &program,
        "this.rabbit",
        "moveToward",
        "this.fox"
    ));
    assert!(has_method_call(
        &program,
        "this.fox",
        "moveToward",
        "this.rabbit"
    ));
    assert!(has_position_response(&program));
    assert!(has_method_call(
        &program,
        "this.rider",
        "setVehicle",
        "this.horse"
    ));
}

#[test]
fn interaction_program_passes_creative_grading() {
    let report = grade_creative_project(creative_input(Some(multi_object_program())));

    assert!(
        report.passed,
        "inter-object project should satisfy creative grading"
    );
    assert_eq!(report.lesson, "creative-design-project");
    assert_eq!(
        report
            .steps
            .iter()
            .find(|step| step.name == "add-control-structure")
            .unwrap()
            .status,
        StepStatus::Ready
    );
    assert_eq!(
        report
            .steps
            .iter()
            .find(|step| step.name == "add-event-or-interaction")
            .unwrap()
            .status,
        StepStatus::Ready
    );
}

#[test]
fn interaction_program_survives_json_round_trip() {
    let program = multi_object_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert!(has_method_call(
        &restored,
        "this.rider",
        "setVehicle",
        "this.horse"
    ));
}
