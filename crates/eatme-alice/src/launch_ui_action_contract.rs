use crate::launch_artifacts::artifact_info;
use crate::launch_edit_procedure::{
    DEFAULT_PROCEDURE_EDIT_HOOK, UiActionEditProcedureProbe, probe_edit_procedure_preconditions,
};
use crate::launch_object_placement::{DEFAULT_OBJECT_PLACEMENT_HOOK, UiActionObjectPlacementProbe};
use crate::launch_object_transform::{
    DEFAULT_OBJECT_TRANSFORM_HOOK, UiActionObjectTransformProbe,
    probe_object_transform_preconditions,
};
use crate::launch_reopen_project::{
    DEFAULT_PROJECT_REOPEN_HOOK, UiActionReopenProjectProbe, probe_project_reopen_preconditions,
};
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
    window_verification_probe: Option<&UiActionProbe>,
    activation_probe: Option<&UiActionProbe>,
    desktop_save_shortcut_probe: Option<&UiActionProbe>,
    desktop_run_shortcut_probe: Option<&UiActionProbe>,
    run_window_probe: Option<&UiActionProbe>,
    desktop_run_toolbar_probe: Option<&UiActionProbe>,
    run_window_after_toolbar_probe: Option<&UiActionProbe>,
    desktop_run_execution_probe: Option<&UiActionProbe>,
    place_object_precondition_probe: Option<&UiActionNoGoProbe>,
    object_placement_probe: Option<&UiActionObjectPlacementProbe>,
    object_transform_probe: Option<&UiActionObjectTransformProbe>,
    edit_procedure_candidate_probe: Option<&UiActionEditProcedureProbe>,
    run_world_candidate_probe: Option<&UiActionRunWorldProbe>,
    save_project_candidate_probe: Option<&UiActionSaveProjectProbe>,
    reopen_project_candidate_probe: Option<&UiActionReopenProjectProbe>,
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
    let object_transform_proven =
        object_transform_probe.is_some_and(UiActionObjectTransformProbe::proves_transform);
    let reopen_project_proven =
        reopen_project_candidate_probe.is_some_and(UiActionReopenProjectProbe::proves_reopen);
    let include_objects_first_actions =
        object_transform_probe.is_some() || reopen_project_candidate_probe.is_some();
    let contract_passed = if include_objects_first_actions {
        reopen_project_proven
    } else {
        save_project_proven
    };
    let transform_no_go_probe = include_objects_first_actions
        .then(|| {
            object_placement_probe
                .filter(|probe| probe.proves_placement())
                .filter(|_| !object_transform_proven)
                .map(probe_object_transform_preconditions)
        })
        .flatten();
    let transform_precondition_probes = transform_no_go_probe.iter().collect::<Vec<_>>();
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
    let reopen_project_no_go_probe = include_objects_first_actions
        .then(|| {
            save_project_candidate_probe
                .filter(|probe| probe.proves_save())
                .filter(|_| !reopen_project_proven)
                .map(probe_project_reopen_preconditions)
        })
        .flatten();
    let reopen_project_precondition_probes = reopen_project_no_go_probe.iter().collect::<Vec<_>>();
    let candidate_affordance_probes = object_placement_probe
        .into_iter()
        .map(serde_json::to_value)
        .chain(object_transform_probe.into_iter().map(serde_json::to_value))
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
        .chain(
            reopen_project_candidate_probe
                .into_iter()
                .map(serde_json::to_value),
        )
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let required_actions = required_actions(
        placement_status,
        object_transform_proven,
        edit_procedure_proven,
        run_world_proven,
        save_project_proven,
        reopen_project_proven,
        include_objects_first_actions,
    );
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": if contract_passed { "passed" } else { "blocked" },
        "blocking_reason": ui_action_blocking_reason(placement_status, object_transform_proven, edit_procedure_proven, run_world_proven, save_project_proven, reopen_project_proven, include_objects_first_actions),
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "executed_action_probes": window_verification_probe
            .into_iter()
            .chain(activation_probe)
            .chain(desktop_save_shortcut_probe)
            .chain(desktop_run_shortcut_probe)
            .chain(run_window_probe)
            .chain(desktop_run_toolbar_probe)
            .chain(run_window_after_toolbar_probe)
            .chain(desktop_run_execution_probe)
            .collect::<Vec<_>>(),
        "action_precondition_probes": action_precondition_probes
            .into_iter()
            .chain(transform_precondition_probes)
            .chain(edit_procedure_precondition_probes)
            .chain(run_world_precondition_probes)
            .chain(save_project_precondition_probes)
            .chain(reopen_project_precondition_probes)
            .collect::<Vec<_>>(),
        "candidate_affordance_probes": candidate_affordance_probes,
        "required_actions": required_actions
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    artifact_info(&path)
}

