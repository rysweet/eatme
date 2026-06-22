use serde_json::Value;

use super::setup_readiness_models::{EvidenceHandoffResponse, SetupPreflightResponse};

pub fn preflight_is_ready(preflight: &SetupPreflightResponse, scenario: &str) -> bool {
    preflight.status == "ready"
        && preflight.platform == "lookingglass"
        && preflight.scenario == scenario
        && preflight.classroom_readiness.ready_to_create_project
        && preflight.classroom_readiness.ready_for_lab_handoff
        && preflight.classroom_readiness.ready_for_evidence_handoff
        && preflight
            .unsupported_capabilities
            .iter()
            .any(|capability| capability.contains("Java desktop Alice launch"))
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
