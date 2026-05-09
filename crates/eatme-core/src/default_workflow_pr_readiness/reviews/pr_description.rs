use super::super::REQUIRED_LOCAL_QA_COMMANDS;
use super::super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrDescriptionReview {
    Current,
}

pub struct PrDescriptionReviewerUpdater;

const REQUIRED_EVIDENCE_FRAGMENT_GROUPS: &[&[&str]] = &[
    &["Final verdict:"],
    &[
        "GitHub Actions:",
        "GitHub Actions for the evaluated head",
        "Required GitHub Actions",
    ],
    &["Local QA:", "Local QA passed", "Local QA commands passed"],
    &["Scenario evidence:"],
    &["Docs impact:"],
    &["Focused diff:"],
    &["Quality audit cycles:"],
    &[
        "Evidence boundary:",
        "Does not claim full UI automation",
        "no full Alice UI automation",
    ],
];

impl PrDescriptionReviewerUpdater {
    pub fn review(
        pr_number: u64,
        body: &str,
        evaluated_head: &str,
    ) -> Result<PrDescriptionReview, ReadinessError> {
        let pr_marker = format!("#{pr_number}");
        let has_change_report = body.contains("Files modified:")
            || body.contains("Files modified by")
            || body.contains("No-op justification:")
            || body.contains("Workflow-accepted no-op justification:");

        if !body.contains(&pr_marker)
            || !body.contains(evaluated_head)
            || REQUIRED_EVIDENCE_FRAGMENT_GROUPS
                .iter()
                .any(|fragments| !contains_any(body, fragments))
            || REQUIRED_LOCAL_QA_COMMANDS
                .iter()
                .any(|command| !body.contains(command))
            || !has_change_report
        {
            return Err(ReadinessError::new(
                ReadinessErrorKind::StalePrDescription,
                "PR description does not contain current bounded readiness evidence",
            ));
        }

        Ok(PrDescriptionReview::Current)
    }
}

fn contains_any(body: &str, fragments: &[&str]) -> bool {
    fragments.iter().any(|fragment| body.contains(fragment))
}
