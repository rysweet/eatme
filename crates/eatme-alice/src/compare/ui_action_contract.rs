use crate::launch_edit_procedure::DEFAULT_PROCEDURE_SELECTOR;
use crate::launch_run_world::DEFAULT_RUN_SELECTOR;
use required_action::validate_required_action_no_go_contracts;
use save::{has_passed_save_project_candidate_affordance_probe, has_save_project_no_go_probe};

mod required_action;

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
    if has_passed_action_probe(contract, "activate-specific-alice-window")
        && !has_passed_action_probe(contract, "dispatch-save-project-shortcut")
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record passed dispatch-save-project-shortcut probe after Alice window activation"
        ));
    }
    if !has_place_object_candidate_affordance_probe(contract) {
        issues.push(format!(
            "{role} ui-action-contract.json must record the Alice-side object placement command hook candidate probe"
        ));
    }
    if !has_passed_place_object_candidate_affordance_probe(contract)
        && !has_place_object_no_go_probe(contract)
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record a no-go precondition probe for place-object"
        ));
    }
    if has_passed_place_object_candidate_affordance_probe(contract)
        && !has_passed_edit_procedure_candidate_affordance_probe(contract)
        && !has_edit_procedure_no_go_probe(contract)
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record either passed edit-procedure-or-code-block proof or a no-go precondition probe after object placement passes"
        ));
    }
    if has_passed_edit_procedure_candidate_affordance_probe(contract)
        && !has_passed_action_probe(contract, "dispatch-run-world-shortcut")
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record passed dispatch-run-world-shortcut probe after edit-procedure-or-code-block proof"
        ));
    }
    if has_passed_action_probe(contract, "dispatch-run-world-shortcut")
        && !has_action_probe(contract, "observe-run-window-after-shortcut")
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record observe-run-window-after-shortcut probe after desktop Run shortcut dispatch"
        ));
    }
    if has_passed_edit_procedure_candidate_affordance_probe(contract)
        && !has_passed_run_world_candidate_affordance_probe(contract)
        && !has_run_world_no_go_probe(contract)
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record either passed run-world proof or a no-go precondition probe after edit-procedure-or-code-block passes"
        ));
    }
    if has_passed_run_world_candidate_affordance_probe(contract)
        && !has_passed_save_project_candidate_affordance_probe(contract)
        && !has_save_project_no_go_probe(contract)
    {
        issues.push(format!(
            "{role} ui-action-contract.json must record either passed save-project proof or a no-go precondition probe after run-world passes"
        ));
    }
    validate_required_action_no_go_contracts(role, contract, issues);
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
    has_action_probe_with_status(contract, probe_id, Some("passed"))
}

fn has_action_probe(contract: &serde_json::Value, probe_id: &str) -> bool {
    has_action_probe_with_status(contract, probe_id, None)
}

fn has_action_probe_with_status(
    contract: &serde_json::Value,
    probe_id: &str,
    status: Option<&str>,
) -> bool {
    contract
        .get("executed_action_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str) == Some(probe_id)
                    && status.is_none_or(|expected| {
                        probe.get("status").and_then(serde_json::Value::as_str) == Some(expected)
                    })
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

fn has_place_object_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("alice-side-object-placement-command-hook")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("place-object")
                    && probe
                        .get("object_identifier")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.starts_with("alice-gallery://"))
                    && probe
                        .get("candidate_hook_path")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.ends_with("tools/eatme-place-object"))
                    && matches!(
                        probe.get("status").and_then(serde_json::Value::as_str),
                        Some("passed" | "blocked" | "failed")
                    )
            })
        })
        .unwrap_or(false)
}

fn has_passed_place_object_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("alice-side-object-placement-command-hook")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("place-object")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("passed")
                    && probe.get("placement_artifact").is_some_and(|artifact| {
                        artifact
                            .get("size_bytes")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0)
                            > 0
                    })
                    && probe.get("scene_or_project_diff").is_some_and(|artifact| {
                        artifact
                            .get("size_bytes")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0)
                            > 0
                    })
            })
        })
        .unwrap_or(false)
}

