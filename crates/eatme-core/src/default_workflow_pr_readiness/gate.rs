use super::error::ReadinessErrorKind;
use std::fmt::Write as _;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessEvidence {
    pr_number: u64,
    branch: String,
    evaluated_head: String,
    exact_head_verified: bool,
    workflow_completed: bool,
    github_actions_green: bool,
    local_qa_passed: bool,
    scenario_evidence_reviewed: bool,
    docs_impact_reviewed: bool,
    focused_diff_reviewed: bool,
    pr_state_reviewed: bool,
    pr_description_current: bool,
    quality_audit_cycle_count: usize,
    final_quality_audit_cycle_clean: bool,
    files_modified: Vec<String>,
    noop_justification: Option<String>,
}

impl ReadinessEvidence {
    pub fn new(
        pr_number: u64,
        branch: impl Into<String>,
        evaluated_head: impl Into<String>,
    ) -> Self {
        Self {
            pr_number,
            branch: branch.into(),
            evaluated_head: evaluated_head.into(),
            exact_head_verified: false,
            workflow_completed: false,
            github_actions_green: false,
            local_qa_passed: false,
            scenario_evidence_reviewed: false,
            docs_impact_reviewed: false,
            focused_diff_reviewed: false,
            pr_state_reviewed: false,
            pr_description_current: false,
            quality_audit_cycle_count: 0,
            final_quality_audit_cycle_clean: false,
            files_modified: Vec::new(),
            noop_justification: None,
        }
    }

    pub fn with_exact_head_verified(mut self, verified: bool) -> Self {
        self.exact_head_verified = verified;
        self
    }

    pub fn with_workflow_completed(mut self, completed: bool) -> Self {
        self.workflow_completed = completed;
        self
    }

    pub fn with_github_actions_green(mut self, green: bool) -> Self {
        self.github_actions_green = green;
        self
    }

    pub fn with_local_qa_passed(mut self, passed: bool) -> Self {
        self.local_qa_passed = passed;
        self
    }

    pub fn with_scenario_evidence_reviewed(mut self, reviewed: bool) -> Self {
        self.scenario_evidence_reviewed = reviewed;
        self
    }

    pub fn with_docs_impact_reviewed(mut self, reviewed: bool) -> Self {
        self.docs_impact_reviewed = reviewed;
        self
    }

    pub fn with_focused_diff_reviewed(mut self, reviewed: bool) -> Self {
        self.focused_diff_reviewed = reviewed;
        self
    }

    pub fn with_pr_state_reviewed(mut self, reviewed: bool) -> Self {
        self.pr_state_reviewed = reviewed;
        self
    }

    pub fn with_pr_description_current(mut self, current: bool) -> Self {
        self.pr_description_current = current;
        self
    }

    pub fn with_quality_audit_cycles(mut self, count: usize, final_clean: bool) -> Self {
        self.quality_audit_cycle_count = count;
        self.final_quality_audit_cycle_clean = final_clean;
        self
    }

    pub fn with_files_modified(mut self, files: Vec<String>) -> Self {
        self.files_modified = files;
        self
    }

    pub fn with_noop_justification(mut self, justification: impl Into<String>) -> Self {
        self.noop_justification = Some(justification.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessStatus {
    MergeReady,
    NotMergeReady,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessVerdict {
    status: ReadinessStatus,
    pr_number: u64,
    branch: String,
    evaluated_head: String,
    blockers: Vec<ReadinessErrorKind>,
    files_modified: Vec<String>,
    noop_justification: Option<String>,
}

impl ReadinessVerdict {
    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn has_blocker(&self, blocker: ReadinessErrorKind) -> bool {
        self.blockers.contains(&blocker)
    }

    pub fn blockers(&self) -> &[ReadinessErrorKind] {
        &self.blockers
    }
}

pub struct ReadinessGate;

impl ReadinessGate {
    pub fn evaluate(evidence: ReadinessEvidence) -> ReadinessVerdict {
        let mut blockers = Vec::new();
        if !evidence.exact_head_verified {
            blockers.push(ReadinessErrorKind::WrongHead);
        }
        if !evidence.workflow_completed {
            blockers.push(ReadinessErrorKind::IncompleteWorkflow);
        }
        if !evidence.github_actions_green {
            blockers.push(ReadinessErrorKind::IncompleteChecks);
        }
        if !evidence.local_qa_passed {
            blockers.push(ReadinessErrorKind::MissingLocalQa);
        }
        if !evidence.scenario_evidence_reviewed {
            blockers.push(ReadinessErrorKind::MissingScenarioEvidence);
        }
        if !evidence.docs_impact_reviewed {
            blockers.push(ReadinessErrorKind::MissingDocsImpact);
        }
        if !evidence.focused_diff_reviewed {
            blockers.push(ReadinessErrorKind::UnfocusedDiff);
        }
        if !evidence.pr_state_reviewed {
            blockers.push(ReadinessErrorKind::MissingPrStateReview);
        }
        if !evidence.pr_description_current {
            blockers.push(ReadinessErrorKind::StalePrDescription);
        }
        if evidence.quality_audit_cycle_count < 3 {
            blockers.push(ReadinessErrorKind::MissingQualityAuditCycle);
        } else if !evidence.final_quality_audit_cycle_clean {
            blockers.push(ReadinessErrorKind::UncleanFinalAuditCycle);
        }
        if evidence.files_modified.is_empty() && missing_noop_justification(&evidence) {
            blockers.push(ReadinessErrorKind::MissingNoopJustification);
        }

        let status = if blockers.is_empty() {
            ReadinessStatus::MergeReady
        } else {
            ReadinessStatus::NotMergeReady
        };

        ReadinessVerdict {
            status,
            pr_number: evidence.pr_number,
            branch: evidence.branch,
            evaluated_head: evidence.evaluated_head,
            blockers,
            files_modified: evidence.files_modified,
            noop_justification: evidence.noop_justification,
        }
    }
}

fn missing_noop_justification(evidence: &ReadinessEvidence) -> bool {
    match evidence.noop_justification.as_deref() {
        Some(justification) => justification.trim().is_empty(),
        None => true,
    }
}

pub struct ChangeReporter;

impl ChangeReporter {
    pub fn format_final_output(verdict: &ReadinessVerdict) -> String {
        let status = match verdict.status {
            ReadinessStatus::MergeReady => "MERGE_READY",
            ReadinessStatus::NotMergeReady => "NOT_MERGE_READY",
        };
        let mut output = format!(
            "{status}\nPR: #{}\nBranch: {}\nEvaluated head: {}\n",
            verdict.pr_number, verdict.branch, verdict.evaluated_head
        );

        if verdict.files_modified.is_empty() {
            let justification = verdict
                .noop_justification
                .as_deref()
                .unwrap_or("missing no-op justification");
            writeln!(
                output,
                "Workflow-accepted no-op justification: {justification}"
            )
            .expect("writing to String cannot fail");
        } else {
            output.push_str("Files modified:\n");
            for file in &verdict.files_modified {
                writeln!(output, "- {file}").expect("writing to String cannot fail");
            }
        }

        if !verdict.blockers.is_empty() {
            output.push_str("Blockers:\n");
            for blocker in &verdict.blockers {
                writeln!(output, "- {blocker:?}").expect("writing to String cannot fail");
            }
        }

        output
    }
}
