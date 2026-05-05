use crate::schema::EatmeScenarioAsset;

use super::scenario_contains_all_boundary_phrases;

pub(super) fn validate_real_ui_action_contract(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if !scenario.artifacts.contains_key("ui_action_contract") {
        errors.push("artifacts.ui_action_contract must be defined".into());
    }
    let action_evidence = [
        "specific_alice_window_detected",
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
    if !scenario_contains_all_boundary_phrases(
        scenario,
        &[
            "ui_action_automation_unimplemented",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
        ],
    ) {
        errors.push(
            "real UI action contract must state ui_action_automation_unimplemented and the current boundary: not full UI automation, not creative assessment, and not learner-world grading"
                .into(),
        );
    }
}
