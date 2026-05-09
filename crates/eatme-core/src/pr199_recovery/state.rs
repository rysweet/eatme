use crate::pr199_recovery::evidence::{
    AliceActionEvidence, AliceEvidenceKind, AliceEvidenceTarget, Pr199RecoveryEvidence,
};
use crate::pr199_recovery::qa;
use crate::pr199_recovery::service::Pr199Metadata;
use crate::pr199_recovery::workflow;
use anyhow::Result;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Pr199RecoveryReport {
    pub pr: u32,
    pub status: String,
    pub workflow: Pr199WorkflowReport,
    pub alice: Pr199AliceReport,
    pub qa: Pr199QaReport,
    pub pr_metadata: Pr199Metadata,
    pub blockers: Vec<ReadinessBlocker>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pr199WorkflowReport {
    pub proof: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pr199AliceReport {
    pub original_actions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pr199QaReport {
    pub required_commands: [&'static str; 5],
    pub observed_commands: Vec<String>,
    pub required_commands_passed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadinessBlocker {
    pub code: String,
    pub field: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

impl ReadinessBlocker {
    pub fn new(
        code: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            field: field.into(),
            message: message.into(),
            action: None,
            target: None,
        }
    }

    pub fn with_action(mut self, action: impl Into<String>, target: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self.target = Some(target.into());
        self
    }
}

pub fn evaluate(evidence: Pr199RecoveryEvidence) -> Result<Pr199RecoveryReport> {
    let mut blockers = Vec::new();

    if evidence.pr != 199 {
        blockers.push(ReadinessBlocker::new(
            "wrong_pr_scope",
            "pr",
            format!(
                "PR #199 recovery evidence cannot be used for PR #{}.",
                evidence.pr
            ),
        ));
    }

    let (workflow_report, workflow_blockers) =
        workflow::evaluate_workflow(evidence.workflow_proof.as_deref());
    blockers.extend(workflow_blockers);

    let (alice_report, alice_blockers) = evaluate_alice_actions(&evidence.alice_actions);
    blockers.extend(alice_blockers);

    let (qa_report, qa_blockers) = qa::evaluate_qa(&evidence.qa_commands);
    blockers.extend(qa_blockers);

    let pr_metadata = Pr199Metadata::from_optional_value(evidence.pr_metadata)?;
    if pr_metadata.number != 199 {
        blockers.push(ReadinessBlocker::new(
            "wrong_pr_scope",
            "pr",
            format!(
                "PR metadata is scoped to PR #{} instead of PR #199.",
                pr_metadata.number
            ),
        ));
    }

    let status = if blockers.is_empty() {
        "ready"
    } else {
        "not_ready"
    }
    .to_string();

    Ok(Pr199RecoveryReport {
        pr: evidence.pr,
        status,
        workflow: workflow_report,
        alice: alice_report,
        qa: qa_report,
        pr_metadata,
        blockers,
    })
}

fn evaluate_alice_actions(
    actions: &[AliceActionEvidence],
) -> (Pr199AliceReport, Vec<ReadinessBlocker>) {
    let mut real_original_actions = Vec::new();
    let mut blockers = Vec::new();

    for evidence in actions {
        match (&evidence.target, &evidence.kind) {
            (AliceEvidenceTarget::Original, AliceEvidenceKind::Real) => {
                real_original_actions.push(evidence.action.clone());
            }
            (AliceEvidenceTarget::Original, AliceEvidenceKind::Missing) => {
                blockers.push(missing_real_action_blocker(
                    &evidence.action,
                    evidence.target.as_str(),
                ));
            }
            (AliceEvidenceTarget::Original, AliceEvidenceKind::Synthetic) => {
                blockers.push(ReadinessBlocker::new(
                    "invalid_alice_action_evidence",
                    format!("alice.original.actions.{}", evidence.action),
                    format!(
                        "Synthetic or reconstructed Alice {} evidence cannot satisfy PR #199 recovery readiness.",
                        evidence.action
                    ),
                ));
                blockers.push(missing_real_action_blocker(
                    &evidence.action,
                    evidence.target.as_str(),
                ));
            }
        }
    }

    (
        Pr199AliceReport {
            original_actions: real_original_actions,
        },
        blockers,
    )
}

fn missing_real_action_blocker(action: &str, target: &str) -> ReadinessBlocker {
    ReadinessBlocker::new(
        "missing_real_action_evidence",
        format!("alice.{target}.actions.{action}"),
        format!(
            "Original Alice {action} action evidence is missing and must remain blocked until real evidence exists.",
        ),
    )
    .with_action(action, target)
}
