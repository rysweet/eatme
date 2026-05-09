use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
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
    summarize_items(names.len(), names.iter().map(String::as_str))
}

pub(crate) fn summarize_paths(paths: &[PathBuf]) -> String {
    summarize_items(paths.len(), paths.iter().map(|path| path.display()))
}

fn summarize_items<I, T>(count: usize, items: I) -> String
where
    I: IntoIterator<Item = T>,
    T: fmt::Display,
{
    if count == 0 {
        return "0".into();
    }

    let mut summary = format!("{count} [");
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            summary.push_str(", ");
        }
        let _ = write!(&mut summary, "{item}");
    }
    summary.push(']');
    summary
}
