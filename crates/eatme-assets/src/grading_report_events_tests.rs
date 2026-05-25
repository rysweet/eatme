use super::*;
use eatme_core::ast::{Procedure, Program, Statement};
use std::{
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

// --- Test fixtures ---

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

fn program_with_event_only() -> Program {
    Program::new(vec![Procedure {
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
    }])
}

fn program_with_collision_only() -> Program {
    Program::new(vec![Procedure {
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
    }])
}

fn events_input_all_ready(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn events_input_blocked_assets(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn events_input_blocked_deps(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn events_input_both_blocked(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn small_events_program(seed: usize) -> Program {
    Program::new(vec![Procedure {
        name: format!("smallEventProgram{seed}"),
        parameters: vec![],
        body: vec![
            Statement::EventListener {
                event: format!("SceneActivated{seed}"),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec![format!("\"Hello {seed}\"")],
                }],
            },
            Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "think".into(),
                    arguments: vec![format!("\"Ouch {seed}\"")],
                }],
            },
        ],
    }])
}

fn large_events_program(statement_count: usize) -> Program {
    let mut body = Vec::with_capacity(statement_count);
    for index in 0..statement_count {
        if index % 2 == 0 {
            body.push(Statement::EventListener {
                event: format!("SceneActivated{index}"),
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec![format!("\"Hello {index}\"")],
                    },
                    Statement::IfElse {
                        condition: format!("score_{index} > 0"),
                        if_body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "turn".into(),
                            arguments: vec!["LEFT".into(), "0.25".into()],
                        }],
                        else_body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "turn".into(),
                            arguments: vec!["RIGHT".into(), "0.25".into()],
                        }],
                    },
                ],
            });
        } else {
            body.push(Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![
                    Statement::MethodCall {
                        object: "this.dog".into(),
                        method: "say".into(),
                        arguments: vec![format!("\"Bounce {index}\"")],
                    },
                    Statement::DoInOrder {
                        body: vec![
                            Statement::MethodCall {
                                object: "this.cat".into(),
                                method: "move".into(),
                                arguments: vec!["FORWARD".into(), "1.0".into()],
                            },
                            Statement::MethodCall {
                                object: "this.dog".into(),
                                method: "move".into(),
                                arguments: vec!["BACKWARD".into(), "1.0".into()],
                            },
                        ],
                    },
                ],
            });
        }
    }

    Program::new(vec![Procedure {
        name: "stressEventsProgram".into(),
        parameters: vec![],
        body,
    }])
}

fn assert_ready_events_pipeline(report: &GradingReport) {
    assert_eq!(report.steps.len(), 7);
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
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
    assert_eq!(
        report.steps[5].status,
        StepStatus::NotYetTested,
        "run-world"
    );
    assert_eq!(report.steps[6].status, StepStatus::Ready, "save-project");
    assert_eq!(report.quality_scores.len(), 1);
    assert_eq!(report.quality_scores[0].dimension, "entity_types");
    assert_eq!(report.quality_scores[0].score, 100);
}

// --- Schema and structure tests ---

#[test]
fn schema_version_is_grading_v1() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_events_collision_proximity_game() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.lesson, "events-collision-proximity-game");
}

#[test]
fn always_produces_seven_steps() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps.len(), 7);
}

#[test]
fn step_names_in_order() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
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

// --- depends_on field tests ---

#[test]
fn root_steps_have_empty_dependencies() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert!(report.steps[0].depends_on.is_empty(), "validate-assets");
    assert!(report.steps[1].depends_on.is_empty(), "check-dependencies");
}

#[test]
fn launch_smoke_depends_on_first_two() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn add_event_listener_depends_on_launch_smoke() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn add_collision_listener_depends_on_add_event_listener() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[4].depends_on, vec!["add-event-listener"]);
}

#[test]
fn run_world_depends_on_add_collision_listener() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[5].depends_on, vec!["add-collision-listener"]);
}

#[test]
fn save_project_depends_on_run_world() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[6].depends_on, vec!["run-world"]);
}

// --- All ready with complete program ---

#[test]
fn all_ready_complete_program_report_does_not_pass() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert!(
        !report.passed,
        "report should not pass because run-world is not-yet-tested"
    );
}

