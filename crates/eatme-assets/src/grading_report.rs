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
    };
    let check_deps = StepGrade {
        name: "check-dependencies".into(),
        status: if input.deps_available {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: input.deps_reason,
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
            }
        } else {
            StepGrade {
                name: "launch-smoke".into(),
                status: StepStatus::Blocked,
                reason: format!("Blocked by: {}", blockers.join(", ")),
            }
        }
    };
    let passed = validate_assets.status == StepStatus::Ready
        && check_deps.status == StepStatus::Ready
        && launch_smoke.status == StepStatus::Ready;
    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "building-a-scene-first-world".into(),
        passed,
        steps: vec![validate_assets, check_deps, launch_smoke],
    }
}

#[cfg(test)]
#[path = "grading_report_tests.rs"]
mod tests;
