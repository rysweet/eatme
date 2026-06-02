use crate::launch_path_validation::artifact_info_under;
use crate::launch_run_world::UiActionRunWorldProbe;
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(crate) const DEFAULT_PROJECT_SAVE_HOOK: &str = "tools/eatme-save-project";
pub(crate) const DEFAULT_SAVE_SELECTOR: &str = "scene.myFirstMethod";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionSaveProjectProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub save_selector: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub saved_project_artifact: Option<ArtifactInfo>,
    pub save_artifact: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
}

impl UiActionSaveProjectProbe {
    pub fn proves_save(&self) -> bool {
        self.status == "passed"
            && self.saved_project_artifact.is_some()
            && self.save_artifact.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ProjectSaveHookResult {
    schema_version: String,
    status: String,
    save_selector: String,
    saved_project_artifact: String,
    save_artifact: String,
}

pub(crate) fn probe_project_save_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    run_world_probe: &UiActionRunWorldProbe,
    display: &str,
) -> UiActionSaveProjectProbe {
    let hook_path = alice_home.join(DEFAULT_PROJECT_SAVE_HOOK);
    let evidence_dir = run_dir.join("project-save");
    let edited_project = run_dir.join("procedure-edit").join("edited-project.a3p");
    if !run_world_probe.proves_run() {
        return blocked_save_project_probe(
            &hook_path,
            "blocked: run-world proof is required before project save would be safe",
            Some(missing_project_save_affordance()),
        );
    }
    if !hook_path.is_file() {
        return blocked_save_project_probe(
            &hook_path,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_PROJECT_SAVE_HOOK}; project save remains unproven"
            ),
            Some(missing_project_save_affordance()),
        );
    }
    if !edited_project.is_file() {
        return failed_save_project_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "procedure edit did not leave an edited project at {}",
                edited_project.display()
            )],
        );
    }

    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_save_project_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "creating project save evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--project".to_string(),
                edited_project.display().to_string(),
                "--save-selector".to_string(),
                DEFAULT_SAVE_SELECTOR.to_string(),
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
            return failed_save_project_probe(
                &hook_path,
                Some(format!(
                    "{} --project {} --save-selector {} --evidence-dir {} --json",
                    hook_path.display(),
                    edited_project.display(),
                    DEFAULT_SAVE_SELECTOR,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                vec![format!("project save hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_save_project_probe(
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            vec!["project save hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ProjectSaveHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_save_project_probe(
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                vec![format!(
                    "project save hook stdout is not valid save JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_save_hook_result(&result);
    let saved_project_artifact = artifact_info_under(
        &evidence_dir,
        &result.saved_project_artifact,
        "saved_project_artifact",
        "project-save evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let save_artifact = artifact_info_under(
        &evidence_dir,
        &result.save_artifact,
        "save_artifact",
        "project-save evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    if saved_project_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("saved_project_artifact must be non-empty".into());
    }
    if save_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("save_artifact must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side project save hook returned non-empty saved project and save evidence for {DEFAULT_SAVE_SELECTOR}"
        )
    } else {
        format!(
            "project save hook ran but did not prove save: {}",
            validation_errors.join("; ")
        )
    };

    UiActionSaveProjectProbe {
        id: "alice-side-project-save-command-hook".into(),
        action_id: "save-project".into(),
        status: status.into(),
        detail,
        save_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        saved_project_artifact,
        save_artifact,
        validation_errors,
        missing_affordance: None,
    }
}

pub(crate) fn probe_project_save_preconditions(
    run_world_probe: &UiActionRunWorldProbe,
) -> UiActionNoGoProbe {
    let run_ready = run_world_probe.proves_run();
    let blocking_reason = if run_ready {
        "blocked: missing deterministic-alice-project-save-affordance"
    } else {
        "blocked: run-world proof is required before project save would be safe"
    };

    UiActionNoGoProbe {
        id: "project-save-precondition".into(),
        action_id: "save-project".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence: "saved .a3p project artifact exists, is non-empty, and can be read after the first-lesson run proof".into(),
        missing_affordance: missing_project_save_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "run-world".into(),
                passed: run_ready,
                detail: "run-world hook returned non-empty run artifact and runtime/log evidence".into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-project-save-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, save control contract, or persistence verification hook for proving the edited project was saved".into(),
            },
        ],
    }
}

fn missing_project_save_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-project-save-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given an edited Alice project after run-world proof, deterministically save the project and return proof that the saved .a3p is readable.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_PROJECT_SAVE_HOOK}, accessibility target, save control contract, or persistence verification hook currently accepts an edited project and returns project-save proof."),
        next_implementation: "Add one stable affordance: either an Alice-side save-project command hook defined by this contract, or a desktop automation contract with named save control plus saved-project evidence.".into(),
    }
}

fn blocked_save_project_probe(
    hook_path: &Path,
    detail: &str,
    missing_affordance: Option<UiActionMissingAffordance>,
) -> UiActionSaveProjectProbe {
    UiActionSaveProjectProbe {
        id: "alice-side-project-save-command-hook".into(),
        action_id: "save-project".into(),
        status: "blocked".into(),
        detail: detail.into(),
        save_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        saved_project_artifact: None,
        save_artifact: None,
        validation_errors: Vec::new(),
        missing_affordance,
    }
}

fn failed_save_project_probe(
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    validation_errors: Vec<String>,
) -> UiActionSaveProjectProbe {
    UiActionSaveProjectProbe {
        id: "alice-side-project-save-command-hook".into(),
        action_id: "save-project".into(),
        status: "failed".into(),
        detail: format!(
            "project save hook did not prove save: {}",
            validation_errors.join("; ")
        ),
        save_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        saved_project_artifact: None,
        save_artifact: None,
        validation_errors,
        missing_affordance: None,
    }
}

fn validate_save_hook_result(result: &ProjectSaveHookResult) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-project-save-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-project-save-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "saved" {
        errors.push(format!("status must be saved, got {:?}", result.status));
    }
    if result.save_selector != DEFAULT_SAVE_SELECTOR {
        errors.push(format!(
            "save_selector must be {:?}, got {:?}",
            DEFAULT_SAVE_SELECTOR, result.save_selector
        ));
    }
    if result.saved_project_artifact.is_empty() {
        errors.push("saved_project_artifact must not be empty".into());
    }
    if result.save_artifact.is_empty() {
        errors.push("save_artifact must not be empty".into());
    }
    errors
}

#[cfg(test)]
mod tests;
