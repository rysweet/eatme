use crate::launch_artifacts::artifact_info;
use crate::launch_path_validation::artifact_info_under;
use eatme_core::{ArtifactInfo, AssertionResult, CommandRunner, CommandSpec};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) const DEFAULT_CLASS_PORTABILITY_HOOK: &str = "tools/eatme-class-portability";
const SOURCE_PROJECT_NAME: &str = "learner-source-world";
const DESTINATION_PROJECT_NAME: &str = "peer-import-world";
const MODIFIED_CLASS_NAME: &str = "learner-modified-character-class";
const BEHAVIOR_SELECTOR: &str = "modified-class-behavior-visible-after-import";

#[derive(Clone, Debug, Serialize)]
pub struct DesktopClassPortabilityProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub source_project: String,
    pub destination_project: String,
    pub modified_class: String,
    pub behavior_selector: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub exported_class_package: Option<ArtifactInfo>,
    pub import_report: Option<ArtifactInfo>,
    pub save_reopen_report: Option<ArtifactInfo>,
    pub post_import_behavior: Option<ArtifactInfo>,
    pub validation_errors: Vec<String>,
    pub missing_affordance: Option<ClassPortabilityMissingAffordance>,
}

impl DesktopClassPortabilityProbe {
    pub fn proves_portability(&self) -> bool {
        self.status == "passed"
            && self.exported_class_package.is_some()
            && self.import_report.is_some()
            && self.save_reopen_report.is_some()
            && self.post_import_behavior.is_some()
            && self.validation_errors.is_empty()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ClassPortabilityMissingAffordance {
    pub id: String,
    pub summary: String,
    pub next_implementation: String,
    pub required_backend: String,
}

#[derive(Debug, Deserialize)]
struct ClassPortabilityHookResult {
    schema_version: String,
    status: String,
    source_project: String,
    destination_project: String,
    modified_class: String,
    behavior_selector: String,
    exported_class_package: String,
    import_report: String,
    save_reopen_report: String,
    post_import_behavior: String,
}

pub(crate) fn probe_desktop_class_portability_hook(
    runner: &impl CommandRunner,
    alice_home: &Path,
    run_dir: &Path,
    starter_project: &Path,
    display: &str,
    preflight_ready: bool,
) -> DesktopClassPortabilityProbe {
    let hook_path = alice_home.join(DEFAULT_CLASS_PORTABILITY_HOOK);
    let evidence_dir = run_dir.join("portability");
    let source_project = resolve_starter_project(alice_home, starter_project);

    if !preflight_ready {
        return blocked_probe(
            &hook_path,
            source_project,
            "blocked: Alice desktop window, visual evidence, and launch log must be captured before class portability actions are safe",
        );
    }
    if !hook_path.is_file() {
        return blocked_probe(
            &hook_path,
            source_project,
            &format!(
                "blocked: Alice checkout does not expose {DEFAULT_CLASS_PORTABILITY_HOOK}; desktop modified-class export/import/save/reopen evidence remains unproven"
            ),
        );
    }
    if !source_project.is_file() {
        return failed_probe(
            &hook_path,
            source_project,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "source starter project is not readable at {}",
                resolve_starter_project(alice_home, starter_project).display()
            )],
        );
    }
    if let Err(error) = fs::create_dir_all(&evidence_dir) {
        return failed_probe(
            &hook_path,
            source_project,
            None,
            None,
            String::new(),
            String::new(),
            vec![format!(
                "creating class portability evidence dir {} failed: {error}",
                evidence_dir.display()
            )],
        );
    }

