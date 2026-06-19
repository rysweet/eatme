use crate::launch_artifacts::artifact_info;
use crate::launch_edit_procedure::UiActionEditProcedureProbe;
use crate::launch_object_placement::UiActionObjectPlacementProbe;
use crate::launch_object_transform::UiActionObjectTransformProbe;
use crate::launch_reopen_project::UiActionReopenProjectProbe;
use crate::launch_run_world::UiActionRunWorldProbe;
use crate::launch_save_project::UiActionSaveProjectProbe;
use anyhow::Result;
use eatme_core::{ArtifactInfo, AssertionResult};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const CONTRACT_SCHEMA: &str = "eatme.alice-objects-first-full-path/v1";
const SCENARIO_PATH: &str = "assets/scenarios/eatme/alice-objects-first-full-path.yaml";

#[derive(Debug)]
pub(crate) struct ObjectsFirstFullPathEvidence {
    pub assertions: BTreeMap<String, AssertionResult>,
    pub failure_category: Option<String>,
    pub command: Value,
    pub scenario: Value,
    pub evidence: Value,
    pub persistence_assertions: Value,
}

pub(crate) struct FullPathVisualEvidence<'a> {
    pub screenshot: Option<&'a ArtifactInfo>,
    pub screenshot_error: Option<&'a str>,
    pub ui_action_contract: Option<&'a ArtifactInfo>,
}

pub(crate) struct FullPathPhaseProbes<'a> {
    pub object_placement: &'a UiActionObjectPlacementProbe,
    pub object_transform: &'a UiActionObjectTransformProbe,
    pub edit_procedure: &'a UiActionEditProcedureProbe,
    pub run_world: &'a UiActionRunWorldProbe,
    pub save_project: &'a UiActionSaveProjectProbe,
    pub reopen_project: &'a UiActionReopenProjectProbe,
}

