use std::path::PathBuf;

use super::{RecoveryError, required_text, summarize_names};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRun {
    pub name: String,
    pub conclusion: Option<CheckConclusion>,
}

impl CheckRun {
    pub fn completed(name: impl Into<String>, conclusion: CheckConclusion) -> Self {
        Self {
            name: name.into(),
            conclusion: Some(conclusion),
        }
    }

    pub fn in_progress(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            conclusion: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckRollup {
    pub success: Vec<String>,
    pub failure: Vec<String>,
    pub pending: Vec<String>,
    pub cancelled: Vec<String>,
    pub skipped: Vec<String>,
}

impl CheckRollup {
    pub(crate) fn from_runs(check_runs: Vec<CheckRun>) -> Self {
        let mut rollup = Self::default();
        for run in check_runs {
            match run.conclusion {
                Some(CheckConclusion::Success) => rollup.success.push(run.name),
                Some(CheckConclusion::Failure) => rollup.failure.push(run.name),
                Some(CheckConclusion::Cancelled) => rollup.cancelled.push(run.name),
                Some(CheckConclusion::Skipped) => rollup.skipped.push(run.name),
                None => rollup.pending.push(run.name),
            }
        }
        rollup
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "success={}; failure={}; pending={}; cancelled={}; skipped={}",
            summarize_names(&self.success),
            summarize_names(&self.failure),
            summarize_names(&self.pending),
            summarize_names(&self.cancelled),
            summarize_names(&self.skipped)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrStateInput {
    pub pr_number: u32,
    pub branch: String,
    pub head_sha: String,
    pub changed_files: Vec<PathBuf>,
    pub check_runs: Vec<CheckRun>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrState {
    pub pr_number: u32,
    pub branch: String,
    pub head_sha: String,
    pub changed_files: Vec<PathBuf>,
    pub check_rollup: CheckRollup,
}

pub struct PrStateCollector;

impl PrStateCollector {
    pub fn collect(input: PrStateInput) -> Result<PrState, RecoveryError> {
        if input.pr_number != 199 {
            return Err(RecoveryError::new(
                "unexpected_pr_number",
                "PR #199 recovery must not collect state for another PR",
            ));
        }
        let branch = required_text(Some(input.branch.as_str()), "pr_state_missing_branch")?;
        let head_sha = required_text(Some(input.head_sha.as_str()), "pr_state_missing_head")?;

        Ok(PrState {
            pr_number: input.pr_number,
            branch,
            head_sha,
            changed_files: input.changed_files,
            check_rollup: CheckRollup::from_runs(input.check_runs),
        })
    }
}
