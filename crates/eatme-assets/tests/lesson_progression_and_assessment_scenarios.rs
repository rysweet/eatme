use eatme_assets::{
    ArraysArithmeticGradingInput, GradingReport, ParametersGradingInput, SceneBuildingGradingInput,
    SequencingGradingInput, StepStatus, VariablesGradingInput, for_building_a_scene,
    grade_arrays_and_arithmetic, grade_parameters, grade_scene_building, grade_sequencing,
    grade_variables,
};
use eatme_core::ast::{
    ArithmeticOperator, CameraPose, Parameter, Procedure, Program, SceneLayout, SceneObject,
    SequenceBlock, SequenceKind, Statement, Vec3,
};

fn ready_scene_input(scene: SceneLayout) -> SceneBuildingGradingInput {
    SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        student_scene: Some(scene),
    }
}

fn ready_program_input(program: Program) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        student_program: Some(program),
    }
}

fn hello_world_scene() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: true,
        objects: vec![
            SceneObject {
                name: "textBanner".into(),
                kind: "SProp".into(),
                position: Some(Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: -4.0,
                }),
                size: Some(1.2),
                color: Some("yellow".into()),
                opacity: Some(1.0),
            },
            SceneObject {
                name: "cameraTarget".into(),
                kind: "SModel".into(),
                position: Some(Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: -2.0,
                }),
                size: Some(0.8),
                color: Some("blue".into()),
                opacity: Some(0.9),
            },
        ],
        camera: Some(CameraPose {
            position: Vec3 {
                x: 0.0,
                y: 4.0,
                z: 12.0,
            },
        }),
    }
}

fn scene_save_progression() -> Vec<SceneLayout> {
    vec![
        SceneLayout {
            ground_present: true,
            sky_present: true,
            objects: vec![
                SceneObject {
                    name: "hero".into(),
                    kind: "SBiped".into(),
                    position: Some(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    size: Some(1.0),
                    color: Some("red".into()),
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
        },
        SceneLayout {
            ground_present: true,
            sky_present: true,
            objects: vec![
                SceneObject {
                    name: "hero".into(),
                    kind: "SBiped".into(),
                    position: Some(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    size: Some(1.0),
                    color: Some("red".into()),
                    opacity: Some(1.0),
                },
                SceneObject {
                    name: "tree".into(),
                    kind: "SProp".into(),
                    position: Some(Vec3 {
                        x: 2.0,
                        y: 0.0,
                        z: -3.0,
                    }),
                    size: Some(1.5),
                    color: Some("green".into()),
                    opacity: Some(1.0),
                },
            ],
            camera: Some(CameraPose {
                position: Vec3 {
                    x: 0.0,
                    y: 4.0,
                    z: 10.0,
                },
            }),
        },
        hello_world_scene(),
    ]
}

fn complete_variables_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
                Statement::VariableAssignment {
                    name: "speed".into(),
                    value: "1.0".into(),
                },
            ],
        }],
        functions: vec![],
    }
}

fn variable_mistake_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "move".into(),
                    arguments: vec!["\"FORWARD\"".into(), "\"fast\"".into()],
                },
            ],
        }],
        functions: vec![],
    }
}

fn complete_parameters_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveByAmount".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "distance".into()],
                }],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveByAmount".into(),
                    arguments: vec!["1.0".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn complete_arrays_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::ArrayDeclaration {
                    name: "friends".into(),
                    element_type: "SThing".into(),
                    elements: vec!["this.cat".into(), "this.dog".into()],
                },
                Statement::ArrayAccess {
                    array: "friends".into(),
                    index: "0".into(),
                    target: "firstFriend".into(),
                },
                Statement::ForEachArray {
                    item_name: "friend".into(),
                    array: "friends".into(),
                    body: vec![Statement::MethodCall {
                        object: "friend".into(),
                        method: "say".into(),
                        arguments: vec!["\"hello\"".into()],
                    }],
                },
                Statement::ArithmeticExpression {
                    operator: ArithmeticOperator::Add,
                    left: "score".into(),
                    right: "1".into(),
                    result: "sum".into(),
                },
                Statement::ArithmeticExpression {
                    operator: ArithmeticOperator::Subtract,
                    left: "score".into(),
                    right: "1".into(),
                    result: "difference".into(),
                },
                Statement::ArithmeticExpression {
                    operator: ArithmeticOperator::Multiply,
                    left: "score".into(),
                    right: "2".into(),
                    result: "product".into(),
                },
                Statement::ArithmeticExpression {
                    operator: ArithmeticOperator::Divide,
                    left: "score".into(),
                    right: "2".into(),
                    result: "quotient".into(),
                },
            ],
        }],
        functions: vec![],
    }
}