#[allow(clippy::too_many_arguments)]
fn required_actions(
    placement_status: &str,
    object_transform_proven: bool,
    edit_procedure_proven: bool,
    run_world_proven: bool,
    save_project_proven: bool,
    reopen_project_proven: bool,
    include_objects_first_actions: bool,
) -> Vec<serde_json::Value> {
    let mut actions = vec![
        serde_json::json!({
            "id": "verify-specific-alice-window",
            "required_evidence": "wmctrl or xwininfo output identifies the Alice main window"
        }),
        serde_json::json!({
            "id": "activate-specific-alice-window",
            "required_evidence": "wmctrl -ia or xdotool windowfocus succeeds against the detected Alice window id"
        }),
        serde_json::json!({
            "id": "place-object",
            "decision": if placement_status != "passed" { "no_go" } else { "ready" },
            "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
            "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_OBJECT_PLACEMENT_HOOK,
                "inputs": ["open_project", "object_identifier", "evidence_dir"],
                "outputs": ["placement_artifact", "scene_or_project_diff"],
                "unsafe_until_available": placement_status != "passed"
            }
        }),
    ];
    if include_objects_first_actions {
        actions.push(serde_json::json!({
            "id": "transform-object",
            "decision": if !object_transform_proven { "no_go" } else { "ready" },
            "required_evidence": "artifact proves the named visible object was positioned or transformed after it was added",
            "missing_affordance_id": "deterministic-alice-object-transform-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_OBJECT_TRANSFORM_HOOK,
                "inputs": ["project_after_object_placement", "object_identifier", "position", "scale", "evidence_dir"],
                "outputs": ["transform_artifact", "transformed_project_artifact"],
                "unsafe_until_available": !object_transform_proven
            }
        }));
    }
    actions.extend([
        serde_json::json!({
            "id": "edit-procedure-or-code-block",
            "decision": if !edit_procedure_proven { "no_go" } else { "ready" },
            "required_evidence": if include_objects_first_actions { "artifact proves a movement command was added to the named procedure" } else { "artifact proves a procedure or code block was edited" },
            "missing_affordance_id": "deterministic-alice-procedure-edit-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_PROCEDURE_EDIT_HOOK,
                "inputs": if include_objects_first_actions { vec!["project_after_object_transform", "procedure_selector", "edit_spec", "evidence_dir"] } else { vec!["project_after_object_placement", "procedure_selector", "edit_spec", "evidence_dir"] },
                "outputs": ["edited_project_artifact", "procedure_or_code_diff"],
                "unsafe_until_available": !edit_procedure_proven
            }
        }),
        serde_json::json!({
            "id": "run-world",
            "decision": if !run_world_proven { "no_go" } else { "ready" },
            "required_evidence": "artifact proves the world run control or equivalent runtime entry point executed after the first-lesson edit",
            "missing_affordance_id": "deterministic-alice-world-run-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_WORLD_RUN_HOOK,
                "inputs": ["edited_project", "run_selector", "evidence_dir"],
                "outputs": ["run_artifact", "runtime_or_log_evidence"],
                "unsafe_until_available": !run_world_proven
            }
        }),
        serde_json::json!({
            "id": "save-project",
            "decision": if !save_project_proven { "no_go" } else { "ready" },
            "required_evidence": "saved .a3p project artifact exists, is non-empty, and can be read after the first-lesson run proof",
            "missing_affordance_id": "deterministic-alice-project-save-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_PROJECT_SAVE_HOOK,
                "inputs": ["edited_project", "save_selector", "evidence_dir"],
                "outputs": ["saved_project_artifact", "save_artifact"],
                "unsafe_until_available": !save_project_proven
            }
        }),
    ]);
    if include_objects_first_actions {
        actions.push(serde_json::json!({
            "id": "reopen-project",
            "decision": if !reopen_project_proven { "no_go" } else { "ready" },
            "required_evidence": "saved .a3p project artifact is reopened and persisted object, transform, procedure movement, and run state are verified",
            "missing_affordance_id": "deterministic-alice-project-reopen-affordance",
            "contract_required": {
                "candidate_backend": DEFAULT_PROJECT_REOPEN_HOOK,
                "inputs": ["saved_project", "reopen_selector", "evidence_dir"],
                "outputs": ["reopened_project_artifact", "reopen_artifact", "reopened_state_artifact"],
                "unsafe_until_available": !reopen_project_proven
            }
        }));
    }
    actions
}

