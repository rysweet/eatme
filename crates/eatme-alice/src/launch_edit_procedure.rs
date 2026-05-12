use crate::launch_artifacts::artifact_info;
use crate::launch_object_placement::UiActionObjectPlacementProbe;
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};
use std::time::Duration;

pub(crate) const DEFAULT_PROCEDURE_EDIT_HOOK: &str = "tools/eatme-edit-procedure";
pub(crate) const DEFAULT_PROCEDURE_SELECTOR: &str = "scene.eatmeFirstLessonStep";
const DEFAULT_EDIT_SPEC: &str = "append-comment:eatme first lesson edit proof";
pub(crate) const EDIT_PROCEDURE_PROOF_ARTIFACT: &str = "first-lesson-code-editor-action-proof.json";

#[derive(Clone, Debug, Serialize)]
pub struct UiActionEditProcedureProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub procedure_selector: String,
    pub edit_spec: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub edited_project_artifact: Option<ArtifactInfo>,
    pub procedure_or_code_diff: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<UiActionMissingAffordance>,
    pub edit_procedure_verified: bool,
    pub proof_detail: Option<String>,
}

impl UiActionEditProcedureProbe {
    pub fn proves_edit(&self) -> bool {
        let hook_proved = self.status == "passed"
            && self.edited_project_artifact.is_some()
            && self.procedure_or_code_diff.is_some()
            && self.validation_errors.is_empty();
        hook_proved || self.edit_procedure_verified
    }

    /// Check for the proof artifact file in the run directory.
    /// If found and valid JSON, sets `edit_procedure_verified=true` with proof details.
    /// If missing or invalid, sets `edit_procedure_verified=false`.
    pub(crate) fn with_proof_artifact_check(mut self, run_dir: &Path) -> Self {
        let proof_path = run_dir.join(EDIT_PROCEDURE_PROOF_ARTIFACT);
        let content = match fs::read_to_string(&proof_path) {
            Ok(content) => content,
            Err(_) => return self,
        };
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(value) => {
                self.edit_procedure_verified = true;
                let summary = value.to_string();
                let truncated = if summary.len() > 500 {
                    format!("{}…", &summary[..497])
                } else {
                    summary
                };
                self.proof_detail = Some(truncated.clone());
                if !self.proves_edit() || self.detail.contains("blocked") {
                    self.detail =
                        format!("{} [proof artifact verified: {}]", self.detail, truncated);
                }
            }
            Err(err) => {
                self.edit_procedure_verified = false;
                self.proof_detail =
                    Some(format!("invalid JSON in {}: {err}", proof_path.display()));
            }
        }
        self
    }
}

#[derive(Debug, Deserialize)]
struct ProcedureEditHookResult {
    schema_version: String,
    status: String,
    procedure_selector: String,
    edited_project_artifact: String,
    procedure_or_code_diff: String,
}

