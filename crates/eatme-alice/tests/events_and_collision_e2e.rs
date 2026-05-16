// Events and collision E2E tests: validates the student-facing contract
// of the events-collision-proximity-game grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use eatme_assets::{EventsGradingInput, StepStatus, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};
use std::env;
use std::fs;
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

// -------------------------------------------------------------------
// Test 1: Complete program — all preconditions ready, AST checks pass
// -------------------------------------------------------------------

#[test]
fn events_grading_all_ready_with_complete_program() {
    let report = grade_events_and_collision(all_ready_input(Some(complete_events_program())));

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

fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> PathBuf {
    PathBuf::from(env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()))
}

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
    let screenshot_bytes = fs::read(&screenshot_path)
        .unwrap_or_else(|e| panic!("reading screenshot {}: {e}", screenshot_path.display()));
    assert!(
        screenshot_bytes.starts_with(&[0x89, b'P', b'N', b'G']),
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
fn real_alice_events_grading_baseline_no_program() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice events baseline grading (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let report = grade_events_and_collision(all_ready_input(None));

    // Precondition steps pass (assets + deps + launch-smoke are synthetic-ready)
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

    // All 4 interaction steps blocked without a program
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
    assert!(!report.passed, "report should not pass without a program");
}

// -------------------------------------------------------------------
// Phase 3: Complete grading — synthetic AST with EventListener +
// CollisionListener. Validates that the grading pipeline recognizes
// both constructs, marks steps Ready, and the AST survives JSON
// round-trip (save/reopen contract).
// -------------------------------------------------------------------

#[test]
fn real_alice_events_grading_complete_program() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice events complete grading (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let program = complete_events_program();
    let report = grade_events_and_collision(all_ready_input(Some(program.clone())));

    // Precondition steps
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");

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

    // JSON round-trip of the complete AST
    let json = serde_json::to_string(&program).expect("program should serialize");
    let restored: Program =
        serde_json::from_str(&json).expect("program should deserialize from JSON");
    assert_eq!(program, restored, "AST must survive JSON round-trip");

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
