use super::LessonTargetEvidence;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ContractEvidenceItem {
    pub id: String,
    pub state: String,
    pub required: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContractDiagnostic {
    pub code: String,
    pub severity: String,
    pub field: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

pub(crate) fn evidence_item(
    id: impl Into<String>,
    state: impl Into<String>,
    summary: impl Into<String>,
) -> ContractEvidenceItem {
    ContractEvidenceItem {
        id: id.into(),
        state: state.into(),
        required: true,
        summary: summary.into(),
    }
}

pub(crate) fn error_diagnostic(
    code: impl Into<String>,
    field: impl Into<String>,
    message: impl Into<String>,
) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.into(),
        severity: "error".into(),
        field: field.into(),
        message: message.into(),
        expected: None,
    }
}

pub(super) fn readiness_contract_evidence(
    execute_requested: Option<bool>,
    target_evidence: &[LessonTargetEvidence],
    readiness_status: &str,
) -> Vec<ContractEvidenceItem> {
    let (baseline, modernized) = required_target_evidence(target_evidence);
    let mut evidence = Vec::with_capacity(8 + missing_required_action_count(target_evidence));
    evidence.push(evidence_item(
        "comparison_manifest",
        "present",
        "comparison manifest was parsed",
    ));
    evidence.push(evidence_item(
        "execute_requested",
        if execute_requested == Some(true) {
            "present"
        } else {
            "missing"
        },
        "comparison manifest records executed target evidence",
    ));

    for (role, target) in [("baseline", baseline), ("modernized", modernized)] {
        push_target_contract_evidence(&mut evidence, role, target);
    }

    evidence.push(desktop_pixel_observation_evidence(modernized));
    evidence.push(evidence_item(
        "first_lesson_completion",
        match readiness_status {
            "ready" => "present",
            "blocked_until_ui_automation" => "blocked",
            _ => "missing",
        },
        "first lesson completion evidence is bounded by the readiness status",
    ));

    evidence
}

fn required_target_evidence(
    target_evidence: &[LessonTargetEvidence],
) -> (Option<&LessonTargetEvidence>, Option<&LessonTargetEvidence>) {
    let mut baseline = None;
    let mut modernized = None;
    for target in target_evidence {
        match target.role.as_str() {
            "baseline" => baseline = Some(target),
            "modernized" => modernized = Some(target),
            _ => {}
        }
    }
    (baseline, modernized)
}

fn push_target_contract_evidence(
    evidence: &mut Vec<ContractEvidenceItem>,
    role: &str,
    target: Option<&LessonTargetEvidence>,
) {
    evidence.push(evidence_item(
        format!("{role}.launch_manifest"),
        if target.is_some_and(|target| target.launch_manifest_present) {
            "present"
        } else {
            "missing"
        },
        format!("{role} target embeds launch manifest evidence"),
    ));
    evidence.push(evidence_item(
        format!("{role}.ui_action_contract"),
        if target.is_some_and(|target| target.ui_action_contract_readable) {
            "present"
        } else {
            "missing"
        },
        format!("{role} target has readable UI action contract evidence"),
    ));

    if let Some(target) = target {
        evidence.extend(target.missing_required_actions.iter().map(|action| {
            evidence_item(
                format!("{role}.required_action.{action}"),
                "missing",
                format!("{role} UI action contract includes required action {action}"),
            )
        }));
    }
}

fn desktop_pixel_observation_evidence(
    target: Option<&LessonTargetEvidence>,
) -> ContractEvidenceItem {
    let state = target
        .and_then(|target| target.desktop_run_pixel_observation.as_ref())
        .map(|observation| readiness_evidence_state(&observation.status))
        .unwrap_or("missing");
    evidence_item(
        "modernized.desktop_pixel_observation",
        state,
        "modernized desktop Run pixel observation evidence",
    )
}

fn readiness_evidence_state(state: &str) -> &str {
    match state {
        "observed" | "ready" => "present",
        other => other,
    }
}

pub(super) fn readiness_diagnostics(
    execute_requested: Option<bool>,
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> Vec<ContractDiagnostic> {
    let (baseline, modernized) = required_target_evidence(target_evidence);
    let mut diagnostics =
        Vec::with_capacity(issues.len() + missing_required_action_count(target_evidence) + 3);
    if execute_requested != Some(true) {
        diagnostics.push(error_diagnostic(
            "execution_not_requested",
            "execute_requested",
            "comparison manifest must be produced with --execute to contain target launch evidence",
        ));
    }

    for (role, target) in [("baseline", baseline), ("modernized", modernized)] {
        if !target.is_some_and(|target| target.launch_manifest_present) {
            diagnostics.push(error_diagnostic(
                "missing_target_evidence",
                format!("targets.{role}.launch_manifest"),
                format!("{role} target is missing embedded launch_manifest"),
            ));
        }
    }
    push_missing_required_action_diagnostics(&mut diagnostics, target_evidence);
    for issue in issues {
        if !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_str() == issue.as_str())
        {
            diagnostics.push(error_diagnostic(
                "contract_validation_failed",
                "manifest",
                issue.clone(),
            ));
        }
    }
    diagnostics
}

fn push_missing_required_action_diagnostics(
    diagnostics: &mut Vec<ContractDiagnostic>,
    target_evidence: &[LessonTargetEvidence],
) {
    for target in target_evidence {
        for action in &target.missing_required_actions {
            diagnostics.push(ContractDiagnostic {
                code: "missing_required_action".into(),
                severity: "error".into(),
                field: format!(
                    "targets.{}.ui_action_contract.required_actions",
                    target.role
                ),
                message: format!(
                    "{} automation scenarios are missing required action {action:?}",
                    target.role
                ),
                expected: Some(action.clone()),
            });
        }
    }
}

fn missing_required_action_count(target_evidence: &[LessonTargetEvidence]) -> usize {
    target_evidence
        .iter()
        .map(|target| target.missing_required_actions.len())
        .sum()
}
