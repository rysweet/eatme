use eatme_assets::{
    EventsGradingInput, GradingInput, ParametersGradingInput, VariablesGradingInput,
    grade_events_and_collision, grade_first_lesson_readiness, grade_parameters, grade_variables,
    score_event_quality, score_parameter_quality, score_variable_quality,
};
use eatme_core::ast::{ArithmeticOperator, Parameter, Procedure, Program, Statement};

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

fn parameter_program(parameters: Vec<Parameter>) -> Program {
    Program {
        procedures: vec![Procedure {
            name: "configure".into(),
            parameters,
            body: vec![],
        }],
        functions: vec![],
    }
}

fn listener_program(body: Vec<Statement>) -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body,
    }])
}

fn variable_program(body: Vec<Statement>) -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body,
        }],
        functions: vec![],
    }
}

fn quality_score<'a>(
    scores: &'a [eatme_assets::QualityScore],
    dimension: &str,
) -> &'a eatme_assets::QualityScore {
    scores
        .iter()
        .find(|score| score.dimension == dimension)
        .unwrap_or_else(|| panic!("missing quality score for {dimension}"))
}

#[test]
fn parameter_quality_handles_missing_program_no_parameters_and_all_specific_types() {
    let missing = score_parameter_quality(None);
    let none = quality_score(&missing, "parameter_types");
    assert_eq!(none.score, 0);
    assert_eq!(
        none.feedback,
        "No student program provided for parameter quality scoring"
    );

    let no_parameters = score_parameter_quality(Some(&parameter_program(vec![])));
    let no_parameters = quality_score(&no_parameters, "parameter_types");
    assert_eq!(no_parameters.score, 0);
    assert_eq!(no_parameters.feedback, "No parameters found to assess");

    let all_specific = score_parameter_quality(Some(&parameter_program(vec![
        Parameter {
            name: "distance".into(),
            param_type: "DecimalNumber".into(),
        },
        Parameter {
            name: "target".into(),
            param_type: "Bunny".into(),
        },
    ])));
    let all_specific = quality_score(&all_specific, "parameter_types");
    assert_eq!(all_specific.score, 100);
    assert_eq!(all_specific.feedback, "All 2 parameters use specific types");
}

#[test]
fn event_quality_handles_missing_program_no_listeners_and_all_explicit_entities() {
    let missing = score_event_quality(None);
    let missing = quality_score(&missing, "entity_types");
    assert_eq!(missing.score, 0);
    assert_eq!(
        missing.feedback,
        "No student program provided for event quality scoring"
    );

    let no_listeners = score_event_quality(Some(&listener_program(vec![Statement::MethodCall {
        object: "cat".into(),
        method: "say".into(),
        arguments: vec!["\"outside\"".into()],
    }])));
    let no_listeners = quality_score(&no_listeners, "entity_types");
    assert_eq!(no_listeners.score, 0);
    assert_eq!(
        no_listeners.feedback,
        "No listener entity references found to assess"
    );

    let all_explicit = score_event_quality(Some(&listener_program(vec![
        Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"hello\"".into()],
            }],
        },
        Statement::CollisionListener {
            object_a: "this.cat".into(),
            object_b: "this.dog".into(),
            body: vec![],
        },
    ])));
    let all_explicit = quality_score(&all_explicit, "entity_types");
    assert_eq!(all_explicit.score, 100);
    assert_eq!(
        all_explicit.feedback,
        "All 3 listener entity references use explicit scene entities"
    );
}

#[test]
fn variable_quality_handles_missing_program_no_variables_and_nested_references() {
    let missing = score_variable_quality(None);
    let missing = quality_score(&missing, "variable_usage");
    assert_eq!(missing.score, 0);
    assert_eq!(
        missing.feedback,
        "No student program provided for variable quality scoring"
    );

    let no_variables = score_variable_quality(Some(&variable_program(vec![Statement::Comment {
        text: "nothing here".into(),
    }])));
    let no_variables = quality_score(&no_variables, "variable_usage");
    assert_eq!(no_variables.score, 0);
    assert_eq!(
        no_variables.feedback,
        "No declared variables found to assess"
    );

    let nested = score_variable_quality(Some(&variable_program(vec![
        Statement::VariableDeclaration {
            name: "speed".into(),
            var_type: "DecimalNumber".into(),
            initial_value: "0.5".into(),
        },
        Statement::VariableDeclaration {
            name: "direction".into(),
            var_type: "String".into(),
            initial_value: "\"left\"".into(),
        },
        Statement::ArrayDeclaration {
            name: "moves".into(),
            element_type: "String".into(),
            elements: vec!["direction".into(), "speed".into()],
        },
        Statement::ArithmeticExpression {
            operator: ArithmeticOperator::Add,
            left: "speed".into(),
            right: "1".into(),
            result: "speed".into(),
        },
        Statement::IfElse {
            condition: "direction == \"left\"".into(),
            if_body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "move".into(),
                arguments: vec!["direction".into(), "speed".into()],
            }],
            else_body: vec![Statement::ReturnStatement {
                expression: "speed".into(),
            }],
        },
    ])));
    let nested = quality_score(&nested, "variable_usage");
    assert_eq!(nested.score, 100);
    assert_eq!(
        nested.feedback,
        "All 2 declared variables are referenced after declaration"
    );
}

#[test]
fn variable_quality_reports_partial_usage_when_some_variables_are_unused() {
    let scores = score_variable_quality(Some(&variable_program(vec![
        Statement::VariableDeclaration {
            name: "speed".into(),
            var_type: "DecimalNumber".into(),
            initial_value: "0.5".into(),
        },
        Statement::VariableDeclaration {
            name: "direction".into(),
            var_type: "String".into(),
            initial_value: "\"left\"".into(),
        },
        Statement::VariableDeclaration {
            name: "unused".into(),
            var_type: "WholeNumber".into(),
            initial_value: "1".into(),
        },
        Statement::MethodCall {
            object: "this.cat".into(),
            method: "move".into(),
            arguments: vec!["direction".into(), "speed".into()],
        },
    ])));
    let score = quality_score(&scores, "variable_usage");
    assert_eq!(score.score, 66);
    assert_eq!(
        score.feedback,
        "2 of 3 declared variables are referenced after declaration"
    );
}

#[test]
fn grading_reports_serialize_quality_scores_for_each_dimension() {
    let parameter_report = grade_parameters(parameter_input(parameter_program(vec![Parameter {
        name: "distance".into(),
        param_type: "DecimalNumber".into(),
    }])));
    let events_report = grade_events_and_collision(events_input(listener_program(vec![
        Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"hello\"".into()],
            }],
        },
    ])));
    let variables_report = grade_variables(variables_input(variable_program(vec![
        Statement::VariableDeclaration {
            name: "speed".into(),
            var_type: "DecimalNumber".into(),
            initial_value: "0.5".into(),
        },
        Statement::MethodCall {
            object: "this.cat".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        },
    ])));

    for (report, expected_dimension) in [
        (parameter_report, "parameter_types"),
        (events_report, "entity_types"),
        (variables_report, "variable_usage"),
    ] {
        let json = serde_json::to_value(&report).unwrap();
        let quality_scores = json["quality_scores"]
            .as_array()
            .expect("quality_scores array");
        assert_eq!(quality_scores.len(), 1);
        assert_eq!(quality_scores[0]["dimension"], expected_dimension);
        assert!(quality_scores[0]["score"].is_number());
        assert!(quality_scores[0]["feedback"].is_string());
    }
}

#[test]
fn reports_without_quality_scores_omit_the_json_field() {
    let report = grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
    });

    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("quality_scores").is_none());
}
