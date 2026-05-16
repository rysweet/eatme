// Events and collision E2E tests: validates the student-facing contract
// of the events-collision-proximity-game grading pipeline.
// Exercises: AST construction → grading report → JSON serialization →
// save/reopen round-trip.

use eatme_assets::{EventsGradingInput, StepStatus, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};

// --- Shared fixtures ---

fn complete_events_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
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
            ..Default::default()
        }],
        ..Default::default()
    }
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
    let program = Program {
        procedures: vec![Procedure {
            name: "collisionOnly".into(),
            body: vec![Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Ouch!\"".into()],
                }],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
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
    let program = Program {
        procedures: vec![Procedure {
            name: "eventOnly".into(),
            body: vec![Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                }],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
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
