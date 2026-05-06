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
    let place_object_probe_recorded = has_place_object_candidate_affordance_probe(contract);
    let place_object_proven = has_passed_place_object_candidate_affordance_probe(contract);
    let edit_procedure_proven = has_passed_edit_procedure_candidate_affordance_probe(contract);
    let run_world_proven = has_passed_run_world_candidate_affordance_probe(contract);
    let save_project_proven = has_passed_save_project_candidate_affordance_probe(contract);

    if !place_object_probe_recorded {
        issues.push(format!(
            "{role} ui-action-contract.json must record the Alice-side object placement command hook candidate probe"
        ));
    }
    if !place_object_proven && !has_place_object_no_go_probe(contract) {
        issues.push(format!(
            "{role} ui-action-contract.json must record a no-go precondition probe for place-object"
        ));
    }
    if place_object_proven && !edit_procedure_proven && !has_edit_procedure_no_go_probe(contract) {
        issues.push(format!(
            "{role} ui-action-contract.json must record either passed edit-procedure-or-code-block proof or a no-go precondition probe after object placement passes"
        ));
    }
    if edit_procedure_proven && !has_passed_action_probe(contract, "dispatch-run-world-shortcut") {
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
    if edit_procedure_proven && !run_world_proven && !has_run_world_no_go_probe(contract) {
        issues.push(format!(
            "{role} ui-action-contract.json must record either passed run-world proof or a no-go precondition probe after edit-procedure-or-code-block passes"
        ));
    }
    if run_world_proven && !save_project_proven && !has_save_project_no_go_probe(contract) {
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

struct NoGoProbeContract {
    probe_id: &'static str,
    action_id: &'static str,
    missing_affordance: MissingAffordanceContract,
    preconditions: &'static [NoGoPrecondition],
}

struct MissingAffordanceContract {
    id: &'static str,
    required_capability_contains: &'static [&'static str],
    missing_contract_contains: &'static [&'static str],
    next_implementation_contains: &'static [&'static str],
}

struct NoGoPrecondition {
    id: &'static str,
    passed: bool,
}

const PLACE_OBJECT_NO_GO: NoGoProbeContract = NoGoProbeContract {
    probe_id: "place-object-precondition",
    action_id: "place-object",
    missing_affordance: MissingAffordanceContract {
        id: "deterministic-alice-object-gallery-placement-affordance",
        required_capability_contains: &["named object identifier", "without coordinate guessing"],
        missing_contract_contains: &["No Alice-side command", "returns proof of placement"],
        next_implementation_contains: &[
            "Alice-side object placement command",
            "named gallery selector",
        ],
    },
    preconditions: &[NoGoPrecondition {
        id: "deterministic-alice-object-gallery-placement-affordance",
        passed: false,
    }],
};

const EDIT_PROCEDURE_NO_GO: NoGoProbeContract = NoGoProbeContract {
    probe_id: "edit-procedure-precondition",
    action_id: "edit-procedure-or-code-block",
    missing_affordance: MissingAffordanceContract {
        id: "deterministic-alice-procedure-edit-affordance",
        required_capability_contains: &[
            "procedure or code-block selector",
            "return proof of the edit",
        ],
        missing_contract_contains: &[
            "No Alice-side command",
            "procedure/code-block selector",
            "procedure/code diff",
        ],
        next_implementation_contains: &["procedure edit command", "named editor target"],
    },
    preconditions: &[
        NoGoPrecondition {
            id: "place-object",
            passed: true,
        },
        NoGoPrecondition {
            id: "deterministic-alice-procedure-edit-affordance",
            passed: false,
        },
    ],
};

const RUN_WORLD_NO_GO: NoGoProbeContract = NoGoProbeContract {
    probe_id: "run-world-precondition",
    action_id: "run-world",
    missing_affordance: MissingAffordanceContract {
        id: "deterministic-alice-world-run-affordance",
        required_capability_contains: &["run the world", "return proof that execution reached"],
        missing_contract_contains: &["No Alice-side command", "returns world-run proof"],
        next_implementation_contains: &["run-world command", "named run control"],
    },
    preconditions: &[
        NoGoPrecondition {
            id: "edit-procedure-or-code-block",
            passed: true,
        },
        NoGoPrecondition {
            id: "deterministic-alice-world-run-affordance",
            passed: false,
        },
    ],
};

fn has_place_object_no_go_probe(contract: &serde_json::Value) -> bool {
    has_no_go_probe(contract, &PLACE_OBJECT_NO_GO)
}

fn has_place_object_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    candidate_affordance_probes(contract).any(|probe| {
        string_field(probe, "id") == Some("alice-side-object-placement-command-hook")
            && string_field(probe, "action_id") == Some("place-object")
            && string_field(probe, "object_identifier")
                .is_some_and(|value| value.starts_with("alice-gallery://"))
            && string_field(probe, "candidate_hook_path")
                .is_some_and(|value| value.ends_with("tools/eatme-place-object"))
            && matches!(
                string_field(probe, "status"),
                Some("passed" | "blocked" | "failed")
            )
    })
}

