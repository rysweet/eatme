#![cfg_attr(not(test), allow(dead_code))]

use crate::launch_path_validation::{
    artifact_info_under, canonical_artifact_under, normal_components,
};
use crate::launch_save_project::{DEFAULT_SAVE_SELECTOR, UiActionSaveProjectProbe};
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) const DEFAULT_PROJECT_REOPEN_HOOK: &str = "tools/eatme-reopen-project";
const EXPECTED_SOURCE_PREFIX: &str = "project-save";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionReopenProjectProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub reopen_selector: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub source_saved_project_artifact: String,
    pub reopened_project_artifact: Option<ArtifactInfo>,
    pub reopen_artifact: Option<ArtifactInfo>,
    pub reopened_state_artifact: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
}

impl UiActionReopenProjectProbe {
    pub fn proves_reopen(&self) -> bool {
        self.status == "passed"
            && !self.source_saved_project_artifact.is_empty()
            && self.reopened_project_artifact.is_some()
            && self.reopen_artifact.is_some()
            && self.reopened_state_artifact.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ProjectReopenHookResult {
    schema_version: String,
    status: String,
    source_saved_project_artifact: String,
    reopen_selector: String,
    reopened_project_artifact: String,
    reopen_artifact: String,
    reopened_state_artifact: String,
    state_verification: String,
}

pub(crate) fn probe_project_reopen_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    save_project_probe: &UiActionSaveProjectProbe,
    display: &str,
) -> UiActionReopenProjectProbe {
    probe_project_reopen_hook_with_selector(
        runner,
        alice_home,
        run_dir,
        save_project_probe,
        DEFAULT_SAVE_SELECTOR,
        display,
    )
}

pub(crate) fn probe_project_reopen_hook_with_selector(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    save_project_probe: &UiActionSaveProjectProbe,
    reopen_selector: &str,
    display: &str,
) -> UiActionReopenProjectProbe {
    let hook_path = alice_home.join(DEFAULT_PROJECT_REOPEN_HOOK);
    let evidence_dir = run_dir.join("project-reopen");

    if !save_project_probe.proves_save() {
        return blocked_reopen_project_probe(
            &hook_path,
            "blocked: save-project proof is required before project reopen would be safe",
            Some(missing_project_reopen_affordance()),
        );
    }
    if !hook_path.is_file() {
        return blocked_reopen_project_probe(
            &hook_path,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_PROJECT_REOPEN_HOOK}; project reopen remains unproven"
            ),
            Some(missing_project_reopen_affordance()),
        );
    }

    let saved_project = match saved_project_path(run_dir, save_project_probe) {
        Ok(path) => path,
        Err(error) => {
            return failed_reopen_project_probe(
                &hook_path,
                None,
                None,
                String::new(),
                String::new(),
                String::new(),
                vec![error],
            );
        }
    };

    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_reopen_project_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            String::new(),
            vec![format!(
                "creating project reopen evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--saved-project".to_string(),
                saved_project.display().to_string(),
                "--reopen-selector".to_string(),
                reopen_selector.to_string(),
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
            return failed_reopen_project_probe(
                &hook_path,
                Some(format!(
                    "{} --saved-project {} --reopen-selector {} --evidence-dir {} --json",
                    hook_path.display(),
                    saved_project.display(),
                    reopen_selector,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                String::new(),
                vec![format!("project reopen hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_reopen_project_probe(
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            String::new(),
            vec!["project reopen hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ProjectReopenHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_reopen_project_probe(
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                String::new(),
                vec![format!(
                    "project reopen hook stdout is not valid reopen JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors =
        validate_reopen_hook_result(&result, run_dir, &saved_project, reopen_selector);
    let reopened_project_artifact = artifact_info_under(
        &evidence_dir,
        &result.reopened_project_artifact,
        "reopened_project_artifact",
        "project-reopen evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let reopen_artifact = artifact_info_under(
        &evidence_dir,
        &result.reopen_artifact,
        "reopen_artifact",
        "project-reopen evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let reopened_state_artifact = artifact_info_under(
        &evidence_dir,
        &result.reopened_state_artifact,
        "reopened_state_artifact",
        "project-reopen evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();

    if reopened_project_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("reopened_project_artifact must be non-empty".into());
    }
    if reopen_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("reopen_artifact must be non-empty".into());
    }
    if reopened_state_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("reopened_state_artifact must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side project reopen hook reopened saved artifact {} and returned state evidence for {reopen_selector}",
            result.source_saved_project_artifact
        )
    } else {
        format!(
            "project reopen hook ran but did not prove reopen: {}",
            validation_errors.join("; ")
        )
    };

    UiActionReopenProjectProbe {
        id: "alice-side-project-reopen-command-hook".into(),
        action_id: "reopen-project".into(),
        status: status.into(),
        detail,
        reopen_selector: reopen_selector.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        source_saved_project_artifact: result.source_saved_project_artifact,
        reopened_project_artifact,
        reopen_artifact,
        reopened_state_artifact,
        validation_errors,
        missing_affordance: None,
    }
}

pub(crate) fn probe_project_reopen_preconditions(
    save_project_probe: &UiActionSaveProjectProbe,
) -> UiActionNoGoProbe {
    let save_ready = save_project_probe.proves_save();
    let blocking_reason = if save_ready {
        "blocked: missing deterministic-alice-project-reopen-affordance"
    } else {
        "blocked: save-project proof is required before project reopen would be safe"
    };

    UiActionNoGoProbe {
        id: "project-reopen-precondition".into(),
        action_id: "reopen-project".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence: "saved .a3p artifact is reopened in a new or reset Alice session, reopened state evidence is captured, and the source is the saved project artifact, not the original bundled starter project".into(),
        missing_affordance: missing_project_reopen_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "save-project".into(),
                passed: save_ready,
                detail:
                    "save-project hook returned non-empty saved project and save evidence artifacts"
                        .into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-project-reopen-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, reopen control contract, or state verification hook for proving the saved project can be reopened".into(),
            },
        ],
    }
}

fn missing_project_reopen_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-project-reopen-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given project-save proof, deterministically reopen the saved .a3p in a new or reset Alice session and return reopened-state proof.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_PROJECT_REOPEN_HOOK}, accessibility target, reopen control contract, or state verification hook currently accepts a saved project and returns project-reopen proof."),
        next_implementation: "Add one stable affordance: either an Alice-side reopen-project command hook defined by this contract, or a desktop automation contract with named reopen control plus reopened-project and state evidence.".into(),
    }
}

fn blocked_reopen_project_probe(
    hook_path: &Path,
    detail: &str,
    missing_affordance: Option<UiActionMissingAffordance>,
) -> UiActionReopenProjectProbe {
    UiActionReopenProjectProbe {
        id: "alice-side-project-reopen-command-hook".into(),
        action_id: "reopen-project".into(),
        status: "blocked".into(),
        detail: detail.into(),
        reopen_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        source_saved_project_artifact: String::new(),
        reopened_project_artifact: None,
        reopen_artifact: None,
        reopened_state_artifact: None,
        validation_errors: Vec::new(),
        missing_affordance,
    }
}

fn failed_reopen_project_probe(
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    source_saved_project_artifact: String,
    validation_errors: Vec<String>,
) -> UiActionReopenProjectProbe {
    UiActionReopenProjectProbe {
        id: "alice-side-project-reopen-command-hook".into(),
        action_id: "reopen-project".into(),
        status: "failed".into(),
        detail: format!(
            "project reopen hook did not prove reopen: {}",
            validation_errors.join("; ")
        ),
        reopen_selector: DEFAULT_SAVE_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        source_saved_project_artifact,
        reopened_project_artifact: None,
        reopen_artifact: None,
        reopened_state_artifact: None,
        validation_errors,
        missing_affordance: None,
    }
}

fn validate_reopen_hook_result(
    result: &ProjectReopenHookResult,
    run_dir: &Path,
    expected_saved_project: &Path,
    expected_selector: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-project-reopen-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-project-reopen-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "reopened" {
        errors.push(format!("status must be reopened, got {:?}", result.status));
    }
    if result.reopen_selector != expected_selector {
        errors.push(format!(
            "reopen_selector must be {:?}, got {:?}",
            expected_selector, result.reopen_selector
        ));
    }
    if result.state_verification != "passed" && result.state_verification != "matched" {
        errors.push(format!(
            "state_verification must be passed or matched, got {:?}",
            result.state_verification
        ));
    }
    validate_source_saved_project(
        &result.source_saved_project_artifact,
        run_dir,
        expected_saved_project,
        &mut errors,
    );
    if result.reopened_project_artifact.is_empty() {
        errors.push("reopened_project_artifact must not be empty".into());
    }
    if result.reopen_artifact.is_empty() {
        errors.push("reopen_artifact must not be empty".into());
    }
    if result.reopened_state_artifact.is_empty() {
        errors.push("reopened_state_artifact must not be empty".into());
    }
    errors
}

fn validate_source_saved_project(
    source: &str,
    run_dir: &Path,
    expected_saved_project: &Path,
    errors: &mut Vec<String>,
) {
    if source.is_empty() {
        errors.push("source_saved_project_artifact must not be empty".into());
        return;
    }

    let path = Path::new(source);
    let components = normal_components(path);
    if path.is_absolute() || components.is_none() {
        errors.push(
            "source_saved_project_artifact must be a simple relative path under project-save"
                .into(),
        );
        return;
    }

    let components = components.unwrap();
    if components.first().map(String::as_str) != Some(EXPECTED_SOURCE_PREFIX) {
        errors
            .push("source_saved_project_artifact must reopen the saved artifact, not the bundled starter project".into());
        return;
    }

    let full_path = run_dir.join(path);
    let project_save_dir = run_dir.join(EXPECTED_SOURCE_PREFIX);
    let source_resolved = match canonical_artifact_under(
        &project_save_dir,
        &full_path,
        "source_saved_project_artifact",
        "project-save evidence dir",
    ) {
        Ok(path) => path,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    let expected_resolved = match expected_saved_project.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            errors.push(format!(
                "save-project saved_project_artifact {} is not a readable artifact: {error:#}",
                expected_saved_project.display()
            ));
            return;
        }
    };
    if source_resolved != expected_resolved {
        errors.push(
            "source_saved_project_artifact must match save-project saved_project_artifact from the same run"
                .into(),
        );
    }
}

fn saved_project_path(
    run_dir: &Path,
    save_project_probe: &UiActionSaveProjectProbe,
) -> std::result::Result<PathBuf, String> {
    let artifact = save_project_probe
        .saved_project_artifact
        .as_ref()
        .ok_or_else(|| "save-project proof did not include saved_project_artifact".to_string())?;
    let path = Path::new(&artifact.path);
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        run_dir.join(path)
    };
    Ok(full_path)
}
