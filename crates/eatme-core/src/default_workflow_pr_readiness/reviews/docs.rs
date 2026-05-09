use super::super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocsImpact {
    pub changed_files: Vec<String>,
    pub docs_files: Vec<String>,
    pub no_docs_impact_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocsImpactReview {
    passed: bool,
}

impl DocsImpactReview {
    pub fn passed(&self) -> bool {
        self.passed
    }
}

pub struct DocsImpactReviewer;

impl DocsImpactReviewer {
    pub fn review(impact: DocsImpact) -> Result<DocsImpactReview, ReadinessError> {
        let has_docs = !impact.docs_files.is_empty();
        let has_no_impact_reason = impact
            .no_docs_impact_reason
            .as_deref()
            .is_some_and(|reason| !reason.trim().is_empty());

        if has_docs || has_no_impact_reason || impact.changed_files.is_empty() {
            Ok(DocsImpactReview { passed: true })
        } else {
            Err(ReadinessError::new(
                ReadinessErrorKind::MissingDocsImpact,
                "documentation impact must be reviewed or explicitly ruled out",
            ))
        }
    }
}
