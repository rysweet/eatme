// Variables E2E tests: validates the student-facing contract
// of the variables-scorekeeper-timekeeper grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::{StepStatus, VariablesGradingInput, grade_variables};
use eatme_core::ast::{Procedure, Program, Statement, VariableDeclaration};

// --- Shared fixtures ---

fn complete_variables_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["score".into()],
                },
                Statement::VariableAssignment {
                    variable: "score".into(),
                    value: "score + 1".into(),
                },
            ],
        }],
        functions: vec![],
        variable_declarations: vec![VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        }],
    }
}

fn all_ready_input(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
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
fn variables_grading_all_ready_with_complete_program() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));

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
        "declare-variable"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "use-variable-in-method"
    );
    assert_eq!(report.steps[5].status, StepStatus::Ready, "modify-variable");

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
fn variables_grading_blocked_without_program() {
    let report = grade_variables(all_ready_input(None));

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
// Test 3: Missing variable declaration — declare-variable blocked
// -------------------------------------------------------------------

#[test]
fn variables_grading_missing_declaration_blocks_downstream() {
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
    let report = grade_variables(all_ready_input(Some(program)));

    // declare-variable: no VariableDeclaration → blocked
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3]
            .reason
            .contains("No VariableDeclaration found")
    );

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "use-variable-in-method"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "modify-variable"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 4: Missing variable assignment — modify-variable blocked
// -------------------------------------------------------------------

#[test]
fn variables_grading_missing_assignment_blocks_downstream() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["score".into()],
            }],
        }],
        functions: vec![],
        variable_declarations: vec![VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        }],
    };
    let report = grade_variables(all_ready_input(Some(program)));

    // declare-variable found VariableDeclaration → ready
    assert_eq!(report.steps[3].status, StepStatus::Ready);

    // use-variable-in-method found variable arg → ready
    assert_eq!(report.steps[4].status, StepStatus::Ready);

    // modify-variable: no VariableAssignment → blocked
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert!(
        report.steps[5]
            .reason
            .contains("No VariableAssignment found")
    );

    // Downstream cascade
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 5: AST survives JSON round-trip
// -------------------------------------------------------------------

#[test]
fn ast_with_variables_survives_json_round_trip() {
    let program = complete_variables_program();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson
// -------------------------------------------------------------------

#[test]
fn variables_grading_report_schema_version_and_lesson() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "variables-scorekeeper-timekeeper");
}

// -------------------------------------------------------------------
// Test 7: Eight steps in expected order
// -------------------------------------------------------------------

#[test]
fn variables_grading_report_has_eight_steps() {
    let report = grade_variables(all_ready_input(Some(complete_variables_program())));
    assert_eq!(report.steps.len(), 8);
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "declare-variable",
            "use-variable-in-method",
            "modify-variable",
            "run-world",
            "save-project",
        ]
    );
}
