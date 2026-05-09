mod error;
mod gate;
mod head;
mod reviews;
mod services;

pub use error::{ReadinessError, ReadinessErrorKind};
pub use gate::{
    ChangeReporter, ReadinessEvidence, ReadinessGate, ReadinessStatus, ReadinessVerdict,
};
pub use head::{
    CheckConclusion, CheckStatus, CollectedEvidence, EvidenceCollector, LocalQARunner,
    LocalQaCommandList, LocalQaCommandOutput, LocalQaReport, PrHeadEvidence, PrHeadSynchronizer,
    PrMetadata, PrReviewDecision, PrReviewState, ReviewEvidence, StatusCheck, VerifiedPrHead,
};
pub use reviews::{
    AuditFix, DiffScopeReview, DiffScopeReviewer, DocsImpact, DocsImpactReview, DocsImpactReviewer,
    FocusedFile, PrDescriptionReview, PrDescriptionReviewerUpdater, QualityAuditCycle,
    QualityAuditCycleRunner, QualityAuditReport, ScenarioClaim, ScenarioEvidence,
    ScenarioEvidenceReview, ScenarioEvidenceReviewer,
};
pub use services::{
    ExternalServiceRetryPolicy, GitCliPrHeadAdapter, GitHubCliPrService, GitHubPrService,
    PrHeadService,
};

pub(crate) const REQUIRED_LOCAL_QA_COMMANDS: [&str; 4] = [
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];
