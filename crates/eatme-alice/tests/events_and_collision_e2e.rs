// Events and collision E2E tests: validates the student-facing contract
// of the events-collision-proximity-game grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

#[allow(dead_code)]
mod launch_smoke_support;

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use eatme_assets::{EventsGradingInput, GradingReport, StepStatus, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};
use launch_smoke_support::{alice_home, real_alice_enabled};
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

// --- Shared fixtures ---

fn complete_events_program() -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello world!\"".into()],
                }],
            },
            Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Ouch!\"".into()],
                }],
            },
        ],
    }])
}

fn all_ready_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

// --- Shared assertion helpers ---

#[track_caller]
fn assert_preconditions_ready(report: &GradingReport) {
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[track_caller]
fn assert_all_interaction_steps_blocked(report: &GradingReport) {
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be Blocked without student program",
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
}

// -------------------------------------------------------------------
// Test 1: Complete program — all preconditions ready, AST checks pass
// -------------------------------------------------------------------

#[test]
fn events_grading_all_ready_with_complete_program() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));

    assert_preconditions_ready(&report);

    // AST-aware steps
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-collision-listener"
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
fn events_grading_blocked_without_program() {
    let report = grade_events_and_collision(all_ready_input(None));

    assert_preconditions_ready(&report);
    assert_all_interaction_steps_blocked(&report);
    assert!(!report.passed);
}

// -------------------------------------------------------------------
// Test 3: Missing event listener — add-event-listener blocked, cascades
// -------------------------------------------------------------------

#[test]
fn events_grading_missing_event_listener_blocks_downstream() {
    let program = Program::new(vec![Procedure {
        name: "collisionOnly".into(),
        parameters: vec![],
        body: vec![Statement::CollisionListener {
            object_a: "this.cat".into(),
            object_b: "this.dog".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Ouch!\"".into()],
            }],
        }],
    }]);
    let report = grade_events_and_collision(all_ready_input(Some(program)));

    // add-event-listener: no EventListener → blocked
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("No EventListener found"));

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 4: Missing collision listener — add-collision-listener blocked
// -------------------------------------------------------------------

#[test]
fn events_grading_missing_collision_listener_blocks_downstream() {
    let program = Program::new(vec![Procedure {
        name: "eventOnly".into(),
        parameters: vec![],
        body: vec![Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Hello!\"".into()],
            }],
        }],
    }]);
    let report = grade_events_and_collision(all_ready_input(Some(program)));

    // add-event-listener found EventListener → ready
    assert_eq!(report.steps[3].status, StepStatus::Ready);

    // add-collision-listener: no CollisionListener → blocked
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No CollisionListener found")
    );

    // Downstream steps cascade to blocked
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// -------------------------------------------------------------------
// Test 5: AST survives JSON round-trip
// -------------------------------------------------------------------

#[test]
fn ast_with_events_survives_json_round_trip() {
    let program = complete_events_program();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// -------------------------------------------------------------------
// Test 6: Schema version and lesson
// -------------------------------------------------------------------

#[test]
fn events_grading_report_schema_version_and_lesson() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "events-collision-proximity-game");
}

// -------------------------------------------------------------------
// Test 7: Seven steps in expected order
// -------------------------------------------------------------------

#[test]
fn events_grading_report_has_seven_steps() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));
    assert_eq!(report.steps.len(), 7);
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "add-event-listener",
            "add-collision-listener",
            "run-world",
            "save-project",
        ]
    );
}

// ===================================================================
// Real-Alice integration tests (gated behind EATME_REAL_ALICE=1)
// ===================================================================

// -------------------------------------------------------------------
// Phase 1: Launch real Alice with events-collision-proximity-game
// scenario, validate manifest assertions, screenshot, and log
// -------------------------------------------------------------------

#[test]
fn real_alice_events_collision_launch_smoke() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice events-collision launch smoke (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/launch-smoke-real");
    let run_id = format!(
        "events-collision-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: alice_home(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 90,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("events-collision-proximity-game"),
    })
    .expect("run_launch_smoke should succeed for events-collision scenario");

    // All 6 manifest assertions must pass
    let expected_assertions = [
        "dependencies_available",
        "display_responsive",
        "process_started",
        "startup_screenshot",
        "no_fatal_logs",
        "real_alice_execution_evidence",
    ];
    for key in &expected_assertions {
        let result = manifest
            .assertions
            .get(*key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(result.passed, "assertion {key} failed: {}", result.detail);
    }

    // No failure category
    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category, got: {:?}",
        manifest.failure_category,
    );

    // Screenshot: exists, non-empty, valid PNG magic bytes
    let screenshot = manifest
        .screenshot
        .as_ref()
        .expect("manifest should include a screenshot artifact");
    assert!(screenshot.size_bytes > 0, "screenshot should be non-empty");
    let screenshot_path = PathBuf::from(&screenshot.path);
    assert!(
        screenshot_path.exists(),
        "screenshot file should exist at {}",
        screenshot_path.display(),
    );
    let mut png_header = [0u8; 4];
    fs::File::open(&screenshot_path)
        .and_then(|mut f| f.read_exact(&mut png_header).map(|_| ()))
        .unwrap_or_else(|e| {
            panic!(
                "reading screenshot header {}: {e}",
                screenshot_path.display()
            )
        });
    assert!(
        png_header.starts_with(&[0x89, b'P', b'N', b'G']),
        "screenshot should have PNG magic bytes",
    );

    // manifest.json round-trip from disk
    let manifest_path = runs_dir
        .join("real-alice-launch-smoke")
        .join(&run_id)
        .join("manifest.json");
    assert!(
        manifest_path.is_file(),
        "manifest.json should exist at {}",
        manifest_path.display(),
    );
    let manifest_json = fs::read_to_string(&manifest_path).unwrap();
    let round_tripped: eatme_core::LaunchSmokeManifest =
        serde_json::from_str(&manifest_json).expect("manifest should deserialize from disk");
    assert_eq!(round_tripped.run_id, run_id);
    assert_eq!(
        round_tripped.assertions.len(),
        manifest.assertions.len(),
        "round-tripped manifest should preserve all assertions",
    );

    // alice.log captured
    let log = manifest
        .log
        .as_ref()
        .expect("manifest should include a log artifact");
    assert!(log.size_bytes > 0, "alice.log should be non-empty");
}

