// Creative/Design project E2E tests: validates the student-facing contract
// of the creative project grading pipeline.
// Test 6 (below) adds a real-Alice integration path gated by EATME_REAL_ALICE=1.

use eatme_assets::{CreativeProjectGradingInput, StepStatus, grade_creative_project};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

fn complete_creative_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Welcome!\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.dog".into(),
                        method: "walk".into(),
                        arguments: vec!["FORWARD".into(), "1.0".into()],
                    },
                    Statement::CountLoop {
                        count: 3,
                        body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "turn".into(),
                            arguments: vec!["LEFT".into(), "0.25".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "say".into(),
                            arguments: vec!["\"Game on!\"".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "doSpecialMove".into(),
                parameters: vec![Parameter {
                    name: "speed".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn all_ready_input(program: Option<Program>) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn creative_grading_all_ready() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert!(report.passed, "report should pass: {:?}", report.steps);
    assert_eq!(report.lesson, "creative-design-project");
}

#[test]
fn creative_grading_blocked_without_program() {
    let report = grade_creative_project(all_ready_input(None));
    assert!(!report.passed);
}

#[test]
fn creative_grading_minimal_fails() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Hello\"".into()],
        }],
    }]);
    let report = grade_creative_project(all_ready_input(Some(program)));
    assert!(
        !report.passed,
        "minimal program should not pass creative assessment"
    );
}

#[test]
fn creative_report_has_nine_steps() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert_eq!(report.steps.len(), 9, "3 preconditions + 6 interaction");
}

#[test]
fn creative_schema_and_lesson() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "creative-design-project");
}

// ===================================================================
// Real-Alice integration tests — gated behind EATME_REAL_ALICE=1
// ===================================================================

// -------------------------------------------------------------------
// Test 6: Real Alice launch + creative project grading integration
// -------------------------------------------------------------------
//
// Launches real Alice with the design-process-story-or-game scenario,
// verifies the launch succeeds, then parses the starter project,
// augments it with student-added EventListener + additional procedures +
// control structures to satisfy all creative grading criteria, and
// feeds it through the grading pipeline.

#[test]
fn real_alice_creative_project_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice creative project integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    // --- Phase 1: Launch real Alice with the lesson-8 scenario ---

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/creative-project-real");
    let run_id = format!(
        "real-creative-{}",
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
        scenario: eatme_alice::LaunchSmokeScenario::new("design-process-story-or-game"),
    })
    .expect("run_launch_smoke should succeed for creative project scenario");

    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category for creative scenario, got: {:?}",
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
    // The starter project (amazonMinimum.a3p) has MethodCall statements and
    // may have IfElse, but the a3p parser does NOT extract EventListener or
    // CollisionListener from the starter. The student must add event
    // handling, additional procedures, and control structures.

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

    // Augment the starter with student-added constructs to satisfy all
    // creative grading criteria:
    //   - build-scene-with-objects: ≥2 MethodCall (starter already has these)
    //   - create-custom-procedure: ≥2 procedures or parameterized procedure
    //   - add-control-structure: CountLoop or IfElse in procedure body
    //   - add-event-or-interaction: EventListener or CollisionListener
    let mut student_program = starter_program;

    // Add a second procedure with a parameter (satisfies create-custom-procedure)
    student_program.procedures.push(Procedure {
        name: "doSpecialMove".into(),
        parameters: vec![Parameter {
            name: "speed".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        }],
    });

    // Add control structure + event listener to first procedure
    if let Some(first_proc) = student_program.procedures.first_mut() {
        // Ensure ≥2 MethodCall (may already exist from parsed XML, add extra)
        first_proc.body.push(Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Welcome!\"".into()],
        });
        first_proc.body.push(Statement::MethodCall {
            object: "this.dog".into(),
            method: "walk".into(),
            arguments: vec!["FORWARD".into(), "1.0".into()],
        });
        // Add control structure (satisfies add-control-structure)
        first_proc.body.push(Statement::CountLoop {
            count: 3,
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "turn".into(),
                arguments: vec!["LEFT".into(), "0.25".into()],
            }],
        });
        // Add event listener (satisfies add-event-or-interaction)
        first_proc.body.push(Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Game on!\"".into()],
            }],
        });
    }

    // --- Phase 3: Run grading pipeline and verify pass/fail signals ---

    let report = grade_creative_project(CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "Real Alice launch succeeded; assets validated".into(),
        deps_available: true,
        deps_reason: "All dependencies available (verified via real launch)".into(),
        student_program: Some(student_program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "creative-design-project");

    // Preconditions: validate-assets, check-dependencies, launch-smoke → Ready
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // AST steps: build-scene-with-objects, create-custom-procedure,
    // add-control-structure, add-event-or-interaction → all Ready
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "build-scene-with-objects must be Ready when ≥2 MethodCall present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "create-custom-procedure must be Ready when ≥2 procedures present"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "add-control-structure must be Ready when CountLoop present"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "add-event-or-interaction must be Ready when EventListener present"
    );

    // run-world and save-project → Ready (all creative criteria met)
    assert_eq!(
        report.steps[7].status,
        StepStatus::Ready,
        "run-world must be Ready when all creative steps pass"
    );
    assert_eq!(
        report.steps[8].status,
        StepStatus::Ready,
        "save-project must be Ready"
    );

    // Overall: passed is true because all steps are Ready.
    assert!(
        report.passed,
        "report.passed must be true when all creative constructs present"
    );

    // Grading report must survive JSON round-trip.
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("creative-design-project"),
        "JSON must contain lesson name"
    );
    assert!(
        json.contains("eatme.assets/grading/v1"),
        "JSON must contain schema version"
    );

    // Manifest round-trip: verify the launch manifest was persisted.
    let manifest_dir = runs_dir.join("design-process-story-or-game").join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
