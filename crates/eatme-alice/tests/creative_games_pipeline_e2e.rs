use std::path::{Path, PathBuf};

use eatme_assets::{
    CreativeProjectGradingInput, GamesNarrativeGradingInput, GradingReport, StepStatus,
    grade_creative_project, grade_games_and_narrative,
};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

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
    let json = serde_json::to_string_pretty(&report).expect("serialize grading report");
    let restored: GradingReport = serde_json::from_str(&json).expect("deserialize grading report");
    assert_eq!(
        restored, report,
        "{label} should survive report JSON round-trip"
    );
    restored
}

#[test]
fn creative_project_pipeline_parses_fixture_and_emits_json_report() {
    let mut program = parse_fixture_program("amazonMinimum");
    assert_program_extracted("amazonMinimum", &program);

    let first = program
        .procedures
        .first_mut()
        .expect("amazonMinimum should include a procedure");
    first.body.push(Statement::MethodCall {
        object: "this.rabbit".into(),
        method: "say".into(),
        arguments: vec!["\"Welcome!\"".into()],
    });
    first.body.push(Statement::MethodCall {
        object: "this.ground".into(),
        method: "turn".into(),
        arguments: vec!["LEFT".into(), "0.25".into()],
    });
    first.body.push(Statement::CountLoop {
        count: 3,
        body: vec![Statement::MethodCall {
            object: "this.rabbit".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "0.5".into()],
        }],
    });
    first.body.push(Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::MethodCall {
            object: "this.rabbit".into(),
            method: "say".into(),
            arguments: vec!["\"Go!\"".into()],
        }],
    });
    program.procedures.push(Procedure {
        name: "doSpecialMove".into(),
        parameters: vec![Parameter {
            name: "speed".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.rabbit".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        }],
    });

    let report = assert_report_roundtrip(
        "creative-project",
        grade_creative_project(CreativeProjectGradingInput {
            assets_valid: true,
            asset_reason: "fixture parsed from a3p".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "creative-design-project");
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].status, StepStatus::Ready);
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert_eq!(report.steps[7].status, StepStatus::Ready);
    assert_eq!(report.steps[8].status, StepStatus::Ready);
    assert!(report.passed);
}

#[test]
fn games_narrative_pipeline_parses_fixture_and_emits_json_report() {
    let mut program = parse_fixture_program("amazonMinimum");
    assert_program_extracted("amazonMinimum", &program);

    let first = program
        .procedures
        .first_mut()
        .expect("amazonMinimum should include a procedure");
    first.body.push(Statement::VariableDeclaration {
        name: "score".into(),
        var_type: "WholeNumber".into(),
        initial_value: "0".into(),
    });
    first.body.push(Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::IfElse {
            condition: "score less than 10".into(),
            if_body: vec![Statement::MethodCall {
                object: "this.rabbit".into(),
                method: "move".into(),
                arguments: vec!["FORWARD".into(), "1.0".into()],
            }],
            else_body: vec![Statement::MethodCall {
                object: "this.rabbit".into(),
                method: "say".into(),
                arguments: vec!["\"Finished\"".into()],
            }],
        }],
    });
    first.body.push(Statement::CollisionListener {
        object_a: "this.rabbit".into(),
        object_b: "this.tree".into(),
        body: vec![Statement::VariableAssignment {
            name: "score".into(),
            value: "score + 10".into(),
        }],
    });
    first.body.push(Statement::DoInOrder {
        body: vec![
            Statement::MethodCall {
                object: "this.narrator".into(),
                method: "say".into(),
                arguments: vec!["\"Welcome, traveler.\"".into()],
            },
            Statement::MethodCall {
                object: "this.narrator".into(),
                method: "think".into(),
                arguments: vec!["\"Choose wisely.\"".into()],
            },
        ],
    });

    let report = assert_report_roundtrip(
        "games-narrative",
        grade_games_and_narrative(GamesNarrativeGradingInput {
            assets_valid: true,
            asset_reason: "fixture parsed from a3p".into(),
            deps_available: true,
            deps_reason: "grading dependencies available".into(),
            student_program: Some(program),
        }),
    );

    assert_eq!(report.lesson, "games-and-interactive-narrative");
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].status, StepStatus::Ready);
    assert_eq!(report.steps[7].status, StepStatus::Ready);
    assert_eq!(report.steps[8].status, StepStatus::Ready);
    assert_eq!(report.steps[9].status, StepStatus::Ready);
    assert_eq!(report.steps[10].status, StepStatus::Ready);
    assert!(report.passed);
}
