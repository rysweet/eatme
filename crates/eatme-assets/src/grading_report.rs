use eatme_core::ast::Program;
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
    let validate_assets = StepGrade {
        name: "validate-assets".into(),
        status: if input.assets_valid {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: input.asset_reason,
        depends_on: vec![],
    };
    let check_deps = StepGrade {
        name: "check-dependencies".into(),
        status: if input.deps_available {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: input.deps_reason,
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

    let preconditions_blocked = launch_smoke.status == StepStatus::Blocked;

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

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "building-a-scene-first-world".into(),
        passed,
        steps: vec![
            validate_assets,
            check_deps,
            launch_smoke,
            place_object,
            edit_code,
            run_world,
        ],
    }
}

fn interaction_step(
    name: &str,
    deps: &[&str],
    upstream_blocked: bool,
    description: &str,
) -> StepGrade {
    if upstream_blocked {
        StepGrade {
            name: name.into(),
            status: StepStatus::Blocked,
            reason: format!("Blocked by: {}", deps.join(", ")),
            depends_on: deps.iter().map(|d| (*d).into()).collect(),
        }
    } else {
        StepGrade {
            name: name.into(),
            status: StepStatus::NotYetTested,
            reason: format!("{} — requires human interaction", description),
            depends_on: deps.iter().map(|d| (*d).into()).collect(),
        }
    }
}

pub struct LoopsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_loops_and_conditionals(_input: LoopsGradingInput) -> GradingReport {
    todo!("Implementation pending — TDD red phase")
}

#[cfg(test)]
#[path = "grading_report_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "grading_report_loops_tests.rs"]
mod loops_tests;