pub(crate) fn write_objects_first_full_path_contract(
    run_dir: &Path,
    visual: FullPathVisualEvidence<'_>,
    probes: FullPathPhaseProbes<'_>,
) -> Result<ObjectsFirstFullPathEvidence> {
    let object_placement_probe = probes.object_placement;
    let object_transform_probe = probes.object_transform;
    let edit_procedure_probe = probes.edit_procedure;
    let run_world_probe = probes.run_world;
    let save_project_probe = probes.save_project;
    let reopen_project_probe = probes.reopen_project;
    let object_id = object_identity(object_transform_probe, edit_procedure_probe);
    let movement = movement_from_edit(edit_procedure_probe);
    let before_state = read_artifact_json(run_dir, save_project_probe.save_artifact.as_ref());
    let after_state = read_artifact_json(
        run_dir,
        reopen_project_probe.reopened_state_artifact.as_ref(),
    );
    let persistence = persistence_assertions(
        &before_state.value,
        &after_state.value,
        run_world_probe,
        save_project_probe,
        reopen_project_probe,
    );
    let persistence_json = persistence
        .iter()
        .map(PersistenceAssertion::as_contract_json)
        .collect::<Vec<_>>();
    let persistence_manifest_json = persistence
        .iter()
        .map(|assertion| {
            (
                assertion.id.to_string(),
                json!({
                    "passed": assertion.passed,
                    "detail": assertion.detail,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    let phases = json!({
        "create-or-open-project": {
            "status": "passed",
            "starter_project": "core/resources/target/distribution/application/starter-projects/africa.a3p"
        },
        "place-object": {
            "status": phase_status(object_placement_probe.proves_placement()),
            "object_identifier": object_placement_probe.object_identifier,
            "object_id": object_id
        },
        "transform-object": {
            "status": phase_status(object_transform_probe.proves_transform()),
            "object_id": object_transform_probe.object_id,
            "transform": object_transform_probe.transform
        },
        "edit-movement-procedure": {
            "status": phase_status(edit_procedure_probe.proves_edit() && movement.is_some()),
            "procedure_selector": edit_procedure_probe.procedure_selector,
            "movement": movement
        },
        "run-world": {
            "status": phase_status(run_world_probe.proves_run()),
            "run_selector": run_world_probe.run_selector
        },
        "save-project": {
            "status": phase_status(save_project_probe.proves_save()),
            "save_selector": save_project_probe.save_selector
        },
        "reopen-project": {
            "status": phase_status(reopen_project_probe.proves_reopen()),
            "reopen_selector": reopen_project_probe.reopen_selector,
            "source_saved_project_artifact": reopen_project_probe.source_saved_project_artifact
        },
        "verify-persistence": {
            "status": phase_status(persistence.iter().all(|assertion| assertion.passed))
        }
    });

    let contract = json!({
        "schema_version": CONTRACT_SCHEMA,
        "status": phase_status(all_phases_passed(
            &probes,
            movement.as_ref(),
            &persistence,
        )),
        "phases": phases,
        "project_state": {
            "before_reopen": before_state.value,
            "before_reopen_read_error": before_state.error,
            "after_reopen": after_state.value,
            "after_reopen_read_error": after_state.error
        },
        "persistence_assertions": persistence_json
    });
    let contract_path = run_dir.join("objects-first-full-path-contract.json");
    fs::write(&contract_path, serde_json::to_vec_pretty(&contract)?)?;
    let contract_artifact = artifact_info(&contract_path)?;

    let phase_artifacts = phase_artifacts(visual.ui_action_contract, &contract_artifact, &probes);
    let evidence = json!({
        "screenshot": screenshot_status(visual.screenshot, visual.screenshot_error),
        "phase_artifacts": phase_artifacts
    });
    let assertions = manifest_assertions(
        object_transform_probe,
        edit_procedure_probe,
        movement.as_ref(),
        reopen_project_probe,
        &persistence,
    );
    let failure_category = failure_category(&probes, movement.as_ref(), &persistence);

    Ok(ObjectsFirstFullPathEvidence {
        assertions,
        failure_category,
        command: json!({"argv": ["eatme", "alice", "objects-first-full-path"]}),
        scenario: json!({"id": "alice-objects-first-full-path", "path": SCENARIO_PATH}),
        evidence,
        persistence_assertions: Value::Object(persistence_manifest_json),
    })
}

fn all_phases_passed(
    probes: &FullPathPhaseProbes<'_>,
    movement: Option<&Value>,
    persistence: &[PersistenceAssertion],
) -> bool {
    probes.object_placement.proves_placement()
        && probes.object_transform.proves_transform()
        && probes.edit_procedure.proves_edit()
        && movement.is_some()
        && probes.run_world.proves_run()
        && probes.save_project.proves_save()
        && probes.reopen_project.proves_reopen()
        && persistence.iter().all(|assertion| assertion.passed)
}

fn phase_status(passed: bool) -> &'static str {
    if passed { "passed" } else { "failed" }
}

fn object_identity(
    object_transform_probe: &UiActionObjectTransformProbe,
    edit_procedure_probe: &UiActionEditProcedureProbe,
) -> String {
    if !object_transform_probe.object_id.is_empty() {
        return object_transform_probe.object_id.clone();
    }
    movement_from_edit(edit_procedure_probe)
        .and_then(|movement| {
            movement
                .get("object_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default()
}

fn movement_from_edit(edit_procedure_probe: &UiActionEditProcedureProbe) -> Option<Value> {
    serde_json::from_str::<Value>(&edit_procedure_probe.stdout)
        .ok()
        .and_then(|value| {
            let object_id = value.get("object_id").and_then(Value::as_str)?;
            let movement = value.get("movement")?;
            if movement.get("operation").and_then(Value::as_str) != Some("move") {
                return None;
            }
            if movement
                .get("distance_meters")
                .and_then(Value::as_f64)
                .is_none_or(|distance| distance <= 0.0)
            {
                return None;
            }
            let mut movement = movement.clone();
            if let Value::Object(ref mut object) = movement {
                object.insert("object_id".into(), Value::String(object_id.to_string()));
            }
            Some(movement)
        })
}

struct ArtifactJson {
    value: Value,
    error: Option<String>,
}

fn read_artifact_json(run_dir: &Path, artifact: Option<&ArtifactInfo>) -> ArtifactJson {
    let Some(artifact) = artifact else {
        return ArtifactJson {
            value: Value::Null,
            error: Some("artifact missing".into()),
        };
    };
    let path = Path::new(&artifact.path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        run_dir.join(path)
    };
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(value) => ArtifactJson { value, error: None },
            Err(error) => ArtifactJson {
                value: Value::Null,
                error: Some(format!("{} is not JSON: {error}", path.display())),
            },
        },
        Err(error) => ArtifactJson {
            value: Value::Null,
            error: Some(format!("{} is not readable: {error}", path.display())),
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct PersistenceAssertion {
    id: &'static str,
    passed: bool,
    detail: &'static str,
}

impl PersistenceAssertion {
    fn as_contract_json(&self) -> Value {
        json!({
            "id": self.id,
            "status": phase_status(self.passed),
            "detail": self.detail,
        })
    }
}

fn persistence_assertions(
    before: &Value,
    after: &Value,
    run_world_probe: &UiActionRunWorldProbe,
    save_project_probe: &UiActionSaveProjectProbe,
    reopen_project_probe: &UiActionReopenProjectProbe,
) -> Vec<PersistenceAssertion> {
    vec![
        PersistenceAssertion {
            id: "saved_project_artifact_exists",
            passed: save_project_probe.proves_save(),
            detail: "save hook returned a non-empty saved project artifact",
        },
        PersistenceAssertion {
            id: "reopened_same_saved_project",
            passed: reopen_project_probe.proves_reopen(),
            detail: "reopen hook loaded the saved artifact from the same run",
        },
        PersistenceAssertion {
            id: "object_identity_persisted",
            passed: matching_non_null(before, after, "object_id"),
            detail: "object identity matched before save and after reopen",
        },
        PersistenceAssertion {
            id: "object_transform_persisted",
            passed: both_non_null(before, after, "transform"),
            detail: "object transform matched before save and after reopen",
        },
        PersistenceAssertion {
            id: "movement_procedure_persisted",
            passed: matching_non_null(before, after, "procedure_selector")
                && both_non_null(before, after, "movement"),
            detail: "movement procedure selector and movement semantics persisted",
        },
        PersistenceAssertion {
            id: "world_run_completed_before_save",
            passed: run_world_probe.proves_run(),
            detail: "world run proof was captured before save proof",
        },
    ]
}

fn both_non_null(left: &Value, right: &Value, field: &str) -> bool {
    !left.get(field).unwrap_or(&Value::Null).is_null()
        && !right.get(field).unwrap_or(&Value::Null).is_null()
}

fn matching_non_null(left: &Value, right: &Value, field: &str) -> bool {
    let left = left.get(field).unwrap_or(&Value::Null);
    let right = right.get(field).unwrap_or(&Value::Null);
    !left.is_null() && left == right
}

fn phase_artifacts(
    ui_action_contract: Option<&ArtifactInfo>,
    contract_artifact: &ArtifactInfo,
    probes: &FullPathPhaseProbes<'_>,
) -> Value {
    json!({
        "ui_action_contract": artifact_json(ui_action_contract),
        "objects_first_full_path_contract": artifact_json(Some(contract_artifact)),
        "object_placement": artifact_json(probes.object_placement.placement_artifact.as_ref()),
        "object_transform": artifact_json(probes.object_transform.transform_artifact.as_ref()),
        "procedure_edit": artifact_json(probes.edit_procedure.procedure_or_code_diff.as_ref()),
        "world_run": artifact_json(probes.run_world.run_artifact.as_ref()),
        "project_state_before_reopen": artifact_json(probes.save_project.save_artifact.as_ref()),
        "project_reopen": artifact_json(probes.reopen_project.reopen_artifact.as_ref()),
        "project_state_after_reopen": artifact_json(probes.reopen_project.reopened_state_artifact.as_ref())
    })
}

fn artifact_json(artifact: Option<&ArtifactInfo>) -> Value {
    match artifact {
        Some(artifact) => json!(artifact),
        None => Value::Null,
    }
}

fn screenshot_status(screenshot: Option<&ArtifactInfo>, screenshot_error: Option<&str>) -> Value {
    match screenshot {
        Some(artifact) if artifact.size_bytes > 0 => {
            json!({"status": "captured", "artifact": artifact})
        }
        _ => json!({"status": "unavailable", "error": screenshot_error}),
    }
}

fn manifest_assertions(
    object_transform_probe: &UiActionObjectTransformProbe,
    edit_procedure_probe: &UiActionEditProcedureProbe,
    movement: Option<&Value>,
    reopen_project_probe: &UiActionReopenProjectProbe,
    persistence: &[PersistenceAssertion],
) -> BTreeMap<String, AssertionResult> {
    let mut assertions = BTreeMap::new();
    assertions.insert(
        "transform_object".into(),
        assertion_from_bool(
            object_transform_probe.proves_transform(),
            object_transform_probe.detail.clone(),
        ),
    );
    assertions.insert(
        "edit_movement_procedure".into(),
        assertion_from_bool(
            edit_procedure_probe.proves_edit() && movement.is_some(),
            edit_procedure_probe.detail.clone(),
        ),
    );
    assertions.insert(
        "reopen_project".into(),
        assertion_from_bool(
            reopen_project_probe.proves_reopen(),
            reopen_project_probe.detail.clone(),
        ),
    );
    for assertion in persistence {
        assertions.insert(
            assertion.id.into(),
            assertion_from_bool(assertion.passed, assertion.detail),
        );
    }
    assertions
}

fn assertion_from_bool(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}

fn failure_category(
    probes: &FullPathPhaseProbes<'_>,
    movement: Option<&Value>,
    persistence: &[PersistenceAssertion],
) -> Option<String> {
    if !probes.object_placement.proves_placement() {
        Some("object_placement_missing".into())
    } else if !probes.object_transform.proves_transform() {
        Some("object_transform_missing".into())
    } else if !probes.edit_procedure.proves_edit() || movement.is_none() {
        Some("movement_procedure_missing".into())
    } else if !probes.run_world.proves_run() {
        Some("world_run_missing".into())
    } else if !probes.save_project.proves_save() {
        Some("project_save_missing".into())
    } else if !probes.reopen_project.proves_reopen() {
        Some("project_reopen_missing".into())
    } else if persistence
        .iter()
        .any(|assertion| assertion.id == "object_identity_persisted" && !assertion.passed)
    {
        Some("persistence_assertion_failed".into())
    } else {
        None
    }
}
