use crate::launch_path_validation::artifact_info_under;
use crate::launch_ui_actions::UiActionMissingAffordance;
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(crate) const DEFAULT_OBJECT_PLACEMENT_HOOK: &str = "tools/eatme-place-object";
const DEFAULT_OBJECT_IDENTIFIER: &str = "alice-gallery://animals/bunny";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionObjectPlacementProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub object_identifier: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub placement_artifact: Option<ArtifactInfo>,
    pub scene_or_project_diff: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
}

impl UiActionObjectPlacementProbe {
    pub fn proves_placement(&self) -> bool {
        self.status == "passed"
            && self.placement_artifact.is_some()
            && self.scene_or_project_diff.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ObjectPlacementHookResult {
    schema_version: String,
    status: String,
    object_identifier: String,
    placement_artifact: String,
    scene_or_project_diff: String,
}

pub(crate) fn default_object_identifier() -> &'static str {
    DEFAULT_OBJECT_IDENTIFIER
}

pub(crate) fn missing_object_placement_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-object-gallery-placement-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given an open Alice starter project and a named object identifier, deterministically add that object to the scene without coordinate guessing.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_OBJECT_PLACEMENT_HOOK}, accessibility target, stable menu action, or scene-graph verification hook currently accepts a named object identifier and returns proof of placement."),
        next_implementation: "Add one stable affordance: either the Alice-side object placement command hook defined by this contract, or a UI automation contract with a named gallery selector plus scene-graph or saved-project diff verification.".into(),
    }
}

