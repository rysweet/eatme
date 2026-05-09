use super::{
    LessonSessionReadinessReport, REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID,
    desktop_proof::DesktopProofContract,
    output::{
        LessonTargetEvidence, ReadinessOutput, build_launch_smoke_readiness_output,
        launch_smoke_limitations, launch_smoke_unproven_claims, not_yet_shown, shown_evidence,
    },
    progress::LessonReadinessEvidenceProgress,
    string_field,
};
use crate::compare::LessonSessionContractCheck;
use anyhow::Result;
use std::path::Path;

mod evidence_progress;
use evidence_progress::launch_smoke_evidence_progress;

const REQUIRED_LAUNCH_SMOKE_ASSERTIONS: &[&str] = &[
    "display_responsive",
    "process_started",
    "startup_screenshot",
    "no_fatal_logs",
    "real_alice_execution_evidence",
];

struct LaunchSmokeTargets {
    evidence: Vec<LessonTargetEvidence>,
    role_statuses: Vec<(&'static str, &'static str)>,
    issues: Vec<String>,
}

struct LaunchSmokeReportParts<'a> {
    manifest_path: &'a Path,
    scenario_id: Option<String>,
    contract_check: LessonSessionContractCheck,
    execute_requested: Option<bool>,
    issues: Vec<String>,
    target_evidence: Vec<LessonTargetEvidence>,
    readiness_status: String,
    readiness_output: ReadinessOutput,
    evidence_progress: LessonReadinessEvidenceProgress,
}

pub(super) fn check_launch_smoke_readiness(
    manifest_path: &Path,
    manifest: &serde_json::Value,
    contract_check: LessonSessionContractCheck,
    scenario_id: Option<String>,
    execute_requested: Option<bool>,
    mut issues: Vec<String>,
) -> Result<LessonSessionReadinessReport> {
    push_execute_requested_issue(execute_requested, &mut issues);

    let targets = manifest
        .get("targets")
        .and_then(serde_json::Value::as_object);
    let inspected_targets = inspect_required_launch_smoke_targets(targets);
    issues.extend(inspected_targets.issues);

    let readiness_status = if issues.is_empty() {
        "ready"
    } else {
        "incomplete"
    }
    .to_string();
    let readiness_output = build_launch_smoke_readiness_output(
        scenario_id.as_deref(),
        &readiness_status,
        &inspected_targets.role_statuses,
    );
    let evidence_progress = launch_smoke_evidence_progress(
        &readiness_output.required_evidence,
        &inspected_targets.evidence,
        &issues,
    );

    Ok(launch_smoke_readiness_report(LaunchSmokeReportParts {
        manifest_path,
        scenario_id,
        contract_check,
        execute_requested,
        issues,
        target_evidence: inspected_targets.evidence,
        readiness_status,
        readiness_output,
        evidence_progress,
    }))
}

fn push_execute_requested_issue(execute_requested: Option<bool>, issues: &mut Vec<String>) {
    if execute_requested != Some(true) {
        issues.push(
            "comparison manifest must be produced with --execute to contain target launch-smoke manifest evidence"
                .into(),
        );
    }
}

fn launch_smoke_readiness_report(
    parts: LaunchSmokeReportParts<'_>,
) -> LessonSessionReadinessReport {
    let LaunchSmokeReportParts {
        manifest_path,
        scenario_id,
        contract_check,
        execute_requested,
        issues,
        target_evidence,
        readiness_status,
        readiness_output,
        evidence_progress,
    } = parts;
    let shown_evidence = shown_evidence(&evidence_progress, &[]);
    let not_yet_shown = not_yet_shown(&evidence_progress, &[]);
    let desktop_proof_contract = launch_smoke_desktop_proof_contract(&target_evidence, &issues);

    LessonSessionReadinessReport {
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
        desktop_next_action: None,
        unproven_claims: launch_smoke_unproven_claims(),
        evidence_progress,
        evidence_boundaries: Vec::new(),
        required_evidence: readiness_output.required_evidence,
        no_go_contracts: readiness_output.no_go_contracts,
        lesson_session_readiness: readiness_output.lesson_session_readiness,
        role_readiness: readiness_output.role_readiness,
        contract_check,
        execute_requested,
        target_evidence,
        issues,
        limitations: launch_smoke_limitations(),
    }
}