fn sequencing_report(complete: bool) -> GradingReport {
    let sequence_blocks = if complete {
        Some(vec![
            SequenceBlock {
                kind: SequenceKind::DoInOrder,
                steps: vec!["walk".into(), "turn".into()],
            },
            SequenceBlock {
                kind: SequenceKind::DoTogether,
                steps: vec!["wave".into(), "smile".into()],
            },
        ])
    } else {
        Some(vec![SequenceBlock {
            kind: SequenceKind::DoInOrder,
            steps: vec!["walk".into(), "turn".into()],
        }])
    };

    grade_sequencing(SequencingGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        sequence_blocks,
    })
}

fn can_start_lesson(target_lesson: usize, reports: &[GradingReport]) -> bool {
    reports
        .iter()
        .take(target_lesson.saturating_sub(1))
        .all(|report| report.passed)
}

fn ready_steps(report: &GradingReport) -> usize {
    report
        .steps
        .iter()
        .filter(|step| step.status == StepStatus::Ready)
        .count()
}

#[test]
fn lesson_one_completion_hello_world_scene_meets_all_criteria() {
    let report = grade_scene_building(ready_scene_input(hello_world_scene()));

    assert!(report.passed, "{report:?}");
    assert_eq!(report.lesson, "building-a-scene-first-world");
    assert_eq!(report.steps.len(), 9);
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
}

#[test]
fn lesson_progression_gate_requires_lesson_two_before_lesson_three() {
    let lesson_one = grade_scene_building(ready_scene_input(hello_world_scene()));
    let incomplete_lesson_two = sequencing_report(false);
    let complete_lesson_two = sequencing_report(true);

    assert!(lesson_one.passed);
    assert!(!incomplete_lesson_two.passed);
    assert!(!can_start_lesson(
        3,
        &[lesson_one.clone(), incomplete_lesson_two]
    ));
    assert!(can_start_lesson(3, &[lesson_one, complete_lesson_two]));
}

#[test]
fn assessment_rubric_covers_machine_and_human_review_dimensions() {
    let rubric = for_building_a_scene();
    let report = grade_scene_building(ready_scene_input(hello_world_scene()));

    assert!(report.passed);
    assert_eq!(rubric.lesson, report.lesson);
    assert_eq!(rubric.machine_assessable.len(), 6);
    assert_eq!(rubric.human_review_needed.len(), 6);
    assert!(
        rubric
            .machine_assessable
            .iter()
            .any(|aspect| aspect.name == "world-ran-without-errors")
    );
    assert!(
        rubric
            .human_review_needed
            .iter()
            .any(|aspect| aspect.name == "student-can-explain-choices")
    );
}

#[test]
fn common_mistake_detection_reports_helpful_variable_feedback() {
    let report = grade_variables(ready_program_input(variable_mistake_program()));
    let use_variable = report
        .steps
        .iter()
        .find(|step| step.name == "use-variable-in-method")
        .expect("use-variable-in-method step");

    assert_eq!(use_variable.status, StepStatus::Blocked);
    assert!(
        use_variable
            .reason
            .contains("No variable used in a method call found in student program")
    );
    assert!(
        use_variable
            .reason
            .contains("save the project, and rerun grading")
    );
}

#[test]
fn incremental_save_keeps_each_scene_snapshot_valid() {
    let mut previous_ready = 0;

    for scene in scene_save_progression() {
        let saved = serde_json::to_vec(&scene).expect("scene should serialize");
        let restored: SceneLayout =
            serde_json::from_slice(&saved).expect("scene should deserialize");
        let report = grade_scene_building(ready_scene_input(restored));
        let current_ready = ready_steps(&report);

        assert!(
            current_ready >= previous_ready,
            "scene saves should not lose completed work"
        );
        previous_ready = current_ready;
    }

    assert!(
        previous_ready >= 8,
        "final save should be nearly or fully complete"
    );
}

#[test]
fn portfolio_review_summarizes_five_completed_lessons() {
    let reports = [
        grade_scene_building(ready_scene_input(hello_world_scene())),
        sequencing_report(true),
        grade_variables(ready_program_input(complete_variables_program())),
        grade_parameters(ParametersGradingInput {
            assets_valid: true,
            asset_reason: "assets ok".into(),
            deps_available: true,
            deps_reason: "deps ok".into(),
            student_program: Some(complete_parameters_program()),
        }),
        grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
            assets_valid: true,
            asset_reason: "assets ok".into(),
            deps_available: true,
            deps_reason: "deps ok".into(),
            student_program: Some(complete_arrays_program()),
        }),
    ];

    let completed = reports.iter().filter(|report| report.passed).count();
    let total_ready: usize = reports.iter().map(ready_steps).sum();
    let average_ready = total_ready as f32 / reports.len() as f32;

    assert_eq!(reports.len(), 5);
    assert_eq!(completed, 5, "all portfolio lessons should be complete");
    assert!(
        average_ready >= 7.0,
        "portfolio should show strong readiness statistics"
    );
}
