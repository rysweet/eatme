// Loops and conditionals E2E tests: validates the student-facing contract
// of the loops-and-conditionals-mini-challenge grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::grading_report::{LoopsGradingInput, StepStatus, grade_loops_and_conditionals};
use eatme_core::ast::{Procedure, Program, Statement};

// --- Shared fixtures ---

fn complete_program() -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![
            Statement::CountLoop {
                count: 3,
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
            },
            Statement::IfElse {
                condition: "this.cat isCloseTo this.dog".into(),
                if_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                }],
                else_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "think".into(),
                    arguments: vec!["\"Hmm...\"".into()],
                }],
            },
        ],
    }])
}

fn all_ready_input(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
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
fn loops_grading_all_ready_with_complete_program() {
    let report = grade_loops_and_conditionals(all_ready_input(Some(complete_program())));

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
        "build-counting-loop"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-conditional-branch"
    );

    // Runtime step — requires execution, so not-yet-tested
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world"
    );

    // Save/reopen round-trip — not-yet-tested does NOT cascade
    assert_eq!(report.steps[6].status, StepStatus::Ready, "save-project");

    // Passed is false because run-world is not-yet-tested
    assert!(!report.passed);
}

// -------------------------------------------------------------------
// Test 2: No student program — all interaction steps blocked
// -------------------------------------------------------------------

#[test]
fn loops_grading_blocked_without_program() {
    let report = grade_loops_and_conditionals(all_ready_input(None));

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
// Test 3: Missing loop — build-counting-loop blocked, cascades
// -------------------------------------------------------------------

#[test]
fn loops_grading_missing_loop_blocks_downstream() {
    let program = Program::new(vec![Procedure {
        name: "conditionalOnly".into(),
        body: vec![Statement::IfElse {
            condition: "this.cat isCloseTo this.dog".into(),
            if_body: vec![],
            else_body: vec![],
        }],
    }]);
    let report = grade_loops_and_conditionals(all_ready_input(Some(program)));

    // build-counting-loop: no CountLoop → blocked
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("No CountLoop found"));

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-conditional-branch"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 4: Missing conditional — add-conditional-branch blocked, cascades
// -------------------------------------------------------------------

#[test]
fn loops_grading_missing_conditional_blocks_downstream() {
    let program = Program::new(vec![Procedure {
        name: "loopOnly".into(),
        body: vec![Statement::CountLoop {
            count: 5,
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into()],
            }],
        }],
    }]);
    let report = grade_loops_and_conditionals(all_ready_input(Some(program)));

    // build-counting-loop found CountLoop → ready
    assert_eq!(report.steps[3].status, StepStatus::Ready);

    // add-conditional-branch: no IfElse → blocked
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(report.steps[4].reason.contains("No IfElse found"));

    // Downstream steps cascade to blocked
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 5: AST survives JSON round-trip
// -------------------------------------------------------------------

#[test]
fn ast_survives_json_round_trip() {
    let program = complete_program();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson
// -------------------------------------------------------------------

#[test]
fn grading_report_schema_version_and_lesson() {
    let report = grade_loops_and_conditionals(all_ready_input(Some(complete_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
}

// -------------------------------------------------------------------
// Test 7: Seven steps in expected order
// -------------------------------------------------------------------

#[test]
fn grading_report_has_seven_steps() {
    let report = grade_loops_and_conditionals(all_ready_input(Some(complete_program())));
    assert_eq!(report.steps.len(), 7);
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "build-counting-loop",
            "add-conditional-branch",
            "run-world",
            "save-project",
        ]
    );
}
