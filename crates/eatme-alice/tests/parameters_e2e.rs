// Parameters E2E tests

use eatme_assets::{ParametersGradingInput, StepStatus, grade_parameters};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn complete_parameters_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveAnimal".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
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

fn all_ready_input(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn parameters_grading_all_ready() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));
    assert!(report.passed);
    assert_eq!(report.lesson, "parameters-mini-challenge");
}

#[test]
fn parameters_grading_blocked_without_program() {
    let report = grade_parameters(all_ready_input(None));
    assert!(!report.passed);
}

#[test]
fn parameters_grading_no_params_blocks() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Hello\"".into()],
        }],
    }]);
    let report = grade_parameters(all_ready_input(Some(program)));
    assert!(!report.passed);
    let create = report
        .steps
        .iter()
        .find(|s| s.name == "create-parameterized-procedure")
        .unwrap();
    assert_ne!(create.status, StepStatus::Ready);
}

#[test]
fn parameters_report_has_seven_steps() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));
    assert_eq!(report.steps.len(), 7);
}

#[test]
fn parameters_schema_and_lesson() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "parameters-mini-challenge");
}

#[test]
fn ast_with_parameters_survives_json_round_trip() {
    let program = complete_parameters_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