#[test]
fn all_ready_complete_program_preconditions_ready() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[test]
fn all_ready_complete_program_add_event_listener_ready() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert!(
        report.steps[3].reason.contains("EventListener found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn all_ready_complete_program_add_collision_listener_ready() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert!(
        report.steps[4].reason.contains("CollisionListener found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn all_ready_complete_program_run_world_not_yet_tested() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
}

#[test]
fn all_ready_complete_program_save_project_ready() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert!(
        report.steps[6].reason.contains("round-trip"),
        "save-project reason should mention round-trip: {}",
        report.steps[6].reason
    );
}

// --- No student program ---

#[test]
fn no_program_all_interaction_steps_blocked() {
    let report = grade_events_and_collision(events_input_all_ready(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Missing event listener construct ---

#[test]
fn missing_event_listener_blocks_and_cascades() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(program_with_collision_only())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3].reason.contains("No EventListener found"),
        "reason: {}",
        report.steps[3].reason
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-collision-listener"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Missing collision listener construct ---

#[test]
fn missing_collision_listener_blocks_and_cascades() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(program_with_event_only())));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "add-event-listener still ready"
    );
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No CollisionListener found"),
        "reason: {}",
        report.steps[4].reason
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report =
        grade_events_and_collision(events_input_blocked_assets(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[0].reason,
        "3 scenario assets failed validation"
    );
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Blocked dependencies scenario ---

#[test]
fn blocked_deps_cascades_all_downstream() {
    let report =
        grade_events_and_collision(events_input_blocked_deps(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[1].reason,
        "Missing required tools: Xvfb, wmctrl"
    );
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert!(
        report.steps[2].reason.contains("check-dependencies"),
        "launch-smoke reason: {}",
        report.steps[2].reason
    );
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Both blocked scenario ---

#[test]
fn both_blocked_launch_smoke_mentions_both_blockers() {
    let report =
        grade_events_and_collision(events_input_both_blocked(Some(complete_events_program())));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_all_steps_blocked() {
    let report =
        grade_events_and_collision(events_input_both_blocked(Some(complete_events_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    for i in 2..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Nested AST detection ---

#[test]
fn nested_event_inside_collision_listener_is_detected() {
    let program = Program::new(vec![Procedure {
        name: "eventInsideCollision".into(),
        parameters: vec![],
        body: vec![Statement::CollisionListener {
            object_a: "this.cat".into(),
            object_b: "this.dog".into(),
            body: vec![Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![],
            }],
        }],
    }]);
    let report = grade_events_and_collision(events_input_all_ready(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener nested inside CollisionListener should be detected"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener at top level should be detected"
    );
}

#[test]
fn nested_collision_inside_event_listener_is_detected() {
    let program = Program::new(vec![Procedure {
        name: "collisionInsideEvent".into(),
        parameters: vec![],
        body: vec![Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![],
            }],
        }],
    }]);
    let report = grade_events_and_collision(events_input_all_ready(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "EventListener at top level should be detected"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "CollisionListener nested inside EventListener should be detected"
    );
}

// --- JSON serialization ---

#[test]
fn report_serializes_to_expected_json_shape() {
    let report =
        grade_events_and_collision(events_input_all_ready(Some(complete_events_program())));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "events-collision-proximity-game");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 7);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "add-event-listener");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][4]["name"], "add-collision-listener");
    assert_eq!(json["steps"][4]["status"], "ready");
    assert_eq!(json["steps"][5]["name"], "run-world");
    assert_eq!(json["steps"][5]["status"], "not-yet-tested");
    assert_eq!(json["steps"][6]["name"], "save-project");
    assert_eq!(json["steps"][6]["status"], "ready");
}

#[test]
fn grades_large_program_under_100ms() {
    const LARGE_PROGRAM_STATEMENTS: usize = 240;
    const MAX_ELAPSED: Duration = Duration::from_millis(100);

    let program = large_events_program(LARGE_PROGRAM_STATEMENTS);
    let warmup = grade_events_and_collision(events_input_all_ready(Some(program.clone())));
    assert_ready_events_pipeline(&warmup);

    let start = Instant::now();
    let report = grade_events_and_collision(events_input_all_ready(Some(program)));
    let elapsed = start.elapsed();

    assert_ready_events_pipeline(&report);
    assert!(
        elapsed < MAX_ELAPSED,
        "grading {LARGE_PROGRAM_STATEMENTS} statements took {elapsed:?}, expected under {MAX_ELAPSED:?}"
    );
}

#[test]
fn grades_hundred_small_programs_with_high_sequential_throughput() {
    const PROGRAM_COUNT: usize = 100;
    const MAX_TOTAL: Duration = Duration::from_millis(100);

    let warmup = grade_events_and_collision(events_input_all_ready(Some(small_events_program(0))));
    assert_ready_events_pipeline(&warmup);

    let start = Instant::now();
    for seed in 0..PROGRAM_COUNT {
        let report =
            grade_events_and_collision(events_input_all_ready(Some(small_events_program(seed))));
        assert_ready_events_pipeline(&report);
    }
    let elapsed = start.elapsed();
    let throughput = PROGRAM_COUNT as f64 / elapsed.as_secs_f64();

    assert!(
        elapsed < MAX_TOTAL,
        "graded {PROGRAM_COUNT} small programs in {elapsed:?} ({throughput:.0} programs/sec), expected under {MAX_TOTAL:?}"
    );
}

#[test]
fn grades_programs_concurrently_across_multiple_threads() {
    const THREADS: usize = 4;
    const GRADES_PER_THREAD: usize = 25;

    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::with_capacity(THREADS);

    for thread_index in 0..THREADS {
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();

            let mut completed = 0;
            for iteration in 0..GRADES_PER_THREAD {
                let program = if iteration % 2 == 0 {
                    small_events_program(thread_index * GRADES_PER_THREAD + iteration)
                } else {
                    large_events_program(240)
                };
                let report = grade_events_and_collision(events_input_all_ready(Some(program)));
                assert_ready_events_pipeline(&report);
                completed += 1;
            }

            completed
        }));
    }

    let completed: usize = handles
        .into_iter()
        .map(|handle| handle.join().expect("grading thread should finish cleanly"))
        .sum();

    assert_eq!(completed, THREADS * GRADES_PER_THREAD);
}
