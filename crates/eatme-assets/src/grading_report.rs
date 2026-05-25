use eatme_core::ast::{Program, Statement};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GradingReport {
    pub schema_version: String,
    pub lesson: String,
    pub passed: bool,
    pub steps: Vec<StepGrade>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quality_scores: Vec<QualityScore>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct QualityScore {
    pub score: u8,
    pub dimension: String,
    pub feedback: String,
}

impl GradingReport {
    pub(crate) fn new(
        schema_version: impl Into<String>,
        lesson: impl Into<String>,
        passed: bool,
        steps: Vec<StepGrade>,
    ) -> Self {
        Self {
            schema_version: schema_version.into(),
            lesson: lesson.into(),
            passed,
            steps,
            quality_scores: vec![],
        }
    }

    pub(crate) fn with_quality_scores(mut self, quality_scores: Vec<QualityScore>) -> Self {
        self.quality_scores = quality_scores;
        self
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct StepGrade {
    pub name: String,
    pub status: StepStatus,
    pub reason: String,
    pub depends_on: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
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
        && place_object.status == StepStatus::Ready
        && edit_code.status == StepStatus::Ready
        && run_world.status == StepStatus::Ready;

    steps.extend([place_object, edit_code, run_world]);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "building-a-scene-first-world",
        passed,
        steps,
    )
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

pub(crate) fn build_preconditions(
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

    // Precondition steps are all Ready when !preconditions_blocked, so skip them.
    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "loops-and-conditionals-mini-challenge",
        passed,
        steps,
    )
}

pub(crate) fn cascade_blocked(name: &str, deps: &[&str]) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: format!("Blocked by: {}", deps.join(", ")),
        depends_on: deps.iter().map(|d| (*d).into()).collect(),
    }
}

pub(crate) fn no_program_chain(steps: &[(&str, &str)]) -> Vec<StepGrade> {
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

pub(crate) fn ast_check_step(name: &str, dep: &str, found: bool, construct: &str) -> StepGrade {
    let (status, reason) = if found {
        (
            StepStatus::Ready,
            format!("{construct} found in student program"),
        )
    } else {
        (
            StepStatus::Blocked,
            format!("No {construct} found in student program"),
        )
    };
    StepGrade {
        name: name.into(),
        status,
        reason,
        depends_on: vec![dep.into()],
    }
}

fn evaluate_loops_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => {
            return no_program_chain(&[
                ("build-counting-loop", "launch-smoke"),
                ("add-conditional-branch", "build-counting-loop"),
                ("run-world", "add-conditional-branch"),
                ("save-project", "run-world"),
            ]);
        }
    };

    let (has_loop, has_conditional) = ast_find_constructs(program);

    let build_loop = ast_check_step("build-counting-loop", "launch-smoke", has_loop, "CountLoop");
    let loop_blocked = build_loop.status == StepStatus::Blocked;

    let add_cond = if loop_blocked {
        cascade_blocked("add-conditional-branch", &["build-counting-loop"])
    } else {
        ast_check_step(
            "add-conditional-branch",
            "build-counting-loop",
            has_conditional,
            "IfElse",
        )
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
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => {}
            Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => {
                if !(*has_loop && *has_cond) {
                    stmt_find_constructs(body, has_loop, has_cond);
                }
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                if !(*has_loop && *has_cond) {
                    for method in methods {
                        stmt_find_constructs(&method.body, has_loop, has_cond);
                        if *has_loop && *has_cond {
                            break;
                        }
                    }
                }
            }
        }
        if *has_loop && *has_cond {
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
