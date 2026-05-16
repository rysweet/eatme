// Parameters E2E tests: validates the student-facing contract
// of the parameters-procedure-generalization grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::{ParametersGradingInput, StepStatus, grade_parameters};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

// --- Shared fixtures ---

fn complete_parameters_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "greet".into(),
                    arguments: vec!["\"Hello\"".into()],
                }],
            },
            Procedure {
                name: "greet".into(),
                parameters: vec![Parameter {
                    name: "message".into(),
                    param_type: "String".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["message".into()],
                }],
            },
        ],
        functions: vec![],
        variable_declarations: vec![],
    }
}

fn all_ready_input(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
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
fn parameters_grading_all_ready_with_complete_program() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));

    // Precondition steps
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST-aware steps
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "create-parameterized-procedure"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "call-with-argument"
    );

    // Runtime step — requires execution, so not-yet-tested
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world"
    );

    // Save/reopen round-trip — actual verification, should be Ready
    assert_eq!(report.steps[6].status, StepStatus::Ready, "save-project");

    // Passed is false because run-world is not-yet-tested
    assert!(!report.passed);
}

// -------------------------------------------------------------------
// Test 2: No student program — all interaction steps blocked
// -------------------------------------------------------------------

#[test]
fn parameters_grading_blocked_without_program() {
    let report = grade_parameters(all_ready_input(None));

    // Preconditions still pass
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // All 4 interaction steps blocked
    for i in 3..=6 {
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
// Test 3: Missing parameterized procedure — blocked, cascades
// -------------------------------------------------------------------

#[test]
fn parameters_grading_missing_param_procedure_blocks_downstream() {
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
    let report = grade_parameters(all_ready_input(Some(program)));

    // create-parameterized-procedure: no parameterized procedure → blocked
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3]
            .reason
            .contains("No parameterized procedure found")
    );

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "call-with-argument"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 4: Parameterized procedure but no call — call-with-argument blocked
// -------------------------------------------------------------------

#[test]
fn parameters_grading_no_call_blocks_downstream() {
    let program = Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into()],
                }],
            },
            Procedure {
                name: "greet".into(),
                parameters: vec![Parameter {
                    name: "message".into(),
                    param_type: "String".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["message".into()],
                }],
            },
        ],
        functions: vec![],
        variable_declarations: vec![],
    };
    let report = grade_parameters(all_ready_input(Some(program)));

    // create-parameterized-procedure found → ready
    assert_eq!(report.steps[3].status, StepStatus::Ready);

    // call-with-argument: no call with argument → blocked
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No call with argument found")
    );

    // Downstream cascade
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 5: AST survives JSON round-trip
// -------------------------------------------------------------------

#[test]
fn ast_with_parameters_survives_json_round_trip() {
    let program = complete_parameters_program();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson
// -------------------------------------------------------------------

#[test]
fn parameters_grading_report_schema_version_and_lesson() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "parameters-procedure-generalization");
}

// -------------------------------------------------------------------
// Test 7: Seven steps in expected order
// -------------------------------------------------------------------

#[test]
fn parameters_grading_report_has_seven_steps() {
    let report = grade_parameters(all_ready_input(Some(complete_parameters_program())));
    assert_eq!(report.steps.len(), 7);
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "create-parameterized-procedure",
            "call-with-argument",
            "run-world",
            "save-project",
        ]
    );
}
