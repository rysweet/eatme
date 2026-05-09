use crate::pr199_recovery::state::{Pr199WorkflowReport, ReadinessBlocker};

pub const REAL_DEFAULT_WORKFLOW_NO_TIMEOUT: &str = "RealDefaultWorkflowNoTimeout";

pub fn evaluate_workflow(proof: Option<&str>) -> (Pr199WorkflowReport, Vec<ReadinessBlocker>) {
    match proof {
        Some(REAL_DEFAULT_WORKFLOW_NO_TIMEOUT) => (
            Pr199WorkflowReport {
                proof: Some(REAL_DEFAULT_WORKFLOW_NO_TIMEOUT.to_string()),
            },
            Vec::new(),
        ),
        Some(invalid) => (
            Pr199WorkflowReport {
                proof: Some(invalid.to_string()),
            },
            vec![ReadinessBlocker::new(
                "invalid_workflow_proof",
                "workflow_proof",
                format!(
                    "{invalid} is not accepted for PR #199 recovery readiness; only \
                     RealDefaultWorkflowNoTimeout proves the real default-workflow path."
                ),
            )],
        ),
        None => (
            Pr199WorkflowReport { proof: None },
            vec![ReadinessBlocker::new(
                "missing_workflow_proof",
                "workflow_proof",
                "PR #199 recovery readiness requires RealDefaultWorkflowNoTimeout workflow proof.",
            )],
        ),
    }
}