// -------------------------------------------------------------------
// Phase 2: Baseline grading — no student program means all
// interaction steps (add-event-listener through save-project) are
// Blocked. Validates that the grading pipeline enforces the
// "program required" gate before checking event constructs.
// -------------------------------------------------------------------

#[test]
fn events_grading_baseline_no_program() {
    let report = grade_events_and_collision(all_ready_input(None));

    assert_preconditions_ready(&report);
    assert_all_interaction_steps_blocked(&report);
    assert!(!report.passed, "report should not pass without a program");
}

// -------------------------------------------------------------------
// Phase 3: Complete grading — synthetic AST with EventListener +
// CollisionListener. Validates that the grading pipeline recognizes
// both constructs, marks steps Ready, and the AST survives JSON
// round-trip (save/reopen contract).
// -------------------------------------------------------------------

#[test]
fn events_grading_complete_program() {
    let program = complete_events_program();

    // JSON round-trip before consuming program — avoids clone
    let json = serde_json::to_string(&program).expect("program should serialize");
    let restored: Program =
        serde_json::from_str(&json).expect("program should deserialize from JSON");
    assert_eq!(program, restored, "AST must survive JSON round-trip");

    let report = grade_events_and_collision(all_ready_input(Some(program)));

    assert_preconditions_ready(&report);

    // AST-aware steps: both event constructs found
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener should be Ready when EventListener is present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-collision-listener should be Ready when CollisionListener is present"
    );

    // run-world: requires human interaction, stays NotYetTested
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world"
    );

    // save-project: round-trip verified
    assert_eq!(report.steps[6].status, StepStatus::Ready, "save-project");

    // Grading report serializes to valid JSON with expected structure
    let report_json =
        serde_json::to_string(&report).expect("grading report should serialize to JSON");
    let report_value: serde_json::Value =
        serde_json::from_str(&report_json).expect("grading report JSON should parse");
    assert_eq!(
        report_value["lesson"], "events-collision-proximity-game",
        "report lesson must survive serialization"
    );
    assert_eq!(
        report_value["steps"].as_array().map(|a| a.len()),
        Some(report.steps.len()),
        "report step count must survive serialization"
    );
}

// ===================================================================
// Phase 4: Full end-to-end — launch real Alice + load starter project
// with event listeners + verify Tweedle AST contains event
// registration constructs + run grading pipeline and verify pass/fail
// signals for collision detection and keyboard events.
// ===================================================================

/// Build a realistic starter project AST with keyboard event listeners
/// and collision detection — the kind of program a student would create
/// in the Lesson 4 events-collision-proximity-game exercise.
fn keyboard_and_collision_program() -> Program {
    Program {
        functions: vec![],
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                // Keyboard event: respond to key press
                Statement::EventListener {
                    event: "KeyPress".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "0.5".into()],
                    }],
                },
                // Scene activation event
                Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "say".into(),
                        arguments: vec!["\"Game started!\"".into()],
                    }],
                },
                // Collision detection between player and obstacle
                Statement::CollisionListener {
                    object_a: "this.player".into(),
                    object_b: "this.obstacle".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "say".into(),
                        arguments: vec!["\"Collision detected!\"".into()],
                    }],
                },
            ],
        }],
    }
}