pub(crate) fn probe_object_placement_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    starter_project: &Path,
    object_identifier: &str,
    display: &str,
) -> UiActionObjectPlacementProbe {
    let hook_path = alice_home.join(DEFAULT_OBJECT_PLACEMENT_HOOK);
    let evidence_dir = run_dir.join("object-placement");
    if !hook_path.is_file() {
        return UiActionObjectPlacementProbe {
            id: "alice-side-object-placement-command-hook".into(),
            action_id: "place-object".into(),
            status: "blocked".into(),
            detail: format!(
                "blocked: Alice checkout does not expose {DEFAULT_OBJECT_PLACEMENT_HOOK}; object placement remains unproven"
            ),
            object_identifier: object_identifier.into(),
            candidate_hook_path: hook_path.display().to_string(),
            command: None,
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
            placement_artifact: None,
            scene_or_project_diff: None,
            validation_errors: Vec::new(),
            missing_affordance: Some(missing_object_placement_affordance()),
        };
    }

    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_object_placement_probe(
            object_identifier,
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "creating object placement evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--project".to_string(),
                starter_project.display().to_string(),
                "--object".to_string(),
                object_identifier.to_string(),
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
            return failed_object_placement_probe(
                object_identifier,
                &hook_path,
                Some(format!(
                    "{} --project {} --object {} --evidence-dir {} --json",
                    hook_path.display(),
                    starter_project.display(),
                    object_identifier,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                vec![format!("object placement hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_object_placement_probe(
            object_identifier,
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            vec!["object placement hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ObjectPlacementHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_object_placement_probe(
                object_identifier,
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                vec![format!(
                    "object placement hook stdout is not valid placement JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_hook_result(&result, object_identifier);
    let placement_artifact = artifact_info_under(
        &evidence_dir,
        &result.placement_artifact,
        "placement_artifact",
        "object-placement evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let scene_or_project_diff = artifact_info_under(
        &evidence_dir,
        &result.scene_or_project_diff,
        "scene_or_project_diff",
        "object-placement evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    if placement_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("placement_artifact must be non-empty".into());
    }
    if scene_or_project_diff
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("scene_or_project_diff must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side object placement hook returned non-empty placement artifact and scene/project diff for {object_identifier}"
        )
    } else {
        format!(
            "object placement hook ran but did not prove placement: {}",
            validation_errors.join("; ")
        )
    };

    UiActionObjectPlacementProbe {
        id: "alice-side-object-placement-command-hook".into(),
        action_id: "place-object".into(),
        status: status.into(),
        detail,
        object_identifier: object_identifier.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        placement_artifact,
        scene_or_project_diff,
        validation_errors,
        missing_affordance: None,
    }
}

fn failed_object_placement_probe(
    object_identifier: &str,
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    validation_errors: Vec<String>,
) -> UiActionObjectPlacementProbe {
    UiActionObjectPlacementProbe {
        id: "alice-side-object-placement-command-hook".into(),
        action_id: "place-object".into(),
        status: "failed".into(),
        detail: format!(
            "object placement hook did not prove placement: {}",
            validation_errors.join("; ")
        ),
        object_identifier: object_identifier.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        placement_artifact: None,
        scene_or_project_diff: None,
        validation_errors,
        missing_affordance: None,
    }
}

fn validate_hook_result(
    result: &ObjectPlacementHookResult,
    object_identifier: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-object-placement-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-object-placement-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "placed" {
        errors.push(format!("status must be placed, got {:?}", result.status));
    }
    if result.object_identifier != object_identifier {
        errors.push(format!(
            "object_identifier must be {:?}, got {:?}",
            object_identifier, result.object_identifier
        ));
    }
    if result.placement_artifact.is_empty() {
        errors.push("placement_artifact must not be empty".into());
    }
    if result.scene_or_project_diff.is_empty() {
        errors.push("scene_or_project_diff must not be empty".into());
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use eatme_core::CommandOutput;
    use eatme_test_support::FakeCommandRunner;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn object_placement_hook_blocks_when_alice_side_command_is_absent() {
        let root = unique_test_dir("object-placement-hook-absent");
        let alice_home = root.join("alice");
        let run_dir = root.join("runs");
        let project = alice_home.join("starter.a3p");
        fs::create_dir_all(&alice_home).unwrap();
        fs::write(&project, "project").unwrap();
        let runner = FakeCommandRunner::default();

        let probe = probe_object_placement_hook(
            &runner,
            &alice_home,
            &run_dir,
            &project,
            default_object_identifier(),
            ":99",
        );

        assert_eq!(probe.status, "blocked");
        assert_eq!(probe.action_id, "place-object");
        assert!(probe.missing_affordance.is_some());
        assert!(runner.commands().is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn object_placement_hook_passes_only_with_artifact_and_scene_diff_proof() {
        let root = unique_test_dir("object-placement-hook-passed");
        let alice_home = root.join("alice");
        let tools = alice_home.join("tools");
        let run_dir = root.join("runs");
        let evidence_dir = run_dir.join("object-placement");
        let project = alice_home.join("starter.a3p");
        fs::create_dir_all(&tools).unwrap();
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(tools.join("eatme-place-object"), "#!/bin/sh\n").unwrap();
        fs::write(&project, "project").unwrap();
        fs::write(evidence_dir.join("placement.json"), r#"{"placed":true}"#).unwrap();
        fs::write(
            evidence_dir.join("scene.diff.json"),
            r#"{"added":["bunny"]}"#,
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "tools/eatme-place-object --json".into(),
            exit_status: Some(0),
            stdout: serde_json::json!({
                "schema_version": "eatme.alice-object-placement-result/v1",
                "status": "placed",
                "object_identifier": default_object_identifier(),
                "placement_artifact": "placement.json",
                "scene_or_project_diff": "scene.diff.json"
            })
            .to_string(),
            stderr: String::new(),
        });

        let probe = probe_object_placement_hook(
            &runner,
            &alice_home,
            &run_dir,
            &project,
            default_object_identifier(),
            ":99",
        );

        assert_eq!(probe.status, "passed");
        assert!(probe.proves_placement());
        assert!(probe.placement_artifact.unwrap().size_bytes > 0);
        assert!(probe.scene_or_project_diff.unwrap().size_bytes > 0);
        assert!(probe.validation_errors.is_empty());
        assert_eq!(runner.commands().len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn object_placement_hook_rejects_paths_outside_evidence_dir() {
        let root = unique_test_dir("object-placement-hook-bad-path");
        let alice_home = root.join("alice");
        let tools = alice_home.join("tools");
        let run_dir = root.join("runs");
        let project = alice_home.join("starter.a3p");
        fs::create_dir_all(&tools).unwrap();
        fs::write(tools.join("eatme-place-object"), "#!/bin/sh\n").unwrap();
        fs::write(&project, "project").unwrap();
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "tools/eatme-place-object --json".into(),
            exit_status: Some(0),
            stdout: serde_json::json!({
                "schema_version": "eatme.alice-object-placement-result/v1",
                "status": "placed",
                "object_identifier": default_object_identifier(),
                "placement_artifact": "../placement.json",
                "scene_or_project_diff": "scene.diff.json"
            })
            .to_string(),
            stderr: String::new(),
        });

        let probe = probe_object_placement_hook(
            &runner,
            &alice_home,
            &run_dir,
            &project,
            default_object_identifier(),
            ":99",
        );

        assert_eq!(probe.status, "failed");
        assert!(!probe.proves_placement());
        assert!(
            probe
                .validation_errors
                .iter()
                .any(|error| error.contains("simple relative path"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn object_placement_hook_rejects_symlink_escape_from_evidence_dir() {
        let root = unique_test_dir("object-placement-hook-symlink-escape");
        let alice_home = root.join("alice");
        let tools = alice_home.join("tools");
        let run_dir = root.join("runs");
        let evidence_dir = run_dir.join("object-placement");
        let outside_dir = root.join("outside");
        let project = alice_home.join("starter.a3p");
        fs::create_dir_all(&tools).unwrap();
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();
        fs::write(tools.join("eatme-place-object"), "#!/bin/sh\n").unwrap();
        fs::write(&project, "project").unwrap();
        fs::write(outside_dir.join("placement.json"), r#"{"placed":true}"#).unwrap();
        fs::write(
            evidence_dir.join("scene.diff.json"),
            r#"{"added":["bunny"]}"#,
        )
        .unwrap();
        std::os::unix::fs::symlink(
            outside_dir.join("placement.json"),
            evidence_dir.join("placement.json"),
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "tools/eatme-place-object --json".into(),
            exit_status: Some(0),
            stdout: serde_json::json!({
                "schema_version": "eatme.alice-object-placement-result/v1",
                "status": "placed",
                "object_identifier": default_object_identifier(),
                "placement_artifact": "placement.json",
                "scene_or_project_diff": "scene.diff.json"
            })
            .to_string(),
            stderr: String::new(),
        });

        let probe = probe_object_placement_hook(
            &runner,
            &alice_home,
            &run_dir,
            &project,
            default_object_identifier(),
            ":99",
        );

        assert_eq!(probe.status, "failed");
        assert!(!probe.proves_placement());
        assert!(
            probe
                .validation_errors
                .iter()
                .any(|error| error.contains("must stay under"))
        );
        let _ = fs::remove_dir_all(root);
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("eatme-alice-object-placement-tests")
            .join(format!("{prefix}-{nonce}"))
    }
}