fn inspect_required_launch_smoke_targets(
    targets: Option<&serde_json::Map<String, serde_json::Value>>,
) -> LaunchSmokeTargets {
    let mut inspected = LaunchSmokeTargets {
        evidence: Vec::new(),
        role_statuses: Vec::new(),
        issues: Vec::new(),
    };

    for role in ["baseline", "modernized"] {
        let Some(target) = targets.and_then(|entries| entries.get(role)) else {
            inspected.issues.push(format!(
                "comparison manifest is missing {role} target evidence"
            ));
            inspected.role_statuses.push((role, "not_ready"));
            continue;
        };

        let mut role_issues = Vec::new();
        inspected
            .evidence
            .push(inspect_launch_smoke_target_evidence(
                role,
                target,
                &mut role_issues,
            ));
        inspected
            .role_statuses
            .push((role, role_status(&role_issues)));
        inspected.issues.extend(role_issues);
    }

    inspected
}

fn role_status(issues: &[String]) -> &'static str {
    if issues.is_empty() {
        "ready"
    } else {
        "not_ready"
    }
}

fn inspect_launch_smoke_target_evidence(
    role: &str,
    target: &serde_json::Value,
    issues: &mut Vec<String>,
) -> LessonTargetEvidence {
    let target_id = string_field(target, "target_id");
    let target_status = string_field(target, "status");
    let failure_category = string_field(target, "failure_category");
    push_target_status_issues(role, target, target_status.as_deref(), issues);

    let launch_manifest = target
        .get("launch_manifest")
        .filter(|value| !value.is_null());
    let Some(launch_manifest) = launch_manifest else {
        issues.push(format!("{role} target is missing embedded launch_manifest"));
        return missing_launch_smoke_target_evidence(
            role,
            target_id,
            target_status,
            failure_category,
        );
    };

    push_launch_manifest_identity_issues(role, launch_manifest, issues);
    let missing_assertions = missing_launch_smoke_assertions(role, launch_manifest, issues);
    push_missing_artifact_metadata_issues(role, launch_manifest, issues);

    present_launch_smoke_target_evidence(
        role,
        target_id,
        target_status,
        failure_category,
        missing_assertions,
    )
}

fn push_target_status_issues(
    role: &str,
    target: &serde_json::Value,
    target_status: Option<&str>,
    issues: &mut Vec<String>,
) {
    if target_status != Some("passed") {
        issues.push(format!("{role} target status must be passed"));
    }
    if field_is_non_null(target, "failure_category") {
        issues.push(format!("{role} target failure_category must be null"));
    }
}

fn push_launch_manifest_identity_issues(
    role: &str,
    launch_manifest: &serde_json::Value,
    issues: &mut Vec<String>,
) {
    let scenario_id = launch_manifest
        .get("scenario_id")
        .and_then(serde_json::Value::as_str);
    if scenario_id != Some(REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID) {
        issues.push(format!(
            "{role} launch_manifest scenario_id must be {REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID:?}"
        ));
    }
    if field_is_non_null(launch_manifest, "failure_category") {
        issues.push(format!(
            "{role} launch_manifest failure_category must be null"
        ));
    }
}

fn missing_launch_smoke_assertions(
    role: &str,
    launch_manifest: &serde_json::Value,
    issues: &mut Vec<String>,
) -> Vec<String> {
    let mut missing = Vec::new();
    for assertion in REQUIRED_LAUNCH_SMOKE_ASSERTIONS {
        if !launch_smoke_assertion_passed(launch_manifest, assertion) {
            missing.push((*assertion).to_string());
            issues.push(format!(
                "{role} launch_manifest assertion {assertion:?} must pass"
            ));
        }
    }
    missing
}

