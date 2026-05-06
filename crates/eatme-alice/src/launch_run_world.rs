use crate::launch_artifacts::artifact_info;
use crate::launch_edit_procedure::UiActionEditProcedureProbe;
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};
use std::time::Duration;

pub(crate) const DEFAULT_WORLD_RUN_HOOK: &str = "tools/eatme-run-world";
pub(crate) const DEFAULT_RUN_SELECTOR: &str = "scene.eatmeFirstLessonStep";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionRunWorldProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub run_selector: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub run_artifact: Option<ArtifactInfo>,
    pub runtime_or_log_evidence: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
}

impl UiActionRunWorldProbe {
    pub fn proves_run(&self) -> bool {
        self.status == "passed"
            && self.run_artifact.is_some()
            && self.runtime_or_log_evidence.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct WorldRunHookResult {
    schema_version: String,
    status: String,
    run_selector: String,
    run_artifact: String,
    runtime_or_log_evidence: String,
}

pub(crate) fn probe_run_world_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    edit_procedure_probe: &UiActionEditProcedureProbe,
    display: &str,
) -> UiActionRunWorldProbe {
    let hook_path = alice_home.join(DEFAULT_WORLD_RUN_HOOK);
    let evidence_dir = run_dir.join("world-run");
    let edited_project = run_dir.join("procedure-edit").join("edited-project.a3p");
    if !edit_procedure_probe.proves_edit() {
        return blocked_run_world_probe(
            &hook_path,
            "blocked: procedure/code-block edit proof is required before world run would be safe",
            Some(missing_world_run_affordance()),
        );
    }
    if !hook_path.is_file() {
        return blocked_run_world_probe(
            &hook_path,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_WORLD_RUN_HOOK}; world run remains unproven"
            ),
            Some(missing_world_run_affordance()),
        );
    }
    if !edited_project.is_file() {
        return failed_run_world_probe(
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
        return failed_run_world_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "creating world run evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--project".to_string(),
                edited_project.display().to_string(),
                "--run-selector".to_string(),
                DEFAULT_RUN_SELECTOR.to_string(),
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
            return failed_run_world_probe(
                &hook_path,
                Some(format!(
                    "{} --project {} --run-selector {} --evidence-dir {} --json",
                    hook_path.display(),
                    edited_project.display(),
                    DEFAULT_RUN_SELECTOR,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                vec![format!("world run hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_run_world_probe(
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            vec!["world run hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<WorldRunHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_run_world_probe(
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                vec![format!(
                    "world run hook stdout is not valid run JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_run_hook_result(&result);
    let run_artifact = hook_artifact(&evidence_dir, &result.run_artifact, "run_artifact")
        .map_err(|error| validation_errors.push(error))
        .ok();
    let runtime_or_log_evidence = hook_artifact(
        &evidence_dir,
        &result.runtime_or_log_evidence,
        "runtime_or_log_evidence",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    if run_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("run_artifact must be non-empty".into());
    }
    if runtime_or_log_evidence
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("runtime_or_log_evidence must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side world run hook returned non-empty run artifact and runtime/log evidence for {DEFAULT_RUN_SELECTOR}"
        )
    } else {
        format!(
            "world run hook ran but did not prove execution: {}",
            validation_errors.join("; ")
        )
    };

    UiActionRunWorldProbe {
        id: "alice-side-world-run-command-hook".into(),
        action_id: "run-world".into(),
        status: status.into(),
        detail,
        run_selector: DEFAULT_RUN_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        run_artifact,
        runtime_or_log_evidence,
        validation_errors,
        missing_affordance: None,
    }
}

pub(crate) fn probe_run_world_preconditions(
    edit_procedure_probe: &UiActionEditProcedureProbe,
) -> UiActionNoGoProbe {
    let edit_ready = edit_procedure_probe.proves_edit();
    let blocking_reason = if edit_ready {
        "blocked: missing deterministic-alice-world-run-affordance"
    } else {
        "blocked: procedure/code-block edit proof is required before world run would be safe"
    };

    UiActionNoGoProbe {
        id: "run-world-precondition".into(),
        action_id: "run-world".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence: "artifact proves the world run control or equivalent runtime entry point executed after the first-lesson edit".into(),
        missing_affordance: missing_world_run_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "edit-procedure-or-code-block".into(),
                passed: edit_ready,
                detail: "procedure/code-block edit hook returned a non-empty edited project and procedure/code diff".into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-world-run-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, run control contract, or runtime hook for proving the edited world was run".into(),
            },
        ],
    }
}

fn missing_world_run_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-world-run-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given an edited Alice project, deterministically run the world or equivalent runtime entry point and return proof that execution reached the edited world.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_WORLD_RUN_HOOK}, accessibility target, run control contract, or runtime verification hook currently accepts an edited project and returns world-run proof."),
        next_implementation: "Add one stable affordance: either an Alice-side run-world command hook defined by this contract, or a desktop automation contract with named run control plus runtime/log evidence.".into(),
    }
}

fn blocked_run_world_probe(
    hook_path: &Path,
    detail: &str,
    missing_affordance: Option<UiActionMissingAffordance>,
) -> UiActionRunWorldProbe {
    UiActionRunWorldProbe {
        id: "alice-side-world-run-command-hook".into(),
        action_id: "run-world".into(),
        status: "blocked".into(),
        detail: detail.into(),
        run_selector: DEFAULT_RUN_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        run_artifact: None,
        runtime_or_log_evidence: None,
        validation_errors: Vec::new(),
        missing_affordance,
    }
}

fn failed_run_world_probe(
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    validation_errors: Vec<String>,
) -> UiActionRunWorldProbe {
    UiActionRunWorldProbe {
        id: "alice-side-world-run-command-hook".into(),
        action_id: "run-world".into(),
        status: "failed".into(),
        detail: format!(
            "world run hook did not prove execution: {}",
            validation_errors.join("; ")
        ),
        run_selector: DEFAULT_RUN_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        run_artifact: None,
        runtime_or_log_evidence: None,
        validation_errors,
        missing_affordance: None,
    }
}

fn validate_run_hook_result(result: &WorldRunHookResult) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-world-run-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-world-run-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "ran" {
        errors.push(format!("status must be ran, got {:?}", result.status));
    }
    if result.run_selector != DEFAULT_RUN_SELECTOR {
        errors.push(format!(
            "run_selector must be {:?}, got {:?}",
            DEFAULT_RUN_SELECTOR, result.run_selector
        ));
    }
    if result.run_artifact.is_empty() {
        errors.push("run_artifact must not be empty".into());
    }
    if result.runtime_or_log_evidence.is_empty() {
        errors.push("runtime_or_log_evidence must not be empty".into());
    }
    errors
}

fn hook_artifact(
    evidence_dir: &Path,
    relative_path: &str,
    field: &str,
) -> std::result::Result<ArtifactInfo, String> {
    let path = Path::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "{field} must be a simple relative path under world-run evidence dir"
        ));
    }

    let full_path = evidence_dir.join(path);
    artifact_info(&full_path).map_err(|error| {
        format!(
            "{field} {} is not a readable artifact: {error:#}",
            full_path.display()
        )
    })
}

#[cfg(test)]
mod tests;
