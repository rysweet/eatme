use std::fs;
use std::path::{Path, PathBuf};

use eatme_assets::{
    SceneBuildingGradingInput, StepStatus, grade_scene_building, score_event_quality,
    score_parameter_quality, score_variable_quality, validate_scenario_asset,
};
use eatme_core::ast::{
    CameraPose, Parameter, Procedure, Program, SceneLayout, SceneObject, Statement, Vec3,
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn design_process_story_or_game_yaml() -> String {
    let path = repository_root().join("assets/scenarios/eatme/design-process-story-or-game.yaml");
    fs::read_to_string(path).expect("design process scenario should exist")
}

fn ready_count(statuses: &[StepStatus]) -> usize {
    statuses
        .iter()
        .filter(|status| **status == StepStatus::Ready)
        .count()
}

fn simple_scene_layout() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: true,
        objects: vec![
            SceneObject {
                name: "bunny".into(),
                kind: "SBiped".into(),
                position: Some(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                size: Some(1.0),
                color: Some("white".into()),
                opacity: Some(1.0),
            },
            SceneObject {
                name: "tree".into(),
                kind: "SProp".into(),
                position: None,
                size: Some(1.5),
                color: Some("green".into()),
                opacity: Some(1.0),
            },
        ],
        camera: None,
    }
}

fn polished_scene_layout() -> SceneLayout {
    SceneLayout {
        camera: Some(CameraPose {
            position: Vec3 {
                x: 0.0,
                y: 4.0,
                z: 12.0,
            },
        }),
        objects: vec![
            SceneObject {
                name: "bunny".into(),
                kind: "SBiped".into(),
                position: Some(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                size: Some(1.0),
                color: Some("white".into()),
                opacity: Some(1.0),
            },
            SceneObject {
                name: "tree".into(),
                kind: "SProp".into(),
                position: Some(Vec3 {
                    x: 2.5,
                    y: 0.0,
                    z: -3.0,
                }),
                size: Some(1.5),
                color: Some("green".into()),
                opacity: Some(1.0),
            },
        ],
        ..simple_scene_layout()
    }
}

fn code_review_program(param_type: &str, explicit_entities: bool, use_variable: bool) -> Program {
    let cat_ref = if explicit_entities { "this.cat" } else { "cat" };
    let dog_ref = if explicit_entities { "this.dog" } else { "dog" };
    let variable_argument = if use_variable { "speed" } else { "distance" };

    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![Parameter {
                name: "distance".into(),
                param_type: param_type.into(),
            }],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: cat_ref.into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), variable_argument.into()],
                    }],
                },
                Statement::CollisionListener {
                    object_a: cat_ref.into(),
                    object_b: dog_ref.into(),
                    body: vec![Statement::MethodCall {
                        object: cat_ref.into(),
                        method: "say".into(),
                        arguments: vec!["\"hello\"".into()],
                    }],
                },
            ],
        }],
        functions: vec![],
    }
}

#[test]
fn storyboarding_contract_bridges_scene_plan_to_named_alice_concepts() {
    let scenario_path =
        repository_root().join("assets/scenarios/eatme/design-process-story-or-game.yaml");
    let validation = validate_scenario_asset(&scenario_path).expect("scenario should validate");
    let contract = design_process_story_or_game_yaml();

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        contract.contains("sketch or describe at least two scenes")
            || contract.contains("sketch at least two scenes or game states"),
        "storyboard should require multiple scenes or game states"
    );
    assert!(
        contract.contains("design-to-code bridge card")
            && contract.contains("at least one Alice concept"),
        "storyboard contract should connect the scene plan to implementation concepts"
    );
}

#[test]
fn iterative_development_v2_adds_features_that_v1_is_missing() {
    let v1 = grade_scene_building(SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_scene: Some(simple_scene_layout()),
    });
    let v2 = grade_scene_building(SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_scene: Some(polished_scene_layout()),
    });

    let v1_statuses: Vec<_> = v1.steps.iter().map(|step| step.status.clone()).collect();
    let v2_statuses: Vec<_> = v2.steps.iter().map(|step| step.status.clone()).collect();

    assert!(!v1.passed, "v1 should still expose iteration gaps");
    assert!(
        v2.passed,
        "v2 should satisfy the full scene-building rubric"
    );
    assert!(ready_count(&v2_statuses) > ready_count(&v1_statuses));

    let v1_position_camera = v1
        .steps
        .iter()
        .find(|step| step.name == "position-camera")
        .expect("position-camera step");
    let v1_set_properties = v1
        .steps
        .iter()
        .find(|step| step.name == "set-object-properties")
        .expect("set-object-properties step");
    let v1_save = v1
        .steps
        .iter()
        .find(|step| step.name == "save-project")
        .expect("save-project step");
    let v2_set_properties = v2
        .steps
        .iter()
        .find(|step| step.name == "set-object-properties")
        .expect("set-object-properties step");
    let v2_save = v2
        .steps
        .iter()
        .find(|step| step.name == "save-project")
        .expect("save-project step");

    assert_eq!(
        v1_position_camera.status,
        StepStatus::Blocked,
        "v1 position-camera"
    );
    assert_eq!(
        v1_set_properties.status,
        StepStatus::Blocked,
        "v1 set-object-properties"
    );
    assert_eq!(
        v2_set_properties.status,
        StepStatus::Ready,
        "v2 set-object-properties"
    );
    assert_eq!(v1_save.status, StepStatus::Blocked, "v1 save-project");
    assert_eq!(v2_save.status, StepStatus::Ready, "v2 save-project");
}

#[test]
fn code_review_quality_patterns_reward_specific_types_explicit_entities_and_used_variables() {
    let strong = code_review_program("DecimalNumber", true, true);
    let weak = code_review_program("Object", false, false);

    let strong_parameter = score_parameter_quality(Some(&strong));
    let weak_parameter = score_parameter_quality(Some(&weak));
    assert!(strong_parameter[0].score > weak_parameter[0].score);

    let strong_event = score_event_quality(Some(&strong));
    let weak_event = score_event_quality(Some(&weak));
    assert!(strong_event[0].score > weak_event[0].score);
    assert!(strong_event[0].feedback.contains("explicit scene entities"));

    let strong_variable = score_variable_quality(Some(&strong));
    let weak_variable = score_variable_quality(Some(&weak));
    assert!(strong_variable[0].score > weak_variable[0].score);
    assert!(
        weak_variable[0]
            .feedback
            .contains("0 of 1 declared variables")
    );
}
