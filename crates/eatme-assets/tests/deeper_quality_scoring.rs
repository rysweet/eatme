use eatme_assets::{
    EventsGradingInput, GradingReport, ParametersGradingInput, VariablesGradingInput,
    grade_events_and_collision, grade_parameters, grade_variables,
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
