use crate::schema::EatmeScenarioAsset;

use super::honest_boundary::{
    REAL_UI_ACTION_BOUNDARY_PHRASES, scenario_contains_all_boundary_phrases,
};

pub(super) fn validate_real_ui_action_contract(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if !scenario.artifacts.contains_key("ui_action_contract") {
        errors.push("artifacts.ui_action_contract must be defined".into());
    }
    let action_evidence = [
        "specific_alice_window_detected",
        "activate_alice_window_ui_action",
        "place_object_ui_action",
        "edit_procedure_ui_action",
        "run_world_ui_action",
        "save_project_ui_action",
        "ui_action_artifact_captured",
    ];
    for required in action_evidence {
        let mentioned = scenario.steps.iter().any(|step| {
            step.evidence
                .iter()
                .any(|evidence| evidence.contains(required))
        });
        if !mentioned {
            errors.push(format!("real UI action contract must inspect {required}"));
        }
    }
    if !scenario_contains_all_boundary_phrases(scenario, REAL_UI_ACTION_BOUNDARY_PHRASES) {
        errors.push(format!(
            "real UI action contract must state the current boundary: {}",
            REAL_UI_ACTION_BOUNDARY_PHRASES.join(", ")
        ));
    }
}
