use crate::launch_save_project::DEFAULT_SAVE_SELECTOR;

pub(super) fn has_save_project_no_go_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("project-save-precondition")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("save-project")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("blocked")
                    && probe.get("decision").and_then(serde_json::Value::as_str) == Some("no_go")
                    && probe
                        .get("missing_affordance")
                        .is_some_and(has_save_project_missing_affordance)
                    && probe
                        .get("preconditions")
                        .and_then(serde_json::Value::as_array)
                        .map(|preconditions| {
                            has_precondition(preconditions, "run-world", true)
                                && has_precondition(
                                    preconditions,
                                    "deterministic-alice-project-save-affordance",
                                    false,
                                )
                        })
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

pub(super) fn has_passed_save_project_candidate_affordance_probe(
    contract: &serde_json::Value,
) -> bool {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("alice-side-project-save-command-hook")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("save-project")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("passed")
                    && probe
                        .get("save_selector")
                        .and_then(serde_json::Value::as_str)
                        == Some(DEFAULT_SAVE_SELECTOR)
                    && probe
                        .get("candidate_hook_path")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.ends_with("tools/eatme-save-project"))
                    && probe
                        .get("saved_project_artifact")
                        .is_some_and(non_empty_artifact)
                    && probe.get("save_artifact").is_some_and(non_empty_artifact)
            })
        })
        .unwrap_or(false)
}

fn non_empty_artifact(artifact: &serde_json::Value) -> bool {
    artifact
        .get("size_bytes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        > 0
}

fn has_precondition(preconditions: &[serde_json::Value], id: &str, expected_passed: bool) -> bool {
    preconditions.iter().any(|precondition| {
        precondition.get("id").and_then(serde_json::Value::as_str) == Some(id)
            && precondition
                .get("passed")
                .and_then(serde_json::Value::as_bool)
                == Some(expected_passed)
    })
}

fn has_save_project_missing_affordance(value: &serde_json::Value) -> bool {
    value.get("id").and_then(serde_json::Value::as_str)
        == Some("deterministic-alice-project-save-affordance")
        && value.get("kind").and_then(serde_json::Value::as_str) == Some("backend_or_ui_affordance")
        && value
            .get("required_capability")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("save the project") && detail.contains("saved .a3p is readable")
            })
        && value
            .get("missing_contract")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("No Alice-side command")
                    && detail.contains("returns project-save proof")
            })
        && value
            .get("next_implementation")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("save-project command") && detail.contains("named save control")
            })
}
