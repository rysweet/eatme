use eatme_assets::{
    EventsGradingInput, GradingReport, ParametersGradingInput, VariablesGradingInput,
    grade_events_and_collision, grade_parameters, grade_variables, score_event_quality,
    score_parameter_quality, score_variable_quality,
};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn parameter_input(student_program: Program) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(student_program),
    }
}

fn events_input(student_program: Program) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(student_program),
    }
}

fn variables_input(student_program: Program) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: Some(student_program),
    }
}

fn parameter_program(param_type: &str) -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveAnimal".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: param_type.into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "distance".into()],
                }],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveAnimal".into(),
                    arguments: vec!["2.0".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn events_program(cat_ref: &str, dog_ref: &str) -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: cat_ref.into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello world!\"".into()],
                }],
            },
            Statement::CollisionListener {
                object_a: cat_ref.into(),
                object_b: dog_ref.into(),
                body: vec![Statement::MethodCall {
                    object: cat_ref.into(),
                    method: "say".into(),
                    arguments: vec!["\"Ouch!\"".into()],
                }],
            },
        ],
    }])
}

fn variables_program(argument_name: &str) -> Program {
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
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), argument_name.into()],
                },
                Statement::VariableAssignment {
                    name: "speed".into(),
                    value: "1.0".into(),
                },
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), argument_name.into()],
                },
            ],
        }],
        functions: vec![],
    }
}

fn score_for(report: &GradingReport, dimension: &str) -> u8 {
    report
        .quality_scores
        .iter()
        .find(|score| score.dimension == dimension)
        .unwrap_or_else(|| panic!("missing quality score for {dimension}"))
        .score
}

fn status_snapshot(report: &GradingReport) -> Vec<(String, String)> {
    report
        .steps
        .iter()
        .map(|step| (step.name.clone(), format!("{:?}", step.status)))
        .collect()
}

#[test]
fn specific_parameter_types_score_higher_without_changing_grading() {
    let specific = grade_parameters(parameter_input(parameter_program("DecimalNumber")));
    let generic = grade_parameters(parameter_input(parameter_program("Object")));

    assert_eq!(specific.passed, generic.passed);
    assert_eq!(status_snapshot(&specific), status_snapshot(&generic));
    assert!(score_for(&specific, "parameter_types") > score_for(&generic, "parameter_types"));
}

#[test]
fn explicit_event_entity_types_score_higher_without_changing_grading() {
    let explicit = grade_events_and_collision(events_input(events_program("this.cat", "this.dog")));
    let implicit = grade_events_and_collision(events_input(events_program("cat", "dog")));

    assert_eq!(explicit.passed, implicit.passed);
    assert_eq!(status_snapshot(&explicit), status_snapshot(&implicit));
    assert!(score_for(&explicit, "entity_types") > score_for(&implicit, "entity_types"));
}

#[test]
fn actually_used_variables_score_higher_without_changing_grading() {
    let used = grade_variables(variables_input(variables_program("speed")));
    let unused = grade_variables(variables_input(variables_program("distance")));

    assert_eq!(used.passed, unused.passed);
    assert_eq!(status_snapshot(&used), status_snapshot(&unused));
    assert!(score_for(&used, "variable_usage") > score_for(&unused, "variable_usage"));
}

#[test]
fn parameter_quality_trims_types_and_penalizes_generic_aliases() {
    let program = Program {
        procedures: vec![Procedure {
            name: "mixedParameters".into(),
            parameters: vec![
                Parameter {
                    name: "distance".into(),
                    param_type: " DecimalNumber ".into(),
                },
                Parameter {
                    name: "target".into(),
                    param_type: "Any".into(),
                },
                Parameter {
                    name: "mystery".into(),
                    param_type: "Unknown".into(),
                },
                Parameter {
                    name: "blank".into(),
                    param_type: "   ".into(),
                },
            ],
            body: vec![],
        }],
        functions: vec![],
    };

    let scores = score_parameter_quality(Some(&program));

    assert_eq!(scores[0].dimension, "parameter_types");
    assert_eq!(scores[0].score, 25);
    assert_eq!(
        scores[0].feedback,
        "1 of 4 parameters use specific types; prefer concrete parameter types over Object or empty types"
    );
}

#[test]
fn event_quality_counts_nested_listener_references_but_ignores_non_listener_calls() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::MethodCall {
                object: "cat".into(),
                method: "say".into(),
                arguments: vec!["\"outside\"".into()],
            },
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::DoInOrder {
                    body: vec![
                        Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "say".into(),
                            arguments: vec!["\"inside\"".into()],
                        },
                        Statement::IfElse {
                            condition: "true".into(),
                            if_body: vec![Statement::MethodCall {
                                object: "cat".into(),
                                method: "jump".into(),
                                arguments: vec![],
                            }],
                            else_body: vec![],
                        },
                    ],
                }],
            },
            Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.dog".into(),
                    method: "turn".into(),
                    arguments: vec!["LEFT".into(), "0.25".into()],
                }],
            },
        ],
    }]);

    let scores = score_event_quality(Some(&program));

    assert_eq!(scores[0].dimension, "entity_types");
    assert_eq!(scores[0].score, 60);
    assert_eq!(
        scores[0].feedback,
        "3 of 5 listener entity references use explicit scene entities like this.cat"
    );
}

