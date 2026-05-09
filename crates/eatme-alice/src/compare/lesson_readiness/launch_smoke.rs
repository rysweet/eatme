use super::{
    LessonSessionReadinessReport, REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID,
    desktop_proof::DesktopProofContract,
    output::{
        LessonTargetEvidence, build_launch_smoke_readiness_output, launch_smoke_limitations,
        launch_smoke_unproven_claims, not_yet_shown, shown_evidence,
    },
    progress::{
        LessonReadinessEvidenceProgress, LessonReadinessEvidenceProgressItem, progress_item,
    },
    string_field,
};
use crate::compare::LessonSessionContractCheck;
use anyhow::Result;
use std::path::Path;

const REQUIRED_LAUNCH_SMOKE_ASSERTIONS: &[&str] = &[
    "display_responsive",
    "process_started",
    "startup_screenshot",
    "no_fatal_logs",
    "real_alice_execution_evidence",
];

pub(super) fn check_launch_smoke_readiness(
    manifest_path: &Path,
    manifest: &serde_json::Value,
    contract_check: LessonSessionContractCheck,
    scenario_id: Option<String>,
    execute_requested: Option<bool>,
    mut issues: Vec<String>,
) -> Result<LessonSessionReadinessReport> {
    if execute_requested != Some(true) {
        issues.push(
            "comparison manifest must be produced with --execute to contain target launch-smoke manifest evidence"
                .into(),
        );
    }

    let targets = manifest
        .get("targets")
        .and_then(serde_json::Value::as_object);
    let mut target_evidence = Vec::new();
    let mut role_statuses = Vec::new();

    for role in ["baseline", "modernized"] {
        let mut role_issues = Vec::new();
        match targets.and_then(|entries| entries.get(role)) {
            Some(target) => {
                let evidence = inspect_launch_smoke_target_evidence(role, target, &mut role_issues);
                let role_status = if role_issues.is_empty() {
                    "ready"
                } else {
                    "not_ready"
                };
                target_evidence.push(evidence);
                role_statuses.push((role, role_status));
                issues.extend(role_issues);
            }
            None => {
                issues.push(format!(
                    "comparison manifest is missing {role} target evidence"
                ));
                role_statuses.push((role, "not_ready"));
            }
        }
    }

    let readiness_status = if issues.is_empty() {
        "ready"
    } else {
        "incomplete"
    }
    .to_string();
    let readiness_output = build_launch_smoke_readiness_output(
        scenario_id.as_deref(),
        &readiness_status,
        &role_statuses,
    );
    let evidence_progress = launch_smoke_evidence_progress(
        &readiness_output.required_evidence,
        &target_evidence,
        &issues,
    );
    let shown_evidence = shown_evidence(&evidence_progress, &[]);
    let not_yet_shown = not_yet_shown(&evidence_progress, &[]);

    Ok(LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: manifest_path.display().to_string(),
        scenario_id,
        passed: issues.is_empty(),
        status: readiness_output.status,
        readiness_status,
        blocked_reason: readiness_output.blocked_reason,
        human_summary: readiness_output.human_summary,
        desktop_proof_contract: launch_smoke_desktop_proof_contract(&target_evidence, &issues),
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
    })
}

fn inspect_launch_smoke_target_evidence(
    role: &str,
    target: &serde_json::Value,
    issues: &mut Vec<String>,
) -> LessonTargetEvidence {
    let target_id = string_field(target, "target_id");
    let target_status = string_field(target, "status");
    let failure_category = string_field(target, "failure_category");
    if target_status.as_deref() != Some("passed") {
        issues.push(format!("{role} target status must be passed"));
    }
    if field_is_non_null(target, "failure_category") {
        issues.push(format!("{role} target failure_category must be null"));
    }

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
            missing_assertions: REQUIRED_LAUNCH_SMOKE_ASSERTIONS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            missing_required_actions: Vec::new(),
            blockers: Vec::new(),
            no_go_contracts: Vec::new(),
        };
    };

    if launch_manifest
        .get("scenario_id")
        .and_then(serde_json::Value::as_str)
        != Some(REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID)
    {
        issues.push(format!(
            "{role} launch_manifest scenario_id must be {REAL_ALICE_LAUNCH_SMOKE_SCENARIO_ID:?}"
        ));
    }
    if field_is_non_null(launch_manifest, "failure_category") {
        issues.push(format!(
            "{role} launch_manifest failure_category must be null"
        ));
    }

    let mut missing_assertions = Vec::new();
    for assertion in REQUIRED_LAUNCH_SMOKE_ASSERTIONS {
        if !launch_smoke_assertion_passed(launch_manifest, assertion) {
            missing_assertions.push((*assertion).to_string());
            issues.push(format!(
                "{role} launch_manifest assertion {assertion:?} must pass"
            ));
        }
    }
    for artifact in ["window_list", "screenshot", "log"] {
        if !artifact_metadata_present(launch_manifest, artifact) {
            issues.push(format!(
                "{role} launch_manifest {artifact} metadata must be present"
            ));
        }
    }

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

