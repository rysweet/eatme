use eatme_assets::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput, GradingInput,
    InheritanceOopGradingInput, LoopsGradingInput, NestedControlFlowGradingInput,
    ParametersGradingInput, SceneBuildingGradingInput, SequencingGradingInput, StepStatus,
    TextbookIntegrationGradingInput, VariablesGradingInput, grade_arrays_and_arithmetic,
    grade_comments, grade_creative_project, grade_events_and_collision,
    grade_first_lesson_readiness, grade_functions, grade_games_and_narrative,
    grade_inheritance_oop, grade_loops_and_conditionals, grade_nested_control_flow,
    grade_parameters, grade_scene_building, grade_sequencing, grade_textbook_integration,
    grade_variables,
};
use eatme_core::ast::{
    ArithmeticOperator, CameraPose, Function, Parameter, Procedure, Program, SceneLayout,
    SceneObject, SequenceBlock, SequenceKind, Statement, Vec3,
};

fn ready_reason() -> String {
    "all grading prerequisites satisfied".into()
}

fn scene_layout() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: true,
        objects: vec![
            SceneObject {
                name: "rabbit".into(),
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
                name: "fox".into(),
                kind: "SBiped".into(),
                position: Some(Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: -1.0,
                }),
                size: Some(1.1),
                color: Some("orange".into()),
                opacity: Some(1.0),
            },
            SceneObject {
                name: "tree".into(),
                kind: "SProp".into(),
                position: Some(Vec3 {
                    x: -3.0,
                    y: 0.0,
                    z: 2.5,
                }),
                size: Some(1.8),
                color: Some("green".into()),
                opacity: Some(1.0),
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

fn sequence_blocks() -> Vec<SequenceBlock> {
    vec![
        SequenceBlock {
            kind: SequenceKind::DoInOrder,
            steps: vec!["rabbit.say".into(), "fox.think".into()],
        },
        SequenceBlock {
            kind: SequenceKind::DoTogether,
            steps: vec!["rabbit.move".into(), "fox.move".into()],
        },
    ]
}

fn comprehensive_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::Comment {
                        text: "Coordinate the forest game story with reusable logic".into(),
                    },
                    Statement::VariableDeclaration {
                        name: "score".into(),
                        var_type: "WholeNumber".into(),
                        initial_value: "0".into(),
                    },
                    Statement::VariableDeclaration {
                        name: "timer".into(),
                        var_type: "WholeNumber".into(),
                        initial_value: "10".into(),
                    },
                    Statement::VariableDeclaration {
                        name: "speed".into(),
                        var_type: "DecimalNumber".into(),
                        initial_value: "1.5".into(),
                    },
                    Statement::ArrayDeclaration {
                        name: "waypoints".into(),
                        element_type: "DecimalNumber".into(),
                        elements: vec!["1.0".into(), "2.0".into(), "3.0".into()],
                    },
                    Statement::ArrayAccess {
                        array: "waypoints".into(),
                        index: "1".into(),
                        target: "currentWaypoint".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Add,
                        left: "score".into(),
                        right: "1".into(),
                        result: "scorePlusOne".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Subtract,
                        left: "timer".into(),
                        right: "1".into(),
                        result: "timeRemaining".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Multiply,
                        left: "speed".into(),
                        right: "2".into(),
                        result: "doubleSpeed".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Divide,
                        left: "doubleSpeed".into(),
                        right: "2".into(),
                        result: "normalizedSpeed".into(),
                    },
                    Statement::DoInOrder {
                        body: vec![
                            Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "say".into(),
                                arguments: vec!["\"Welcome to the forest mission\"".into()],
                            },
                            Statement::MethodCall {
                                object: "this.fox".into(),
                                method: "think".into(),
                                arguments: vec!["\"Let's start the quest\"".into()],
                            },
                        ],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![
                            Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "move".into(),
                                arguments: vec!["FORWARD".into(), "speed".into()],
                            },
                            Statement::IfElse {
                                condition: "score > 0 && timer < 20".into(),
                                if_body: vec![
                                    Statement::VariableAssignment {
                                        name: "score".into(),
                                        value: "scorePlusOne".into(),
                                    },
                                    Statement::MethodCall {
                                        object: "this.rabbit".into(),
                                        method: "say".into(),
                                        arguments: vec!["\"Scene ready\"".into()],
                                    },
                                ],
                                else_body: vec![Statement::MethodCall {
                                    object: "this.rabbit".into(),
                                    method: "think".into(),
                                    arguments: vec!["\"Waiting\"".into()],
                                }],
                            },
                        ],
                    },
                    Statement::CollisionListener {
                        object_a: "this.rabbit".into(),
                        object_b: "this.fox".into(),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "score".into(),
                                value: "scorePlusOne".into(),
                            },
                            Statement::IfElse {
                                condition: "timer > 0 || score == 0".into(),
                                if_body: vec![Statement::CountLoop {
                                    count: 2,
                                    body: vec![Statement::ForEachArray {
                                        item_name: "step".into(),
                                        array: "waypoints".into(),
                                        body: vec![Statement::IfElse {
                                            condition: "step < 3 && score > 0 || score == 0".into(),
                                            if_body: vec![Statement::MethodCall {
                                                object: "this.fox".into(),
                                                method: "move".into(),
                                                arguments: vec!["FORWARD".into(), "step".into()],
                                            }],
                                            else_body: vec![Statement::MethodCall {
                                                object: "this.fox".into(),
                                                method: "turn".into(),
                                                arguments: vec!["LEFT".into(), "0.25".into()],
                                            }],
                                        }],
                                    }],
                                }],
                                else_body: vec![Statement::MethodCall {
                                    object: "this.fox".into(),
                                    method: "say".into(),
                                    arguments: vec!["\"Game over\"".into()],
                                }],
                            },
                        ],
                    },
                    Statement::MethodCall {
                        object: "this".into(),
                        method: "moveHero".into(),
                        arguments: vec!["normalizedSpeed".into(), "\"Go!\"".into()],
                    },
                    Statement::FunctionCall {
                        object: "this".into(),
                        function: "computeBonus".into(),
                        arguments: vec!["score".into(), "timer".into()],
                    },
                    Statement::VariableAssignment {
                        name: "timer".into(),
                        value: "timeRemaining".into(),
                    },
                    Statement::UserTypeDeclaration {
                        name: "FlyingRabbit".into(),
                        extends: Some("SBiped".into()),
                        methods: vec![Procedure {
                            name: "celebrate".into(),
                            parameters: vec![],
                            body: vec![Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "say".into(),
                                arguments: vec!["\"Victory!\"".into()],
                            }],
                        }],
                    },
                ],
            },
            Procedure {
                name: "moveHero".into(),
                parameters: vec![
                    Parameter {
                        name: "distance".into(),
                        param_type: "DecimalNumber".into(),
                    },
                    Parameter {
                        name: "message".into(),
                        param_type: "TextString".into(),
                    },
                ],
                body: vec![
                    Statement::MethodCall {
                        object: "this.rabbit".into(),
                        method: "say".into(),
                        arguments: vec!["message".into()],
                    },
                    Statement::MethodCall {
                        object: "this.rabbit".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "distance".into()],
                    },
                ],
            },
        ],
        functions: vec![Function {
            name: "computeBonus".into(),
            return_type: "WholeNumber".into(),
            body: vec![Statement::ReturnStatement {
                expression: "score + timer".into(),
            }],
        }],
    }
}

