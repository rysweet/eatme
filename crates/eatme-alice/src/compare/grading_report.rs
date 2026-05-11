use super::LessonSessionReadinessReport;
use serde::Serialize;

/// Per-step completion status for the Building a Scene first lesson.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct FirstLessonGradingReport {
    pub schema_version: String,
    pub scenario_id: String,
    pub steps: Vec<GradingStep>,
}

/// One canonical lesson step with its completion status.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct GradingStep {
    pub id: String,
    pub name: String,
    pub status: GradingStepStatus,
}

/// Closed status set for each grading step.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GradingStepStatus {
    Ready,
    Blocked,
    NotYetTested,
}

impl std::fmt::Display for GradingStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "ready"),
            Self::Blocked => write!(f, "blocked"),
            Self::NotYetTested => write!(f, "not_yet_tested"),
        }
    }
}

/// Build a grading report from an existing readiness report.
pub fn first_lesson_grading_report(
    readiness: &LessonSessionReadinessReport,
) -> FirstLessonGradingReport {
    let _ = readiness;
    todo!("implementation pending — TDD step")
}

#[cfg(test)]
mod tests;
