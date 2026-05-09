use std::cell::Cell;
use std::time::Duration;

use eatme_assets::default_workflow_readiness::{
    CheckConclusion, CheckStatus, CommandEvidence, CommandStatus, DocsImpactReview,
    ExternalServiceError, ExternalServiceErrorKind, GitHubCheckRun, GitHubEvidenceText,
    GitHubPullRequest, GitHubReadinessAdapter, GitHubReadinessClient, QualityAuditCycle,
    ReadinessEvidenceDraft, RetryPolicy,
};

const PR_NUMBER: u64 = 193;
const BRANCH: &str = "feat/issue-176-eatme-wave7-gap-matrix-lane-follow-default-workflo";
const HEAD: &str = "8255dcb33d4c22214c971fa22e7e6d7b9237c0b3";

const REQUIRED_COMMANDS: [&str; 4] = [
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

#[test]
fn github_adapter_builds_readiness_input_from_pr_metadata_checks_and_evidence() {
    let adapter = GitHubReadinessAdapter::with_retry_policy(
        FakeGitHubClient::ready(),
        RetryPolicy::no_retry(),
    );

    let input = adapter
        .build_input(readiness_draft())
        .expect("ready fake GitHub evidence should build input");

    assert_eq!(input.pr_number, PR_NUMBER);
    assert_eq!(input.head_ref_name, BRANCH);
    assert_eq!(input.head_ref_oid, HEAD);
    assert_eq!(input.check_runs.len(), 2);
    assert!(
        input
            .check_runs
            .iter()
            .all(|check| check.head_sha == HEAD && check.conclusion == CheckConclusion::Success)
    );
    assert_eq!(input.pr_evidence.location, "PR body");
    assert_eq!(input.pr_evidence.head_sha, HEAD);
    assert!(input.pr_evidence.records_github_checks);
    assert!(input.pr_evidence.records_no_manual_merge);
    assert_eq!(input.pr_evidence.recorded_commands, REQUIRED_COMMANDS);
}

#[test]
fn github_adapter_retries_retryable_external_failures() {
    let client = FakeGitHubClient::fails_once();
    let adapter = GitHubReadinessAdapter::with_retry_policy(
        client,
        RetryPolicy::new(2, Duration::from_millis(0)),
    );

    let input = adapter
        .build_input(readiness_draft())
        .expect("transient pull request failure should be retried");

    assert_eq!(input.head_ref_oid, HEAD);
}

#[test]
fn github_adapter_does_not_retry_permanent_external_failures() {
    let client = FakeGitHubClient::permanent_failure();
    let adapter = GitHubReadinessAdapter::with_retry_policy(
        client,
        RetryPolicy::new(3, Duration::from_millis(0)),
    );

    let error = adapter
        .build_input(readiness_draft())
        .expect_err("permanent GitHub failures must be surfaced");

    assert_eq!(error.kind(), &ExternalServiceErrorKind::CommandFailed);
    assert!(error.message().contains("bad request"));
}

struct FakeGitHubClient {
    mode: FakeMode,
    pull_request_calls: Cell<usize>,
}

impl FakeGitHubClient {
    fn ready() -> Self {
        Self {
            mode: FakeMode::Ready,
            pull_request_calls: Cell::new(0),
        }
    }

    fn fails_once() -> Self {
        Self {
            mode: FakeMode::FailsOnce,
            pull_request_calls: Cell::new(0),
        }
    }

    fn permanent_failure() -> Self {
        Self {
            mode: FakeMode::PermanentFailure,
            pull_request_calls: Cell::new(0),
        }
    }
}

enum FakeMode {
    Ready,
    FailsOnce,
    PermanentFailure,
}

impl GitHubReadinessClient for FakeGitHubClient {
    fn pull_request(&self, _pr_number: u64) -> Result<GitHubPullRequest, ExternalServiceError> {
        let calls = self.pull_request_calls.get() + 1;
        self.pull_request_calls.set(calls);

        match self.mode {
            FakeMode::FailsOnce if calls == 1 => Err(ExternalServiceError::new(
                ExternalServiceErrorKind::TemporarilyUnavailable,
                "GitHub temporarily unavailable",
            )),
            FakeMode::PermanentFailure => Err(ExternalServiceError::new(
                ExternalServiceErrorKind::CommandFailed,
                "bad request",
            )),
            FakeMode::Ready | FakeMode::FailsOnce => Ok(github_pull_request()),
        }
    }

    fn check_runs(&self, _pr_number: u64) -> Result<Vec<GitHubCheckRun>, ExternalServiceError> {
        Ok(vec![
            github_check("Build MkDocs site"),
            github_check("fmt, clippy, tests, module size, coverage"),
        ])
    }
}

fn readiness_draft() -> ReadinessEvidenceDraft {
    ReadinessEvidenceDraft {
        pr_number: PR_NUMBER,
        local_branch: BRANCH.into(),
        local_head_sha: HEAD.into(),
        command_evidence: REQUIRED_COMMANDS
            .iter()
            .map(|command| CommandEvidence {
                command: (*command).into(),
                status: CommandStatus::Passed,
                head_sha: HEAD.into(),
                used_timeout_wrapper: false,
            })
            .collect(),
        quality_audit_cycles: vec![
            audit_cycle(1, false),
            audit_cycle(2, false),
            audit_cycle(3, true),
        ],
        changed_files: vec![
            "docs/default-workflow-pr-readiness.md".into(),
            "crates/eatme-assets/src/default_workflow_readiness/github.rs".into(),
        ],
        docs_impact: DocsImpactReview {
            mkdocs_strict_passed: true,
            bounded_claims: vec!["current-head command evidence".into()],
        },
        manual_merge_attempted: false,
    }
}

fn github_pull_request() -> GitHubPullRequest {
    GitHubPullRequest {
        head_ref_name: BRANCH.into(),
        head_ref_oid: HEAD.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        evidence_texts: vec![GitHubEvidenceText {
            location: "PR body".into(),
            body: format!(
                "{HEAD}\n{}\n{}\n{}\n{}\nGitHub checks\nDiff scope\nDocs impact\nQuality audit\nNo manual merge",
                REQUIRED_COMMANDS[0],
                REQUIRED_COMMANDS[1],
                REQUIRED_COMMANDS[2],
                REQUIRED_COMMANDS[3]
            ),
        }],
    }
}

fn github_check(name: &str) -> GitHubCheckRun {
    GitHubCheckRun {
        name: name.into(),
        status: CheckStatus::Completed,
        conclusion: CheckConclusion::Success,
    }
}

fn audit_cycle(number: usize, clean: bool) -> QualityAuditCycle {
    QualityAuditCycle {
        seek: format!("SEEK {number}: reviewed external service evidence"),
        validate: format!("VALIDATE {number}: checked GitHub service response mapping"),
        fix: format!("FIX {number}: no-op or bounded adapter correction for {HEAD}"),
        clean,
    }
}
