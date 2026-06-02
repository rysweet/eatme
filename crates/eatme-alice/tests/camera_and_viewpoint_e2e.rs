// Camera-and-viewpoint E2E tests: validates camera-focused Alice.org lesson flows.

use eatme_assets::{CreativeProjectGradingInput, StepStatus, grade_creative_project};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn camera_controls_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "setViewPoint".into(),
                        arguments: vec!["TOP".into()],
                    },
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "moveTo".into(),
                        arguments: vec!["this.castle".into(), "2.0".into()],
                    },
                    Statement::MethodCall {
                        object: "this.knight".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "1.0".into()],
                    },
                    Statement::CountLoop {
                        count: 2,
                        body: vec![Statement::MethodCall {
                            object: "this.camera".into(),
                            method: "turn".into(),
                            arguments: vec!["LEFT".into(), "0.125".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this".into(),
                            method: "followHero".into(),
                            arguments: vec!["this.knight".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "followHero".into(),
                parameters: vec![Parameter {
                    name: "target".into(),
                    param_type: "Object".into(),
                }],
                body: vec![
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "setVehicle".into(),
                        arguments: vec!["target".into()],
                    },
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "pointAt".into(),
                        arguments: vec!["target".into()],
                    },
                ],
            },
        ],
        functions: vec![],
    }
}

fn creative_input(program: Option<Program>) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "camera fixture parsed".into(),
        deps_available: true,
        deps_reason: "camera scenario grading ready".into(),
        student_program: program,
    }
}

fn has_camera_method(program: &Program, method: &str) -> bool {
    program
        .procedures
        .iter()
        .any(|procedure| has_camera_method_in_statements(&procedure.body, method))
}

fn has_camera_method_in_statements(statements: &[Statement], method: &str) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::MethodCall {
            object,
            method: call_method,
            ..
        } => object == "this.camera" && call_method == method,
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => has_camera_method_in_statements(body, method),
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            has_camera_method_in_statements(if_body, method)
                || has_camera_method_in_statements(else_body, method)
        }
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method_decl| has_camera_method_in_statements(&method_decl.body, method)),
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

fn has_camera_move_to_target(program: &Program, target: &str) -> bool {
    program
        .procedures
        .iter()
        .any(|procedure| has_camera_move_to_target_in_statements(&procedure.body, target))
}

fn has_camera_move_to_target_in_statements(statements: &[Statement], target: &str) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::MethodCall {
            object,
            method,
            arguments,
        } => {
            object == "this.camera"
                && method == "moveTo"
                && arguments.iter().any(|argument| argument == target)
        }
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => {
            has_camera_move_to_target_in_statements(body, target)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            has_camera_move_to_target_in_statements(if_body, target)
                || has_camera_move_to_target_in_statements(else_body, target)
        }
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method_decl| has_camera_move_to_target_in_statements(&method_decl.body, target)),
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
fn camera_program_contains_viewpoint_move_and_follow_controls() {
    let program = camera_controls_program();

    assert!(has_camera_method(&program, "setViewPoint"));
    assert!(has_camera_move_to_target(&program, "this.castle"));
    assert!(has_camera_method(&program, "setVehicle"));
}

#[test]
fn camera_controls_grade_as_a_creative_project() {
    let report = grade_creative_project(creative_input(Some(camera_controls_program())));

    assert!(
        report.passed,
        "camera-focused project should satisfy creative grading"
    );
    assert_eq!(report.lesson, "creative-design-project");
    for name in [
        "build-scene-with-objects",
        "create-custom-procedure",
        "add-control-structure",
        "add-event-or-interaction",
        "run-world",
        "save-project",
    ] {
        let step = report.steps.iter().find(|step| step.name == name).unwrap();
        assert_eq!(step.status, StepStatus::Ready, "{name} should be ready");
    }
}

#[test]
fn camera_controls_ast_survives_json_round_trip() {
    let program = camera_controls_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert!(has_camera_method(&restored, "pointAt"));
}