    let command_for_error = format!(
        "{} --source-project {} --source-project-name {} --destination-project-name {} --modified-class {} --behavior-selector {} --evidence-dir {} --json",
        hook_path.display(),
        source_project.display(),
        SOURCE_PROJECT_NAME,
        DESTINATION_PROJECT_NAME,
        MODIFIED_CLASS_NAME,
        BEHAVIOR_SELECTOR,
        evidence_dir.display()
    );
    let output = runner.run(
        &CommandSpec::new(hook_path.display().to_string())
            .args([
                "--source-project".to_string(),
                source_project.display().to_string(),
                "--source-project-name".to_string(),
                SOURCE_PROJECT_NAME.to_string(),
                "--destination-project-name".to_string(),
                DESTINATION_PROJECT_NAME.to_string(),
                "--modified-class".to_string(),
                MODIFIED_CLASS_NAME.to_string(),
                "--behavior-selector".to_string(),
                BEHAVIOR_SELECTOR.to_string(),
                "--evidence-dir".to_string(),
                evidence_dir.display().to_string(),
                "--json".to_string(),
            ])
            .cwd(alice_home)
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(90)),
    );

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return failed_probe(
                &hook_path,
                source_project,
                Some(command_for_error),
                None,
                String::new(),
                String::new(),
                vec![format!("class portability hook failed to run: {error:#}")],
            );
        }
    };
    if output.exit_status != Some(0) {
        return failed_probe(
            &hook_path,
            source_project,
            Some(output.command),
            output.exit_status,
            output.stdout,
            output.stderr,
            vec!["class portability hook exited unsuccessfully".into()],
        );
    }

    let result = match serde_json::from_str::<ClassPortabilityHookResult>(&output.stdout) {
        Ok(result) => result,
        Err(error) => {
            return failed_probe(
                &hook_path,
                source_project,
                Some(output.command),
                output.exit_status,
                output.stdout,
                output.stderr,
                vec![format!(
                    "class portability hook stdout is not valid portability JSON: {error}"
                )],
            );
        }
    };

    let mut validation_errors = validate_hook_result(&result);
    let exported_class_package = artifact_info_under(
        &evidence_dir,
        &result.exported_class_package,
        "exported_class_package",
        "class portability evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let import_report = artifact_info_under(
        &evidence_dir,
        &result.import_report,
        "import_report",
        "class portability evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let save_reopen_report = artifact_info_under(
        &evidence_dir,
        &result.save_reopen_report,
        "save_reopen_report",
        "class portability evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();
    let post_import_behavior = artifact_info_under(
        &evidence_dir,
        &result.post_import_behavior,
        "post_import_behavior",
        "class portability evidence dir",
    )
    .map_err(|error| validation_errors.push(error))
    .ok();

    for (label, artifact) in [
        ("exported_class_package", &exported_class_package),
        ("import_report", &import_report),
        ("save_reopen_report", &save_reopen_report),
        ("post_import_behavior", &post_import_behavior),
    ] {
        if artifact
            .as_ref()
            .map(|artifact| artifact.size_bytes == 0)
            .unwrap_or(false)
        {
            validation_errors.push(format!("{label} must be non-empty"));
        }
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        "Alice-side class portability hook produced export package, destination import report, and post-import behavior evidence".to_string()
    } else {
        format!(
            "class portability hook ran but did not prove desktop portability: {}",
            validation_errors.join("; ")
        )
    };

    DesktopClassPortabilityProbe {
        id: "alice-side-class-portability-command-hook".into(),
        action_id: "modified-class-export-import-save-reopen".into(),
        status: status.into(),
        detail,
        source_project: SOURCE_PROJECT_NAME.into(),
        destination_project: DESTINATION_PROJECT_NAME.into(),
        modified_class: MODIFIED_CLASS_NAME.into(),
        behavior_selector: BEHAVIOR_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(output.command),
        exit_status: output.exit_status,
        stdout: output.stdout,
        stderr: output.stderr,
        exported_class_package,
        import_report,
        save_reopen_report,
        post_import_behavior,
        validation_errors,
        missing_affordance: None,
    }
}

pub(crate) fn write_desktop_class_portability_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
    probe: &DesktopClassPortabilityProbe,
) -> anyhow::Result<ArtifactInfo> {
    let evidence_dir = run_dir.join("portability");
    fs::create_dir_all(&evidence_dir)?;
    let path = evidence_dir.join("desktop-class-portability-contract.json");
    let contract = serde_json::json!({
        "schema_version": "eatme.desktop-class-portability-contract/v1",
        "status": if probe.proves_portability() { "passed" } else { probe.status.as_str() },
        "blocking_reason": if probe.proves_portability() { serde_json::Value::Null } else { serde_json::json!(probe.detail) },
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "candidate_affordance_probes": [probe],
        "required_actions": required_actions(probe.proves_portability()),
        "does_not_claim": [
            "LookingGlass browser class behavior support",
            "desktop class portability when tools/eatme-class-portability is absent",
            "manual instructor review",
            "class portability without export, import, save/reopen, and post-import behavior artifacts"
        ]
    });
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &contract)?;
    writer.flush()?;
    artifact_info(&path)
}

pub(crate) fn desktop_class_portability_assertion(
    probe: &DesktopClassPortabilityProbe,
) -> AssertionResult {
    if probe.proves_portability() {
        AssertionResult::pass(
            "desktop Alice modified-class export/import/save/reopen behavior evidence was captured",
        )
    } else {
        AssertionResult::fail(probe.detail.clone())
    }
}

pub(crate) fn desktop_class_portability_failure_category(
    probe: &DesktopClassPortabilityProbe,
) -> &'static str {
    if probe.status == "blocked" {
        "class_portability_desktop_contract_blocked"
    } else {
        "class_portability_desktop_evidence_missing"
    }
}

fn required_actions(proven: bool) -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "id": "modified-class-export-import-save-reopen",
        "decision": if proven { "ready" } else { "no_go" },
        "required_evidence": "desktop Alice artifact set proving modified class export, import into a different project, destination save/reopen, and visible post-import behavior",
        "missing_affordance_id": "deterministic-alice-class-portability-affordance",
        "contract_required": {
            "candidate_backend": DEFAULT_CLASS_PORTABILITY_HOOK,
            "inputs": [
                "source_project",
                "source_project_name",
                "destination_project_name",
                "modified_class",
                "behavior_selector",
                "evidence_dir"
            ],
            "outputs": [
                "exported_class_package",
                "import_report",
                "save_reopen_report",
                "post_import_behavior"
            ],
            "unsafe_until_available": !proven
        }
    })]
}

