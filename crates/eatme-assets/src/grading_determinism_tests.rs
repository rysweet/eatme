use crate::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput, GradingInput,
    InheritanceOopGradingInput, LoopsGradingInput, NestedControlFlowGradingInput,
    ParametersGradingInput, SceneBuildingGradingInput, SequencingGradingInput,
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
use serde::Serialize;

fn assert_deterministic<T, F>(label: &str, run: F)
where
    T: PartialEq + Serialize + std::fmt::Debug,
    F: Fn() -> T,
{
    let first = run();
    let second = run();
    assert_eq!(
        first, second,
        "{label} should produce identical structured output"
    );

    let first_json = serde_json::to_string(&first).expect("serialize first grading result");
    let second_json = serde_json::to_string(&second).expect("serialize second grading result");
    assert_eq!(
        first_json, second_json,
        "{label} should serialize identically for repeated runs"
    );
}

fn ready_asset_reason() -> String {
    "All scenario assets passed validation".into()
}

fn ready_deps_reason() -> String {
    "All required tools available".into()
}

fn sample_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::Comment {
                        text: "Move the rabbit to explain the scoring logic.".into(),
                    },
                    Statement::VariableDeclaration {
                        name: "score".into(),
                        var_type: "WholeNumber".into(),
                        initial_value: "0".into(),
                    },
                    Statement::VariableAssignment {
                        name: "score".into(),
                        value: "1".into(),
                    },
                    Statement::ArrayDeclaration {
                        name: "steps".into(),
                        element_type: "Text".into(),
                        elements: vec!["\"start\"".into(), "\"middle\"".into(), "\"end\"".into()],
                    },
                    Statement::ArrayAccess {
                        array: "steps".into(),
                        index: "0".into(),
                        target: "currentStep".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Add,
                        left: "score".into(),
                        right: "1".into(),
                        result: "scorePlusOne".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Subtract,
                        left: "score".into(),
                        right: "1".into(),
                        result: "scoreMinusOne".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Multiply,
                        left: "score".into(),
                        right: "2".into(),
                        result: "doubleScore".into(),
                    },
                    Statement::ArithmeticExpression {
                        operator: ArithmeticOperator::Divide,
                        left: "score".into(),
                        right: "2".into(),
                        result: "halfScore".into(),
                    },
                    Statement::CountLoop {
                        count: 3,
                        body: vec![Statement::IfElse {
                            condition: "score < 10 && lives > 0".into(),
                            if_body: vec![Statement::IfElse {
                                condition: "score > 3 || score == 3".into(),
                                if_body: vec![Statement::CountLoop {
                                    count: 1,
                                    body: vec![Statement::MethodCall {
                                        object: "this.rabbit".into(),
                                        method: "move".into(),
                                        arguments: vec!["1.0".into()],
                                    }],
                                }],
                                else_body: vec![],
                            }],
                            else_body: vec![Statement::Comment {
                                text: "fallback".into(),
                            }],
                        }],
                    },
                    Statement::ForEachArray {
                        item_name: "step".into(),
                        array: "steps".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.rabbit".into(),
                            method: "say".into(),
                            arguments: vec!["step".into()],
                        }],
                    },
                    Statement::DoInOrder {
                        body: vec![
                            Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "say".into(),
                                arguments: vec!["\"Ready?\"".into()],
                            },
                            Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "say".into(),
                                arguments: vec!["\"Go!\"".into()],
                            },
                        ],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![
                            Statement::VariableAssignment {
                                name: "score".into(),
                                value: "score + 1".into(),
                            },
                            Statement::IfElse {
                                condition: "score > 0".into(),
                                if_body: vec![Statement::MethodCall {
                                    object: "this.rabbit".into(),
                                    method: "turn".into(),
                                    arguments: vec!["LEFT".into()],
                                }],
                                else_body: vec![],
                            },
                        ],
                    },
                    Statement::CollisionListener {
                        object_a: "this.rabbit".into(),
                        object_b: "this.fox".into(),
                        body: vec![
                            Statement::MethodCall {
                                object: "this.rabbit".into(),
                                method: "say".into(),
                                arguments: vec!["\"Ouch!\"".into()],
                            },
                            Statement::VariableAssignment {
                                name: "score".into(),
                                value: "score - 1".into(),
                            },
                        ],
                    },
                    Statement::FunctionCall {
                        object: "this".into(),
                        function: "computeBonus".into(),
                        arguments: vec!["score".into()],
                    },
                    Statement::UserTypeDeclaration {
                        name: "HelperBunny".into(),
                        extends: Some("SBunny".into()),
                        methods: vec![Procedure {
                            name: "hopTwice".into(),
                            parameters: vec![],
                            body: vec![Statement::MethodCall {
                                object: "this".into(),
                                method: "hop".into(),
                                arguments: vec!["2".into()],
                            }],
                        }],
                    },
                ],
            },
            Procedure {
                name: "announceScore".into(),
                parameters: vec![Parameter {
                    name: "value".into(),
                    param_type: "WholeNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.rabbit".into(),
                    method: "say".into(),
                    arguments: vec!["value".into()],
                }],
            },
        ],
        functions: vec![Function {
            name: "computeBonus".into(),
            return_type: "WholeNumber".into(),
            body: vec![Statement::ReturnStatement {
                expression: "1".into(),
            }],
        }],
    }
}

