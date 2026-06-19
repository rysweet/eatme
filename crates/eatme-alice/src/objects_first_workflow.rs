use crate::launch::assertions::bool_assert;
use crate::launch_path_validation::canonical_artifact_under;
use crate::launch_reopen_project::UiActionReopenProjectProbe;
use anyhow::Result;
use eatme_core::{ArtifactInfo, AssertionResult};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const OBJECTS_FIRST_SCENARIO_ID: &str = "alice-objects-first-world";

#[derive(Debug, Deserialize)]
struct PersistedState {
    object: PersistedObject,
    procedure: PersistedProcedure,
    world_run: PersistedWorldRun,
}

#[derive(Debug, Deserialize)]
struct PersistedObject {
    name: String,
    visible: bool,
    transform: PersistedTransform,
}

#[derive(Debug, Deserialize)]
struct PersistedTransform {
    position: PersistedPosition,
    scale: f64,
}

#[derive(Debug, Deserialize)]
struct PersistedPosition {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Deserialize)]
struct PersistedProcedure {
    movement: PersistedMovement,
}

#[derive(Debug, Deserialize)]
struct PersistedMovement {
    object: String,
    method: String,
}

#[derive(Debug, Deserialize)]
struct PersistedWorldRun {
    status: String,
}

pub(crate) fn is_objects_first_scenario(id: &str) -> bool {
    id == OBJECTS_FIRST_SCENARIO_ID
}

pub(crate) fn create_or_open_project_assertion(
    process_started: bool,
    starter_project: &Path,
) -> AssertionResult {
    bool_assert(
        process_started && starter_project.is_file(),
        format!(
            "Alice opened or created a project from {}",
            starter_project.display()
        ),
    )
}

pub(crate) fn persisted_state_assertion(
    run_dir: &Path,
    reopen_probe: &UiActionReopenProjectProbe,
) -> AssertionResult {
    match persisted_state_errors(run_dir, reopen_probe) {
        Ok(()) => AssertionResult::pass(
            "reopened project kept bunny visible, transformed, movable in scene.myFirstMethod, and runnable",
        ),
        Err(error) => AssertionResult::fail(error),
    }
}

pub(crate) fn record_evidence_summary(
    run_dir: &Path,
    all_required_proof: bool,
    persisted_state: &AssertionResult,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("objects-first-evidence.json");
    let json = serde_json::json!({
        "schema_version": "eatme.alice-objects-first-evidence/v1",
        "scenario_id": OBJECTS_FIRST_SCENARIO_ID,
        "complete": all_required_proof && persisted_state.passed,
        "major_steps": [
            "create_or_open_project",
            "add_visible_object",
            "position_and_transform_object",
            "edit_movement_procedure",
            "run_world",
            "save_project",
            "reopen_project",
            "verify_persisted_state"
        ],
        "persisted_state": {
            "passed": persisted_state.passed,
            "detail": persisted_state.detail
        }
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    let mut artifact = crate::launch_artifacts::artifact_info(&path)?;
    artifact.path = "objects-first-evidence.json".into();
    Ok(artifact)
}

fn persisted_state_errors(
    run_dir: &Path,
    reopen_probe: &UiActionReopenProjectProbe,
) -> std::result::Result<(), String> {
    if !reopen_probe.proves_reopen() {
        return Err(
            "project reopen proof is required before persisted state can be trusted".into(),
        );
    }
    let artifact = reopen_probe
        .reopened_state_artifact
        .as_ref()
        .ok_or_else(|| {
            "project reopen proof did not include persisted state artifact".to_string()
        })?;
    let state_path = artifact_path(run_dir, artifact)?;
    let project_reopen_dir = run_dir.join("project-reopen");
    canonical_artifact_under(
        &project_reopen_dir,
        &state_path,
        "reopened_state_artifact",
        "project-reopen evidence dir",
    )?;
    let state: PersistedState = serde_json::from_str(
        &fs::read_to_string(&state_path)
            .map_err(|error| format!("reading persisted state failed: {error:#}"))?,
    )
    .map_err(|error| format!("persisted state JSON is malformed: {error}"))?;
    let mut errors = Vec::new();
    if state.object.name != "bunny" {
        errors.push(format!(
            "object.name must be bunny, got {}",
            state.object.name
        ));
    }
    if !state.object.visible {
        errors.push("object.visible must be true".into());
    }
    if (state.object.transform.position.x - 1.5).abs() > f64::EPSILON {
        errors.push("object transform position.x must be 1.5".into());
    }
    if (state.object.transform.position.y - 0.0).abs() > f64::EPSILON {
        errors.push("object transform position.y must be 0.0".into());
    }
    if (state.object.transform.position.z - -2.0).abs() > f64::EPSILON {
        errors.push("object transform position.z must be -2.0".into());
    }
    if (state.object.transform.scale - 1.25).abs() > f64::EPSILON {
        errors.push("object transform scale must be 1.25".into());
    }
    if state.procedure.movement.object != "bunny" {
        errors.push(format!(
            "procedure movement object must be bunny, got {}",
            state.procedure.movement.object
        ));
    }
    if state.procedure.movement.method != "move" {
        errors.push(format!(
            "procedure movement method must be move, got {}",
            state.procedure.movement.method
        ));
    }
    if state.world_run.status != "ran" {
        errors.push(format!(
            "world_run.status must be ran, got {}",
            state.world_run.status
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn artifact_path(run_dir: &Path, artifact: &ArtifactInfo) -> std::result::Result<PathBuf, String> {
    let path = Path::new(&artifact.path);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(run_dir.join(path))
    }
}
