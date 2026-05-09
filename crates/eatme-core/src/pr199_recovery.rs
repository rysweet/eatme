pub mod evidence;
pub mod qa;
pub mod service;
pub mod state;
pub mod workflow;

pub use evidence::{AliceActionEvidence, Pr199RecoveryEvidence};
pub use qa::QaCommandProof;
pub use state::{Pr199RecoveryReport, ReadinessBlocker};

use anyhow::Result;

pub fn evaluate_pr199_recovery_readiness(
    evidence: Pr199RecoveryEvidence,
) -> Result<Pr199RecoveryReport> {
    state::evaluate(evidence)
}