fn ui_action_blocking_reason(
    placement_status: &str,
    object_transform_proven: bool,
    edit_procedure_proven: bool,
    run_world_proven: bool,
    save_project_proven: bool,
    reopen_project_proven: bool,
    include_objects_first_actions: bool,
) -> &'static str {
    if !include_objects_first_actions {
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
    } else if reopen_project_proven {
        "Deterministic object placement, transform, movement edit, world run, save, reopen, and persisted-state evidence exist."
    } else if save_project_proven {
        "Deterministic object placement, transform, movement edit, world run, and project save evidence exist, but project reopen evidence is not wired yet."
    } else if run_world_proven {
        "Deterministic object placement, transform, movement edit, and world run evidence exist, but project save and reopen evidence are not wired yet."
    } else if edit_procedure_proven {
        "Deterministic object placement, transform, and movement edit evidence exist, but world run, project save, and reopen evidence are not wired yet."
    } else if object_transform_proven {
        "Deterministic object placement and transform evidence exist, but movement edit, world run, project save, and reopen evidence are not wired yet."
    } else if placement_status == "passed" {
        "Deterministic object placement evidence exists, but object transform, movement edit, world run, project save, and reopen evidence are not wired yet."
    } else {
        "The harness can activate a detected Alice window when present, but deterministic object placement, transform, movement edit, world run, project save, and reopen evidence are not wired yet."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_run_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/ui-action-contract-tests")
            .join(format!("{nonce}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn artifact(path: &str) -> eatme_core::ArtifactInfo {
        eatme_core::ArtifactInfo {
            path: path.into(),
            size_bytes: 16,
            sha256: format!("{path}-sha"),
        }
    }

    fn object_placement_probe(status: &str) -> UiActionObjectPlacementProbe {
        UiActionObjectPlacementProbe {
            id: "alice-side-object-placement-command-hook".into(),
            action_id: "place-object".into(),
            status: status.into(),
            detail: "placement detail".into(),
            object_identifier: "alice-gallery://animals/bunny".into(),
            candidate_hook_path: DEFAULT_OBJECT_PLACEMENT_HOOK.into(),
            command: Some("tools/eatme-place-object --json".into()),
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            placement_artifact: (status == "passed").then(|| artifact("placement.json")),
            scene_or_project_diff: (status == "passed").then(|| artifact("scene.diff.json")),
            validation_errors: Vec::new(),
            missing_affordance: None,
        }
    }

    #[test]
    fn write_ui_action_contract_records_blocked_follow_on_actions_after_placement() {
        let run_dir = test_run_dir();
        let placement_probe = object_placement_probe("passed");

        let contract = write_ui_action_contract(
            &run_dir,
            true,
            true,
            true,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&placement_probe),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let json = fs::read_to_string(run_dir.join("ui-action-contract.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(
            contract.path,
            run_dir
                .join("ui-action-contract.json")
                .display()
                .to_string()
        );
        assert_eq!(value["schema_version"], "eatme.ui-action-contract/v1");
        assert_eq!(value["required_actions"][2]["decision"], "ready");
        assert_eq!(value["required_actions"][3]["decision"], "no_go");
        assert_eq!(
            value["candidate_affordance_probes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            value["action_precondition_probes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            value["blocking_reason"]
                .as_str()
                .unwrap()
                .contains("procedure/code-block edit")
        );

        let _ = fs::remove_dir_all(run_dir);
    }
}
