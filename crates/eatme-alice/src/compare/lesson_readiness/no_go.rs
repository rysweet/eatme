use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionNoGoContract {
    pub target_role: String,
    pub affordance: String,
    pub decision: String,
    pub reason: String,
    pub missing_affordance_id: Option<String>,
}

pub(super) fn ui_action_no_go_contracts(
    role: &str,
    contract: &serde_json::Value,
) -> Vec<LessonSessionNoGoContract> {
    let mut contracts = BTreeMap::new();
    collect_precondition_no_go_contracts(role, contract, &mut contracts);
    collect_required_action_no_go_contracts(role, contract, &mut contracts);
    contracts.into_values().collect()
}

fn collect_precondition_no_go_contracts(
    role: &str,
    contract: &serde_json::Value,
    contracts: &mut BTreeMap<String, LessonSessionNoGoContract>,
) {
    let Some(probes) = contract
        .get("action_precondition_probes")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };
    for probe in probes {
        if probe.get("decision").and_then(serde_json::Value::as_str) != Some("no_go") {
            continue;
        }
        let action_id = probe
            .get("action_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let missing_affordance_id = probe
            .get("missing_affordance")
            .and_then(|affordance| affordance.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let reason =
            evidence_gap_reason(affordance_name(action_id, missing_affordance_id.as_deref()));
        upsert_no_go_contract(contracts, role, action_id, missing_affordance_id, reason);
    }
}

fn collect_required_action_no_go_contracts(
    role: &str,
    contract: &serde_json::Value,
    contracts: &mut BTreeMap<String, LessonSessionNoGoContract>,
) {
    let Some(actions) = contract
        .get("required_actions")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };
    for action in actions {
        let unsafe_until_available = action
            .get("contract_required")
            .and_then(|contract| contract.get("unsafe_until_available"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let decision_is_no_go =
            action.get("decision").and_then(serde_json::Value::as_str) == Some("no_go");
        if !unsafe_until_available && !decision_is_no_go {
            continue;
        }
        let action_id = action
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let missing_affordance_id = action
            .get("missing_affordance_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let reason =
            evidence_gap_reason(affordance_name(action_id, missing_affordance_id.as_deref()));
        upsert_no_go_contract(contracts, role, action_id, missing_affordance_id, reason);
    }
}

fn evidence_gap_reason(affordance: &str) -> String {
    let affordance = affordance.replace('_', " ");
    format!(
        "Evidence gap: required evidence is missing or incomplete for {affordance}; cannot report this action as supported."
    )
}

fn upsert_no_go_contract(
    contracts: &mut BTreeMap<String, LessonSessionNoGoContract>,
    role: &str,
    action_id: &str,
    missing_affordance_id: Option<String>,
    reason: String,
) {
    let key = format!(
        "{}:{}:{}",
        role,
        action_id,
        missing_affordance_id.as_deref().unwrap_or("")
    );
    contracts
        .entry(key)
        .or_insert_with(|| LessonSessionNoGoContract {
            target_role: role.into(),
            affordance: affordance_name(action_id, missing_affordance_id.as_deref()).into(),
            decision: "no_go".into(),
            reason,
            missing_affordance_id,
        });
}

fn affordance_name(action_id: &str, missing_affordance_id: Option<&str>) -> &'static str {
    match action_id {
        "place-object" => "object_placement",
        "edit-procedure-or-code-block" => "procedure_edit",
        "run-world" => "world_run",
        "save-project" => "project_save",
        _ => match missing_affordance_id {
            Some("deterministic-alice-object-gallery-placement-affordance") => "object_placement",
            Some("deterministic-alice-procedure-edit-affordance") => "procedure_edit",
            Some("deterministic-alice-world-run-affordance") => "world_run",
            Some("deterministic-alice-project-save-affordance") => "project_save",
            _ => "unknown",
        },
    }
}
