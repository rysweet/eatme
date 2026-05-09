use std::fs;
use std::path::Path;

mod gates;
pub(crate) mod github;
mod types;
use types::{Blocker, Decision, ReadinessEvidence, ReadinessReport};

const EVIDENCE_SCHEMA_VERSION: &str = "eatme.default-workflow-pr-readiness-evidence/v1";

pub(crate) fn evaluate_pr_readiness_evidence(path: &Path) -> ReadinessOutcome {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return ReadinessOutcome::input_error(format!(
                "failed to read evidence file {}: {error}",
                path.display()
            ));
        }
    };

    let evidence = match serde_json::from_str::<ReadinessEvidence>(&content) {
        Ok(evidence) => evidence,
        Err(error) => {
            return ReadinessOutcome::input_error(format!(
                "failed to parse readiness evidence {}: {error}",
                path.display()
            ));
        }
    };

    gates::evaluate_readiness(&evidence)
}

pub(crate) struct ReadinessOutcome {
    pub(crate) report: ReadinessReport,
    pub(crate) exit_code: i32,
}

impl ReadinessOutcome {
    fn input_error(message: String) -> Self {
        Self {
            report: ReadinessReport {
                decision: Decision::NotMergeReady,
                pr_number: None,
                head_ref_name: None,
                head_ref_oid: None,
                local_head: None,
                files_modified: Vec::new(),
                no_op_justification: None,
                blockers: vec![Blocker::new("input", "malformed_evidence", message)],
            },
            exit_code: 2,
        }
    }
}