fn launch_smoke_evidence_progress(
    required_evidence: &[String],
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> LessonReadinessEvidenceProgress {
    let baseline = target_evidence
        .iter()
        .find(|target| target.role == "baseline");
    let modernized = target_evidence
        .iter()
        .find(|target| target.role == "modernized");
    let targets = [baseline, modernized];
    let items = vec![
        progress_item(
            &required_evidence[0],
            if baseline.is_some() && modernized.is_some() {
                "present"
            } else {
                "missing"
            },
            "baseline and modernized target entries for launch-smoke readiness",
        ),
        progress_item(
            &required_evidence[1],
            if targets
                .into_iter()
                .flatten()
                .all(|target| target.launch_manifest_present)
                && baseline.is_some()
                && modernized.is_some()
            {
                "present"
            } else {
                "missing"
            },
            "embedded launch-smoke manifest metadata for both targets",
        ),
        progress_item(
            &required_evidence[2],
            launch_smoke_target_status_state(&targets, issues),
            "target status and failure-category metadata for both targets",
        ),
        progress_item(
            &required_evidence[3],
            if targets
                .into_iter()
                .flatten()
                .all(|target| target.missing_assertions.is_empty())
                && baseline.is_some()
                && modernized.is_some()
            {
                "present"
            } else {
                "invalid"
            },
            "required launch-smoke assertions for both targets",
        ),
        progress_item(
            &required_evidence[4],
            if issues
                .iter()
                .any(|issue| issue.contains("metadata must be present"))
            {
                "missing"
            } else if baseline.is_some() && modernized.is_some() {
                "present"
            } else {
                "missing"
            },
            "window-list, screenshot, and log artifact metadata only",
        ),
    ];

    let present = count_launch_smoke_state(&items, "present");
    let missing = count_launch_smoke_state(&items, "missing");
    let invalid = count_launch_smoke_state(&items, "invalid");
    let not_observed = count_launch_smoke_state(&items, "not_observed");
    let blocked = count_launch_smoke_state(&items, "blocked");
    let total_required = items.len();
    let summary = format!(
        "{present} of {total_required} required launch-smoke evidence items are present; {missing} missing, {invalid} invalid, {not_observed} not observed, {blocked} blocked."
    );

    LessonReadinessEvidenceProgress {
        total_required,
        present,
        missing,
        invalid,
        not_observed,
        blocked,
        summary,
        next_actionable_blocker: launch_smoke_next_blocker(issues),
        next_missing_real_desktop_proof: None,
        items,
    }
}

fn launch_smoke_target_status_state(
    targets: &[Option<&LessonTargetEvidence>; 2],
    issues: &[String],
) -> &'static str {
    if targets.iter().any(|target| target.is_none()) {
        return "missing";
    }
    if issues.iter().any(|issue| {
        issue.contains("target status must be passed")
            || issue.contains("target failure_category must be null")
            || issue.contains("launch_manifest failure_category must be null")
    }) {
        return "invalid";
    }
    if targets.iter().flatten().all(|target| {
        target.target_status.as_deref() == Some("passed")
            && target.failure_category.is_none()
            && target.launch_manifest_present
    }) {
        "present"
    } else {
        "invalid"
    }
}

fn count_launch_smoke_state(items: &[LessonReadinessEvidenceProgressItem], state: &str) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn launch_smoke_next_blocker(issues: &[String]) -> Option<String> {
    issues
        .first()
        .map(|issue| format!("next launch-smoke readiness evidence gap: {issue}"))
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
