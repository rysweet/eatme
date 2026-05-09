use std::cell::Cell;
use std::time::Duration;

use eatme_assets::default_workflow_readiness::{
    CheckConclusion, CheckStatus, CommandEvidence, CommandStatus, DocsImpactReview,
    ExternalServiceError, ExternalServiceErrorKind, GhCliReadinessClient, GitHubCheckRun,
    GitHubEvidenceText, GitHubPullRequest, GitHubReadinessAdapter, GitHubReadinessClient,
    QualityAuditCycle, ReadinessEvidenceDraft, RetryPolicy,
};

const PR_NUMBER: u64 = 193;
const BRANCH: &str = "feat/issue-176-eatme-wave7-gap-matrix-lane-follow-default-workflo";
const HEAD: &str = "8255dcb33d4c22214c971fa22e7e6d7b9237c0b3";
const NEXT_HEAD: &str = "1111111111111111111111111111111111111111";

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
    assert!(input.pr_evidence.trusted_provenance);
    assert_eq!(input.pr_evidence.head_sha, HEAD);
    assert!(input.pr_evidence.records_github_checks);
    assert!(input.pr_evidence.records_no_manual_merge);
    assert_eq!(input.pr_evidence.recorded_commands, REQUIRED_COMMANDS);
}

#[test]
fn github_adapter_ignores_untrusted_comments_as_readiness_evidence() {
    let adapter = GitHubReadinessAdapter::with_retry_policy(
        FakeGitHubClient::untrusted_comment_only(),
        RetryPolicy::no_retry(),
    );

    let input = adapter
        .build_input(readiness_draft())
        .expect("untrusted comments should not prevent metadata collection");

    assert!(!input.pr_evidence.trusted_provenance);
    assert_eq!(input.pr_evidence.head_sha, "");
    assert!(input.pr_evidence.recorded_commands.is_empty());
}

#[test]
fn gh_cli_client_uses_supported_check_fields_and_maps_final_states() {
    let script_path = fake_gh_script();
    let client = GhCliReadinessClient::with_binary(script_path.to_string_lossy());

    let checks = client
        .check_runs(PR_NUMBER)
        .expect("supported gh pr checks fields should parse");

    assert_eq!(checks.len(), 2);
    assert_eq!(checks[0].status, CheckStatus::Completed);
    assert_eq!(checks[0].conclusion, CheckConclusion::Success);
    assert!(checks[0].required);
    assert_eq!(checks[1].status, CheckStatus::Completed);
    assert_eq!(checks[1].conclusion, CheckConclusion::Skipped);
    assert!(!checks[1].required);
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

#[test]
fn github_adapter_blocks_head_changes_during_evidence_collection() {
    let client = FakeGitHubClient::head_changes();
    let adapter = GitHubReadinessAdapter::with_retry_policy(
        client,
        RetryPolicy::new(2, Duration::from_millis(0)),
    );

    let error = adapter
        .build_input(readiness_draft())
        .expect_err("PR head changes during collection must block readiness evidence");

    assert_eq!(error.kind(), &ExternalServiceErrorKind::InvalidResponse);
    assert!(error.message().contains("head changed"));
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

    fn head_changes() -> Self {
        Self {
            mode: FakeMode::HeadChanges,
            pull_request_calls: Cell::new(0),
        }
    }

    fn untrusted_comment_only() -> Self {
        Self {
            mode: FakeMode::UntrustedCommentOnly,
            pull_request_calls: Cell::new(0),
        }
    }
}

enum FakeMode {
    Ready,
    FailsOnce,
    PermanentFailure,
    HeadChanges,
    UntrustedCommentOnly,
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
            FakeMode::HeadChanges if calls >= 2 => Ok(github_pull_request_with_head(NEXT_HEAD)),
            FakeMode::UntrustedCommentOnly => Ok(github_pull_request_with_untrusted_comment()),
            FakeMode::Ready | FakeMode::FailsOnce | FakeMode::HeadChanges => {
                Ok(github_pull_request())
            }
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
    github_pull_request_with_head(HEAD)
}

fn github_pull_request_with_head(head: &str) -> GitHubPullRequest {
    GitHubPullRequest {
        head_ref_name: BRANCH.into(),
        head_ref_oid: head.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        evidence_texts: vec![GitHubEvidenceText {
            location: "PR body".into(),
            trusted: true,
            body: format!(
                "{head}\n{}\n{}\n{}\n{}\nGitHub checks\nDiff scope\nDocs impact\nQuality audit\nNo manual merge",
                REQUIRED_COMMANDS[0],
                REQUIRED_COMMANDS[1],
                REQUIRED_COMMANDS[2],
                REQUIRED_COMMANDS[3]
            ),
        }],
    }
}

fn github_pull_request_with_untrusted_comment() -> GitHubPullRequest {
    GitHubPullRequest {
        head_ref_name: BRANCH.into(),
        head_ref_oid: HEAD.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        evidence_texts: vec![GitHubEvidenceText {
            location: "PR comment 1 (NONE)".into(),
            trusted: false,
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
        required: true,
    }
}

fn fake_gh_script() -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("eatme-fake-gh-{}-{}", std::process::id(), "checks"));
    std::fs::create_dir_all(&dir).expect("fake gh temp dir should be created");
    let script_path = dir.join("gh");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
if [ "$1" = "pr" ] && [ "$2" = "checks" ]; then
  case "$5 $6" in
     *conclusion*) echo "unexpected conclusion field" >&2; exit 9 ;;
  esac
  if [ "$4" = "--required" ]; then
    cat <<'JSON'
[{"name":"tests","state":"SUCCESS","bucket":"pass"}]
JSON
    exit 0
  fi
  cat <<'JSON'
[{"name":"tests","state":"SUCCESS","bucket":"pass"},{"name":"manual real Alice launch smoke","state":"SKIPPED","bucket":"skipping"}]
JSON
  exit 0
fi
echo "unexpected arguments: $*" >&2
exit 1
"#,
    )
    .expect("fake gh script should be written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&script_path)
            .expect("fake gh metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script_path, permissions)
            .expect("fake gh script should be executable");
    }

    script_path
}

fn audit_cycle(number: usize, clean: bool) -> QualityAuditCycle {
    QualityAuditCycle {
        seek: format!("SEEK {number}: reviewed external service evidence"),
        validate: format!("VALIDATE {number}: checked GitHub service response mapping"),
        fix: format!("FIX {number}: no-op or bounded adapter correction for {HEAD}"),
        clean,
    }
}