#[test]
fn variable_quality_tracks_nested_array_and_arithmetic_usage() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::VariableDeclaration {
                    name: "index".into(),
                    var_type: "WholeNumber".into(),
                    initial_value: "0".into(),
                },
                Statement::VariableDeclaration {
                    name: "unused".into(),
                    var_type: "Text".into(),
                    initial_value: "\"ghost\"".into(),
                },
                Statement::ArrayAccess {
                    array: "path".into(),
                    index: "index".into(),
                    target: "speed".into(),
                },
                Statement::ArithmeticExpression {
                    operator: eatme_core::ast::ArithmeticOperator::Add,
                    left: "speed".into(),
                    right: "index".into(),
                    result: "speed".into(),
                },
            ],
        }],
        functions: vec![],
    };

    let scores = score_variable_quality(Some(&program));

    assert_eq!(scores[0].dimension, "variable_usage");
    assert_eq!(scores[0].score, 66);
    assert_eq!(
        scores[0].feedback,
        "2 of 3 declared variables are referenced after declaration"
    );
}

#[test]
fn variable_quality_does_not_count_substring_matches_as_usage() {
    let program = Program {
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
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["speedLimit".into()],
                },
            ],
        }],
        functions: vec![],
    };

    let scores = score_variable_quality(Some(&program));

    assert_eq!(scores[0].dimension, "variable_usage");
    assert_eq!(scores[0].score, 0);
    assert_eq!(
        scores[0].feedback,
        "0 of 1 declared variables are referenced after declaration"
    );
}

#[test]
fn parameter_quality_aggregates_types_across_multiple_procedures() {
    let program = Program {
        procedures: vec![
            Procedure {
                name: "configure".into(),
                parameters: vec![
                    Parameter {
                        name: "distance".into(),
                        param_type: "WholeNumber".into(),
                    },
                    Parameter {
                        name: "target".into(),
                        param_type: "Object".into(),
                    },
                ],
                body: vec![],
            },
            Procedure {
                name: "decorate".into(),
                parameters: vec![Parameter {
                    name: "pet".into(),
                    param_type: "Bunny".into(),
                }],
                body: vec![],
            },
        ],
        functions: vec![],
    };

    let scores = score_parameter_quality(Some(&program));

    assert_eq!(scores[0].dimension, "parameter_types");
    assert_eq!(scores[0].score, 66);
    assert_eq!(
        scores[0].feedback,
        "2 of 3 parameters use specific types; prefer concrete parameter types over Object or empty types"
    );
}

#[test]
fn event_quality_traverses_user_type_methods_and_foreach_listener_bodies() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::UserTypeDeclaration {
            name: "Helper".into(),
            extends: None,
            methods: vec![Procedure {
                name: "listen".into(),
                parameters: vec![],
                body: vec![Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::ForEachArray {
                        item_name: "friend".into(),
                        array: "friends".into(),
                        body: vec![
                            Statement::MethodCall {
                                object: "this.cat".into(),
                                method: "say".into(),
                                arguments: vec!["\"hi\"".into()],
                            },
                            Statement::MethodCall {
                                object: "friend".into(),
                                method: "turn".into(),
                                arguments: vec!["LEFT".into(), "0.25".into()],
                            },
                        ],
                    }],
                }],
            }],
        }],
    }]);

    let scores = score_event_quality(Some(&program));

    assert_eq!(scores[0].dimension, "entity_types");
    assert_eq!(scores[0].score, 50);
    assert_eq!(
        scores[0].feedback,
        "1 of 2 listener entity references use explicit scene entities like this.cat"
    );
}

#[test]
fn variable_quality_counts_function_calls_and_nested_user_type_returns() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::VariableDeclaration {
                    name: "target".into(),
                    var_type: "Text".into(),
                    initial_value: "\"cat\"".into(),
                },
                Statement::UserTypeDeclaration {
                    name: "Helper".into(),
                    extends: None,
                    methods: vec![Procedure {
                        name: "compute".into(),
                        parameters: vec![],
                        body: vec![
                            Statement::FunctionCall {
                                object: "math".into(),
                                function: "blend".into(),
                                arguments: vec!["speed".into(), "target".into()],
                            },
                            Statement::CountLoop {
                                count: 2,
                                body: vec![Statement::ReturnStatement {
                                    expression: "target".into(),
                                }],
                            },
                        ],
                    }],
                },
            ],
        }],
        functions: vec![],
    };

    let scores = score_variable_quality(Some(&program));

    assert_eq!(scores[0].dimension, "variable_usage");
    assert_eq!(scores[0].score, 100);
    assert_eq!(
        scores[0].feedback,
        "All 2 declared variables are referenced after declaration"
    );
}