fn sample_scene() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: true,
        objects: vec![
            SceneObject {
                name: "rabbit".into(),
                kind: "SBunny".into(),
                position: Some(Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
                size: Some(1.0),
                color: Some("white".into()),
                opacity: Some(1.0),
            },
            SceneObject {
                name: "fox".into(),
                kind: "SFox".into(),
                position: Some(Vec3 {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                }),
                size: Some(1.2),
                color: Some("orange".into()),
                opacity: Some(0.9),
            },
        ],
        camera: Some(CameraPose {
            position: Vec3 {
                x: 0.0,
                y: 5.0,
                z: 10.0,
            },
        }),
    }
}

fn sample_sequences() -> Vec<SequenceBlock> {
    vec![
        SequenceBlock {
            kind: SequenceKind::DoInOrder,
            steps: vec!["rabbit move".into(), "fox move".into()],
        },
        SequenceBlock {
            kind: SequenceKind::DoTogether,
            steps: vec!["rabbit say".into(), "fox say".into()],
        },
    ]
}

macro_rules! determinism_test {
    ($name:ident, $label:literal, $expr:expr) => {
        #[test]
        fn $name() {
            assert_deterministic($label, || $expr);
        }
    };
}

determinism_test!(
    first_lesson_grading_is_deterministic,
    "grade_first_lesson_readiness",
    grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
    })
);

determinism_test!(
    loops_grading_is_deterministic,
    "grade_loops_and_conditionals",
    grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    scene_building_grading_is_deterministic,
    "grade_scene_building",
    grade_scene_building(SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_scene: Some(sample_scene()),
    })
);

determinism_test!(
    sequencing_grading_is_deterministic,
    "grade_sequencing",
    grade_sequencing(SequencingGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        sequence_blocks: Some(sample_sequences()),
    })
);

determinism_test!(
    arrays_grading_is_deterministic,
    "grade_arrays_and_arithmetic",
    grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    comments_grading_is_deterministic,
    "grade_comments",
    grade_comments(CommentsGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    creative_grading_is_deterministic,
    "grade_creative_project",
    grade_creative_project(CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    events_grading_is_deterministic,
    "grade_events_and_collision",
    grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    functions_grading_is_deterministic,
    "grade_functions",
    grade_functions(FunctionsGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    games_narrative_grading_is_deterministic,
    "grade_games_and_narrative",
    grade_games_and_narrative(GamesNarrativeGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    inheritance_grading_is_deterministic,
    "grade_inheritance_oop",
    grade_inheritance_oop(InheritanceOopGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    nested_control_grading_is_deterministic,
    "grade_nested_control_flow",
    grade_nested_control_flow(NestedControlFlowGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    parameters_grading_is_deterministic,
    "grade_parameters",
    grade_parameters(ParametersGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    textbook_integration_grading_is_deterministic,
    "grade_textbook_integration",
    grade_textbook_integration(TextbookIntegrationGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);

determinism_test!(
    variables_grading_is_deterministic,
    "grade_variables",
    grade_variables(VariablesGradingInput {
        assets_valid: true,
        asset_reason: ready_asset_reason(),
        deps_available: true,
        deps_reason: ready_deps_reason(),
        student_program: Some(sample_program()),
    })
);
