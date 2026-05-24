// Events and collision pass/fail signal tests (Phase 4):
// Validates that the grading pipeline correctly reports pass/fail for
// partial ASTs — keyboard-only, collision-only, and combined programs.
// Phase 4a also launches real Alice end-to-end before grading.

#[allow(dead_code)]
mod events_collision_support;
#[allow(dead_code)]
mod launch_smoke_support;

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use eatme_assets::{StepStatus, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};
use events_collision_support::{all_ready_input, assert_preconditions_ready};
use launch_smoke_support::{alice_home, real_alice_enabled};
use std::env;

// --- Phase 4 fixtures ---

/// Build a starter project AST with keyboard event listeners and
/// collision detection — the kind of program a student would create
/// in the Lesson 4 events-collision-proximity-game exercise.
fn keyboard_and_collision_program() -> Program {
    Program {
        functions: vec![],
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::EventListener {
                    event: "KeyPress".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "0.5".into()],
                    }],
                },
                Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "say".into(),
                        arguments: vec!["\"Game started!\"".into()],
                    }],
                },
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

    assert!(
        manifest.failure_category.is_none(),
        "launch should succeed without failure; got: {:?}",
        manifest.failure_category,
    );

    let program = keyboard_and_collision_program();
    let report = grade_events_and_collision(all_ready_input(Some(program)));

    assert_preconditions_ready(&report);
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
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world stays NotYetTested"
    );
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
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener should PASS — KeyPress EventListener present"
    );
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

    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener should be Blocked (cascaded from event step)"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");

    assert!(
        !report.passed,
        "report should not pass without keyboard event listeners"
    );
}