fn assert_no_blocked_steps(report_name: &str, steps: &[eatme_assets::StepGrade]) {
    assert!(
        steps.iter().all(|step| step.status != StepStatus::Blocked),
        "{report_name} should not have blocked steps: {steps:?}"
    );
}

#[test]
fn single_program_scores_well_across_all_curriculum_areas() {
    let program = comprehensive_program();
    let scene = scene_layout();
    let sequencing = sequence_blocks();

    let first_lesson = grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
    });
    let scene_building = grade_scene_building(SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_scene: Some(scene),
    });
    let sequencing_report = grade_sequencing(SequencingGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        sequence_blocks: Some(sequencing),
    });
    let arrays = grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let events = grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let variables = grade_variables(VariablesGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let loops = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let functions = grade_functions(FunctionsGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let parameters = grade_parameters(ParametersGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let comments = grade_comments(CommentsGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let inheritance = grade_inheritance_oop(InheritanceOopGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let nested_control = grade_nested_control_flow(NestedControlFlowGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let textbook = grade_textbook_integration(TextbookIntegrationGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let games = grade_games_and_narrative(GamesNarrativeGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program.clone()),
    });
    let creative = grade_creative_project(CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: ready_reason(),
        deps_available: true,
        deps_reason: ready_reason(),
        student_program: Some(program),
    });

    for (name, report) in [
        ("first_lesson", &first_lesson),
        ("scene_building", &scene_building),
        ("sequencing", &sequencing_report),
        ("arrays", &arrays),
        ("events", &events),
        ("variables", &variables),
        ("loops", &loops),
        ("functions", &functions),
        ("parameters", &parameters),
        ("comments", &comments),
        ("inheritance", &inheritance),
        ("nested_control", &nested_control),
        ("textbook", &textbook),
        ("games", &games),
        ("creative", &creative),
    ] {
        assert_no_blocked_steps(name, &report.steps);
    }

    assert!(!first_lesson.passed);
    assert_eq!(first_lesson.steps[3].status, StepStatus::NotYetTested);
    assert_eq!(first_lesson.steps[4].status, StepStatus::NotYetTested);
    assert_eq!(first_lesson.steps[5].status, StepStatus::NotYetTested);

    assert!(scene_building.passed);
    assert!(sequencing_report.passed);
    assert!(arrays.passed);
    assert!(!events.passed);
    assert_eq!(events.steps[5].status, StepStatus::NotYetTested);
    assert!(variables.passed);
    assert!(!loops.passed);
    assert_eq!(loops.steps[5].status, StepStatus::NotYetTested);
    assert!(functions.passed);
    assert!(parameters.passed);
    assert!(comments.passed);
    assert!(inheritance.passed);
    assert!(nested_control.passed);
    assert!(textbook.passed);
    assert!(games.passed);
    assert!(creative.passed);

    assert_eq!(events.quality_scores[0].score, 100);
    assert_eq!(variables.quality_scores[0].score, 100);
    assert_eq!(parameters.quality_scores[0].score, 100);
}
