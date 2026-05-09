use super::{ScopeChange, ScopeSurface};
use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckConclusion {
    #[serde(alias = "SUCCESS", alias = "success")]
    Success,
    #[serde(alias = "FAILURE", alias = "failure")]
    Failure,
    #[serde(alias = "PENDING", alias = "pending")]
    Pending,
    #[serde(alias = "CANCELLED", alias = "cancelled")]
    Cancelled,
    #[serde(alias = "SKIPPED", alias = "skipped")]
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckRunEvidence {
    pub name: String,
    pub head_sha: String,
    pub conclusion: CheckConclusion,
    pub required: bool,
    pub workflow_name: Option<String>,
    pub details_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CheckRollupEvidence {
    head_sha: String,
    checks: Vec<CheckRunEvidence>,
}

impl CheckRollupEvidence {
    pub fn for_head(head_sha: impl Into<String>, checks: Vec<CheckRunEvidence>) -> Self {
        Self {
            head_sha: head_sha.into(),
            checks,
        }
    }

    pub fn require_green_current_checks(&self) -> Result<()> {
        if self.checks.is_empty() {
            bail!("no check evidence was available for {}", self.head_sha);
        }
        let failures: Vec<String> = self
            .checks
            .iter()
            .filter_map(|check| {
                if check.head_sha != self.head_sha {
                    return Some(format!(
                        "{} is for wrong head {} instead of {}",
                        check.name, check.head_sha, self.head_sha
                    ));
                }
                match check.conclusion {
                    CheckConclusion::Success => None,
                    CheckConclusion::Skipped if !check.required => None,
                    CheckConclusion::Skipped => Some(format!("{} was skipped", check.name)),
                    CheckConclusion::Failure => Some(format!("{} failed", check.name)),
                    CheckConclusion::Pending => Some(format!("{} is pending", check.name)),
                    CheckConclusion::Cancelled => Some(format!("{} was cancelled", check.name)),
                }
            })
            .collect();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(failures.join("; ")))
        }
    }

    pub fn has_evidence_gap(&self) -> bool {
        self.require_green_current_checks().is_err()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplementalValidation {
    MkdocsStrict,
    AssetValidation,
    GadugiFreshness,
    FullQualityGate,
    Passed { command: String },
}

impl SupplementalValidation {
    pub fn passed(command: impl Into<String>) -> Self {
        Self::Passed {
            command: command.into(),
        }
    }

    pub(crate) fn satisfies(&self, required: &Self) -> bool {
        match (self, required) {
            (Self::MkdocsStrict, Self::MkdocsStrict)
            | (Self::AssetValidation, Self::AssetValidation)
            | (Self::GadugiFreshness, Self::GadugiFreshness)
            | (Self::FullQualityGate, Self::FullQualityGate) => true,
            (Self::Passed { command }, Self::MkdocsStrict) => command.contains("mkdocs build"),
            (Self::Passed { command }, Self::AssetValidation) => {
                command.contains("assets validate")
            }
            (Self::Passed { command }, Self::GadugiFreshness) => {
                command.contains("assets generate-gadugi") && command.contains("--check")
            }
            (Self::Passed { command }, Self::FullQualityGate) => {
                command.contains("quality-gates.sh")
            }
            _ => false,
        }
    }
}

pub fn required_supplemental_validations(
    scope_changes: &[ScopeChange],
    checks: &CheckRollupEvidence,
) -> Vec<SupplementalValidation> {
    let mut required = Vec::new();
    if checks.has_evidence_gap() {
        required.push(SupplementalValidation::FullQualityGate);
    }
    for change in scope_changes {
        match change.surface {
            ScopeSurface::Documentation => {
                push_unique(&mut required, SupplementalValidation::MkdocsStrict)
            }
            ScopeSurface::ScenarioAsset => {
                push_unique(&mut required, SupplementalValidation::AssetValidation);
                push_unique(&mut required, SupplementalValidation::GadugiFreshness);
            }
            ScopeSurface::GeneratedGadugiAdapter => {
                push_unique(&mut required, SupplementalValidation::GadugiFreshness)
            }
            ScopeSurface::ReadinessGuardTest | ScopeSurface::Unrelated => {}
        }
    }
    required
}

fn push_unique(values: &mut Vec<SupplementalValidation>, value: SupplementalValidation) {
    if !values.contains(&value) {
        values.push(value);
    }
}