fn validate_hook_result(result: &ClassPortabilityHookResult) -> Vec<String> {
    let mut errors = Vec::new();
    if result.schema_version != "eatme.alice-class-portability/v1" {
        errors.push("schema_version must be eatme.alice-class-portability/v1".into());
    }
    if result.status != "passed" {
        errors.push("status must be passed".into());
    }
    if result.source_project != SOURCE_PROJECT_NAME {
        errors.push(format!("source_project must be {SOURCE_PROJECT_NAME}"));
    }
    if result.destination_project != DESTINATION_PROJECT_NAME {
        errors.push(format!(
            "destination_project must be {DESTINATION_PROJECT_NAME}"
        ));
    }
    if result.modified_class != MODIFIED_CLASS_NAME {
        errors.push(format!("modified_class must be {MODIFIED_CLASS_NAME}"));
    }
    if result.behavior_selector != BEHAVIOR_SELECTOR {
        errors.push(format!("behavior_selector must be {BEHAVIOR_SELECTOR}"));
    }
    for (field, value) in [
        ("exported_class_package", &result.exported_class_package),
        ("import_report", &result.import_report),
        ("save_reopen_report", &result.save_reopen_report),
        ("post_import_behavior", &result.post_import_behavior),
    ] {
        if value.trim().is_empty() {
            errors.push(format!("{field} must not be empty"));
        }
    }
    errors
}

fn blocked_probe(
    hook_path: &Path,
    source_project_path: PathBuf,
    detail: &str,
) -> DesktopClassPortabilityProbe {
    DesktopClassPortabilityProbe {
        id: "alice-side-class-portability-command-hook".into(),
        action_id: "modified-class-export-import-save-reopen".into(),
        status: "blocked".into(),
        detail: detail.into(),
        source_project: SOURCE_PROJECT_NAME.into(),
        destination_project: DESTINATION_PROJECT_NAME.into(),
        modified_class: MODIFIED_CLASS_NAME.into(),
        behavior_selector: BEHAVIOR_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command: Some(format!(
            "{} --source-project {} --source-project-name {} --destination-project-name {} --modified-class {} --behavior-selector {} --evidence-dir portability --json",
            hook_path.display(),
            source_project_path.display(),
            SOURCE_PROJECT_NAME,
            DESTINATION_PROJECT_NAME,
            MODIFIED_CLASS_NAME,
            BEHAVIOR_SELECTOR
        )),
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
        exported_class_package: None,
        import_report: None,
        save_reopen_report: None,
        post_import_behavior: None,
        validation_errors: Vec::new(),
        missing_affordance: Some(missing_affordance()),
    }
}

fn failed_probe(
    hook_path: &Path,
    _source_project_path: PathBuf,
    command: Option<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    validation_errors: Vec<String>,
) -> DesktopClassPortabilityProbe {
    DesktopClassPortabilityProbe {
        id: "alice-side-class-portability-command-hook".into(),
        action_id: "modified-class-export-import-save-reopen".into(),
        status: "failed".into(),
        detail: format!(
            "desktop class portability evidence is invalid: {}",
            validation_errors.join("; ")
        ),
        source_project: SOURCE_PROJECT_NAME.into(),
        destination_project: DESTINATION_PROJECT_NAME.into(),
        modified_class: MODIFIED_CLASS_NAME.into(),
        behavior_selector: BEHAVIOR_SELECTOR.into(),
        candidate_hook_path: hook_path.display().to_string(),
        command,
        exit_status,
        stdout,
        stderr,
        exported_class_package: None,
        import_report: None,
        save_reopen_report: None,
        post_import_behavior: None,
        validation_errors,
        missing_affordance: Some(missing_affordance()),
    }
}

fn missing_affordance() -> ClassPortabilityMissingAffordance {
    ClassPortabilityMissingAffordance {
        id: "deterministic-alice-class-portability-affordance".into(),
        summary: "Need a desktop Alice backend that modifies a class, exports it, imports it into a different project, saves/reopens the destination, and records behavior evidence.".into(),
        next_implementation: format!(
            "Implement {DEFAULT_CLASS_PORTABILITY_HOOK} in the Alice checkout with the declared inputs and artifacts before claiming desktop class portability."
        ),
        required_backend: DEFAULT_CLASS_PORTABILITY_HOOK.into(),
    }
}

fn resolve_starter_project(alice_home: &Path, starter_project: &Path) -> PathBuf {
    if starter_project.is_absolute() {
        starter_project.to_path_buf()
    } else {
        alice_home.join(starter_project)
    }
}
