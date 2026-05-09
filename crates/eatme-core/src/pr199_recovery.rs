use std::error::Error;
use std::fmt;
use std::path::PathBuf;

mod evidence;
mod qa;
mod service;
mod state;
mod workflow;

pub use evidence::{
    AliceEvidenceBlockerPreserver, EvidenceDelta, EvidenceSnapshot, EvidenceUpdate,
    ExistingEvidenceFile, PushOrNoopDecisionGate, RecoveryDecision, StructuredBlocker,
};
pub use qa::{QaCommand, QaOutcome, QaReport, ScopedQaRunner};
pub use service::{GitHubPrStateClient, GitHubPrStateClientConfig};
pub use state::{CheckConclusion, CheckRollup, CheckRun, PrState, PrStateCollector, PrStateInput};
pub use workflow::{
    DefaultWorkflowInvocation, DefaultWorkflowProof, DefaultWorkflowRecovery, WorkflowSource,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryError {
    code: &'static str,
    message: String,
}

impl RecoveryError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for RecoveryError {}

pub(crate) fn required_text(
    value: Option<&str>,
    code: &'static str,
) -> Result<String, RecoveryError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        Err(RecoveryError::new(
            code,
            "required recovery evidence is missing",
        ))
    } else {
        Ok(value.to_owned())
    }
}

pub(crate) fn summarize_names(names: &[String]) -> String {
    if names.is_empty() {
        "0".into()
    } else {
        format!("{} [{}]", names.len(), names.join(", "))
    }
}

pub(crate) fn summarize_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "0".into();
    }
    let names = paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    format!("{} [{}]", names.len(), names.join(", "))
}
