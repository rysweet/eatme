use super::{
    LessonSessionContractCheck, check_lesson_session_contract,
    first_lesson::FIRST_LESSON_SCENARIO_ID,
    ui_action_contract::{action_ids, inspect_ui_action_contract},
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

const REQUIRED_FIRST_LESSON_ASSERTIONS: &[&str] = &[
    "real_alice_execution_evidence",
    "specific_alice_window_detected",
    "activate_alice_window_ui_action",
    "place_object_candidate_hook_probe",
    "place_object_ui_action",
    "edit_procedure_ui_action",
    "run_world_ui_action",
    "save_project_ui_action",
    "ui_action_artifact_captured",
];

const REQUIRED_UI_ACTION_IDS: &[&str] = &[
    "verify-specific-alice-window",
    "activate-specific-alice-window",
    "place-object",
    "edit-procedure-or-code-block",
    "run-world",
    "save-project",
];

const UI_ACTION_BLOCKED_FAILURE_CATEGORIES: &[&str] = &[
    "ui_action_automation_unimplemented",
    "ui_action_remaining_steps_unimplemented",
];

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionReadinessReport {
    pub schema_version: String,
    pub manifest_path: String,
    pub scenario_id: Option<String>,
    pub passed: bool,
    pub readiness_status: String,
    pub contract_check: LessonSessionContractCheck,
    pub execute_requested: Option<bool>,
    pub target_evidence: Vec<LessonTargetEvidence>,
    pub issues: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LessonTargetEvidence {
    pub role: String,
    pub target_id: Option<String>,
    pub target_status: Option<String>,
    pub failure_category: Option<String>,
    pub launch_manifest_present: bool,
    pub ui_action_contract_path: Option<String>,
    pub ui_action_contract_readable: bool,
    pub required_actions: Vec<String>,
    pub missing_assertions: Vec<String>,
    pub missing_required_actions: Vec<String>,
}

pub fn check_lesson_session_readiness(
    manifest_path: &Path,
) -> Result<LessonSessionReadinessReport> {
    let text = fs::read_to_string(manifest_path)
        .with_context(|| format!("reading comparison manifest {}", manifest_path.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing comparison manifest {}", manifest_path.display()))?;
    let contract_check = check_lesson_session_contract(manifest_path)?;
    let scenario_id = manifest
        .get("scenario_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let execute_requested = manifest
        .get("execute_requested")
        .and_then(serde_json::Value::as_bool);
    let mut issues = contract_check.issues.clone();
    let mut target_evidence = Vec::new();

    if contract_check.session_kind.as_deref() != Some("first_lesson_action_contract") {
        issues.push(
            "lesson readiness evidence is only defined for first_lesson_action_contract".into(),
        );
    }
    if scenario_id.as_deref() != Some(FIRST_LESSON_SCENARIO_ID) {
        issues.push(format!(
            "comparison manifest scenario_id must be {FIRST_LESSON_SCENARIO_ID:?}"
        ));
    }
    if execute_requested != Some(true) {
        issues.push(
            "comparison manifest must be produced with --execute to contain target launch evidence"
                .into(),
        );
    }

    let targets = manifest
        .get("targets")
        .and_then(serde_json::Value::as_object);
    for role in ["baseline", "modernized"] {
        match targets.and_then(|entries| entries.get(role)) {
            Some(target) => {
                let evidence = inspect_target_evidence(manifest_path, role, target, &mut issues);
                target_evidence.push(evidence);
            }
            None => issues.push(format!(
                "comparison manifest is missing {role} target evidence"
            )),
        }
    }

    let readiness_status = if !issues.is_empty() {
        "incomplete"
    } else if target_evidence.iter().any(|target| {
        target
            .failure_category
            .as_deref()
            .is_some_and(is_ui_action_blocked_category)
    }) {
        "blocked_until_ui_automation"
    } else {
        "ready"
    }
    .to_string();

    let limitations = vec![
        "does not automate complete instructor assignment creation".into(),
        "does not automate complete student lesson consumption".into(),
        "does not perform creative assessment".into(),
        "does not grade student worlds".into(),
        "does not prove broad Alice compatibility beyond the selected scenario".into(),
    ];

    Ok(LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: manifest_path.display().to_string(),
        scenario_id,
        passed: issues.is_empty(),
        readiness_status,
        contract_check,
        execute_requested,
        target_evidence,
        issues,
        limitations,
    })
}

fn inspect_target_evidence(
    manifest_path: &Path,
    role: &str,
    target: &serde_json::Value,
    issues: &mut Vec<String>,
) -> LessonTargetEvidence {
    let target_id = string_field(target, "target_id");
    let target_status = string_field(target, "status");
    let failure_category = string_field(target, "failure_category");
    let launch_manifest = target
        .get("launch_manifest")
        .filter(|value| !value.is_null());
    let Some(launch_manifest) = launch_manifest else {
        issues.push(format!("{role} target is missing embedded launch_manifest"));
        return LessonTargetEvidence {
            role: role.into(),
            target_id,
            target_status,
            failure_category,
            launch_manifest_present: false,
            ui_action_contract_path: None,
            ui_action_contract_readable: false,
            required_actions: Vec::new(),
            missing_assertions: REQUIRED_FIRST_LESSON_ASSERTIONS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            missing_required_actions: REQUIRED_UI_ACTION_IDS
                .iter()
                .map(|value| (*value).into())
                .collect(),
        };
    };

    if launch_manifest
        .get("scenario_id")
        .and_then(serde_json::Value::as_str)
        != Some(FIRST_LESSON_SCENARIO_ID)
    {
        issues.push(format!(
            "{role} launch_manifest scenario_id must be {FIRST_LESSON_SCENARIO_ID:?}"
        ));
    }
    if !failure_category
        .as_deref()
        .is_some_and(is_ui_action_blocked_category)
    {
        issues.push(format!(
            "{role} target must be blocked only by a known UI action category until deterministic UI actions exist"
        ));
    }
    if !launch_manifest
        .get("failure_category")
        .and_then(serde_json::Value::as_str)
        .is_some_and(is_ui_action_blocked_category)
    {
        issues.push(format!(
            "{role} launch_manifest failure_category must be a known UI action blocker"
        ));
    }

    let missing_assertions = missing_launch_assertions(launch_manifest);
    for assertion in &missing_assertions {
        issues.push(format!(
            "{role} launch_manifest is missing assertion {assertion:?}"
        ));
    }
    require_passed_assertion(
        issues,
        role,
        launch_manifest,
        "real_alice_execution_evidence",
    );
    require_passed_assertion(
        issues,
        role,
        launch_manifest,
        "specific_alice_window_detected",
    );
    require_passed_assertion(
        issues,
        role,
        launch_manifest,
        "activate_alice_window_ui_action",
    );
    require_passed_assertion(
        issues,
        role,
        launch_manifest,
        "place_object_candidate_hook_probe",
    );
    if !assertion_passed(launch_manifest, "place_object_ui_action") {
        require_passed_assertion(
            issues,
            role,
            launch_manifest,
            "place_object_precondition_no_go_probe",
        );
    }
    require_passed_assertion(issues, role, launch_manifest, "ui_action_artifact_captured");

    let ui_action_contract_path = launch_manifest
        .get("ui_action_contract")
        .and_then(|artifact| artifact.get("path"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let mut required_actions = Vec::new();
    let mut missing_required_actions = REQUIRED_UI_ACTION_IDS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let mut ui_action_contract_readable = false;

    if let Some(path) = &ui_action_contract_path {
        match resolve_artifact_path(manifest_path, path) {
            Ok(resolved) => match fs::read_to_string(&resolved) {
                Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(contract) => {
                        ui_action_contract_readable = true;
                        inspect_ui_action_contract(role, &contract, issues);
                        required_actions = action_ids(&contract);
                        missing_required_actions = REQUIRED_UI_ACTION_IDS
                            .iter()
                            .filter(|id| !required_actions.iter().any(|action| action == **id))
                            .map(|value| (*value).to_string())
                            .collect();
                        for action in &missing_required_actions {
                            issues.push(format!(
                                "{role} ui-action-contract.json is missing required action {action:?}"
                            ));
                        }
                    }
                    Err(error) => issues.push(format!(
                        "{role} ui-action-contract.json is not valid JSON: {error}"
                    )),
                },
                Err(error) => issues.push(format!(
                    "{role} ui-action-contract.json could not be read at {}: {error}",
                    resolved.display()
                )),
            },
            Err(error) => issues.push(format!("{role} ui-action-contract.path is unsafe: {error}")),
        }
    } else {
        issues.push(format!(
            "{role} launch_manifest is missing ui_action_contract.path"
        ));
    }

    LessonTargetEvidence {
        role: role.into(),
        target_id,
        target_status,
        failure_category,
        launch_manifest_present: true,
        ui_action_contract_path,
        ui_action_contract_readable,
        required_actions,
        missing_assertions,
        missing_required_actions,
    }
}

fn is_ui_action_blocked_category(category: &str) -> bool {
    UI_ACTION_BLOCKED_FAILURE_CATEGORIES.contains(&category)
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn missing_launch_assertions(launch_manifest: &serde_json::Value) -> Vec<String> {
    let assertions = launch_manifest
        .get("assertions")
        .and_then(serde_json::Value::as_object);
    REQUIRED_FIRST_LESSON_ASSERTIONS
        .iter()
        .filter(|assertion| {
            assertions
                .map(|entries| !entries.contains_key(**assertion))
                .unwrap_or(true)
        })
        .map(|value| (*value).to_string())
        .collect()
}

fn require_passed_assertion(
    issues: &mut Vec<String>,
    role: &str,
    launch_manifest: &serde_json::Value,
    assertion: &str,
) {
    let passed = launch_manifest
        .get("assertions")
        .and_then(|assertions| assertions.get(assertion))
        .and_then(|entry| entry.get("passed"))
        .and_then(serde_json::Value::as_bool);
    if passed != Some(true) {
        issues.push(format!(
            "{role} launch_manifest assertion {assertion:?} must pass before first-lesson readiness is evidence-ready"
        ));
    }
}

fn assertion_passed(launch_manifest: &serde_json::Value, assertion: &str) -> bool {
    launch_manifest
        .get("assertions")
        .and_then(|assertions| assertions.get(assertion))
        .and_then(|entry| entry.get("passed"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

fn resolve_artifact_path(manifest_path: &Path, artifact_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(artifact_path);
    if path.as_os_str().is_empty() {
        bail!("artifact path must not be empty");
    }
    let evidence_root = comparison_evidence_root(manifest_path);

    if path.is_absolute() {
        let artifact = path
            .canonicalize()
            .with_context(|| format!("resolving artifact path {}", path.display()))?;
        let root = evidence_root.canonicalize().with_context(|| {
            format!(
                "resolving comparison evidence root {}",
                evidence_root.display()
            )
        })?;
        if !artifact.starts_with(&root) {
            bail!(
                "absolute artifact path {} must stay under comparison evidence root {}",
                artifact.display(),
                root.display()
            );
        }
        return Ok(artifact);
    }

    reject_unsafe_relative_path(&path)?;
    let root = canonical_evidence_root(&evidence_root)?;
    if let Some(parent) = manifest_path.parent() {
        let candidate = parent.join(&path);
        if candidate.exists() {
            return canonical_artifact_under_root(&candidate, &root);
        }
    }
    let candidate = evidence_root.join(&path);
    if candidate.exists() {
        return canonical_artifact_under_root(&candidate, &root);
    }
    Ok(candidate)
}

fn comparison_evidence_root(manifest_path: &Path) -> PathBuf {
    let parent = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    for ancestor in parent.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("comparisons") {
            return ancestor.parent().unwrap_or(parent).to_path_buf();
        }
    }
    parent.to_path_buf()
}

fn reject_unsafe_relative_path(path: &Path) -> Result<()> {
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("relative artifact path must not contain parent, current, or root components");
    }
    Ok(())
}

fn canonical_evidence_root(evidence_root: &Path) -> Result<PathBuf> {
    evidence_root.canonicalize().with_context(|| {
        format!(
            "resolving comparison evidence root {}",
            evidence_root.display()
        )
    })
}

fn canonical_artifact_under_root(candidate: &Path, root: &Path) -> Result<PathBuf> {
    let resolved = candidate
        .canonicalize()
        .with_context(|| format!("resolving artifact path {}", candidate.display()))?;
    if !resolved.starts_with(root) {
        bail!(
            "artifact path {} must stay under comparison evidence root {}",
            resolved.display(),
            root.display()
        );
    }
    Ok(resolved)
}