fn push_missing_artifact_metadata_issues(
    role: &str,
    launch_manifest: &serde_json::Value,
    issues: &mut Vec<String>,
) {
    for artifact in ["window_list", "screenshot", "log"] {
        if !artifact_metadata_present(launch_manifest, artifact) {
            issues.push(format!(
                "{role} launch_manifest {artifact} metadata must be present"
            ));
        }
    }
}

fn missing_launch_smoke_target_evidence(
    role: &str,
    target_id: Option<String>,
    target_status: Option<String>,
    failure_category: Option<String>,
) -> LessonTargetEvidence {
    LessonTargetEvidence {
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
        missing_assertions: REQUIRED_LAUNCH_SMOKE_ASSERTIONS
            .iter()
            .map(|value| (*value).into())
            .collect(),
        missing_required_actions: Vec::new(),
        blockers: Vec::new(),
        no_go_contracts: Vec::new(),
    }
}

fn present_launch_smoke_target_evidence(
    role: &str,
    target_id: Option<String>,
    target_status: Option<String>,
    failure_category: Option<String>,
    missing_assertions: Vec<String>,
) -> LessonTargetEvidence {
    LessonTargetEvidence {
        role: role.into(),
        target_id,
        target_status,
        failure_category,
        launch_manifest_present: true,
        ui_action_contract_path: None,
        ui_action_contract_readable: false,
        desktop_run_pixel_boundary: None,
        desktop_run_pixel_observation: None,
        desktop_first_lesson_next_action: None,
        action_assertions: Vec::new(),
        required_actions: Vec::new(),
        missing_assertions,
        missing_required_actions: Vec::new(),
        blockers: Vec::new(),
        no_go_contracts: Vec::new(),
    }
}

fn launch_smoke_assertion_passed(launch_manifest: &serde_json::Value, assertion: &str) -> bool {
    launch_manifest
        .get("assertions")
        .and_then(|assertions| assertions.get(assertion))
        .and_then(|entry| entry.get("passed"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

fn artifact_metadata_present(launch_manifest: &serde_json::Value, artifact: &str) -> bool {
    launch_manifest
        .get(artifact)
        .and_then(serde_json::Value::as_object)
        .and_then(|metadata| metadata.get("path"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|path| !path.trim().is_empty())
}

fn field_is_non_null(value: &serde_json::Value, field: &str) -> bool {
    value
        .get(field)
        .is_some_and(|field_value| !field_value.is_null())
}

fn launch_smoke_desktop_proof_contract(
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> DesktopProofContract {
    if issues.is_empty() {
        DesktopProofContract {
            status: "verified".into(),
            reason_code: "launch_smoke_manifest_ready".into(),
            detail: "launch-smoke readiness is mapped from embedded manifest, assertion, window-list, screenshot, and log metadata only; lesson completion is not proven".into(),
            target_role: "modernized".into(),
            artifact: None,
        }
    } else if target_evidence
        .iter()
        .any(|target| !target.launch_manifest_present)
    {
        DesktopProofContract {
            status: "unsupported_environment".into(),
            reason_code: "launch_smoke_manifest_missing".into(),
            detail: "one or more target launch-smoke manifests are missing; readiness remains bounded to manifest metadata".into(),
            target_role: "modernized".into(),
            artifact: None,
        }
    } else {
        DesktopProofContract {
            status: "launched_but_unverified".into(),
            reason_code: "launch_smoke_manifest_incomplete".into(),
            detail: "launch-smoke manifest metadata is missing, failed, malformed, or incomplete; readiness remains bounded to manifest metadata".into(),
            target_role: "modernized".into(),
            artifact: None,
        }
    }
}
