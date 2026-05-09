use super::{
    LessonSessionContractCheck,
    desktop_evidence::{
        FirstLessonEvidenceBoundary, check_first_lesson_next_action_evidence,
        check_pixel_boundary_evidence, check_pixel_observation_evidence,
        check_visible_desktop_evidence, comparison_evidence_root, first_lesson_evidence_boundaries,
        resolve_artifact_path,
    },
    first_lesson::FIRST_LESSON_SCENARIO_ID,
    lesson_session::check_lesson_session_contract_in_manifest,
    ui_action_contract::{action_ids, inspect_ui_action_contract},
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

mod assertions;
mod desktop_proof;
mod no_go;
mod output;
mod progress;
pub use assertions::LessonActionAssertionEvidence;
use assertions::{
    action_assertions, assertion_passed, missing_launch_assertions, require_passed_assertion,
};
pub use desktop_proof::DesktopProofContract;
use desktop_proof::desktop_proof_contract;
pub use no_go::LessonSessionNoGoContract;
use no_go::ui_action_no_go_contracts;
pub use output::{
    DesktopNextActionSummary, LessonSessionReadinessEnvelope, LessonTargetEvidence,
    LessonTargetEvidenceBlocker, ReadinessEvidenceItem,
};
use output::{
    build_readiness_output, desktop_next_action_summary, limitations, not_yet_shown,
    shown_evidence, unproven_claims,
};
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
    pub desktop_proof_contract: DesktopProofContract,
    pub shown_evidence: Vec<ReadinessEvidenceItem>,
    pub not_yet_shown: Vec<ReadinessEvidenceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_next_action: Option<DesktopNextActionSummary>,
    pub unproven_claims: Vec<String>,
    pub evidence_progress: LessonReadinessEvidenceProgress,
    pub evidence_boundaries: Vec<FirstLessonEvidenceBoundary>,
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

pub fn check_lesson_session_readiness(
    manifest_path: &Path,
) -> Result<LessonSessionReadinessReport> {
    let text = fs::read_to_string(manifest_path)
        .with_context(|| format!("reading comparison manifest {}", manifest_path.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing comparison manifest {}", manifest_path.display()))?;
    let contract_check = check_lesson_session_contract_in_manifest(manifest_path, &manifest)?;
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
        .flat_map(|target| target.no_go_contracts.iter().cloned())
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
    let evidence_boundaries = readiness_evidence_boundaries(manifest_path, &target_evidence);
    let desktop_proof_contract =
        desktop_proof_contract(execute_requested, &target_evidence, &issues);
    let shown_evidence = shown_evidence(&evidence_progress, &evidence_boundaries);
    let not_yet_shown = not_yet_shown(&evidence_progress, &evidence_boundaries);
    let desktop_next_action = desktop_next_action_summary(&target_evidence);
    let unproven_claims = unproven_claims();
    Ok(LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: manifest_path.display().to_string(),
        scenario_id,
        passed: issues.is_empty(),
        status: readiness_output.status,
        readiness_status,
        blocked_reason: readiness_output.blocked_reason,
        human_summary: readiness_output.human_summary,
        desktop_proof_contract,
        shown_evidence,
        not_yet_shown,
        desktop_next_action,
        unproven_claims,
        evidence_progress,
        evidence_boundaries,
        required_evidence: readiness_output.required_evidence,
        no_go_contracts: readiness_output.no_go_contracts,
        lesson_session_readiness: readiness_output.lesson_session_readiness,
        role_readiness: readiness_output.role_readiness,
        contract_check,
        execute_requested,
        target_evidence,
        issues,
        limitations: limitations(),
    })
}

fn readiness_evidence_boundaries(
    manifest_path: &Path,
    target_evidence: &[LessonTargetEvidence],
) -> Vec<FirstLessonEvidenceBoundary> {
    if let Some(boundaries) = target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .and_then(|target| target.desktop_first_lesson_next_action.as_ref())
        .map(|next_action| next_action.evidence_boundaries.clone())
    {
        return boundaries;
    }

    let evidence_root = comparison_evidence_root(manifest_path);
    let canonical_root = evidence_root
        .canonicalize()
        .unwrap_or_else(|_| evidence_root.clone());
    first_lesson_evidence_boundaries(&serde_json::Value::Null, &canonical_root, &evidence_root)
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
            desktop_run_pixel_observation: None,
            desktop_first_lesson_next_action: None,
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
            blockers: required_action_evidence_blockers(role, &[]),
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
    let mut desktop_run_pixel_observation = None;
    let mut desktop_first_lesson_next_action = None;
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
                                "{role} automation scenarios are missing required action {action:?}"
                            ));
                        }
                        if role == "modernized" {
                            let evidence_root = comparison_evidence_root(manifest_path);
                            let pixel_boundary =
                                check_pixel_boundary_evidence(&evidence_root, &resolved);
                            issues.extend(pixel_boundary.issue_when_missing_or_invalid());
                            desktop_run_pixel_boundary = Some(pixel_boundary);
                            let pixel_observation =
                                check_pixel_observation_evidence(&evidence_root, &resolved);
                            issues.extend(pixel_observation.issue_when_missing_or_invalid());
                            desktop_run_pixel_observation = Some(pixel_observation);
                            let first_lesson_next_action =
                                check_first_lesson_next_action_evidence(&evidence_root, &resolved);
                            issues.extend(first_lesson_next_action.issue_when_invalid());
                            issues.extend(first_lesson_next_action.boundary_issues());
                            desktop_first_lesson_next_action = Some(first_lesson_next_action);
                            issues.extend(
                                check_visible_desktop_evidence(&evidence_root, &resolved)
                                    .issue_when_missing(),
                            );
                        }
                    }
                    Err(error) => issues.push(format!(
                        "{role} automation scenario action evidence is not valid JSON: {error}"
                    )),
                },
                Err(error) => issues.push(format!(
                    "{role} automation scenario action evidence could not be read at {}: {error}",
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

    let blockers = required_action_evidence_blockers(role, &action_assertions);
    LessonTargetEvidence {
        role: role.into(),
        target_id,
        target_status,
        failure_category,
        launch_manifest_present: true,
        ui_action_contract_path,
        ui_action_contract_readable,
        desktop_run_pixel_boundary,
        desktop_run_pixel_observation,
        desktop_first_lesson_next_action,
        action_assertions,
        required_actions,
        missing_assertions,
        missing_required_actions,
        blockers,
        no_go_contracts,
    }
}

fn required_action_evidence_blockers(
    role: &str,
    action_assertions: &[LessonActionAssertionEvidence],
) -> Vec<LessonTargetEvidenceBlocker> {
    if !is_original_alice_role(role) {
        return Vec::new();
    }

    REQUIRED_UI_ACTION_IDS
        .iter()
        .filter_map(|action_id| {
            let action = action_assertions
                .iter()
                .find(|action| action.action_id == *action_id);
            let reason = match action {
                Some(action) if action.passed => return None,
                Some(_) => "Required original Alice action evidence from automation scenarios did not pass.",
                None => "Required original Alice action evidence is missing from automation scenarios.",
            };
            Some(LessonTargetEvidenceBlocker {
                code: "missing_real_action_evidence",
                action: (*action_id).to_string(),
                reason: reason.into(),
            })
        })
        .collect()
}

fn is_ui_action_blocked_category(category: &str) -> bool {
    matches!(
        category,
        "alice_window_not_detected"
            | "alice_like_window_not_main"
            | "alice_window_activation_unsupported"
            | "alice_window_activation_failed"
            | "ui_action_automation_unimplemented"
            | "ui_action_remaining_steps_unimplemented"
    )
}

fn is_original_alice_role(role: &str) -> bool {
    role == "baseline" || role == "original Alice"
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}
