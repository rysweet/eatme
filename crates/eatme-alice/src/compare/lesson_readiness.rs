use super::{
    LessonSessionContractCheck, check_lesson_session_contract,
    desktop_evidence::{
        DesktopRunPixelBoundaryEvidence, check_pixel_boundary_evidence,
        check_visible_desktop_evidence, comparison_evidence_root, resolve_artifact_path,
    },
    first_lesson::FIRST_LESSON_SCENARIO_ID,
    ui_action_contract::{action_ids, inspect_ui_action_contract},
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

mod assertions;
mod no_go;
mod output;
mod progress;
pub use assertions::LessonActionAssertionEvidence;
use assertions::{
    action_assertions, assertion_passed, missing_launch_assertions, require_passed_assertion,
};
pub use no_go::LessonSessionNoGoContract;
use no_go::ui_action_no_go_contracts;
pub use output::LessonSessionReadinessEnvelope;
use output::build_readiness_output;
use progress::evidence_progress;
pub use progress::{LessonReadinessEvidenceProgress, LessonReadinessEvidenceProgressItem};

const REQUIRED_FIRST_LESSON_ASSERTIONS: &[&str] = &[
    "real_alice_execution_evidence",
    "specific_alice_window_detected",
    "activate_alice_window_ui_action",
    "save_project_desktop_shortcut_dispatch",
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

const REQUIRED_MODERNIZED_DESKTOP_ASSERTIONS: &[&str] = &[
    "run_world_desktop_toolbar_window_observed",
    "run_world_desktop_execution_observed",
];

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionReadinessReport {
    pub schema_version: String,
    pub manifest_path: String,
    pub scenario_id: Option<String>,
    pub passed: bool,
    pub status: String,
    pub readiness_status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub evidence_progress: LessonReadinessEvidenceProgress,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
    pub lesson_session_readiness: LessonSessionReadinessEnvelope,
    pub role_readiness: Vec<LessonSessionReadinessEnvelope>,
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
    pub desktop_run_pixel_boundary: Option<DesktopRunPixelBoundaryEvidence>,
    pub action_assertions: Vec<LessonActionAssertionEvidence>,
    pub required_actions: Vec<String>,
    pub missing_assertions: Vec<String>,
    pub missing_required_actions: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
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
    let no_go_contracts = target_evidence
        .iter()
        .flat_map(|target| target.no_go_contracts.clone())
        .collect::<Vec<_>>();
    let readiness_output = build_readiness_output(
        scenario_id.as_deref(),
        &readiness_status,
        !issues.is_empty(),
        no_go_contracts,
        FIRST_LESSON_SCENARIO_ID,
    );
    let evidence_progress = evidence_progress(
        &readiness_output.required_evidence,
        &target_evidence,
        &issues,
    );

    let limitations = vec![
        "does not prove full Alice UI automation".into(),
        "does not automate complete instructor assignment creation".into(),
        "does not automate complete student lesson consumption".into(),
        "does not perform creative assessment".into(),
        "does not grade student worlds".into(),
        "does not prove visible rendering correctness".into(),
        "does not prove first-lesson completion".into(),
        "does not prove broad Alice compatibility beyond the selected scenario".into(),
    ];
    Ok(LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: manifest_path.display().to_string(),
        scenario_id,
        passed: issues.is_empty(),
        status: readiness_output.status,
        readiness_status,
        blocked_reason: readiness_output.blocked_reason,
        human_summary: readiness_output.human_summary,
        evidence_progress,
        required_evidence: readiness_output.required_evidence,
        no_go_contracts: readiness_output.no_go_contracts,
        lesson_session_readiness: readiness_output.lesson_session_readiness,
        role_readiness: readiness_output.role_readiness,
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
            desktop_run_pixel_boundary: None,
            action_assertions: Vec::new(),
            required_actions: Vec::new(),
            missing_assertions: REQUIRED_FIRST_LESSON_ASSERTIONS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            missing_required_actions: REQUIRED_UI_ACTION_IDS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            no_go_contracts: Vec::new(),
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
    let action_assertions = action_assertions(launch_manifest);
    for assertion in &missing_assertions {
        issues.push(format!(
            "{role} launch_manifest is missing assertion {assertion:?}"
        ));
    }
    for assertion in [
        "real_alice_execution_evidence",
        "specific_alice_window_detected",
        "activate_alice_window_ui_action",
        "save_project_desktop_shortcut_dispatch",
    ] {
        require_passed_assertion(issues, role, launch_manifest, assertion);
    }
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
    if role == "modernized" {
        for assertion in REQUIRED_MODERNIZED_DESKTOP_ASSERTIONS {
            require_passed_assertion(issues, role, launch_manifest, assertion);
        }
    }

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
    let mut desktop_run_pixel_boundary = None;
    let mut no_go_contracts = Vec::new();

    if let Some(path) = &ui_action_contract_path {
        match resolve_artifact_path(manifest_path, path) {
            Ok(resolved) => match fs::read_to_string(&resolved) {
                Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(contract) => {
                        ui_action_contract_readable = true;
                        inspect_ui_action_contract(role, &contract, issues);
                        no_go_contracts = ui_action_no_go_contracts(role, &contract);
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
                        if role == "modernized" {
                            let pixel_boundary = check_pixel_boundary_evidence(
                                &comparison_evidence_root(manifest_path),
                                &resolved,
                            );
                            issues.extend(pixel_boundary.issue_when_missing_or_invalid());
                            desktop_run_pixel_boundary = Some(pixel_boundary);
                            issues.extend(
                                check_visible_desktop_evidence(
                                    &comparison_evidence_root(manifest_path),
                                    &resolved,
                                )
                                .issue_when_missing(),
                            );
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
        desktop_run_pixel_boundary,
        action_assertions,
        required_actions,
        missing_assertions,
        missing_required_actions,
        no_go_contracts,
    }
}

fn is_ui_action_blocked_category(category: &str) -> bool {
    matches!(
        category,
        "ui_action_automation_unimplemented" | "ui_action_remaining_steps_unimplemented"
    )
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}
