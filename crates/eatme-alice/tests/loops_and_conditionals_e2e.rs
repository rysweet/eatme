// Loops and conditionals E2E tests: validates the student-facing contract
// of the loops-and-conditionals-mini-challenge grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.
// Test 8 (below) adds a real-Alice integration path gated by EATME_REAL_ALICE=1.

use eatme_assets::grading_report::{LoopsGradingInput, StepStatus, grade_loops_and_conditionals};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

// --- Shared fixtures ---

fn complete_program() -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
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
        asset_reason: "All 115 scenario assets passed validation".into(),
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
        parameters: vec![],
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
        parameters: vec![],
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

// ===================================================================
// Real-Alice integration tests — gated behind EATME_REAL_ALICE=1
// ===================================================================

// -------------------------------------------------------------------
// Test 8: Real Alice launch + grading pipeline integration
// -------------------------------------------------------------------
//
// Launches real Alice with the loops-and-conditionals-mini-challenge
// scenario, verifies the launch succeeds, then feeds a representative
// student program through the grading pipeline and asserts the full
// pass/fail signal chain.

#[test]
fn real_alice_loops_and_conditionals_grading_integration() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice loops integration test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    // --- Phase 1: Launch real Alice with the lesson-3 scenario ---

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/loops-and-conditionals-real");
    let run_id = format!(
        "real-loops-{}",
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
        scenario: eatme_alice::LaunchSmokeScenario::new("loops-and-conditionals-mini-challenge"),
    })
    .expect("run_launch_smoke should succeed for loops-and-conditionals");

    // Launch must not have a fatal failure category.
    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category for loops scenario, got: {:?}",
        manifest.failure_category,
    );

    // Core assertions (deps, display, process) should pass.
    for key in ["dependencies_available", "process_started"] {
        let result = manifest
            .assertions
            .get(key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(result.passed, "assertion {key} failed: {}", result.detail);
    }

    // --- Phase 2: Parse real starter project and verify baseline AST ---
    //
    // The starter project (amazonMinimum.a3p) contains IfElse constructs
    // but NO CountLoop — the student must add loops. We verify this baseline,
    // then augment with student-added constructs for Phase 3.

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

    let all_stmts: Vec<&Statement> = starter_program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .collect();

    let has_if_else = all_stmts
        .iter()
        .any(|s| matches!(s, Statement::IfElse { .. }));
    let has_count_loop = all_stmts
        .iter()
        .any(|s| matches!(s, Statement::CountLoop { .. }));

    assert!(
        has_if_else,
        "amazonMinimum.a3p should contain at least one IfElse construct"
    );
    assert!(
        !has_count_loop,
        "amazonMinimum.a3p starter should NOT contain any CountLoop (student adds these)"
    );

    // Augment the starter with student-added CountLoop to simulate a
    // completed student program for the grading pipeline.
    let mut student_program = starter_program;
    if let Some(first_proc) = student_program.procedures.first_mut() {
        first_proc.body.push(Statement::CountLoop {
            count: 3,
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into(), "1.0".into()],
            }],
        });
    }

    // --- Phase 3: Run grading pipeline and verify pass/fail signals ---

    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: "Real Alice launch succeeded; assets validated".into(),
        deps_available: true,
        deps_reason: "All dependencies available (verified via real launch)".into(),
        student_program: Some(student_program),
    });

    // Schema and lesson must match.
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");

    // Preconditions: validate-assets, check-dependencies, launch-smoke → Ready
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST steps: build-counting-loop → Ready, add-conditional-branch → Ready
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "build-counting-loop must be Ready when CountLoop present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-conditional-branch must be Ready when IfElse present"
    );

    // run-world is NotYetTested (requires human interaction).
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world must be NotYetTested"
    );

    // save-project is Ready (doesn't cascade from NotYetTested).
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "save-project must be Ready"
    );

    // Overall: passed is false because run-world is not-yet-tested.
    assert!(
        !report.passed,
        "report.passed must be false when run-world is not-yet-tested"
    );

    // Grading report must survive JSON round-trip.
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("loops-and-conditionals-mini-challenge"),
        "JSON must contain lesson name"
    );
    assert!(
        json.contains("eatme.assets/grading/v1"),
        "JSON must contain schema version"
    );

    // Manifest round-trip: verify the launch manifest was persisted.
    let manifest_dir = runs_dir
        .join("loops-and-conditionals-mini-challenge")
        .join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
