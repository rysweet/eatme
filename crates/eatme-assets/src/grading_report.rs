use eatme_core::ast::{Program, Statement};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct GradingReport {
    pub schema_version: String,
    pub lesson: String,
    pub passed: bool,
    pub steps: Vec<StepGrade>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepGrade {
    pub name: String,
    pub status: StepStatus,
    pub reason: String,
    pub depends_on: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum StepStatus {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "not-yet-tested")]
    NotYetTested,
}

pub struct GradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
}

pub fn grade_first_lesson_readiness(input: GradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let place_object = interaction_step(
        "place-object",
        &["launch-smoke"],
        preconditions_blocked,
        "Place a 3D object into the scene",
    );
    let edit_code = interaction_step(
        "edit-code",
        &["place-object"],
        preconditions_blocked,
        "Edit code to modify object behavior",
    );
    let run_world = interaction_step(
        "run-world",
        &["edit-code"],
        preconditions_blocked,
        "Run the world and observe results",
    );

    let passed = !preconditions_blocked
        && place_object.status != StepStatus::Blocked
        && edit_code.status != StepStatus::Blocked
        && run_world.status != StepStatus::Blocked
        && place_object.status != StepStatus::NotYetTested
        && edit_code.status != StepStatus::NotYetTested
        && run_world.status != StepStatus::NotYetTested;

    steps.extend([place_object, edit_code, run_world]);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "building-a-scene-first-world".into(),
        passed,
        steps,
    }
}

fn interaction_step(
    name: &str,
    deps: &[&str],
    upstream_blocked: bool,
    description: &str,
) -> StepGrade {
    let depends_on: Vec<String> = deps.iter().map(|d| (*d).into()).collect();
    let (status, reason) = if upstream_blocked {
        (
            StepStatus::Blocked,
            format!("Blocked by: {}", deps.join(", ")),
        )
    } else {
        (
            StepStatus::NotYetTested,
            format!("{} — requires human interaction", description),
        )
    };
    StepGrade {
        name: name.into(),
        status,
        reason,
        depends_on,
    }
}

fn build_preconditions(
    assets_valid: bool,
    asset_reason: String,
    deps_available: bool,
    deps_reason: String,
) -> (Vec<StepGrade>, bool) {
    let validate_assets = StepGrade {
        name: "validate-assets".into(),
        status: if assets_valid {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: asset_reason,
        depends_on: vec![],
    };
    let check_deps = StepGrade {
        name: "check-dependencies".into(),
        status: if deps_available {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: deps_reason,
        depends_on: vec![],
    };
    let launch_smoke = {
        let mut blockers = Vec::new();
        if validate_assets.status == StepStatus::Blocked {
            blockers.push("validate-assets");
        }
        if check_deps.status == StepStatus::Blocked {
            blockers.push("check-dependencies");
        }
        if blockers.is_empty() {
            StepGrade {
                name: "launch-smoke".into(),
                status: StepStatus::Ready,
                reason: "All preconditions met".into(),
                depends_on: vec!["validate-assets".into(), "check-dependencies".into()],
            }
        } else {
            StepGrade {
                name: "launch-smoke".into(),
                status: StepStatus::Blocked,
                reason: format!("Blocked by: {}", blockers.join(", ")),
                depends_on: vec!["validate-assets".into(), "check-dependencies".into()],
            }
        }
    };
    let blocked = launch_smoke.status == StepStatus::Blocked;
    (vec![validate_assets, check_deps, launch_smoke], blocked)
}

pub struct LoopsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_loops_and_conditionals(input: LoopsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("build-counting-loop", &["launch-smoke"]),
            cascade_blocked("add-conditional-branch", &["build-counting-loop"]),
            cascade_blocked("run-world", &["add-conditional-branch"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_loops_steps(&input.student_program)
    };

    let passed = steps
        .iter()
        .chain(interaction_steps.iter())
        .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "loops-and-conditionals-mini-challenge".into(),
        passed,
        steps,
    }
}

fn cascade_blocked(name: &str, deps: &[&str]) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: format!("Blocked by: {}", deps.join(", ")),
        depends_on: deps.iter().map(|d| (*d).into()).collect(),
    }
}

fn no_program_chain(steps: &[(&str, &str)]) -> Vec<StepGrade> {
    steps
        .iter()
        .map(|(name, dep)| StepGrade {
            name: (*name).into(),
            status: StepStatus::Blocked,
            reason: "No student program provided".into(),
            depends_on: vec![(*dep).into()],
        })
        .collect()
}

fn ast_check_step(name: &str, dep: &str, found: bool, construct: &str) -> StepGrade {
    let (status, reason) = if found {
        (StepStatus::Ready, format!("{construct} found in student program"))
    } else {
        (StepStatus::Blocked, format!("No {construct} found in student program"))
    };
    StepGrade { name: name.into(), status, reason, depends_on: vec![dep.into()] }
}

fn evaluate_loops_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => return no_program_chain(&[
            ("build-counting-loop", "launch-smoke"),
            ("add-conditional-branch", "build-counting-loop"),
            ("run-world", "add-conditional-branch"),
            ("save-project", "run-world"),
        ]),
    };

    let (has_loop, has_conditional) = ast_find_constructs(program);

    let build_loop = ast_check_step("build-counting-loop", "launch-smoke", has_loop, "CountLoop");
    let loop_blocked = build_loop.status == StepStatus::Blocked;

    let add_cond = if loop_blocked {
        cascade_blocked("add-conditional-branch", &["build-counting-loop"])
    } else {
        ast_check_step("add-conditional-branch", "build-counting-loop", has_conditional, "IfElse")
    };

    let cond_blocked = add_cond.status == StepStatus::Blocked;

    let run_world = if cond_blocked {
        cascade_blocked("run-world", &["add-conditional-branch"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["add-conditional-branch".into()],
        }
    };

    let run_world_blocked = run_world.status == StepStatus::Blocked;

    let save_project = if run_world_blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        StepGrade {
            name: "save-project".into(),
            status: StepStatus::Ready,
            reason: "Save and reopen project to verify persistence".into(),
            depends_on: vec!["run-world".into()],
        }
    };

    vec![build_loop, add_cond, run_world, save_project]
}

