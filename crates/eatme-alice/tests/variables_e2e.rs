// Using Variables E2E tests: validates the student-facing contract
// of the variables grading pipeline.
// Test 7 (below) adds a real-Alice integration path gated by EATME_REAL_ALICE=1.

use eatme_assets::{StepStatus, VariablesGradingInput, grade_variables};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

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

// ===================================================================
// Real-Alice integration tests — gated behind EATME_REAL_ALICE=1
// ===================================================================

// -------------------------------------------------------------------
// Test 7: Real Alice launch + variables grading pipeline integration
// -------------------------------------------------------------------
//
// Launches real Alice with the variables-scorekeeper-timekeeper
// scenario, verifies the launch succeeds, then parses the starter
// project, augments it with student-added VariableDeclaration/
// MethodCall-with-variable/VariableAssignment constructs, and feeds
// it through the grading pipeline.

#[test]
fn real_alice_variables_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice variables integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    // --- Phase 1: Launch real Alice with the lesson-6 scenario ---

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/variables-real");
    let run_id = format!(
        "real-variables-{}",
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
        scenario: eatme_alice::LaunchSmokeScenario::new("variables-scorekeeper-timekeeper"),
    })
    .expect("run_launch_smoke should succeed for variables scenario");

    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category for variables scenario, got: {:?}",
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
    // but NO VariableDeclaration or VariableAssignment — the student adds
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

    // Baseline: the a3p parser never extracts VariableDeclaration or
    // VariableAssignment from XML. Collect once, check twice (matches
    // the loops_and_conditionals reference pattern).
    let all_stmts: Vec<&Statement> = starter_program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .collect();

    let has_var_decl = all_stmts
        .iter()
        .any(|s| matches!(s, Statement::VariableDeclaration { .. }));
    assert!(
        !has_var_decl,
        "amazonMinimum.a3p starter should NOT contain VariableDeclaration"
    );

    let has_var_assign = all_stmts
        .iter()
        .any(|s| matches!(s, Statement::VariableAssignment { .. }));
    assert!(
        !has_var_assign,
        "amazonMinimum.a3p starter should NOT contain VariableAssignment"
    );

    // Augment the starter with student-added constructs.
    let mut student_program = starter_program;
    if let Some(first_proc) = student_program.procedures.first_mut() {
        first_proc.body.push(Statement::VariableDeclaration {
            name: "speed".into(),
            var_type: "DecimalNumber".into(),
            initial_value: "0.5".into(),
        });
        first_proc.body.push(Statement::MethodCall {
            object: "this.cat".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        });
        first_proc.body.push(Statement::VariableAssignment {
            name: "speed".into(),
            value: "1.0".into(),
        });
    }

    // --- Phase 3: Run grading pipeline and verify pass/fail signals ---

    let report = grade_variables(VariablesGradingInput {
        assets_valid: true,
        asset_reason: "Real Alice launch succeeded; assets validated".into(),
        deps_available: true,
        deps_reason: "All dependencies available (verified via real launch)".into(),
        student_program: Some(student_program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "using-variables-mini-challenge");

    // Preconditions: validate-assets, check-dependencies, launch-smoke → Ready
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST steps: declare-variable, use-variable-in-method, modify-variable → Ready
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "declare-variable must be Ready when VariableDeclaration present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "use-variable-in-method must be Ready when MethodCall uses variable arg"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "modify-variable must be Ready when VariableAssignment present"
    );

    // run-world and save-project → Ready (all AST checks passed)
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "run-world must be Ready when all variable steps pass"
    );
    assert_eq!(
        report.steps[7].status,
        StepStatus::Ready,
        "save-project must be Ready"
    );

    // Overall: passed is true because all steps are Ready.
    assert!(
        report.passed,
        "report.passed must be true when all variable constructs present"
    );

    // Grading report must survive JSON round-trip.
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("using-variables-mini-challenge"),
        "JSON must contain lesson name"
    );
    assert!(
        json.contains("eatme.assets/grading/v1"),
        "JSON must contain schema version"
    );

    // Manifest round-trip: verify the launch manifest was persisted.
    let manifest_dir = runs_dir
        .join("variables-scorekeeper-timekeeper")
        .join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