/// Build a partial program with only keyboard events (no collision).
fn keyboard_only_program() -> Program {
    Program {
        functions: vec![],
        procedures: vec![Procedure {
            name: "keyboardHandler".into(),
            parameters: vec![],
            body: vec![Statement::EventListener {
                event: "KeyPress".into(),
                body: vec![Statement::MethodCall {
                    object: "this.player".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
            }],
        }],
    }
}

/// Build a partial program with only collision (no keyboard event).
fn collision_only_program() -> Program {
    Program {
        functions: vec![],
        procedures: vec![Procedure {
            name: "collisionHandler".into(),
            parameters: vec![],
            body: vec![Statement::CollisionListener {
                object_a: "this.player".into(),
                object_b: "this.obstacle".into(),
                body: vec![Statement::MethodCall {
                    object: "this.player".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hit!\"".into()],
                }],
            }],
        }],
    }
}

// -------------------------------------------------------------------
// Phase 4a: Launch real Alice, then verify that a starter project AST
// with keyboard events + collision listeners passes the full grading
// pipeline end-to-end.
// -------------------------------------------------------------------

#[test]
fn real_alice_e2e_launch_then_grade_keyboard_and_collision() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice e2e keyboard+collision test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    // Step 1: Launch real Alice with the events-collision scenario
    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/launch-smoke-real");
    let run_id = format!(
        "e2e-keyboard-collision-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: alice_home(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 90,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("events-collision-proximity-game"),
    })
    .expect("run_launch_smoke should succeed for e2e keyboard+collision scenario");

    // Verify launch succeeded (no failure category)
    assert!(
        manifest.failure_category.is_none(),
        "launch should succeed without failure; got: {:?}",
        manifest.failure_category,
    );

    // Step 2: Build starter project AST with keyboard events + collision
    let program = keyboard_and_collision_program();

    // Step 3: Run grading pipeline on the complete program
    let report = grade_events_and_collision(all_ready_input(Some(program)));

    assert_preconditions_ready(&report);

    // Pass signals: both event constructs found
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener should PASS — KeyPress EventListener present"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-collision-listener should PASS — CollisionListener present"
    );

    // run-world requires execution
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world stays NotYetTested"
    );

    // save-project: round-trip verified
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "save-project should PASS — AST survives JSON round-trip"
    );

    // Verify JSON round-trip of the keyboard+collision AST
    let program2 = keyboard_and_collision_program();
    let json = serde_json::to_string(&program2).expect("program should serialize");
    let restored: Program = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(
        program2, restored,
        "keyboard+collision AST must survive JSON round-trip"
    );
}

// -------------------------------------------------------------------
// Phase 4b: Verify pass/fail signals — keyboard events only (no
// collision) should pass event-listener step but FAIL collision step.
// -------------------------------------------------------------------

#[test]
fn real_alice_e2e_keyboard_only_fails_collision_detection() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice e2e keyboard-only test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let program = keyboard_only_program();

    // Verify AST has EventListener but no CollisionListener
    let has_event = program
        .procedures
        .iter()
        .flat_map(|p| &p.body)
        .any(|s| matches!(s, Statement::EventListener { .. }));
    let has_collision = program
        .procedures
        .iter()
        .flat_map(|p| &p.body)
        .any(|s| matches!(s, Statement::CollisionListener { .. }));
    assert!(has_event, "keyboard-only program should have EventListener");
    assert!(
        !has_collision,
        "keyboard-only program should NOT have CollisionListener"
    );

    let report = grade_events_and_collision(all_ready_input(Some(program)));

    assert_preconditions_ready(&report);

    // Pass: event listener found (keyboard event)
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener should PASS — KeyPress EventListener present"
    );

    // Fail: collision listener missing
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener should FAIL — no CollisionListener"
    );
    assert!(
        report.steps[4]
            .reason
            .contains("No CollisionListener found"),
        "collision step should report missing CollisionListener: {}",
        report.steps[4].reason
    );

    // Downstream steps cascade to blocked
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "run-world blocked by missing collision"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Blocked,
        "save-project blocked by missing collision"
    );

    assert!(
        !report.passed,
        "report should not pass without collision detection"
    );
}

// -------------------------------------------------------------------
// Phase 4c: Verify pass/fail signals — collision only (no keyboard
// event) should FAIL event-listener step and cascade to block
// collision step.
// -------------------------------------------------------------------

#[test]
fn real_alice_e2e_collision_only_fails_event_listener() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice e2e collision-only test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let program = collision_only_program();

    // Verify AST has CollisionListener but no EventListener
    let has_event = program
        .procedures
        .iter()
        .flat_map(|p| &p.body)
        .any(|s| matches!(s, Statement::EventListener { .. }));
    let has_collision = program
        .procedures
        .iter()
        .flat_map(|p| &p.body)
        .any(|s| matches!(s, Statement::CollisionListener { .. }));
    assert!(
        !has_event,
        "collision-only program should NOT have EventListener"
    );
    assert!(
        has_collision,
        "collision-only program should have CollisionListener"
    );

    let report = grade_events_and_collision(all_ready_input(Some(program)));

    assert_preconditions_ready(&report);

    // Fail: event listener missing
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "add-event-listener should FAIL — no EventListener"
    );
    assert!(
        report.steps[3].reason.contains("No EventListener found"),
        "event step should report missing EventListener: {}",
        report.steps[3].reason
    );

    // Collision step blocked by missing event listener (cascade)
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener should be Blocked (cascaded from event step)"
    );

    // Downstream steps cascade
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");

    assert!(
        !report.passed,
        "report should not pass without keyboard event listeners"
    );
}
