use crate::launch_artifacts::artifact_info;
use anyhow::Result;
use eatme_core::{ArtifactInfo, AssertionResult};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn record_ui_action_blockers(
    assertions: &mut BTreeMap<String, AssertionResult>,
    artifact: &ArtifactInfo,
) {
    assertions.insert(
        "place_object_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can add/place an object yet",
        ),
    );
    assertions.insert(
        "edit_procedure_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can edit a procedure or code block yet",
        ),
    );
    assertions.insert(
        "run_world_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can run the world yet",
        ),
    );
    assertions.insert(
        "save_project_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can save the project yet",
        ),
    );
    record_ui_action_artifact(assertions, artifact);
}

pub fn record_preflight_ui_action_blockers(assertions: &mut BTreeMap<String, AssertionResult>) {
    assertions.insert(
        "specific_alice_window_detected".into(),
        AssertionResult::fail("preflight blocked before an Alice window could be verified"),
    );
    assertions.insert(
        "place_object_ui_action".into(),
        AssertionResult::fail("preflight blocked before add/place object automation could run"),
    );
    assertions.insert(
        "edit_procedure_ui_action".into(),
        AssertionResult::fail("preflight blocked before procedure/code-block editing could run"),
    );
    assertions.insert(
        "run_world_ui_action".into(),
        AssertionResult::fail("preflight blocked before world execution could run"),
    );
    assertions.insert(
        "save_project_ui_action".into(),
        AssertionResult::fail("preflight blocked before project save could run"),
    );
}

pub fn record_ui_action_artifact(
    assertions: &mut BTreeMap<String, AssertionResult>,
    artifact: &ArtifactInfo,
) {
    assertions.insert(
        "ui_action_artifact_captured".into(),
        bool_assert(
            artifact.size_bytes > 0,
            "ui action contract artifact exists and is non-empty",
        ),
    );
}

pub fn write_ui_action_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("ui-action-contract.json");
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "No supported deterministic Alice desktop automation is wired for object placement, procedure editing, world run, or project save yet.",
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "required_actions": [
            {
                "id": "verify-specific-alice-window",
                "required_evidence": "wmctrl output identifies an Alice Stage IDE window"
            },
            {
                "id": "place-object",
                "required_evidence": "artifact proves an object was added to the scene and placed"
            },
            {
                "id": "edit-procedure-or-code-block",
                "required_evidence": "artifact proves a procedure or code block was edited"
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

fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}
