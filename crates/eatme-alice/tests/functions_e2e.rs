// Using Functions E2E tests: validates the student-facing contract
// of the functions grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::{FunctionsGradingInput, StepStatus, grade_functions};
use eatme_core::ast::{Function, Procedure, Program, Statement};

fn complete_functions_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::FunctionCall {
                object: "this".into(),
                function: "computeDistance".into(),
                arguments: vec!["this.cat".into(), "this.dog".into()],
            }],
        }],
        functions: vec![Function {
            name: "computeDistance".into(),
            return_type: "DecimalNumber".into(),
            body: vec![
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "getDistanceTo".into(),
                    arguments: vec!["this.dog".into()],
                },
                Statement::ReturnStatement {
                    expression: "this.cat getDistanceTo this.dog".into(),
                },
            ],
        }],
    }
}

fn program_with_function_no_return() -> Program {
    Program {
        procedures: vec![],
        functions: vec![Function {
            name: "computeDistance".into(),
            return_type: "DecimalNumber".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"computing...\"".into()],
            }],
        }],
    }
}

fn program_with_function_no_call() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Hello\"".into()],
            }],
        }],
        functions: vec![Function {
            name: "computeDistance".into(),
            return_type: "DecimalNumber".into(),
            body: vec![Statement::ReturnStatement {
                expression: "1.0".into(),
            }],
        }],
    }
}

fn all_ready_input(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "All scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

// -------------------------------------------------------------------
// Test 1: Complete program — all steps ready
// -------------------------------------------------------------------

#[test]
fn functions_grading_all_ready_with_complete_program() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));
    assert!(report.passed, "report should pass with complete program");
    assert_eq!(report.lesson, "using-functions-mini-challenge");
    for step in &report.steps {
        assert_eq!(
            step.status,
            StepStatus::Ready,
            "step '{}' should be Ready",
            step.name
        );
    }
}

// -------------------------------------------------------------------
// Test 2: No program — all interaction steps blocked
// -------------------------------------------------------------------

#[test]
fn functions_grading_blocked_without_program() {
    let report = grade_functions(all_ready_input(None));
    assert!(!report.passed, "report should not pass without program");
    let interaction_steps: Vec<_> = report
        .steps
        .iter()
        .filter(|s| {
            !["validate-assets", "check-dependencies", "launch-smoke"].contains(&s.name.as_str())
        })
        .collect();
    for step in &interaction_steps {
        assert_ne!(
            step.status,
            StepStatus::Ready,
            "step '{}' should not be Ready without program",
            step.name
        );
    }
}

// -------------------------------------------------------------------
// Test 3: Function without return — downstream blocked
// -------------------------------------------------------------------

#[test]
fn functions_grading_missing_return_blocks_downstream() {
    let report = grade_functions(all_ready_input(Some(program_with_function_no_return())));
    assert!(!report.passed);
    let create_fn = report
        .steps
        .iter()
        .find(|s| s.name == "create-function")
        .unwrap();
    assert_eq!(create_fn.status, StepStatus::Ready);
    let add_return = report
        .steps
        .iter()
        .find(|s| s.name == "add-return-statement")
        .unwrap();
    assert_ne!(add_return.status, StepStatus::Ready);
}

// -------------------------------------------------------------------
// Test 4: Function with return but no call from procedure
// -------------------------------------------------------------------

#[test]
fn functions_grading_missing_call_blocks_downstream() {
    let report = grade_functions(all_ready_input(Some(program_with_function_no_call())));
    assert!(!report.passed);
    let create_fn = report
        .steps
        .iter()
        .find(|s| s.name == "create-function")
        .unwrap();
    assert_eq!(create_fn.status, StepStatus::Ready);
    let add_return = report
        .steps
        .iter()
        .find(|s| s.name == "add-return-statement")
        .unwrap();
    assert_eq!(add_return.status, StepStatus::Ready);
    let call_fn = report
        .steps
        .iter()
        .find(|s| s.name == "call-function-from-procedure")
        .unwrap();
    assert_ne!(call_fn.status, StepStatus::Ready);
}

// -------------------------------------------------------------------
// Test 5: Report has correct step count
// -------------------------------------------------------------------

#[test]
fn functions_grading_report_has_eight_steps() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));
    assert_eq!(
        report.steps.len(),
        8,
        "expected 8 steps (3 preconditions + 5 interaction)"
    );
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson name
// -------------------------------------------------------------------

#[test]
fn functions_grading_report_schema_version_and_lesson() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "using-functions-mini-challenge");
}

// -------------------------------------------------------------------
// Test 7: AST round-trip
// -------------------------------------------------------------------

#[test]
fn ast_with_functions_survives_json_round_trip() {
    let program = complete_functions_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
