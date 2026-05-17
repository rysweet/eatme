// Events and collision E2E tests: validates the student-facing contract
// of the events-collision-proximity-game grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.
//
// Phase 4 pass/fail signal tests are in events_collision_pass_fail_e2e.rs.

#[allow(dead_code)]
mod events_collision_support;
#[allow(dead_code)]
mod launch_smoke_support;

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use eatme_assets::{StepStatus, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};
use events_collision_support::{
    all_ready_input, assert_all_interaction_steps_blocked, assert_preconditions_ready,
    complete_events_program,
};
use launch_smoke_support::{alice_home, real_alice_enabled};
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

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
// Test 8: Grading report JSON round-trip
// -------------------------------------------------------------------

#[test]
fn events_grading_report_survives_json_round_trip() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));

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
