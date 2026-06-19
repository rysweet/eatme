// Parameters E2E tests: validates the student-facing contract
// of the parameters grading pipeline.
// Test 7 (below) adds a real-Alice integration path gated by EATME_REAL_ALICE=1.

use eatme_assets::{ParametersGradingInput, StepStatus, grade_parameters};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

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

// ===================================================================
// Real-Alice integration tests — gated behind EATME_REAL_ALICE=1
// ===================================================================

// -------------------------------------------------------------------
// Test 7: Real Alice launch + parameters grading pipeline integration
// -------------------------------------------------------------------
//
// Launches real Alice with the reusable-methods-and-parameters
// scenario, verifies the launch succeeds, then parses the starter
// project, augments it with student-added Parameter + MethodCall-with-
// arguments constructs, and feeds it through the grading pipeline.

#[test]
fn real_alice_parameters_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice parameters integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    // --- Phase 1: Launch real Alice with the lesson-7 scenario ---

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/parameters-real");
    let run_id = format!(
        "real-parameters-{}",
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
        scenario: eatme_alice::LaunchSmokeScenario::new("reusable-methods-and-parameters"),
    })
    .expect("run_launch_smoke should succeed for parameters scenario");

    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category for parameters scenario, got: {:?}",
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
    // The starter project (amazonMinimum.a3p) has procedures but the a3p
    // parser extracts all Procedure::parameters as empty and all MethodCall
    // arguments as empty. The student must add Parameter and call-with-args.

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

    // Baseline: the a3p parser always produces empty parameters and empty
    // arguments for all procedures and method calls.
    let has_params = starter_program
        .procedures
        .iter()
        .any(|p| !p.parameters.is_empty());
    assert!(
        !has_params,
        "amazonMinimum.a3p starter should NOT have any Procedure with parameters"
    );

    let has_args = starter_program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .any(|s| matches!(s, Statement::MethodCall { arguments, .. } if !arguments.is_empty()));
    assert!(
        !has_args,
        "amazonMinimum.a3p starter should NOT have any MethodCall with arguments"
    );

    // Augment the starter with student-added constructs: a parameterized
    // procedure and a call that passes arguments.
    let mut student_program = starter_program;
    student_program.procedures.push(Procedure {
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
    });
    if let Some(first_proc) = student_program.procedures.first_mut() {
        first_proc.body.push(Statement::MethodCall {
            object: "this".into(),
            method: "moveAnimal".into(),
            arguments: vec!["2.0".into()],
        });
    }

    // --- Phase 3: Run grading pipeline and verify pass/fail signals ---

    let report = grade_parameters(ParametersGradingInput {
        assets_valid: true,
        asset_reason: "Real Alice launch succeeded; assets validated".into(),
        deps_available: true,
        deps_reason: "All dependencies available (verified via real launch)".into(),
        student_program: Some(student_program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "parameters-mini-challenge");

    // Preconditions: validate-assets, check-dependencies, launch-smoke → Ready
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST steps: create-parameterized-procedure, call-with-argument → Ready
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "create-parameterized-procedure must be Ready when Parameter present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "call-with-argument must be Ready when MethodCall has arguments"
    );

    // run-world and save-project → Ready (all AST checks passed)
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "run-world must be Ready when all parameter steps pass"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "save-project must be Ready"
    );

    // Overall: passed is true because all steps are Ready.
    assert!(
        report.passed,
        "report.passed must be true when all parameter constructs present"
    );

    // Grading report must survive JSON round-trip.
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("parameters-mini-challenge"),
        "JSON must contain lesson name"
    );
    assert!(
        json.contains("eatme.assets/grading/v1"),
        "JSON must contain schema version"
    );

    // Manifest round-trip: verify the launch manifest was persisted.
    let manifest_dir = runs_dir
        .join("reusable-methods-and-parameters")
        .join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
