// Using Functions E2E tests: validates the student-facing contract
// of the functions grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.
// Test 8 (below) adds a real-Alice integration path gated by EATME_REAL_ALICE=1.

use eatme_assets::{FunctionsGradingInput, StepStatus, grade_functions};
use eatme_core::ast::{Function, Procedure, Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

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

// ===================================================================
// Real-Alice integration tests — gated behind EATME_REAL_ALICE=1
// ===================================================================

// -------------------------------------------------------------------
// Test 8: Real Alice launch + functions grading pipeline integration
// -------------------------------------------------------------------
//
// Launches real Alice with the functions-as-questions-about-the-world
// scenario, verifies the launch succeeds, then parses the starter
// project, augments it with student-added Function/ReturnStatement/
// FunctionCall constructs, and feeds it through the grading pipeline.

#[test]
fn real_alice_functions_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice functions integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    // --- Phase 1: Launch real Alice with the lesson-5 scenario ---

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/functions-real");
    let run_id = format!(
        "real-functions-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let manifest = eatme_alice::run_launch_smoke(&eatme_alice::LaunchSmokeOptions {
        alice_home: alice_home(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 90,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: eatme_alice::LaunchSmokeScenario::new("functions-as-questions-about-the-world"),
    })
    .expect("run_launch_smoke should succeed for functions scenario");

    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category for functions scenario, got: {:?}",
        manifest.failure_category,
    );

    for key in ["dependencies_available", "process_started"] {
        let result = manifest
            .assertions
            .get(key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(result.passed, "assertion {key} failed: {}", result.detail);
    }

    // --- Phase 2: Parse real starter project and verify baseline AST ---
    //
    // The starter project (amazonMinimum.a3p) has procedures with MethodCall
    // but NO Function, ReturnStatement, or FunctionCall — the student adds
    // those. We verify this baseline, then augment for Phase 3.

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let starter_program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    assert!(
        !starter_program.procedures.is_empty(),
        "parsed starter project should have at least one procedure"
    );

    // Baseline: the a3p parser always produces functions: vec![] (never
    // extracts Function/ReturnStatement/FunctionCall from XML).
    assert!(
        starter_program.functions.is_empty(),
        "amazonMinimum.a3p starter should NOT contain any Function definitions"
    );

    let has_function_call = starter_program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .any(|s| matches!(s, Statement::FunctionCall { .. }));
    assert!(
        !has_function_call,
        "amazonMinimum.a3p starter should NOT contain any FunctionCall statements"
    );

    // Augment the starter with student-added constructs to simulate a
    // completed student program for the grading pipeline.
    let mut student_program = starter_program;
    student_program.functions.push(Function {
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
    });
    if let Some(first_proc) = student_program.procedures.first_mut() {
        first_proc.body.push(Statement::FunctionCall {
            object: "this".into(),
            function: "computeDistance".into(),
            arguments: vec!["this.cat".into(), "this.dog".into()],
        });
    }

    // --- Phase 3: Run grading pipeline and verify pass/fail signals ---

    let report = grade_functions(FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "Real Alice launch succeeded; assets validated".into(),
        deps_available: true,
        deps_reason: "All dependencies available (verified via real launch)".into(),
        student_program: Some(student_program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "using-functions-mini-challenge");

    // Preconditions: validate-assets, check-dependencies, launch-smoke → Ready
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST steps: create-function, add-return-statement,
    // call-function-from-procedure → all Ready
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "create-function must be Ready when Function present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-return-statement must be Ready when ReturnStatement present"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "call-function-from-procedure must be Ready when FunctionCall present"
    );

    // run-world and save-project → Ready (all AST checks passed)
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "run-world must be Ready when all function steps pass"
    );
    assert_eq!(
        report.steps[7].status,
        StepStatus::Ready,
        "save-project must be Ready"
    );

    // Overall: passed is true because all steps are Ready.
    assert!(
        report.passed,
        "report.passed must be true when all function constructs present"
    );

    // Grading report must survive JSON round-trip.
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("using-functions-mini-challenge"),
        "JSON must contain lesson name"
    );
    assert!(
        json.contains("eatme.assets/grading/v1"),
        "JSON must contain schema version"
    );

    // Manifest round-trip: verify the launch manifest was persisted.
    let manifest_dir = runs_dir
        .join("functions-as-questions-about-the-world")
        .join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
