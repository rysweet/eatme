// Shared fixtures and assertion helpers for events-and-collision E2E tests.
//
// Used by:
//   - events_and_collision_e2e.rs (synthetic + Phase 1 real-Alice tests)
//   - events_collision_pass_fail_e2e.rs (Phase 4 pass/fail signal tests)

use eatme_assets::{EventsGradingInput, GradingReport, StepStatus};
use eatme_core::ast::{Procedure, Program, Statement};

// --- Shared fixtures ---

pub fn complete_events_program() -> Program {
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

pub fn all_ready_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 115 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

// --- Shared assertion helpers ---

#[track_caller]
pub fn assert_preconditions_ready(report: &GradingReport) {
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[track_caller]
pub fn assert_all_interaction_steps_blocked(report: &GradingReport) {
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