/// Single-pass AST scan: returns (has_count_loop, has_if_else).
fn ast_find_constructs(program: &Program) -> (bool, bool) {
    let (mut has_loop, mut has_cond) = (false, false);
    for proc in &program.procedures {
        stmt_find_constructs(&proc.body, &mut has_loop, &mut has_cond);
        if has_loop && has_cond {
            return (true, true);
        }
    }
    (has_loop, has_cond)
}

fn stmt_find_constructs(stmts: &[Statement], has_loop: &mut bool, has_cond: &mut bool) {
    for stmt in stmts {
        match stmt {
            Statement::CountLoop { body, .. } => {
                *has_loop = true;
                if !*has_cond {
                    stmt_find_constructs(body, has_loop, has_cond);
                }
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                *has_cond = true;
                if !*has_loop {
                    stmt_find_constructs(if_body, has_loop, has_cond);
                    if !*has_loop {
                        stmt_find_constructs(else_body, has_loop, has_cond);
                    }
                }
            }
            Statement::MethodCall { .. } => {}
            Statement::EventListener { body, .. } => {
                stmt_find_constructs(body, has_loop, has_cond);
            }
            Statement::CollisionListener { body, .. } => {
                stmt_find_constructs(body, has_loop, has_cond);
            }
        }
        if *has_loop && *has_cond {
            return;
        }
    }
}

pub struct EventsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_events_and_collision(input: EventsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("add-event-listener", &["launch-smoke"]),
            cascade_blocked("add-collision-listener", &["add-event-listener"]),
            cascade_blocked("run-world", &["add-collision-listener"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_events_steps(&input.student_program)
    };

    let passed = steps
        .iter()
        .chain(interaction_steps.iter())
        .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "events-collision-proximity-game".into(),
        passed,
        steps,
    }
}

fn evaluate_events_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => return no_program_chain(&[
            ("add-event-listener", "launch-smoke"),
            ("add-collision-listener", "add-event-listener"),
            ("run-world", "add-collision-listener"),
            ("save-project", "run-world"),
        ]),
    };

    let (has_event, has_collision) = ast_find_event_constructs(program);

    let add_event = ast_check_step("add-event-listener", "launch-smoke", has_event, "EventListener");
    let event_blocked = add_event.status == StepStatus::Blocked;

    let add_collision = if event_blocked {
        cascade_blocked("add-collision-listener", &["add-event-listener"])
    } else {
        ast_check_step("add-collision-listener", "add-event-listener", has_collision, "CollisionListener")
    };

    let collision_blocked = add_collision.status == StepStatus::Blocked;

    let run_world = if collision_blocked {
        cascade_blocked("run-world", &["add-collision-listener"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["add-collision-listener".into()],
        }
    };

    let run_world_blocked = run_world.status == StepStatus::Blocked;

    let save_project = if run_world_blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        let round_trip_ok = serde_json::to_string(program)
            .ok()
            .and_then(|j| serde_json::from_str::<Program>(&j).ok())
            .is_some_and(|restored| restored == *program);
        let status = if round_trip_ok { StepStatus::Ready } else { StepStatus::Blocked };
        let reason = if round_trip_ok {
            "Program round-trip (serialize → deserialize → compare) verified"
        } else {
            "Program failed round-trip verification"
        };
        StepGrade {
            name: "save-project".into(),
            status,
            reason: reason.into(),
            depends_on: vec!["run-world".into()],
        }
    };

    vec![add_event, add_collision, run_world, save_project]
}

/// Single-pass AST scan: returns (has_event_listener, has_collision_listener).
fn ast_find_event_constructs(program: &Program) -> (bool, bool) {
    let (mut has_event, mut has_collision) = (false, false);
    for proc in &program.procedures {
        stmt_find_event_constructs(&proc.body, &mut has_event, &mut has_collision);
        if has_event && has_collision {
            return (true, true);
        }
    }
    (has_event, has_collision)
}

fn stmt_find_event_constructs(
    stmts: &[Statement],
    has_event: &mut bool,
    has_collision: &mut bool,
) {
    for stmt in stmts {
        match stmt {
            Statement::EventListener { body, .. } => {
                *has_event = true;
                if !*has_collision {
                    stmt_find_event_constructs(body, has_event, has_collision);
                }
            }
            Statement::CollisionListener { body, .. } => {
                *has_collision = true;
                if !*has_event {
                    stmt_find_event_constructs(body, has_event, has_collision);
                }
            }
            Statement::CountLoop { body, .. } => {
                stmt_find_event_constructs(body, has_event, has_collision);
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                stmt_find_event_constructs(if_body, has_event, has_collision);
                if !(*has_event && *has_collision) {
                    stmt_find_event_constructs(else_body, has_event, has_collision);
                }
            }
            Statement::MethodCall { .. } => {}
        }
        if *has_event && *has_collision {
            return;
        }
    }
}

#[cfg(test)]
#[path = "grading_report_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "grading_report_loops_tests.rs"]
mod loops_tests;

#[cfg(test)]
#[path = "grading_report_events_tests.rs"]
mod events_tests;