fn has_passed_place_object_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    candidate_affordance_probes(contract).any(|probe| {
        string_field(probe, "id") == Some("alice-side-object-placement-command-hook")
            && string_field(probe, "action_id") == Some("place-object")
            && string_field(probe, "status") == Some("passed")
            && probe
                .get("placement_artifact")
                .is_some_and(non_empty_artifact)
            && probe
                .get("scene_or_project_diff")
                .is_some_and(non_empty_artifact)
    })
}

fn has_passed_edit_procedure_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    candidate_affordance_probes(contract).any(|probe| {
        string_field(probe, "id") == Some("alice-side-procedure-edit-command-hook")
            && string_field(probe, "action_id") == Some("edit-procedure-or-code-block")
            && string_field(probe, "status") == Some("passed")
            && string_field(probe, "procedure_selector") == Some(DEFAULT_PROCEDURE_SELECTOR)
            && string_field(probe, "candidate_hook_path")
                .is_some_and(|value| value.ends_with("tools/eatme-edit-procedure"))
            && probe
                .get("edited_project_artifact")
                .is_some_and(non_empty_artifact)
            && probe
                .get("procedure_or_code_diff")
                .is_some_and(non_empty_artifact)
    })
}

fn has_edit_procedure_no_go_probe(contract: &serde_json::Value) -> bool {
    has_no_go_probe(contract, &EDIT_PROCEDURE_NO_GO)
}

fn has_run_world_no_go_probe(contract: &serde_json::Value) -> bool {
    has_no_go_probe(contract, &RUN_WORLD_NO_GO)
}

fn has_passed_run_world_candidate_affordance_probe(contract: &serde_json::Value) -> bool {
    candidate_affordance_probes(contract).any(|probe| {
        string_field(probe, "id") == Some("alice-side-world-run-command-hook")
            && string_field(probe, "action_id") == Some("run-world")
            && string_field(probe, "status") == Some("passed")
            && string_field(probe, "run_selector") == Some(DEFAULT_RUN_SELECTOR)
            && string_field(probe, "candidate_hook_path")
                .is_some_and(|value| value.ends_with("tools/eatme-run-world"))
            && probe.get("run_artifact").is_some_and(non_empty_artifact)
            && probe
                .get("runtime_or_log_evidence")
                .is_some_and(non_empty_artifact)
    })
}

fn has_no_go_probe(contract: &serde_json::Value, expected: &NoGoProbeContract) -> bool {
    action_precondition_probes(contract).any(|probe| no_go_probe_matches(probe, expected))
}

fn no_go_probe_matches(probe: &serde_json::Value, expected: &NoGoProbeContract) -> bool {
    string_field(probe, "id") == Some(expected.probe_id)
        && string_field(probe, "action_id") == Some(expected.action_id)
        && string_field(probe, "status") == Some("blocked")
        && string_field(probe, "decision") == Some("no_go")
        && probe.get("missing_affordance").is_some_and(|affordance| {
            missing_affordance_matches(affordance, &expected.missing_affordance)
        })
        && probe
            .get("preconditions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|preconditions| preconditions_match(preconditions, expected.preconditions))
}

fn missing_affordance_matches(
    value: &serde_json::Value,
    expected: &MissingAffordanceContract,
) -> bool {
    string_field(value, "id") == Some(expected.id)
        && string_field(value, "kind") == Some("backend_or_ui_affordance")
        && field_contains_all(
            value,
            "required_capability",
            expected.required_capability_contains,
        )
        && field_contains_all(
            value,
            "missing_contract",
            expected.missing_contract_contains,
        )
        && field_contains_all(
            value,
            "next_implementation",
            expected.next_implementation_contains,
        )
}

fn field_contains_all(value: &serde_json::Value, field: &str, needles: &[&str]) -> bool {
    string_field(value, field)
        .is_some_and(|detail| needles.iter().all(|needle| detail.contains(needle)))
}

fn preconditions_match(preconditions: &[serde_json::Value], expected: &[NoGoPrecondition]) -> bool {
    expected.iter().all(|expected| {
        preconditions.iter().any(|precondition| {
            string_field(precondition, "id") == Some(expected.id)
                && precondition
                    .get("passed")
                    .and_then(serde_json::Value::as_bool)
                    == Some(expected.passed)
        })
    })
}

fn action_precondition_probes(
    contract: &serde_json::Value,
) -> impl Iterator<Item = &serde_json::Value> {
    contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
}

fn candidate_affordance_probes(
    contract: &serde_json::Value,
) -> impl Iterator<Item = &serde_json::Value> {
    contract
        .get("candidate_affordance_probes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
}

fn string_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(serde_json::Value::as_str)
}

fn non_empty_artifact(artifact: &serde_json::Value) -> bool {
    artifact
        .get("size_bytes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        > 0
}

mod save;
#[cfg(test)]
mod tests;