fn has_passed_edit_procedure_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("alice-side-procedure-edit-command-hook")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("edit-procedure-or-code-block")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("passed")
                    && probe
                        .get("procedure_selector")
                        .and_then(serde_json::Value::as_str)
                        == Some(DEFAULT_PROCEDURE_SELECTOR)
                    && probe
                        .get("candidate_hook_path")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.ends_with("tools/eatme-edit-procedure"))
                    && probe
                        .get("edited_project_artifact")
                        .is_some_and(|artifact| {
                            artifact
                                .get("size_bytes")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0)
                                > 0
                        })
                    && probe.get("procedure_or_code_diff").is_some_and(|artifact| {
                        artifact
                            .get("size_bytes")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0)
                            > 0
                    })
            })
        })
        .unwrap_or(false)
}

fn has_edit_procedure_no_go_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("edit-procedure-precondition")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("edit-procedure-or-code-block")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("blocked")
                    && probe.get("decision").and_then(serde_json::Value::as_str) == Some("no_go")
                    && probe
                        .get("missing_affordance")
                        .is_some_and(has_edit_procedure_missing_affordance)
                    && probe
                        .get("preconditions")
                        .and_then(serde_json::Value::as_array)
                        .map(|preconditions| {
                            has_precondition(preconditions, "place-object", true)
                                && has_precondition(
                                    preconditions,
                                    "deterministic-alice-procedure-edit-affordance",
                                    false,
                                )
                        })
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn has_run_world_no_go_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("run-world-precondition")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("run-world")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("blocked")
                    && probe.get("decision").and_then(serde_json::Value::as_str) == Some("no_go")
                    && probe
                        .get("missing_affordance")
                        .is_some_and(has_run_world_missing_affordance)
                    && probe
                        .get("preconditions")
                        .and_then(serde_json::Value::as_array)
                        .map(|preconditions| {
                            has_precondition(preconditions, "edit-procedure-or-code-block", true)
                                && has_precondition(
                                    preconditions,
                                    "deterministic-alice-world-run-affordance",
                                    false,
                                )
                        })
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn has_passed_run_world_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .map(|probes| {
            probes.iter().any(|probe| {
                probe.get("id").and_then(serde_json::Value::as_str)
                    == Some("alice-side-world-run-command-hook")
                    && probe.get("action_id").and_then(serde_json::Value::as_str)
                        == Some("run-world")
                    && probe.get("status").and_then(serde_json::Value::as_str) == Some("passed")
                    && probe
                        .get("run_selector")
                        .and_then(serde_json::Value::as_str)
                        == Some(DEFAULT_RUN_SELECTOR)
                    && probe
                        .get("candidate_hook_path")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.ends_with("tools/eatme-run-world"))
                    && probe.get("run_artifact").is_some_and(|artifact| {
                        artifact
                            .get("size_bytes")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0)
                            > 0
                    })
                    && probe
                        .get("runtime_or_log_evidence")
                        .is_some_and(|artifact| {
                            artifact
                                .get("size_bytes")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0)
                                > 0
                        })
            })
        })
        .unwrap_or(false)
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
                detail.contains("No Alice-side command")
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

fn has_edit_procedure_missing_affordance(value: &serde_json::Value) -> bool {
    value.get("id").and_then(serde_json::Value::as_str)
        == Some("deterministic-alice-procedure-edit-affordance")
        && value.get("kind").and_then(serde_json::Value::as_str) == Some("backend_or_ui_affordance")
        && value
            .get("required_capability")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("procedure or code-block selector")
                    && detail.contains("return proof of the edit")
            })
        && value
            .get("missing_contract")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("No Alice-side command")
                    && detail.contains("procedure/code-block selector")
                    && detail.contains("procedure/code diff")
            })
        && value
            .get("next_implementation")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("procedure edit command") && detail.contains("named editor target")
            })
}

fn has_run_world_missing_affordance(value: &serde_json::Value) -> bool {
    value.get("id").and_then(serde_json::Value::as_str)
        == Some("deterministic-alice-world-run-affordance")
        && value.get("kind").and_then(serde_json::Value::as_str) == Some("backend_or_ui_affordance")
        && value
            .get("required_capability")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("run the world")
                    && detail.contains("return proof that execution reached")
            })
        && value
            .get("missing_contract")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("No Alice-side command")
                    && detail.contains("returns world-run proof")
            })
        && value
            .get("next_implementation")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| {
                detail.contains("run-world command") && detail.contains("named run control")
            })
}

mod save;
#[cfg(test)]
mod tests;
