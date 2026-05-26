// Joint and vehicle scenario E2E tests: covers nested vehicles, joint posing,
// IK-style pointing, multi-character interaction, and camera follow behaviors.

use eatme_assets::{CreativeProjectGradingInput, StepStatus, grade_creative_project};
use eatme_core::ast::{Procedure, Program, Statement};

fn joint_vehicle_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "vehicleParenting".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "setVehicle".into(),
                        arguments: vec!["this.platform".into()],
                    },
                    Statement::MethodCall {
                        object: "this.camera".into(),
                        method: "setVehicle".into(),
                        arguments: vec!["this.hero".into()],
                    },
                    Statement::MethodCall {
                        object: "this.platform".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "2.0".into()],
                    },
                ],
            },
            Procedure {
                name: "poseBipedJoints".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.hero.leftArm".into(),
                        method: "turn".into(),
                        arguments: vec!["FORWARD".into(), "0.25".into()],
                    },
                    Statement::MethodCall {
                        object: "this.hero.rightLeg".into(),
                        method: "turn".into(),
                        arguments: vec!["BACKWARD".into(), "0.125".into()],
                    },
                    Statement::MethodCall {
                        object: "this.hero.head".into(),
                        method: "turn".into(),
                        arguments: vec!["LEFT".into(), "0.0625".into()],
                    },
                ],
            },
            Procedure {
                name: "pointAtTargetWithIk".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.hero.rightArmIk".into(),
                        method: "solveIk".into(),
                        arguments: vec!["this.target".into()],
                    },
                    Statement::MethodCall {
                        object: "this.hero.rightHand".into(),
                        method: "pointAt".into(),
                        arguments: vec!["this.target".into()],
                    },
                ],
            },
            Procedure {
                name: "bipedMeetAndApproach".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "turnToFace".into(),
                        arguments: vec!["this.friend".into()],
                    },
                    Statement::MethodCall {
                        object: "this.friend".into(),
                        method: "turnToFace".into(),
                        arguments: vec!["this.hero".into()],
                    },
                    Statement::CountLoop {
                        count: 3,
                        body: vec![Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "walkToward".into(),
                            arguments: vec!["this.friend".into(), "0.5".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "cameraFollowPath".into(),
                parameters: vec![],
                body: vec![
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![
                            Statement::MethodCall {
                                object: "this.camera".into(),
                                method: "setVehicle".into(),
                                arguments: vec!["this.hero".into()],
                            },
                            Statement::MethodCall {
                                object: "this.camera".into(),
                                method: "pointAt".into(),
                                arguments: vec!["this.hero".into()],
                            },
                        ],
                    },
                    Statement::CountLoop {
                        count: 4,
                        body: vec![Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "move".into(),
                            arguments: vec!["FORWARD".into(), "1.0".into()],
                        }],
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
        asset_reason: "joint and vehicle fixture parsed".into(),
        deps_available: true,
        deps_reason: "joint and vehicle grading ready".into(),
        student_program: program,
    }
}

fn procedure<'a>(program: &'a Program, name: &str) -> &'a Procedure {
    program
        .procedures
        .iter()
        .find(|procedure| procedure.name == name)
        .unwrap_or_else(|| panic!("missing procedure {name}"))
}

fn has_method_call(statements: &[Statement], object: &str, method: &str, argument: &str) -> bool {
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
        | Statement::ForEachArray { body, .. } => has_method_call(body, object, method, argument),
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            has_method_call(if_body, object, method, argument)
                || has_method_call(else_body, object, method, argument)
        }
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method_decl| has_method_call(&method_decl.body, object, method, argument)),
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
fn vehicle_parenting_covers_camera_character_and_platform_chain() {
    let program = joint_vehicle_program();
    let scene = procedure(&program, "vehicleParenting");

    assert!(has_method_call(
        &scene.body,
        "this.hero",
        "setVehicle",
        "this.platform"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.camera",
        "setVehicle",
        "this.hero"
    ));
}

#[test]
fn joint_manipulation_covers_arms_legs_and_head_pose_changes() {
    let program = joint_vehicle_program();
    let scene = procedure(&program, "poseBipedJoints");

    assert!(has_method_call(
        &scene.body,
        "this.hero.leftArm",
        "turn",
        "0.25"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.hero.rightLeg",
        "turn",
        "0.125"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.hero.head",
        "turn",
        "0.0625"
    ));
}

#[test]
fn ik_pointing_covers_arm_chain_solving_and_targeted_pointing() {
    let program = joint_vehicle_program();
    let scene = procedure(&program, "pointAtTargetWithIk");

    assert!(has_method_call(
        &scene.body,
        "this.hero.rightArmIk",
        "solveIk",
        "this.target"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.hero.rightHand",
        "pointAt",
        "this.target"
    ));
}

#[test]
fn multi_character_interaction_covers_facing_and_walk_toward_behavior() {
    let program = joint_vehicle_program();
    let scene = procedure(&program, "bipedMeetAndApproach");

    assert!(has_method_call(
        &scene.body,
        "this.hero",
        "turnToFace",
        "this.friend"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.hero",
        "walkToward",
        "this.friend"
    ));
}

#[test]
fn camera_follow_covers_vehicle_attachment_and_target_tracking() {
    let program = joint_vehicle_program();
    let scene = procedure(&program, "cameraFollowPath");

    assert!(has_method_call(
        &scene.body,
        "this.camera",
        "setVehicle",
        "this.hero"
    ));
    assert!(has_method_call(
        &scene.body,
        "this.camera",
        "pointAt",
        "this.hero"
    ));
}

#[test]
fn joint_vehicle_scenarios_pass_creative_project_grading() {
    let report = grade_creative_project(creative_input(Some(joint_vehicle_program())));

    assert!(
        report.passed,
        "joint and vehicle project should satisfy creative grading"
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
fn joint_vehicle_scenarios_survive_json_round_trip() {
    let program = joint_vehicle_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, restored);
    assert!(has_method_call(
        &procedure(&restored, "pointAtTargetWithIk").body,
        "this.hero.rightHand",
        "pointAt",
        "this.target"
    ));
}
