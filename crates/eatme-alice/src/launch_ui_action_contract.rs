use crate::launch_artifacts::artifact_info;
use crate::launch_edit_procedure::{
    DEFAULT_PROCEDURE_EDIT_HOOK, UiActionEditProcedureProbe, probe_edit_procedure_preconditions,
};
use crate::launch_object_placement::{DEFAULT_OBJECT_PLACEMENT_HOOK, UiActionObjectPlacementProbe};
use crate::launch_ui_actions::{UiActionNoGoProbe, UiActionProbe};
use anyhow::Result;
use std::fs;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn write_ui_action_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
    activation_probe: Option<&UiActionProbe>,
    place_object_precondition_probe: Option<&UiActionNoGoProbe>,
    object_placement_probe: Option<&UiActionObjectPlacementProbe>,
    edit_procedure_candidate_probe: Option<&UiActionEditProcedureProbe>,
) -> Result<eatme_core::ArtifactInfo> {
    let path = run_dir.join("ui-action-contract.json");
    let placement_status = object_placement_probe
        .map(|probe| probe.status.as_str())
        .unwrap_or("blocked");
    let action_precondition_probes = place_object_precondition_probe
        .into_iter()
        .filter(|_| placement_status != "passed")
        .collect::<Vec<_>>();
    let edit_procedure_proven =
        edit_procedure_candidate_probe.is_some_and(UiActionEditProcedureProbe::proves_edit);
    let edit_procedure_no_go_probe = object_placement_probe
        .filter(|probe| probe.proves_placement())
        .filter(|_| !edit_procedure_proven)
        .map(probe_edit_procedure_preconditions);
    let edit_procedure_precondition_probes = edit_procedure_no_go_probe.iter().collect::<Vec<_>>();
    let candidate_affordance_probes = object_placement_probe
        .into_iter()
        .map(serde_json::to_value)
        .chain(
            edit_procedure_candidate_probe
                .into_iter()
                .map(serde_json::to_value),
        )
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": ui_action_blocking_reason(placement_status),
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "executed_action_probes": activation_probe.into_iter().collect::<Vec<_>>(),
        "action_precondition_probes": action_precondition_probes
            .into_iter()
            .chain(edit_procedure_precondition_probes)
            .collect::<Vec<_>>(),
        "candidate_affordance_probes": candidate_affordance_probes,
        "required_actions": [
            {
                "id": "verify-specific-alice-window",
                "required_evidence": "wmctrl or xwininfo output identifies the Alice main window"
            },
            {
                "id": "activate-specific-alice-window",
                "required_evidence": "wmctrl -ia or xdotool windowfocus succeeds against the detected Alice window id"
            },
            {
                "id": "place-object",
                "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
                "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
                "contract_required": {
                    "candidate_backend": DEFAULT_OBJECT_PLACEMENT_HOOK,
                    "inputs": ["open_project", "object_identifier", "evidence_dir"],
                    "outputs": ["placement_artifact", "scene_or_project_diff"],
                    "unsafe_until_available": placement_status != "passed"
                }
            },
            {
                "id": "edit-procedure-or-code-block",
                "required_evidence": "artifact proves a procedure or code block was edited",
                "missing_affordance_id": "deterministic-alice-procedure-edit-affordance",
                "contract_required": {
                    "candidate_backend": DEFAULT_PROCEDURE_EDIT_HOOK,
                    "inputs": ["project_after_object_placement", "procedure_selector", "edit_spec", "evidence_dir"],
                    "outputs": ["edited_project_artifact", "procedure_or_code_diff"],
                    "unsafe_until_available": !edit_procedure_proven
                }
            },
            {
                "id": "run-world",
                "required_evidence": "artifact proves the world run control was invoked"
            },
            {
                "id": "save-project",
                "required_evidence": "saved .a3p project artifact exists and is non-empty"
            }
        ]
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    artifact_info(&path)
}

fn ui_action_blocking_reason(placement_status: &str) -> &'static str {
    if placement_status == "passed" {
        "Deterministic object placement evidence exists, but the procedure/code-block edit contract, world run, and project save automation are not wired yet."
    } else {
        "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet."
    }
}
