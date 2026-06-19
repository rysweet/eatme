use crate::launch_object_placement::UiActionObjectPlacementProbe;
use crate::launch_path_validation::artifact_info_under;
use crate::launch_ui_actions::UiActionMissingAffordance;
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(crate) const DEFAULT_OBJECT_TRANSFORM_HOOK: &str = "tools/eatme-transform-object";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionObjectTransformProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub object_id: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub transform_artifact: Option<ArtifactInfo>,
    pub transformed_project_artifact: Option<ArtifactInfo>,
    pub transform: Option<Value>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
}

impl UiActionObjectTransformProbe {
    pub fn proves_transform(&self) -> bool {
        self.status == "passed"
            && !self.object_id.is_empty()
            && self.transform_artifact.is_some()
            && self.transformed_project_artifact.is_some()
            && self.transform.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ObjectTransformHookResult {
    schema_version: String,
    status: String,
    object_id: String,
    transform_artifact: String,
    transformed_project_artifact: String,
    #[serde(default)]
    transform: Option<Value>,
}

pub(crate) fn probe_object_transform_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    object_placement_probe: &UiActionObjectPlacementProbe,
    starter_project: &Path,
    display: &str,
) -> UiActionObjectTransformProbe {
    let hook_path = alice_home.join(DEFAULT_OBJECT_TRANSFORM_HOOK);
    let evidence_dir = run_dir.join("object-transform");
    let placed_project = run_dir.join("object-placement").join("placed-project.a3p");
    let project = if placed_project.is_file() {
        placed_project.as_path()
    } else {
        starter_project
    };

    if !object_placement_probe.proves_placement() {
        return blocked_transform_probe(
            &hook_path,
            "blocked: object placement proof is required before object transform would be safe",
            Some(missing_object_transform_affordance()),
        );
    }
    if !hook_path.is_file() {
        return blocked_transform_probe(
            &hook_path,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_OBJECT_TRANSFORM_HOOK}; object transform remains unproven"
            ),
            Some(missing_object_transform_affordance()),
        );
    }
    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_transform_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            String::new(),
            None,
            vec![format!(
                "creating object transform evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--project".to_string(),
                project.display().to_string(),
                "--object-identifier".to_string(),
                object_placement_probe.object_identifier.clone(),
                "--evidence-dir".to_string(),
                evidence_dir.display().to_string(),
                "--json".to_string(),
            ])
            .cwd(alice_home)
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(30)),
    );

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return failed_transform_probe(
                &hook_path,
                Some(format!(
                    "{} --project {} --object-identifier {} --evidence-dir {} --json",
                    hook_path.display(),
                    project.display(),
                    object_placement_probe.object_identifier,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                String::new(),
                None,
                vec![format!("object transform hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_transform_probe(
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            String::new(),
            None,
            vec!["object transform hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ObjectTransformHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_transform_probe(
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                String::new(),
                None,
                vec![format!(
                    "object transform hook stdout is not valid transform JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_transform_hook_result(&result);
    let transform_artifact = artifact_info_under(
        &evidence_dir,
        &result.transform_artifact,
        "transform_artifact",
        "object-transform evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let transform = result.transform.or_else(|| {
        transform_artifact.as_ref().and_then(|artifact| {
            fs::read_to_string(&artifact.path)
                .ok()
                .and_then(|content| serde_json::from_str::<Value>(&content).ok())
                .and_then(|value| value.get("transform").cloned().or(Some(value)))
        })
    });
    let transformed_project_artifact = artifact_info_under(
        &evidence_dir,
        &result.transformed_project_artifact,
        "transformed_project_artifact",
        "object-transform evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    if transform_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("transform_artifact must be non-empty".into());
    }
    if transformed_project_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("transformed_project_artifact must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side object transform hook returned transform and transformed project evidence for {}",
            result.object_id
        )
    } else {
        format!(
            "object transform hook ran but did not prove transform: {}",
            validation_errors.join("; ")
        )
    };

    UiActionObjectTransformProbe {
        id: "alice-side-object-transform-command-hook".into(),
        action_id: "transform-object".into(),
        status: status.into(),
        detail,
        object_id: result.object_id,
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        transform_artifact,
        transformed_project_artifact,
        transform,
        validation_errors,
        missing_affordance: None,
    }
}

fn validate_transform_hook_result(result: &ObjectTransformHookResult) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-object-transform-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-object-transform-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "transformed" {
        errors.push(format!(
            "status must be transformed, got {:?}",
            result.status
        ));
    }
    if result.object_id.is_empty() {
        errors.push("object_id must not be empty".into());
    }
    if result.transform_artifact.is_empty() {
        errors.push("transform_artifact must not be empty".into());
    }
    if result.transformed_project_artifact.is_empty() {
        errors.push("transformed_project_artifact must not be empty".into());
    }
    errors
}

fn missing_object_transform_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-object-transform-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given a placed Alice object, deterministically transform it and return object identity plus transform evidence.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_OBJECT_TRANSFORM_HOOK}, accessibility target, or scene graph transform hook currently returns object transform proof."),
        next_implementation: "Add one stable affordance: either the Alice-side transform-object command hook defined by this contract, or UI automation with scene-graph verification of the transformed object.".into(),
    }
}

fn blocked_transform_probe(
    hook_path: &Path,
    detail: &str,
    missing_affordance: Option<UiActionMissingAffordance>,
) -> UiActionObjectTransformProbe {
    UiActionObjectTransformProbe {
        id: "alice-side-object-transform-command-hook".into(),
        action_id: "transform-object".into(),
        status: "blocked".into(),
        detail: detail.into(),
        object_id: String::new(),
        candidate_hook_path: hook_path.display().to_string(),
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        transform_artifact: None,
        transformed_project_artifact: None,
        transform: None,
        validation_errors: Vec::new(),
        missing_affordance,
    }
}

#[allow(clippy::too_many_arguments)]
fn failed_transform_probe(
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    object_id: String,
    transform: Option<Value>,
    validation_errors: Vec<String>,
) -> UiActionObjectTransformProbe {
    UiActionObjectTransformProbe {
        id: "alice-side-object-transform-command-hook".into(),
        action_id: "transform-object".into(),
        status: "failed".into(),
        detail: format!(
            "object transform hook did not prove transform: {}",
            validation_errors.join("; ")
        ),
        object_id,
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        transform_artifact: None,
        transformed_project_artifact: None,
        transform,
        validation_errors,
        missing_affordance: None,
    }
}
