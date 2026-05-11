use super::LessonSessionReadinessReport;
use super::first_lesson::FIRST_LESSON_SCENARIO_ID;
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

const SCHEMA_VERSION: &str = "eatme.first-lesson-grading-report/v1";

/// Build a grading report from an existing readiness report.
pub fn first_lesson_grading_report(
    readiness: &LessonSessionReadinessReport,
) -> FirstLessonGradingReport {
    let steps = CANONICAL_STEPS
        .iter()
        .map(|spec| GradingStep {
            id: spec.id.into(),
            name: spec.name.into(),
            status: step_status(spec, readiness),
        })
        .collect();

    FirstLessonGradingReport {
        schema_version: SCHEMA_VERSION.into(),
        scenario_id: readiness
            .scenario_id
            .clone()
            .unwrap_or_else(|| FIRST_LESSON_SCENARIO_ID.into()),
        steps,
    }
}

fn step_status(
    spec: &CanonicalStepSpec,
    readiness: &LessonSessionReadinessReport,
) -> GradingStepStatus {
    match spec.kind {
        StepKind::UiAction => ui_action_status(spec.id, readiness),
        StepKind::Boundary => boundary_status(
            spec.boundary_id
                .expect("boundary spec must have boundary_id"),
            readiness,
        ),
        StepKind::MetaBoundary => GradingStepStatus::NotYetTested,
    }
}

fn ui_action_status(
    action_id: &str,
    readiness: &LessonSessionReadinessReport,
) -> GradingStepStatus {
    let passed = readiness.target_evidence.iter().any(|target| {
        target
            .action_assertions
            .iter()
            .any(|a| a.action_id == action_id && a.passed)
    });
    if passed {
        GradingStepStatus::Ready
    } else {
        GradingStepStatus::Blocked
    }
}

fn boundary_status(
    boundary_id: &str,
    readiness: &LessonSessionReadinessReport,
) -> GradingStepStatus {
    let present = readiness
        .evidence_boundaries
        .iter()
        .any(|b| b.id == boundary_id && b.status == "present");
    if present {
        GradingStepStatus::Ready
    } else {
        GradingStepStatus::Blocked
    }
}

// ── Canonical step table ─────────────────────────────────────────────

enum StepKind {
    UiAction,
    Boundary,
    MetaBoundary,
}

struct CanonicalStepSpec {
    id: &'static str,
    name: &'static str,
    kind: StepKind,
    boundary_id: Option<&'static str>,
}

const CANONICAL_STEPS: &[CanonicalStepSpec] = &[
    CanonicalStepSpec {
        id: "verify-specific-alice-window",
        name: "Verify specific Alice window",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "activate-specific-alice-window",
        name: "Activate specific Alice window",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "select-project",
        name: "Select project",
        kind: StepKind::Boundary,
        boundary_id: Some("select_project"),
    },
    CanonicalStepSpec {
        id: "place-object",
        name: "Place object",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "edit-procedure-or-code-block",
        name: "Edit procedure or code block",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "run-world",
        name: "Run world",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "save-project",
        name: "Save project",
        kind: StepKind::UiAction,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "visible-rendering",
        name: "Visible rendering",
        kind: StepKind::Boundary,
        boundary_id: Some("visible_rendering"),
    },
    CanonicalStepSpec {
        id: "grading",
        name: "Grading",
        kind: StepKind::MetaBoundary,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "creative-assessment",
        name: "Creative assessment",
        kind: StepKind::MetaBoundary,
        boundary_id: None,
    },
    CanonicalStepSpec {
        id: "first-lesson-completion",
        name: "First-lesson completion",
        kind: StepKind::MetaBoundary,
        boundary_id: None,
    },
];

#[cfg(test)]
mod tests;
