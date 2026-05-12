use std::{fs, path::Path};

use crate::support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture,
    overwrite_modernized_first_lesson_next_action, write_manifest,
    write_modernized_run_window_evidence_file,
};

pub(super) fn complete_desktop_evidence_manifest() -> std::path::PathBuf {
    write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[
            "place_object_ui_action",
            "edit_procedure_ui_action",
            "run_world_ui_action",
            "save_project_ui_action",
        ],
    })
}

pub(super) fn write_complete_next_action_evidence(manifest_path: &Path, status: &str) {
    write_modernized_run_window_evidence_file(
        manifest_path,
        "desktop-run-pixel-boundary.json",
        r#"{"schema_version":"eatme.alice-desktop-run-pixel-boundary/v1","status":"observed","reason":"Run view attachment and pixel boundary were observed for this complete-evidence fixture."}"#,
    );
    write_modernized_run_window_evidence_file(
        manifest_path,
        "save-project-proof.json",
        r#"{"status":"present"}"#,
    );
    write_modernized_run_window_evidence_file(
        manifest_path,
        "select-project-proof.json",
        r#"{"status":"present"}"#,
    );
    overwrite_modernized_first_lesson_next_action(
        manifest_path,
        &format!(
            r#"{{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"{status}",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["save-project"],
  "save_project_proof_artifact":{{"status":"present","artifact":{{"path":"run-window-evidence/save-project-proof.json"}}}},
  "select_project_proof_artifact":{{"status":"present","artifact":{{"path":"run-window-evidence/select-project-proof.json"}}}},
  "evidence_boundaries":[
    {{"id":"select_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Select Project scenario evidence is present.","claim":"The Select Project boundary has auditable scenario evidence.","does_not_prove":["first-lesson completion"]}},
    {{"id":"procedure_edit","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Procedure/edit scenario evidence is present.","claim":"The procedure/edit boundary has auditable scenario evidence.","does_not_prove":["first-lesson completion"]}},
    {{"id":"save_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Save scenario evidence is present.","claim":"Save action evidence is present for this scenario boundary.","does_not_prove":["Save completion","first-lesson completion"]}},
    {{"id":"visible_rendering","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Visible rendering scenario evidence is present.","claim":"Visible rendering was observed for this scenario boundary.","does_not_prove":["visible rendering correctness"]}},
    {{"id":"grading","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Grading scenario evidence is present.","claim":"The grading boundary has auditable scenario evidence.","does_not_prove":["grading"]}},
    {{"id":"creative_assessment","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Creative assessment scenario evidence is present.","claim":"When creative assessment evidence is missing, limited, or unavailable, the report can surface available evidence and suggest bounded next steps for the learner's creative work in this scenario. It does not grade creativity, judge quality, or mark the lesson complete.","does_not_prove":["creative assessment"]}},
    {{"id":"first_lesson_completion","status":"present","source":"rabbithole","metadata_state":"observed","detail":"First-lesson completion scenario evidence is present.","claim":"The first-lesson completion boundary has auditable scenario evidence.","does_not_prove":["first-lesson completion"]}}
  ],
  "doesNotClaim":["full Alice UI automation","visible rendering correctness","Save completion","grading","creative assessment","first-lesson completion"]
}}"#
        ),
    );
}

pub(super) fn rewrite_target_failure_categories(
    manifest_path: &Path,
    failure_category: serde_json::Value,
    status: &str,
) {
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    for role in ["baseline", "modernized"] {
        manifest["targets"][role]["status"] = serde_json::json!(status);
        manifest["targets"][role]["failure_category"] = failure_category.clone();
        manifest["targets"][role]["launch_manifest"]["failure_category"] = failure_category.clone();
    }
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

pub(super) fn overwrite_ui_action_contracts(manifest_path: &Path, contract: serde_json::Value) {
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    for role in ["baseline", "modernized"] {
        let path = manifest["targets"][role]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap();
        fs::write(path, serde_json::to_string_pretty(&contract).unwrap()).unwrap();
    }
}

pub(super) fn ready_ui_action_contract() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "ready",
        "blocking_reason": "All deterministic first-lesson action proof artifacts are present for this scenario.",
        "preflight_evidence": {
            "specific_alice_window_detected": true,
            "visual_evidence_captured": true,
            "log_captured": true
        },
        "executed_action_probes": [
            passed_probe("verify-specific-alice-window"),
            passed_probe("activate-specific-alice-window"),
            passed_probe("dispatch-save-project-shortcut"),
            passed_probe("dispatch-run-world-shortcut"),
            passed_probe("observe-run-window-after-shortcut")
        ],
        "candidate_affordance_probes": [
            {
                "id": "alice-side-object-placement-command-hook",
                "action_id": "place-object",
                "status": "passed",
                "object_identifier": "alice-gallery://animals/bunny",
                "candidate_hook_path": "/alice/tools/eatme-place-object",
                "placement_artifact": {"size_bytes": 1},
                "scene_or_project_diff": {"size_bytes": 1}
            },
            {
                "id": "alice-side-procedure-edit-command-hook",
                "action_id": "edit-procedure-or-code-block",
                "status": "passed",
                "procedure_selector": "scene.eatmeFirstLesson",
                "candidate_hook_path": "/alice/tools/eatme-edit-procedure",
                "edited_project_artifact": {"size_bytes": 1},
                "procedure_or_code_diff": {"size_bytes": 1}
            },
            {
                "id": "alice-side-world-run-command-hook",
                "action_id": "run-world",
                "status": "passed",
                "run_selector": "scene.eatmeFirstLesson",
                "candidate_hook_path": "/alice/tools/eatme-run-world",
                "run_artifact": {"size_bytes": 1},
                "runtime_or_log_evidence": {"size_bytes": 1}
            },
            {
                "id": "alice-side-project-save-command-hook",
                "action_id": "save-project",
                "status": "passed",
                "save_selector": "scene.eatmeFirstLesson",
                "candidate_hook_path": "/alice/tools/eatme-save-project",
                "saved_project_artifact": {"size_bytes": 1},
                "save_artifact": {"size_bytes": 1},
                "validation_errors": []
            }
        ],
        "required_actions": [
            {"id": "verify-specific-alice-window"},
            {"id": "activate-specific-alice-window"},
            {"id": "place-object"},
            {"id": "edit-procedure-or-code-block"},
            {"id": "run-world"},
            {"id": "save-project"}
        ]
    })
}

fn passed_probe(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "status": "passed",
        "detail": format!("{id} passed with deterministic proof"),
        "command": format!("test-{id}"),
        "exit_status": 0
    })
}
