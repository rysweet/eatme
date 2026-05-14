// Using Variables E2E tests

use eatme_assets::{StepStatus, VariablesGradingInput, grade_variables};
use eatme_core::ast::{Procedure, Program, Statement};

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
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
                Statement::VariableAssignment {
                    name: "speed".into(),
                    value: "1.0".into(),
                },
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
            ],
        }],
        functions: vec![],
    }
}

fn all_ready_input(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn variables_grading_all_ready_with_complete_program() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));
    assert!(report.passed);
    assert_eq!(report.lesson, "using-variables-mini-challenge");
}

#[test]
fn variables_grading_blocked_without_program() {
    let report = grade_variables(all_ready_input(None));
    assert!(!report.passed);
}

#[test]
fn variables_grading_missing_assignment_blocks() {
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
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
            ],
        }],
        functions: vec![],
    };
    let report = grade_variables(all_ready_input(Some(program)));
    assert!(!report.passed);
    let modify = report
        .steps
        .iter()
        .find(|s| s.name == "modify-variable")
        .unwrap();
    assert_ne!(modify.status, StepStatus::Ready);
}

#[test]
fn variables_report_has_eight_steps() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));
    assert_eq!(report.steps.len(), 8);
}

#[test]
fn variables_schema_version_and_lesson() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "using-variables-mini-challenge");
}

#[test]
fn ast_with_variables_survives_json_round_trip() {
    let program = complete_variables_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
