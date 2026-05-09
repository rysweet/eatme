use super::super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScenarioClaim {
    AssetSchemaValid,
    GadugiAdaptersFresh,
    DocsBuildStrictPassed,
    QualityGatesPassed,
    FullUiAutomation,
    VisibleRenderingCorrect,
    GradingComplete,
    CreativeAssessmentComplete,
    FullLessonComplete,
    FullWorldExecution,
    SaveComplete,
    DeployedSharingComplete,
    FullTweedlePlayerDecode,
}

impl ScenarioClaim {
    fn is_overclaim(&self) -> bool {
        matches!(
            self,
            Self::FullUiAutomation
                | Self::VisibleRenderingCorrect
                | Self::GradingComplete
                | Self::CreativeAssessmentComplete
                | Self::FullLessonComplete
                | Self::FullWorldExecution
                | Self::SaveComplete
                | Self::DeployedSharingComplete
                | Self::FullTweedlePlayerDecode
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenarioEvidence {
    pub runnable_artifacts: Vec<String>,
    pub claims: Vec<ScenarioClaim>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenarioEvidenceReview {
    runnable: bool,
    bounded_claims_only: bool,
}

impl ScenarioEvidenceReview {
    pub fn runnable(&self) -> bool {
        self.runnable
    }

    pub fn bounded_claims_only(&self) -> bool {
        self.bounded_claims_only
    }
}

pub struct ScenarioEvidenceReviewer;

impl ScenarioEvidenceReviewer {
    pub fn review(evidence: ScenarioEvidence) -> Result<ScenarioEvidenceReview, ReadinessError> {
        if evidence.runnable_artifacts.is_empty() || evidence.claims.is_empty() {
            return Err(ReadinessError::new(
                ReadinessErrorKind::MissingScenarioEvidence,
                "runnable QA/scenario evidence is required",
            ));
        }
        if evidence.claims.iter().any(ScenarioClaim::is_overclaim) {
            return Err(ReadinessError::new(
                ReadinessErrorKind::OverclaimedScenarioEvidence,
                "scenario evidence contains a claim that was not directly proven",
            ));
        }

        Ok(ScenarioEvidenceReview {
            runnable: true,
            bounded_claims_only: true,
        })
    }
}
