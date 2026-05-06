pub(super) fn inspect_ui_action_contract(
    role: &str,
    contract: &serde_json::Value,
    issues: &mut Vec<String>,
) {
    if contract
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.ui-action-contract/v1")
    {
        issues.push(format!(
            "{role} ui-action-contract.json has unsupported schema_version"
        ));
    }
    if contract.get("status").and_then(serde_json::Value::as_str) != Some("blocked") {
        issues.push(format!(
            "{role} ui-action-contract.json status must remain blocked until UI actions are automated"
        ));
    }
    if contract
        .get("blocking_reason")
        .and_then(serde_json::Value::as_str)
        .map(str::is_empty)
        .unwrap_or(true)
    {
        issues.push(format!(
            "{role} ui-action-contract.json must explain the blocking_reason"
        ));
    }
    let preflight = contract
        .get("preflight_evidence")
        .and_then(serde_json::Value::as_object);
    for field in [
        "specific_alice_window_detected",
        "visual_evidence_captured",
        "log_captured",
    ] {
        if preflight
            .and_then(|entry| entry.get(field))
            .and_then(serde_json::Value::as_bool)
            .is_none()
        {
            issues.push(format!(
                "{role} ui-action-contract.json preflight_evidence.{field} must be a boolean"
            ));
        }
    }
    if contract
        .get("preflight_evidence")
        .and_then(|preflight| preflight.get("specific_alice_window_detected"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
        && !has_passed_action_probe(contract, "activate-specific-alice-window")
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record passed activate-specific-alice-window probe when an Alice window is detected"
        ));
    }
    if !has_place_object_no_go_probe(contract) {
        issues.push(format!(
            "{role} ui-action-contract.json must record a no-go precondition probe for place-object"
        ));
    }
}

pub(super) fn action_ids(contract: &serde_json::Value) -> Vec<String> {
    contract
        .get("required_actions")
        .and_then(serde_json::Value::as_array)
        .map(|actions| {
            actions
                .iter()
                .filter_map(|action| action.get("id").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn has_passed_action_probe(contract: &serde_json::Value, probe_id: &str) -> bool {
    contract
        .get("executed_action_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str) == Some(probe_id)
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("passed")
            })
        })
        .unwrap_or(false)
}

fn has_place_object_no_go_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("place-object-precondition")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("place-object")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("blocked")
                    && probe.get("decision").and_then(serde_json::Value::as_str) == Some("no_go")
                    && probe
                        .get("missing_affordance")
                        .is_some_and(has_place_object_missing_affordance)
                    && probe
                        .get("preconditions")
                        .and_then(serde_json::Value::as_array)
                        .map(|preconditions| {
                            preconditions.iter().any(|precondition| {
                                precondition.get("id").and_then(serde_json::Value::as_str)
                                    == Some(
                                        "deterministic-alice-object-gallery-placement-affordance",
                                    )
                                    && precondition
                                        .get("passed")
                                        .and_then(serde_json::Value::as_bool)
                                        == Some(false)
                            })
                        })
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn has_place_object_missing_affordance(value: &serde_json::Value) -> bool {
    value.get("id").and_then(serde_json::Value::as_str)
        == Some("deterministic-alice-object-gallery-placement-affordance")
        && value.get("kind").and_then(serde_json::Value::as_str) == Some("backend_or_ui_affordance")
        && value
            .get("required_capability")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("named object identifier")
                    && detail.contains("without coordinate guessing")
            })
        && value
            .get("missing_contract")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("No Alice backend command")
                    && detail.contains("returns proof of placement")
            })
        && value
            .get("next_implementation")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("Alice-side object placement command")
                    && detail.contains("named gallery selector")
            })
}
