use serde_json::Value;

use super::setup_readiness_models::{EvidenceHandoffResponse, SetupPreflightResponse};

const REQUIRED_BOUNDARIES: &[&str] = &[
    "Java desktop Alice launch",
    "desktop installer automation",
    "native OpenGL driver diagnosis",
    "native Alice window screenshots",
    "learner-world grading",
    "full Alice UI automation",
];

pub fn preflight_is_ready(preflight: &SetupPreflightResponse, scenario: &str) -> bool {
    preflight.status == "ready"
        && preflight.platform == "lookingglass"
        && preflight.scenario == scenario
        && preflight.classroom_readiness.ready_to_create_project
        && preflight.classroom_readiness.ready_for_lab_handoff
        && preflight.classroom_readiness.ready_for_evidence_handoff
        && contains_all_boundaries(&preflight.unsupported_capabilities)
        && contains_all_boundaries(&preflight.does_not_claim)
        && preflight_matches_scenario(preflight, scenario)
}

pub fn handoff_is_specific(response: &EvidenceHandoffResponse, scenario: &str) -> bool {
    response.status == "handoff-created"
        && response.platform == "lookingglass"
        && response.scenario == scenario
        && !response.evidence_artifact.is_empty()
        && array_contains(&response.handoff, "studentNextActions", "visible result")
        && [
            "blocker category",
            "owner",
            "fallback role",
            "retest signal",
        ]
        .iter()
        .all(|needle| array_contains(&response.handoff, "supportHandoffFields", needle))
        && array_contains_all_boundaries(&response.handoff, "doesNotClaim")
        && handoff_matches_scenario(response, scenario)
}

fn preflight_matches_scenario(preflight: &SetupPreflightResponse, scenario: &str) -> bool {
    match scenario {
        "setup-preflight-ready-to-create" => preflight.checks.iter().any(|check| {
            check.id == "create-project" && check.evidence.contains("create-project routes")
        }),
        "setup-support-lab-readiness" => preflight.checks.iter().any(|check| {
            check.id == "desktop-java-opengl" && check.evidence.contains("doesNotClaim")
        }),
        "instructor-classroom-setup-readiness" => {
            preflight.classroom_readiness.ready_to_create_project
                && preflight.classroom_readiness.ready_for_lab_handoff
        }
        "instructor-student-launch-evidence-handoff" => {
            preflight.classroom_readiness.ready_for_evidence_handoff
        }
        _ => false,
    }
}

fn handoff_matches_scenario(response: &EvidenceHandoffResponse, scenario: &str) -> bool {
    match scenario {
        "setup-preflight-ready-to-create" => {
            array_contains(&response.handoff, "readinessSignals", "create-project")
        }
        "setup-support-lab-readiness" => {
            array_contains(&response.handoff, "supportHandoffFields", "owner")
                && array_contains(&response.handoff, "supportHandoffFields", "retest signal")
        }
        "instructor-classroom-setup-readiness" => {
            array_contains(&response.handoff, "readinessSignals", "project template")
                && array_contains(&response.handoff, "readinessSignals", "create-project")
        }
        "instructor-student-launch-evidence-handoff" => {
            array_contains(&response.handoff, "studentNextActions", "visible result")
                && array_contains(
                    &response.handoff,
                    "studentNextActions",
                    "revision or setup blocker",
                )
        }
        _ => false,
    }
}

fn contains_all_boundaries(values: &[String]) -> bool {
    REQUIRED_BOUNDARIES
        .iter()
        .all(|required| values.iter().any(|value| value == required))
}

fn array_contains_all_boundaries(value: &Value, field: &str) -> bool {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            REQUIRED_BOUNDARIES.iter().all(|required| {
                items
                    .iter()
                    .any(|item| item.as_str().is_some_and(|text| text == *required))
            })
        })
        .unwrap_or(false)
}

fn array_contains(value: &Value, field: &str, needle: &str) -> bool {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|item| {
                item.as_str()
                    .map(|text| text.contains(needle))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
