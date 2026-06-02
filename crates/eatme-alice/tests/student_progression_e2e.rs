use eatme_assets::{
    EventsGradingInput, GradingReport, LoopsGradingInput, StepStatus, VariablesGradingInput,
    grade_events_and_collision, grade_loops_and_conditionals, grade_variables,
};
use eatme_core::ast::{Procedure, Program, Statement};

fn events_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All lesson assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn loops_input(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "All lesson assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn variables_input(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "All lesson assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn ready_steps(report: &GradingReport) -> usize {
    report
        .steps
        .iter()
        .filter(|step| step.status == StepStatus::Ready)
        .count()
}

fn quality_score(report: &GradingReport, dimension: &str) -> u8 {
    report
        .quality_scores
        .iter()
        .find(|score| score.dimension == dimension)
        .map(|score| score.score)
        .unwrap_or(0)
}

#[test]
fn student_progression_grade_improves_as_project_grows() {
    let mut program = Program::new(vec![]);

    assert!(
        program.procedures.is_empty(),
        "student should start with an empty program"
    );

    program.procedures.push(Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![],
    });
    assert_eq!(
        program.procedures.len(),
        1,
        "student adds a procedure before writing code"
    );

    program.procedures[0].body.extend([
        Statement::MethodCall {
            object: "this.cat".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "1.0".into()],
        },
        Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Getting started\"".into()],
        },
    ]);

    let low_events = grade_events_and_collision(events_input(Some(program.clone())));
    let low_loops = grade_loops_and_conditionals(loops_input(Some(program.clone())));
    let low_variables = grade_variables(variables_input(Some(program.clone())));
    let low_score =
        ready_steps(&low_events) + ready_steps(&low_loops) + ready_steps(&low_variables);

    assert_eq!(
        low_score, 9,
        "only shared preconditions should be ready in the starter project"
    );
    assert_eq!(
        low_events.steps[3].status,
        StepStatus::Blocked,
        "no event listener yet"
    );
    assert_eq!(
        low_loops.steps[3].status,
        StepStatus::Blocked,
        "no loop yet"
    );
    assert_eq!(
        low_variables.steps[3].status,
        StepStatus::Blocked,
        "no variable yet"
    );
    assert_eq!(quality_score(&low_events, "entity_types"), 0);
    assert_eq!(quality_score(&low_variables, "variable_usage"), 0);

    program.procedures[0].body.extend([
        Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Ready!\"".into()],
            }],
        },
        Statement::CollisionListener {
            object_a: "this.cat".into(),
            object_b: "this.dog".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "think".into(),
                arguments: vec!["\"Bounce\"".into()],
            }],
        },
    ]);

    let improved_events = grade_events_and_collision(events_input(Some(program.clone())));
    let improved_loops = grade_loops_and_conditionals(loops_input(Some(program.clone())));
    let improved_variables = grade_variables(variables_input(Some(program.clone())));
    let improved_score = ready_steps(&improved_events)
        + ready_steps(&improved_loops)
        + ready_steps(&improved_variables);

    assert!(
        improved_score > low_score,
        "adding events and collision should improve the overall grade"
    );
    assert_eq!(improved_events.steps[3].status, StepStatus::Ready);
    assert_eq!(improved_events.steps[4].status, StepStatus::Ready);
    assert_eq!(improved_events.steps[5].status, StepStatus::NotYetTested);
    assert_eq!(improved_events.steps[6].status, StepStatus::Ready);
    assert_eq!(quality_score(&improved_events, "entity_types"), 100);

    program.procedures[0].body.extend([
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
        Statement::VariableAssignment {
            name: "speed".into(),
            value: "1.0".into(),
        },
        Statement::CountLoop {
            count: 3,
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "move".into(),
                arguments: vec!["FORWARD".into(), "speed".into()],
            }],
        },
        Statement::IfElse {
            condition: "this.cat isCloseTo this.dog".into(),
            if_body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Tagged\"".into()],
            }],
            else_body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "think".into(),
                arguments: vec!["\"Keep trying\"".into()],
            }],
        },
    ]);

    let high_events = grade_events_and_collision(events_input(Some(program.clone())));
    let high_loops = grade_loops_and_conditionals(loops_input(Some(program.clone())));
    let high_variables = grade_variables(variables_input(Some(program.clone())));
    let high_score =
        ready_steps(&high_events) + ready_steps(&high_loops) + ready_steps(&high_variables);

    assert!(
        high_score > improved_score,
        "variables and loops should lift the grade again"
    );
    assert!(
        high_score >= 20,
        "the completed student project should earn a high combined score"
    );
    assert_eq!(high_variables.steps[3].status, StepStatus::Ready);
    assert_eq!(high_variables.steps[4].status, StepStatus::Ready);
    assert_eq!(high_variables.steps[5].status, StepStatus::Ready);
    assert!(
        high_variables.passed,
        "variables lesson should pass once the student uses and updates a variable"
    );
    assert_eq!(high_loops.steps[3].status, StepStatus::Ready);
    assert_eq!(high_loops.steps[4].status, StepStatus::Ready);
    assert_eq!(high_loops.steps[5].status, StepStatus::NotYetTested);
    assert_eq!(high_loops.steps[6].status, StepStatus::Ready);
    assert_eq!(quality_score(&high_variables, "variable_usage"), 100);
    assert_eq!(quality_score(&high_events, "entity_types"), 100);
}
