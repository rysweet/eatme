use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessErrorKind {
    WrongHead,
    ManualMergeOrHistoryRewrite,
    MergeabilityBlocked,
    DraftPullRequest,
    BlockingPrLabel,
    BlockingReviewState,
    StaleReviewEvidence,
    MissingChecks,
    IncompleteChecks,
    FailingChecks,
    MissingLocalQa,
    FailedLocalQa,
    UnsupportedEvidenceSubstitution,
    MissingScenarioEvidence,
    OverclaimedScenarioEvidence,
    MissingDocsImpact,
    UnfocusedDiff,
    StaleGeneratedAsset,
    StalePrDescription,
    MissingQualityAuditCycle,
    UncleanFinalAuditCycle,
    MissingPrStateReview,
    IncompleteWorkflow,
    MissingNoopJustification,
    ExternalServiceUnavailable,
    ExternalServiceFailed,
    MalformedExternalResponse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessError {
    kind: ReadinessErrorKind,
    message: String,
}

impl ReadinessError {
    pub(crate) fn new(kind: ReadinessErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ReadinessErrorKind {
        self.kind
    }
}

impl fmt::Display for ReadinessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for ReadinessError {}
