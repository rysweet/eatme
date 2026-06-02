// Expanded curriculum scenario coverage for camera rigs, audio, property animation,
// multi-scene transfers, and vehicle relationships.

use eatme_assets::{CreativeProjectGradingInput, StepStatus, grade_creative_project};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn expanded_curriculum_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this".into(),
                        method: "setupCameraRig".into(),
                        arguments: vec!["this.hero".into()],
                    },
                    Statement::MethodCall {
                        object: "this".into(),
                        method: "runConcertScene".into(),
                        arguments: vec![],
                    },
                    Statement::MethodCall {
                        object: "this".into(),
                        method: "transitionToFestival".into(),
                        arguments: vec!["this.float".into()],
                    },
                ],
            },
            Procedure {
                name: "setupCameraRig".into(),
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
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "moveToward".into(),
                        arguments: vec!["target".into(), "1.5".into()],
                    },
                ],
            },
            Procedure {
                name: "runConcertScene".into(),
                parameters: vec![],
                body: vec![
                    Statement::DoInOrder {
                        body: vec![
                            Statement::MethodCall {
                                object: "this.soundtrack".into(),
                                method: "playAudio".into(),
                                arguments: vec!["\"festival-theme\"".into()],
                            },
                            Statement::MethodCall {
                                object: "this.soundtrack".into(),
                                method: "setVolume".into(),
                                arguments: vec!["0.65".into()],
                            },
                            Statement::MethodCall {
                                object: "this.hero".into(),
                                method: "setOpacity".into(),
                                arguments: vec!["0.35".into(), "1.0".into()],
                            },
                            Statement::MethodCall {
                                object: "this.lantern".into(),
                                method: "setPaint".into(),
                                arguments: vec!["Color.ORANGE".into(), "1.0".into()],
                            },
                            Statement::MethodCall {
                                object: "this.float".into(),
                                method: "resize".into(),
                                arguments: vec!["1.2".into(), "1.5".into()],
                            },
                        ],
                    },
                    Statement::CountLoop {
                        count: 2,
                        body: vec![Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "moveToward".into(),
                            arguments: vec!["this.stage".into(), "0.5".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.camera".into(),
                            method: "pointAt".into(),
                            arguments: vec!["this.float".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "transitionToFestival".into(),
                parameters: vec![Parameter {
                    name: "floatVehicle".into(),
                    param_type: "Object".into(),
                }],
                body: vec![
                    Statement::MethodCall {
                        object: "this.sceneManager".into(),
                        method: "showScene".into(),
                        arguments: vec!["\"harbor\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.paradeLeader".into(),
                        method: "setVehicle".into(),
                        arguments: vec!["floatVehicle".into()],
                    },
                    Statement::MethodCall {
                        object: "this.paradeLeader".into(),
                        method: "moveToward".into(),
                        arguments: vec!["this.stage".into(), "1.0".into()],
                    },
                    Statement::MethodCall {
                        object: "this.sceneManager".into(),
                        method: "showScene".into(),
                        arguments: vec!["\"festival\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "pointAt".into(),
                        arguments: vec!["this.paradeLeader".into()],
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
        asset_reason: "expanded curriculum fixture parsed".into(),
        deps_available: true,
        deps_reason: "expanded curriculum grading ready".into(),
        student_program: program,
    }
}

fn count_method_calls(statements: &[Statement], method: &str) -> usize {
    statements
        .iter()
        .map(|statement| match statement {
            Statement::MethodCall { method: call, .. } if call == method => 1,
            Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => count_method_calls(body, method),
            Statement::IfElse {
                if_body, else_body, ..
            } => count_method_calls(if_body, method) + count_method_calls(else_body, method),
            Statement::UserTypeDeclaration { methods, .. } => methods
                .iter()
                .map(|method_decl| count_method_calls(&method_decl.body, method))
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
fn expanded_curriculum_program_covers_camera_audio_animation_scene_and_vehicle_scenarios() {
    let program = expanded_curriculum_program();
    let statements = &program.procedures;

    assert!(
        statements
            .iter()
            .any(|procedure| procedure.name == "setupCameraRig")
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "setVehicle"))
            .sum::<usize>(),
        2
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "pointAt"))
            .sum::<usize>(),
        3
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "moveToward"))
            .sum::<usize>(),
        3
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "playAudio"))
            .sum::<usize>(),
        1
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "setVolume"))
            .sum::<usize>(),
        1
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "setOpacity"))
            .sum::<usize>(),
        1
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "setPaint"))
            .sum::<usize>(),
        1
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "resize"))
            .sum::<usize>(),
        1
    );
    assert_eq!(
        program
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "showScene"))
            .sum::<usize>(),
        2
    );
}

#[test]
fn expanded_curriculum_program_passes_creative_grading() {
    let report = grade_creative_project(creative_input(Some(expanded_curriculum_program())));

    assert!(
        report.passed,
        "expanded curriculum project should satisfy creative grading"
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
fn expanded_curriculum_program_survives_json_round_trip() {
    let program = expanded_curriculum_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert_eq!(
        restored
            .procedures
            .iter()
            .map(|procedure| count_method_calls(&procedure.body, "playAudio"))
            .sum::<usize>(),
        1
    );
}
