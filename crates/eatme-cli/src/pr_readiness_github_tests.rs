use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};

use super::pr_readiness::{
    CheckConclusion, CheckStatus, GitHubPrSnapshotRequest, fetch_github_pr_snapshot,
};

const PR_204_BRANCH: &str = "wave7-eatme-nonclaim-audit-1778303500";
const EVIDENCE_HEAD: &str = "1111111111111111111111111111111111111111";
const LOCAL_HEAD: &str = "2222222222222222222222222222222222222222";

#[test]
fn github_snapshot_adapter_fetches_exact_head_and_marks_required_checks() {
    let runner = FakeRunner::succeeding(format!(
        r#"{{
          "number": 204,
          "headRefName": "{PR_204_BRANCH}",
          "headRefOid": "{EVIDENCE_HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "statusCheckRollup": [
            {{"name": "quality-gates", "status": "COMPLETED", "conclusion": "SUCCESS"}},
            {{"name": "optional-preview", "status": "COMPLETED", "conclusion": "SKIPPED"}}
          ]
        }}"#
    ));
    let request = GitHubPrSnapshotRequest {
        owner: "rysweet".into(),
        repo: "eatme".into(),
        pr_number: 204,
        local_head_sha: EVIDENCE_HEAD.into(),
        required_checks: vec!["quality-gates".into(), "missing-required".into()],
    };

    let snapshot = fetch_github_pr_snapshot(&request, &runner).unwrap();
    let specs = runner.specs.lock().unwrap();

    assert_eq!(snapshot.pr_head_sha, EVIDENCE_HEAD);
    assert_eq!(snapshot.local_head_sha, EVIDENCE_HEAD);
    assert_eq!(snapshot.branch, PR_204_BRANCH);
    assert_eq!(snapshot.checks[0].name, "quality-gates");
    assert!(snapshot.checks[0].required);
    assert_eq!(snapshot.checks[0].head_sha, EVIDENCE_HEAD);
    assert_eq!(snapshot.checks[0].status, CheckStatus::Completed);
    assert_eq!(snapshot.checks[0].conclusion, CheckConclusion::Success);
    assert!(
        snapshot.checks.iter().any(|check| {
            check.name == "missing-required"
                && check.required
                && check.status == CheckStatus::Missing
        }),
        "{:#?}",
        snapshot.checks
    );
    assert_eq!(specs[0].program, "gh");
    assert_eq!(
        specs[0].args[0..5],
        ["pr", "view", "204", "--repo", "rysweet/eatme"]
    );
    assert_eq!(specs[0].attempts, 3);
    assert_eq!(specs[0].retry_delay, Duration::from_millis(500));
    assert_eq!(specs[0].timeout, Some(Duration::from_secs(20)));
}

#[test]
fn github_snapshot_adapter_preserves_local_head_separately_from_pr_head() {
    let runner = FakeRunner::succeeding(format!(
        r#"{{
          "number": 204,
          "headRefName": "{PR_204_BRANCH}",
          "headRefOid": "{EVIDENCE_HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "statusCheckRollup": []
        }}"#
    ));
    let request = GitHubPrSnapshotRequest {
        owner: "rysweet".into(),
        repo: "eatme".into(),
        pr_number: 204,
        local_head_sha: LOCAL_HEAD.into(),
        required_checks: vec!["quality-gates".into()],
    };

    let snapshot = fetch_github_pr_snapshot(&request, &runner).unwrap();

    assert_eq!(snapshot.local_head_sha, LOCAL_HEAD);
    assert_eq!(snapshot.pr_head_sha, EVIDENCE_HEAD);
}

#[test]
fn github_snapshot_adapter_surfaces_external_call_failures() {
    let runner = FakeRunner::failing("api rate limit exceeded");
    let request = GitHubPrSnapshotRequest {
        owner: "rysweet".into(),
        repo: "eatme".into(),
        pr_number: 204,
        local_head_sha: EVIDENCE_HEAD.into(),
        required_checks: vec!["quality-gates".into()],
    };

    let error = fetch_github_pr_snapshot(&request, &runner).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("GitHub PR snapshot fetch failed")
    );
    assert!(error.to_string().contains("api rate limit exceeded"));
}

struct FakeRunner {
    output: CommandOutput,
    specs: Mutex<Vec<CommandSpec>>,
}

impl FakeRunner {
    fn succeeding(stdout: String) -> Self {
        Self {
            output: CommandOutput {
                command: "gh pr view".into(),
                exit_status: Some(0),
                stdout,
                stderr: String::new(),
            },
            specs: Mutex::new(Vec::new()),
        }
    }

    fn failing(stderr: &str) -> Self {
        Self {
            output: CommandOutput {
                command: "gh pr view".into(),
                exit_status: Some(1),
                stdout: String::new(),
                stderr: stderr.into(),
            },
            specs: Mutex::new(Vec::new()),
        }
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        self.specs.lock().unwrap().push(spec.clone());
        Ok(self.output.clone())
    }
}
