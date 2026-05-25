use super::{RecoveryError, required_text};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkflowSource {
    RealDefaultWorkflowNoTimeout,
    TimeoutFallback,
    ManualSubstitute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultWorkflowInvocation {
    pub source: WorkflowSource,
    pub outcome: String,
    pub log_reference: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultWorkflowProof {
    pub source: WorkflowSource,
    pub outcome: String,
    pub log_reference: String,
    pub run_id: String,
}

pub struct DefaultWorkflowRecovery;

impl DefaultWorkflowRecovery {
    pub fn validate_invocation(
        invocation: &DefaultWorkflowInvocation,
    ) -> Result<DefaultWorkflowProof, RecoveryError> {
        match invocation.source {
            WorkflowSource::TimeoutFallback => {
                return Err(RecoveryError::new(
                    "default_workflow_timeout_fallback_forbidden",
                    "timeout fallback output is not valid PR #199 recovery proof",
                ));
            }
            WorkflowSource::ManualSubstitute => {
                return Err(RecoveryError::new(
                    "default_workflow_manual_substitute_forbidden",
                    "manual reconstruction cannot substitute for the real default-workflow path",
                ));
            }
            WorkflowSource::RealDefaultWorkflowNoTimeout => {}
        }

        let log_reference = required_text(
            invocation.log_reference.as_deref(),
            "default_workflow_proof_missing",
        )?;
        let run_id = required_text(
            invocation.run_id.as_deref(),
            "default_workflow_proof_missing",
        )?;
        let outcome = required_text(
            Some(invocation.outcome.as_str()),
            "default_workflow_proof_missing",
        )?;

        Ok(DefaultWorkflowProof {
            source: invocation.source.clone(),
            outcome,
            log_reference,
            run_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultWorkflowInvocation, DefaultWorkflowRecovery, WorkflowSource};

    #[test]
    fn validate_invocation_trims_auditable_fields() {
        let proof = DefaultWorkflowRecovery::validate_invocation(&DefaultWorkflowInvocation {
            source: WorkflowSource::RealDefaultWorkflowNoTimeout,
            outcome: "  success  ".into(),
            log_reference: Some("  logs/default-workflow/run-1  ".into()),
            run_id: Some("  default-workflow-run-1  ".into()),
        })
        .unwrap();

        assert_eq!(proof.outcome, "success");
        assert_eq!(proof.log_reference, "logs/default-workflow/run-1");
        assert_eq!(proof.run_id, "default-workflow-run-1");
    }

    #[test]
    fn validate_invocation_rejects_blank_outcome_for_real_runs() {
        let error = DefaultWorkflowRecovery::validate_invocation(&DefaultWorkflowInvocation {
            source: WorkflowSource::RealDefaultWorkflowNoTimeout,
            outcome: "   ".into(),
            log_reference: Some("logs/default-workflow/run-1".into()),
            run_id: Some("default-workflow-run-1".into()),
        })
        .unwrap_err();

        assert_eq!(error.code(), "default_workflow_proof_missing");
    }
}