pub(crate) fn probe_edit_procedure_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    object_placement_probe: &UiActionObjectPlacementProbe,
    display: &str,
) -> UiActionEditProcedureProbe {
    let hook_path = alice_home.join(DEFAULT_PROCEDURE_EDIT_HOOK);
    let evidence_dir = run_dir.join("procedure-edit");
    let placed_project = run_dir.join("object-placement").join("placed-project.a3p");
    if !object_placement_probe.proves_placement() {
        return blocked_edit_procedure_probe(
            &hook_path,
            "blocked: object placement proof is required before procedure/code-block editing would be safe",
            Some(missing_procedure_edit_affordance()),
        );
    }
    if !hook_path.is_file() {
        return blocked_edit_procedure_probe(
            &hook_path,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_PROCEDURE_EDIT_HOOK}; procedure/code-block editing remains unproven"
            ),
            Some(missing_procedure_edit_affordance()),
        );
    }
    if !placed_project.is_file() {
        return failed_edit_procedure_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "object placement did not leave a placed project at {}",
                placed_project.display()
            )],
        );
    }

    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_edit_procedure_probe(
            &hook_path,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "creating procedure edit evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--project".to_string(),
                placed_project.display().to_string(),
                "--procedure-selector".to_string(),
                DEFAULT_PROCEDURE_SELECTOR.to_string(),
                "--edit-spec".to_string(),
                DEFAULT_EDIT_SPEC.to_string(),
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
            return failed_edit_procedure_probe(
                &hook_path,
                Some(format!(
                    "{} --project {} --procedure-selector {} --edit-spec {} --evidence-dir {} --json",
                    hook_path.display(),
                    placed_project.display(),
                    DEFAULT_PROCEDURE_SELECTOR,
                    DEFAULT_EDIT_SPEC,
                    evidence_dir.display()
                )),
                None,
                String::new(),
                String::new(),
                vec![format!("procedure edit hook failed to run: {error:#}")],
            );
        }
    };

    if output.exit_status != Some(0) {
        return failed_edit_procedure_probe(
            &hook_path,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            vec!["procedure edit hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ProcedureEditHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_edit_procedure_probe(
                &hook_path,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                vec![format!(
                    "procedure edit hook stdout is not valid edit JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_edit_hook_result(&result);
    let edited_project_artifact = hook_artifact(
        &evidence_dir,
        &result.edited_project_artifact,
        "edited_project_artifact",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let procedure_or_code_diff = hook_artifact(
        &evidence_dir,
        &result.procedure_or_code_diff,
        "procedure_or_code_diff",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    if edited_project_artifact
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("edited_project_artifact must be non-empty".into());
    }
    if procedure_or_code_diff
        .as_ref()
        .map(|artifact| artifact.size_bytes == 0)
        .unwrap_or(false)
    {
        validation_errors.push("procedure_or_code_diff must be non-empty".into());
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        format!(
            "Alice-side procedure edit hook returned non-empty edited project and procedure/code diff for {DEFAULT_PROCEDURE_SELECTOR}"
        )
    } else {
        format!(
            "procedure edit hook ran but did not prove editing: {}",
            validation_errors.join("; ")
        )
    };

    UiActionEditProcedureProbe {
        id: "alice-side-procedure-edit-command-hook".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: status.into(),
        detail,
        procedure_selector: DEFAULT_PROCEDURE_SELECTOR.into(),
        edit_spec: DEFAULT_EDIT_SPEC.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        edited_project_artifact,
        procedure_or_code_diff,
        validation_errors,
        missing_affordance: None,
        edit_procedure_verified: false,
        proof_detail: None,
    }
}

pub(crate) fn probe_edit_procedure_preconditions(
    object_placement_probe: &UiActionObjectPlacementProbe,
) -> UiActionNoGoProbe {
    let object_placement_ready = object_placement_probe.proves_placement();
    let blocking_reason = if object_placement_ready {
        "blocked: missing deterministic-alice-procedure-edit-affordance"
    } else {
        "blocked: object placement proof is required before procedure/code-block editing would be safe"
    };

    UiActionNoGoProbe {
        id: "edit-procedure-precondition".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence:
            "artifact proves a procedure or code block was edited in the project after object placement"
                .into(),
        missing_affordance: missing_procedure_edit_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "place-object".into(),
                passed: object_placement_ready,
                detail:
                    "object-placement hook returned a non-empty placement artifact and scene/project diff"
                        .into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-procedure-edit-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, or editor automation contract for editing a named procedure or code block".into(),
            },
        ],
    }
}

fn missing_procedure_edit_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-procedure-edit-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given a project after object placement plus a named procedure or code-block selector, deterministically edit that procedure or code block and return proof of the edit.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_PROCEDURE_EDIT_HOOK}, accessibility target, or editor automation contract currently accepts a procedure/code-block selector and returns an edited project artifact plus a procedure/code diff."),
        next_implementation: "Add one stable affordance: either an Alice-side procedure edit command hook defined by this contract, or a UI automation contract with a named editor target plus saved-project or AST diff verification.".into(),
    }
}

fn blocked_edit_procedure_probe(
    hook_path: &Path,
    detail: &str,
    missing_affordance: Option<UiActionMissingAffordance>,
) -> UiActionEditProcedureProbe {
    UiActionEditProcedureProbe {
        id: "alice-side-procedure-edit-command-hook".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: "blocked".into(),
        detail: detail.into(),
        procedure_selector: DEFAULT_PROCEDURE_SELECTOR.into(),
        edit_spec: DEFAULT_EDIT_SPEC.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        edited_project_artifact: None,
        procedure_or_code_diff: None,
        validation_errors: Vec::new(),
        missing_affordance,
        edit_procedure_verified: false,
        proof_detail: None,
    }
}

fn failed_edit_procedure_probe(
    hook_path: &Path,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    validation_errors: Vec<String>,
) -> UiActionEditProcedureProbe {
    UiActionEditProcedureProbe {
        id: "alice-side-procedure-edit-command-hook".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: "failed".into(),
        detail: format!(
            "procedure edit hook did not prove editing: {}",
            validation_errors.join("; ")
        ),
        procedure_selector: DEFAULT_PROCEDURE_SELECTOR.into(),
        edit_spec: DEFAULT_EDIT_SPEC.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        edited_project_artifact: None,
        procedure_or_code_diff: None,
        validation_errors,
        missing_affordance: None,
        edit_procedure_verified: false,
        proof_detail: None,
    }
}

fn validate_edit_hook_result(result: &ProcedureEditHookResult) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-procedure-edit-result/v1" {
        errors.push(format!(
            "schema_version must be eatme.alice-procedure-edit-result/v1, got {:?}",
            result.schema_version
        ));
    }
    if result.status != "edited" {
        errors.push(format!("status must be edited, got {:?}", result.status));
    }
    if result.procedure_selector != DEFAULT_PROCEDURE_SELECTOR {
        errors.push(format!(
            "procedure_selector must be {:?}, got {:?}",
            DEFAULT_PROCEDURE_SELECTOR, result.procedure_selector
        ));
    }
    if result.edited_project_artifact.is_empty() {
        errors.push("edited_project_artifact must not be empty".into());
    }
    if result.procedure_or_code_diff.is_empty() {
        errors.push("procedure_or_code_diff must not be empty".into());
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
            "{field} must be a simple relative path under procedure-edit evidence dir"
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
