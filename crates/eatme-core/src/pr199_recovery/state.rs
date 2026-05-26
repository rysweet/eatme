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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CheckConclusion, CheckRollup, CheckRun, PrStateCollector, PrStateInput};

    #[test]
    fn collect_rejects_blank_branch_and_head_sha() {
        let missing_branch = PrStateCollector::collect(PrStateInput {
            pr_number: 199,
            branch: "   ".into(),
            head_sha: "abc123".into(),
            changed_files: vec![],
            check_runs: vec![],
        })
        .unwrap_err();
        assert_eq!(missing_branch.code(), "pr_state_missing_branch");

        let missing_head = PrStateCollector::collect(PrStateInput {
            pr_number: 199,
            branch: "feat/pr-199".into(),
            head_sha: "   ".into(),
            changed_files: vec![],
            check_runs: vec![],
        })
        .unwrap_err();
        assert_eq!(missing_head.code(), "pr_state_missing_head");
    }

    #[test]
    fn collect_rejects_unexpected_pr_number() {
        let error = PrStateCollector::collect(PrStateInput {
            pr_number: 42,
            branch: "feat/other".into(),
            head_sha: "abc123".into(),
            changed_files: vec![],
            check_runs: vec![],
        })
        .unwrap_err();

        assert_eq!(error.code(), "unexpected_pr_number");
    }

    #[test]
    fn collect_preserves_changed_files_and_rolls_up_checks() {
        let state = PrStateCollector::collect(PrStateInput {
            pr_number: 199,
            branch: " feat/pr-199 ".into(),
            head_sha: " def456 ".into(),
            changed_files: vec![PathBuf::from("src/lib.rs"), PathBuf::from("docs/pr199.md")],
            check_runs: vec![
                CheckRun::completed("workspace", CheckConclusion::Success),
                CheckRun::completed("ubuntu", CheckConclusion::Failure),
                CheckRun::in_progress("qa rerun"),
            ],
        })
        .unwrap();

        assert_eq!(state.branch, "feat/pr-199");
        assert_eq!(state.head_sha, "def456");
        assert_eq!(
            state.changed_files,
            vec![PathBuf::from("src/lib.rs"), PathBuf::from("docs/pr199.md")]
        );
        assert_eq!(state.check_rollup.success, vec!["workspace".to_string()]);
        assert_eq!(state.check_rollup.failure, vec!["ubuntu".to_string()]);
        assert_eq!(state.check_rollup.pending, vec!["qa rerun".to_string()]);
    }

    #[test]
    fn check_rollup_summary_lists_each_category() {
        let rollup = CheckRollup::from_runs(vec![
            CheckRun::completed("workspace tests", CheckConclusion::Success),
            CheckRun::completed("linux", CheckConclusion::Failure),
            CheckRun::in_progress("quality gates"),
            CheckRun::completed("cancelled stale run", CheckConclusion::Cancelled),
            CheckRun::completed("optional preview", CheckConclusion::Skipped),
        ]);

        assert_eq!(
            rollup.summary(),
            "success=1 [workspace tests]; failure=1 [linux]; pending=1 [quality gates]; cancelled=1 [cancelled stale run]; skipped=1 [optional preview]"
        );
    }
}
