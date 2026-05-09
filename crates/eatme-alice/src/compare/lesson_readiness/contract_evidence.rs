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

pub(crate) fn expected_error_diagnostic(
    code: impl Into<String>,
    field: impl Into<String>,
    message: impl Into<String>,
    expected: impl Into<String>,
) -> ContractDiagnostic {
    ContractDiagnostic {
        expected: Some(expected.into()),
        ..error_diagnostic(code, field, message)
    }
}

pub(super) fn readiness_contract_evidence(
    execute_requested: Option<bool>,
    target_evidence: &[LessonTargetEvidence],
    readiness_status: &str,
) -> Vec<ContractEvidenceItem> {
    let mut evidence = vec![
        evidence_item(
            "comparison_manifest",
            "present",
            "comparison manifest was parsed",
        ),
        evidence_item(
            "execute_requested",
            if execute_requested == Some(true) {
                "present"
            } else {
                "missing"
            },
            "comparison manifest records executed target evidence",
        ),
    ];

    for role in ["baseline", "modernized"] {
        let target = target_evidence.iter().find(|target| target.role == role);
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
            for action in &target.missing_required_actions {
                evidence.push(evidence_item(
                    format!("{role}.required_action.{action}"),
                    "missing",
                    format!("{role} UI action contract includes required action {action}"),
                ));
            }
        }
    }

    let pixel_observation = target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .and_then(|target| target.desktop_run_pixel_observation.as_ref());
    evidence.push(evidence_item(
        "modernized.desktop_pixel_observation",
        pixel_observation
            .map(|observation| readiness_evidence_state(&observation.status))
            .unwrap_or("missing"),
        "modernized desktop Run pixel observation evidence",
    ));
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
    let mut diagnostics = Vec::new();
    if execute_requested != Some(true) {
        diagnostics.push(error_diagnostic(
            "execution_not_requested",
            "execute_requested",
            "comparison manifest must be produced with --execute to contain target launch evidence",
        ));
    }

    for role in ["baseline", "modernized"] {
        match target_evidence.iter().find(|target| target.role == role) {
            Some(target) if target.launch_manifest_present => {}
            _ => diagnostics.push(error_diagnostic(
                "missing_target_evidence",
                format!("targets.{role}.launch_manifest"),
                format!("{role} target is missing embedded launch_manifest"),
            )),
        }
    }

    for target in target_evidence {
        for action in &target.missing_required_actions {
            diagnostics.push(expected_error_diagnostic(
                "missing_required_action",
                format!(
                    "targets.{}.ui_action_contract.required_actions",
                    target.role
                ),
                format!(
                    "{} automation scenarios are missing required action {action:?}",
                    target.role
                ),
                action,
            ));
        }
    }

    for issue in issues {
        if diagnostic_message_already_reported(&diagnostics, issue) {
            continue;
        }
        diagnostics.push(error_diagnostic(
            "contract_validation_failed",
            "manifest",
            issue.clone(),
        ));
    }
    diagnostics
}

fn diagnostic_message_already_reported(diagnostics: &[ContractDiagnostic], issue: &str) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == issue)
}
