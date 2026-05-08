const ACTION_NO_GO_REQUIREMENTS: &[(&str, &str)] = &[
    (
        "place-object",
        "deterministic-alice-object-gallery-placement-affordance",
    ),
    (
        "edit-procedure-or-code-block",
        "deterministic-alice-procedure-edit-affordance",
    ),
    ("run-world", "deterministic-alice-world-run-affordance"),
    (
        "save-project",
        "deterministic-alice-project-save-affordance",
    ),
];

pub(super) fn validate_required_action_no_go_contracts(
    role: &str,
    contract: &serde_json::Value,
    issues: &mut Vec<String>,
) {
    for (action_id, missing_affordance_id) in ACTION_NO_GO_REQUIREMENTS {
        if !action_is_proven(contract, action_id)
            && !required_action_has_no_go_contract(contract, action_id, missing_affordance_id)
        {
            issues.push(format!(
                "{role} automation scenario action evidence required action {action_id} must carry a no-go contract until deterministic desktop affordance exists"
            ));
        }
    }
}

fn action_is_proven(contract: &serde_json::Value, action_id: &str) -> bool {
    match action_id {
        "place-object" => super::has_passed_place_object_candidate_affordance_probe(contract),
        "edit-procedure-or-code-block" => {
            super::has_passed_edit_procedure_candidate_affordance_probe(contract)
        }
        "run-world" => super::has_passed_run_world_candidate_affordance_probe(contract),
        "save-project" => super::has_passed_save_project_candidate_affordance_probe(contract),
        _ => false,
    }
}

fn required_action_has_no_go_contract(
    contract: &serde_json::Value,
    action_id: &str,
    missing_affordance_id: &str,
) -> bool {
    contract
        .get("required_actions")
        .and_then(serde_json::Value::as_array)
        .map(|actions| {
            actions.iter().any(|action| {
                action.get("id").and_then(serde_json::Value::as_str) == Some(action_id)
                    && action.get("decision").and_then(serde_json::Value::as_str) == Some("no_go")
                    && action
                        .get("missing_affordance_id")
                        .and_then(serde_json::Value::as_str)
                        == Some(missing_affordance_id)
                    && action
                        .get("contract_required")
                        .and_then(|contract| contract.get("unsafe_until_available"))
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
            })
        })
        .unwrap_or(false)
}
