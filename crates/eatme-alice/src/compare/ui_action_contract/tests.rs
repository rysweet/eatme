use super::*;

#[test]
fn requires_run_world_no_go_after_edit_proof() {
    let mut issues = Vec::new();

    inspect_ui_action_contract(
        "modernized",
        &contract_after_edit_without_run_no_go(),
        &mut issues,
    );

    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("passed run-world proof or a no-go precondition")),
        "issues should name missing run-world proof boundary: {issues:?}"
    );
}

#[test]
fn requires_save_project_no_go_after_run_world_proof() {
    let mut issues = Vec::new();

    inspect_ui_action_contract(
        "modernized",
        &contract_after_run_without_save_no_go(),
        &mut issues,
    );

    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("passed save-project proof or a no-go precondition")),
        "issues should name missing save-project proof boundary: {issues:?}"
    );
}

fn contract_after_edit_without_run_no_go() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "world run remains blocked",
        "preflight_evidence": {
            "specific_alice_window_detected": true,
            "visual_evidence_captured": true,
            "log_captured": true
        },
        "executed_action_probes": [{
            "id": "activate-specific-alice-window",
            "status": "passed"
        }],
        "candidate_affordance_probes": [
            {
                "id": "alice-side-object-placement-command-hook",
                "action_id": "place-object",
                "status": "passed",
                "object_identifier": "alice-gallery://animals/bunny",
                "candidate_hook_path": "/alice/tools/eatme-place-object",
                "placement_artifact": {"path": "object-placement/placed-project.a3p", "size_bytes": 2},
                "scene_or_project_diff": {"path": "object-placement/scene.diff.json", "size_bytes": 2}
            },
            {
                "id": "alice-side-procedure-edit-command-hook",
                "action_id": "edit-procedure-or-code-block",
                "status": "passed",
                "procedure_selector": "scene.eatmeFirstLessonStep",
                "candidate_hook_path": "/alice/tools/eatme-edit-procedure",
                "edited_project_artifact": {"path": "procedure-edit/edited-project.a3p", "size_bytes": 2},
                "procedure_or_code_diff": {"path": "procedure-edit/procedure.diff.json", "size_bytes": 2}
            }
        ],
        "required_actions": []
    })
}

fn contract_after_run_without_save_no_go() -> serde_json::Value {
    let mut contract = contract_after_edit_without_run_no_go();
    contract["candidate_affordance_probes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "alice-side-world-run-command-hook",
            "action_id": "run-world",
            "status": "passed",
            "run_selector": "scene.eatmeFirstLessonStep",
            "candidate_hook_path": "/alice/tools/eatme-run-world",
            "run_artifact": {"path": "world-run/world-run.json", "size_bytes": 2},
            "runtime_or_log_evidence": {"path": "world-run/runtime.log", "size_bytes": 2}
        }));
    contract
}
