use std::path::{Path, PathBuf};

use eatme_assets::grading_report::{
    GradingInput, LoopsGradingInput, grade_first_lesson_readiness, grade_loops_and_conditionals,
};
use eatme_assets::{
    CreativeProjectGradingInput, EventsGradingInput, FunctionsGradingInput, GradingReport,
    ParametersGradingInput, StepStatus, VariablesGradingInput, grade_creative_project,
    grade_events_and_collision, grade_functions, grade_parameters, grade_variables,
};
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real")
        .join(format!("{name}.a3p"))
}

fn parse_fixture_program(name: &str) -> Program {
    let path = fixture_path(name);
    parse_a3p_program(&path).unwrap_or_else(|| panic!("failed to parse {}", path.display()))
}

fn assert_program_extracted(name: &str, program: &Program) {
    assert!(
        !program.procedures.is_empty(),
        "{name} should parse into at least one procedure"
    );
    assert!(
        program
            .procedures
            .iter()
            .flat_map(|procedure| procedure.body.iter())
            .any(|statement| matches!(statement, Statement::MethodCall { .. })),
        "{name} should expose at least one MethodCall after AST extraction"
    );
}

fn assert_report_roundtrip(label: &str, report: GradingReport) -> GradingReport {
    let json = serde_json::to_string_pretty(&report).unwrap();
    let restored: GradingReport = serde_json::from_str(&json).unwrap();
    assert_eq!(
        restored, report,
        "{label} should survive report JSON round-trip"
    );
    restored
}

#[test]
fn first_lesson_pipeline_parses_fixture_and_roundtrips_report() {
    let program = parse_fixture_program("lagoonMinimum");
    assert_program_extracted("lagoonMinimum", &program);

    let report = assert_report_roundtrip(
        "first-lesson",
        grade_first_lesson_readiness(GradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
        }),
    );

    assert_eq!(report.lesson, "building-a-scene-first-world");
    assert_eq!(report.steps.len(), 6);
    assert_eq!(report.steps[3].status, StepStatus::NotYetTested);
    assert_eq!(report.steps[4].status, StepStatus::NotYetTested);
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
    assert!(!report.passed);
}

#[test]
fn events_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("africaMinimum");
    assert_program_extracted("africaMinimum", &program);
    let first = program
        .procedures
        .first_mut()
        .expect("africaMinimum should include a procedure");
    first.body.push(Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::MethodCall {
            object: "this.lion".into(),
            method: "say".into(),
            arguments: vec!["\"Ready!\"".into()],
        }],
    });
    first.body.push(Statement::CollisionListener {
        object_a: "this.lion".into(),
        object_b: "this.gazelle".into(),
        body: vec![Statement::MethodCall {
            object: "this.lion".into(),
            method: "say".into(),
            arguments: vec!["\"Crash!\"".into()],
        }],
    });

    let report = assert_report_roundtrip(
        "events",
        grade_events_and_collision(EventsGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "events-collision-proximity-game");
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert!(!report.passed);
}

#[test]
fn variables_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("amazonMinimum");
    assert_program_extracted("amazonMinimum", &program);
    let first = program
        .procedures
        .first_mut()
        .expect("amazonMinimum should include a procedure");
    first.body.push(Statement::VariableDeclaration {
        name: "speed".into(),
        var_type: "DecimalNumber".into(),
        initial_value: "0.5".into(),
    });
    first.body.push(Statement::MethodCall {
        object: "this.rabbit".into(),
        method: "move".into(),
        arguments: vec!["FORWARD".into(), "speed".into()],
    });
    first.body.push(Statement::VariableAssignment {
        name: "speed".into(),
        value: "1.0".into(),
    });

    let report = assert_report_roundtrip(
        "variables",
        grade_variables(VariablesGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "using-variables-mini-challenge");
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
    assert!(report.passed);
}

#[test]
fn loops_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("amazonMinimum");
    assert_program_extracted("amazonMinimum", &program);
    let first = program
        .procedures
        .first_mut()
        .expect("amazonMinimum should include a procedure");
    first.body.push(Statement::CountLoop {
        count: 3,
        body: vec![Statement::MethodCall {
            object: "this.rabbit".into(),
            method: "hop".into(),
            arguments: vec!["1.0".into()],
        }],
    });

    let report = assert_report_roundtrip(
        "loops",
        grade_loops_and_conditionals(LoopsGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert!(!report.passed);
}

#[test]
fn functions_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("magicMinimum");
    assert_program_extracted("magicMinimum", &program);
    program.functions.push(Function {
        name: "computeDistance".into(),
        return_type: "DecimalNumber".into(),
        body: vec![
            Statement::MethodCall {
                object: "this.dragon".into(),
                method: "getDistanceTo".into(),
                arguments: vec!["this.wizard".into()],
            },
            Statement::ReturnStatement {
                expression: "this.dragon getDistanceTo this.wizard".into(),
            },
        ],
    });
    program
        .procedures
        .first_mut()
        .expect("magicMinimum should include a procedure")
        .body
        .push(Statement::FunctionCall {
            object: "this".into(),
            function: "computeDistance".into(),
            arguments: vec!["this.dragon".into(), "this.wizard".into()],
        });

    let report = assert_report_roundtrip(
        "functions",
        grade_functions(FunctionsGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "using-functions-mini-challenge");
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
    assert!(report.passed);
}

#[test]
fn parameters_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("indiaMinimum");
    assert_program_extracted("indiaMinimum", &program);
    program.procedures.push(Procedure {
        name: "moveAnimal".into(),
        parameters: vec![Parameter {
            name: "distance".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.camel".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "distance".into()],
        }],
    });
    program
        .procedures
        .first_mut()
        .expect("indiaMinimum should include a procedure")
        .body
        .push(Statement::MethodCall {
            object: "this".into(),
            method: "moveAnimal".into(),
            arguments: vec!["2.0".into()],
        });

    let report = assert_report_roundtrip(
        "parameters",
        grade_parameters(ParametersGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "parameters-mini-challenge");
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
    assert!(report.passed);
}

#[test]
fn creative_pipeline_parses_fixture_grades_and_roundtrips_report() {
    let mut program = parse_fixture_program("iceFull");
    assert_program_extracted("iceFull", &program);
    let first = program
        .procedures
        .first_mut()
        .expect("iceFull should include a procedure");
    first.body.push(Statement::MethodCall {
        object: "this.penguin".into(),
        method: "say".into(),
        arguments: vec!["\"Welcome!\"".into()],
    });
    first.body.push(Statement::MethodCall {
        object: "this.seal".into(),
        method: "walk".into(),
        arguments: vec!["FORWARD".into(), "1.0".into()],
    });
    first.body.push(Statement::CountLoop {
        count: 3,
        body: vec![Statement::MethodCall {
            object: "this.penguin".into(),
            method: "turn".into(),
            arguments: vec!["LEFT".into(), "0.25".into()],
        }],
    });
    first.body.push(Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::MethodCall {
            object: "this.penguin".into(),
            method: "say".into(),
            arguments: vec!["\"Game on!\"".into()],
        }],
    });
    program.procedures.push(Procedure {
        name: "doSpecialMove".into(),
        parameters: vec![Parameter {
            name: "speed".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.seal".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        }],
    });

    let report = assert_report_roundtrip(
        "creative-project",
        grade_creative_project(CreativeProjectGradingInput {
            assets_valid: true,
            asset_reason: "real Alice fixture parsed".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "creative-design-project");
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
    assert!(report.passed);
}
