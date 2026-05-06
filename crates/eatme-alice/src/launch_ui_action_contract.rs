use crate::launch_artifacts::artifact_info;
use crate::launch_edit_procedure::{
    DEFAULT_PROCEDURE_EDIT_HOOK, UiActionEditProcedureProbe, probe_edit_procedure_preconditions,
};
use crate::launch_object_placement::{DEFAULT_OBJECT_PLACEMENT_HOOK, UiActionObjectPlacementProbe};
use crate::launch_run_world::{
    DEFAULT_WORLD_RUN_HOOK, UiActionRunWorldProbe, probe_run_world_preconditions,
};
use crate::launch_save_project::{
    DEFAULT_PROJECT_SAVE_HOOK, UiActionSaveProjectProbe, probe_project_save_preconditions,
};
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
    desktop_save_shortcut_probe: Option<&UiActionProbe>,
    desktop_run_shortcut_probe: Option<&UiActionProbe>,
    run_window_probe: Option<&UiActionProbe>,
    place_object_precondition_probe: Option<&UiActionNoGoProbe>,
    object_placement_probe: Option<&UiActionObjectPlacementProbe>,
    edit_procedure_candidate_probe: Option<&UiActionEditProcedureProbe>,
    run_world_candidate_probe: Option<&UiActionRunWorldProbe>,
    save_project_candidate_probe: Option<&UiActionSaveProjectProbe>,
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
    let run_world_proven = run_world_candidate_probe.is_some_and(UiActionRunWorldProbe::proves_run);
    let save_project_proven =
        save_project_candidate_probe.is_some_and(UiActionSaveProjectProbe::proves_save);
    let edit_procedure_no_go_probe = object_placement_probe
        .filter(|probe| probe.proves_placement())
        .filter(|_| !edit_procedure_proven)
        .map(probe_edit_procedure_preconditions);
    let edit_procedure_precondition_probes = edit_procedure_no_go_probe.iter().collect::<Vec<_>>();
    let run_world_no_go_probe = edit_procedure_candidate_probe
        .filter(|probe| probe.proves_edit())
        .filter(|_| !run_world_proven)
        .map(probe_run_world_preconditions);
    let run_world_precondition_probes = run_world_no_go_probe.iter().collect::<Vec<_>>();
    let save_project_no_go_probe = run_world_candidate_probe
        .filter(|probe| probe.proves_run())
        .filter(|_| !save_project_proven)
        .map(probe_project_save_preconditions);
    let save_project_precondition_probes = save_project_no_go_probe.iter().collect::<Vec<_>>();
    let candidate_affordance_probes = object_placement_probe
        .into_iter()
        .map(serde_json::to_value)
        .chain(
            edit_procedure_candidate_probe
                .into_iter()
                .map(serde_json::to_value),
        )
        .chain(
            run_world_candidate_probe
                .into_iter()
                .map(serde_json::to_value),
        )
        .chain(
            save_project_candidate_probe
                .into_iter()
                .map(serde_json::to_value),
        )
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": ui_action_blocking_reason(placement_status, edit_procedure_proven, run_world_proven, save_project_proven),
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "executed_action_probes": activation_probe
            .into_iter()
            .chain(desktop_save_shortcut_probe)
            .chain(desktop_run_shortcut_probe)
            .chain(run_window_probe)
            .collect::<Vec<_>>(),
        "action_precondition_probes": action_precondition_probes
            .into_iter()
            .chain(edit_procedure_precondition_probes)
            .chain(run_world_precondition_probes)
            .chain(save_project_precondition_probes)
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
                "required_evidence": "artifact proves the world run control or equivalent runtime entry point executed after the first-lesson edit",
                "missing_affordance_id": "deterministic-alice-world-run-affordance",
                "contract_required": {
                    "candidate_backend": DEFAULT_WORLD_RUN_HOOK,
                    "inputs": ["edited_project", "run_selector", "evidence_dir"],
                    "outputs": ["run_artifact", "runtime_or_log_evidence"],
                    "unsafe_until_available": !run_world_proven
                }
            },
            {
                "id": "save-project",
                "required_evidence": "saved .a3p project artifact exists, is non-empty, and can be read after the first-lesson run proof",
                "missing_affordance_id": "deterministic-alice-project-save-affordance",
                "contract_required": {
                    "candidate_backend": DEFAULT_PROJECT_SAVE_HOOK,
                    "inputs": ["edited_project", "save_selector", "evidence_dir"],
                    "outputs": ["saved_project_artifact", "save_artifact"],
                    "unsafe_until_available": !save_project_proven
                }
            }
        ]
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    artifact_info(&path)
}

fn ui_action_blocking_reason(
    placement_status: &str,
    edit_procedure_proven: bool,
    run_world_proven: bool,
    save_project_proven: bool,
) -> &'static str {
    if save_project_proven {
        "Deterministic object placement, procedure edit, world run, and project save backend evidence exist, but full desktop lesson automation is not wired yet."
    } else if run_world_proven {
        "Deterministic object placement, procedure edit, and world run evidence exist, but project save automation is not wired yet."
    } else if edit_procedure_proven {
        "Deterministic object placement and procedure edit evidence exist, but world run and project save automation are not wired yet."
    } else if placement_status == "passed" {
        "Deterministic object placement evidence exists, but the procedure/code-block edit contract, world run, and project save automation are not wired yet."
    } else {
        "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet."
    }
}
