// Functions E2E tests: validates the student-facing contract
// of the functions-mini-challenge grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::{FunctionsGradingInput, StepStatus, grade_functions};
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement};

// --- Shared fixtures ---

fn complete_functions_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::FunctionCall {
                function_name: "getGreeting".into(),
            }],
        }],
        functions: vec![Function {
            name: "getGreeting".into(),
            return_type: "String".into(),
            body: vec![Statement::ReturnStatement {
                value: "\"Hello world!\"".into(),
            }],
        }],
        variable_declarations: vec![],
    }
}

fn all_ready_input(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

// -------------------------------------------------------------------
// Test 1: Complete program — all preconditions ready, AST checks pass
// -------------------------------------------------------------------

#[test]
fn functions_grading_all_ready_with_complete_program() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));

    // Precondition steps
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST-aware steps
    assert_eq!(report.steps[3].status, StepStatus::Ready, "create-function");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-return-statement"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "call-function-from-procedure"
    );

    // Runtime step — requires execution, so not-yet-tested
    assert_eq!(
        report.steps[6].status,
        StepStatus::NotYetTested,
        "run-world"
    );

    // Save/reopen round-trip — actual verification, should be Ready
    assert_eq!(report.steps[7].status, StepStatus::Ready, "save-project");

    // Passed is false because run-world is not-yet-tested
    assert!(!report.passed);
}

// -------------------------------------------------------------------
// Test 2: No student program — all interaction steps blocked
// -------------------------------------------------------------------

#[test]
fn functions_grading_blocked_without_program() {
    let report = grade_functions(all_ready_input(None));

    // Preconditions still pass
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // All 5 interaction steps blocked
    for i in 3..=7 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked when no program provided",
            i,
            report.steps[i].name
        );
        assert!(
            report.steps[i]
                .reason
                .contains("No student program provided"),
            "step {} reason should mention no program: {}",
            i,
            report.steps[i].reason
        );
    }
    assert!(!report.passed);
}

// -------------------------------------------------------------------
// Test 3: Missing function — create-function blocked, cascades
// -------------------------------------------------------------------

#[test]
fn functions_grading_missing_function_blocks_downstream() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into()],
            }],
        }],
        functions: vec![],
        variable_declarations: vec![],
    };
    let report = grade_functions(all_ready_input(Some(program)));

    // create-function: no Function → blocked
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("No Function found"));

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-return-statement"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "call-function-from-procedure"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 4: Missing return statement — add-return-statement blocked
// -------------------------------------------------------------------

#[test]
fn functions_grading_missing_return_blocks_downstream() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into()],
            }],
        }],
        functions: vec![Function {
            name: "doSomething".into(),
            return_type: "Void".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Hi\"".into()],
            }],
        }],
        variable_declarations: vec![],
    };
    let report = grade_functions(all_ready_input(Some(program)));

    // create-function found Function → ready
    assert_eq!(report.steps[3].status, StepStatus::Ready);

    // add-return-statement: no ReturnStatement → blocked
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(report.steps[4].reason.contains("No ReturnStatement found"));

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "call-function-from-procedure"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 5: AST survives JSON round-trip
// -------------------------------------------------------------------

#[test]
fn ast_with_functions_survives_json_round_trip() {
    let program = complete_functions_program();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson
// -------------------------------------------------------------------

#[test]
fn functions_grading_report_schema_version_and_lesson() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "functions-mini-challenge");
}

// -------------------------------------------------------------------
// Test 7: Eight steps in expected order
// -------------------------------------------------------------------

#[test]
fn functions_grading_report_has_eight_steps() {
    let report = grade_functions(all_ready_input(Some(complete_functions_program())));
    assert_eq!(report.steps.len(), 8);
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "create-function",
            "add-return-statement",
            "call-function-from-procedure",
            "run-world",
            "save-project",
        ]
    );
}
