use eatme_core::default_workflow_pr_readiness::{
    CheckConclusion, CheckStatus, EvidenceCollector, ExternalServiceRetryPolicy,
    GitCliPrHeadAdapter, GitHubCliPrService, GitHubPrService, PrHeadService, PrReviewDecision,
    PrReviewState, ReadinessErrorKind,
};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

const PR_NUMBER: u64 = 171;
const BRANCH: &str = "wave6-scenario-run-observe-gap-1778302300";
const HEAD: &str = "1778302300abcdef1778302300abcdef17783023";

#[test]
fn github_cli_pr_service_collects_metadata_with_retry_policy() {
    let runner = RecordingRunner::new(vec![ok(json_pr_metadata())]);
    let service = GitHubCliPrService::new(&runner).with_retry_policy(
        ExternalServiceRetryPolicy::new(4, Duration::from_millis(10)),
    );

    let metadata = service.pr_metadata(PR_NUMBER).unwrap();

    assert_eq!(metadata.number, PR_NUMBER);
    assert_eq!(metadata.head_ref_name, BRANCH);
    assert_eq!(metadata.head_ref_oid, HEAD);
    assert!(metadata.mergeable);
    assert!(!metadata.is_draft);
    assert_eq!(metadata.labels, vec!["run-observe-readiness"]);
    assert_eq!(metadata.review_decision, PrReviewDecision::Approved);
    assert_eq!(metadata.latest_reviews.len(), 1);
    assert_eq!(metadata.latest_reviews[0].state, PrReviewState::Approved);
    assert_eq!(metadata.latest_reviews[0].commit_oid, HEAD);
    assert_eq!(metadata.latest_reviews[0].author_login, "reviewer");
    assert_eq!(
        metadata.files,
        vec!["docs/default-workflow-pr-readiness.md".to_string()]
    );
    assert_eq!(metadata.status_checks.len(), 3);
    assert_eq!(metadata.status_checks[0].name, "quality-gates");
    assert_eq!(metadata.status_checks[0].status, CheckStatus::Completed);
    assert_eq!(
        metadata.status_checks[0].conclusion,
        CheckConclusion::Success
    );
    assert!(metadata.status_checks[0].required);
    assert_eq!(metadata.status_checks[1].name, "docs");
    assert_eq!(metadata.status_checks[1].head_sha, HEAD);
    assert!(metadata.status_checks[1].required);
    assert_eq!(
        metadata.status_checks[2].name,
        "manual real Alice launch smoke"
    );
    assert_eq!(
        metadata.status_checks[2].conclusion,
        CheckConclusion::Skipped
    );
    assert!(!metadata.status_checks[2].required);

    let specs = runner.specs();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].shell_display(), expected_gh_command());
    assert_eq!(specs[0].attempts, 4);
    assert_eq!(specs[0].retry_delay, Duration::from_millis(10));
}

#[test]
fn github_cli_pr_service_keeps_live_skipped_status_rollup_entries_optional() {
    let runner = RecordingRunner::new(vec![ok(live_pr_status_rollup_json())]);
    let service = GitHubCliPrService::new(&runner);

    let metadata = service.pr_metadata(PR_NUMBER).unwrap();

    assert_eq!(metadata.status_checks.len(), 5);
    assert_eq!(metadata.status_checks[0].name, "Build MkDocs site");
    assert_eq!(metadata.status_checks[0].head_sha, HEAD);
    assert!(metadata.status_checks[0].required);
    assert_eq!(metadata.status_checks[2].name, "Deploy to GitHub Pages");
    assert_eq!(
        metadata.status_checks[2].conclusion,
        CheckConclusion::Skipped
    );
    assert!(!metadata.status_checks[2].required);
    assert_eq!(
        metadata.status_checks[3].name,
        "manual real Alice launch smoke"
    );
    assert!(!metadata.status_checks[3].required);

    let evidence = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap();
    assert!(evidence.github_actions_green());
}

#[test]
fn github_cli_pr_service_preserves_owner_free_review_required_state() {
    let runner = RecordingRunner::new(vec![ok(owner_free_review_required_json())]);
    let service = GitHubCliPrService::new(&runner);

    let metadata = service.pr_metadata(PR_NUMBER).unwrap();

    assert!(!metadata.is_draft);
    assert_eq!(metadata.review_decision, PrReviewDecision::ReviewRequired);
    assert!(metadata.latest_reviews.is_empty());

    let evidence = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap();
    assert!(evidence.github_actions_green());
}

#[test]
fn github_cli_pr_service_collects_draft_labels_and_review_state_for_readiness_errors() {
    let runner = RecordingRunner::new(vec![ok(draft_changes_requested_json())]);
    let service = GitHubCliPrService::new(&runner);

    let metadata = service.pr_metadata(PR_NUMBER).unwrap();

    assert!(metadata.is_draft);
    assert_eq!(
        metadata.labels,
        vec!["run-observe-readiness", "do-not-merge"]
    );
    assert_eq!(metadata.review_decision, PrReviewDecision::ChangesRequested);
    assert_eq!(metadata.latest_reviews.len(), 1);
    assert_eq!(
        metadata.latest_reviews[0].state,
        PrReviewState::ChangesRequested
    );
    assert_eq!(metadata.latest_reviews[0].commit_oid, HEAD);
}

#[test]
fn github_cli_pr_service_surfaces_external_call_failures_and_bad_json() {
    let failed_runner = RecordingRunner::new(vec![CommandOutput {
        command: expected_gh_command(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "api unavailable".into(),
    }]);
    let failed_error = GitHubCliPrService::new(&failed_runner)
        .pr_metadata(PR_NUMBER)
        .unwrap_err();
    assert_eq!(
        failed_error.kind(),
        ReadinessErrorKind::ExternalServiceFailed
    );

    let malformed_runner = RecordingRunner::new(vec![ok("{not-json}")]);
    let malformed_error = GitHubCliPrService::new(&malformed_runner)
        .pr_metadata(PR_NUMBER)
        .unwrap_err();
    assert_eq!(
        malformed_error.kind(),
        ReadinessErrorKind::MalformedExternalResponse
    );
}

#[test]
fn git_cli_pr_head_adapter_fetches_and_collects_exact_head_evidence() {
    let runner = RecordingRunner::new(vec![
        ok(""),
        ok(format!("{HEAD}\n")),
        ok(format!("{HEAD}\n")),
    ]);
    let adapter = GitCliPrHeadAdapter::new(&runner)
        .with_retry_policy(ExternalServiceRetryPolicy::new(2, Duration::from_millis(5)));

    let evidence = adapter.pr_head_evidence(BRANCH, HEAD).unwrap();

    assert_eq!(evidence.branch, BRANCH);
    assert_eq!(evidence.local_head, HEAD);
    assert_eq!(evidence.remote_head, HEAD);
    assert_eq!(evidence.pr_head_ref_oid, HEAD);
    assert!(!evidence.manually_merged);
    assert!(!evidence.rebased_or_rewritten);

    let specs = runner.specs();
    assert_eq!(specs.len(), 3);
    assert_eq!(
        specs[0].shell_display(),
        format!("git fetch origin {BRANCH}")
    );
    assert_eq!(specs[1].shell_display(), "git rev-parse HEAD");
    assert_eq!(
        specs[2].shell_display(),
        format!("git rev-parse origin/{BRANCH}")
    );
    assert!(specs.iter().all(|spec| spec.attempts == 2));
}

struct RecordingRunner {
    outputs: RefCell<VecDeque<CommandOutput>>,
    specs: RefCell<Vec<CommandSpec>>,
}

impl RecordingRunner {
    fn new(outputs: Vec<CommandOutput>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into()),
            specs: RefCell::new(Vec::new()),
        }
    }

    fn specs(&self) -> Vec<CommandSpec> {
        self.specs.borrow().clone()
    }
}

impl CommandRunner for RecordingRunner {
    fn run(&self, spec: &CommandSpec) -> anyhow::Result<CommandOutput> {
        self.specs.borrow_mut().push(spec.clone());
        Ok(self
            .outputs
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| ok("")))
    }
}

fn ok(stdout: impl Into<String>) -> CommandOutput {
    CommandOutput {
        command: String::new(),
        exit_status: Some(0),
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

fn expected_gh_command() -> String {
    format!(
        "gh pr view {PR_NUMBER} --json {}",
        "number,title,body,headRefName,headRefOid,mergeStateStatus,mergeable,isDraft,labels,reviewDecision,latestReviews,statusCheckRollup,files"
    )
}

fn json_pr_metadata() -> String {
    format!(
        r#"{{
          "number": {PR_NUMBER},
          "title": "Recover run/observe readiness evidence",
          "body": "Default-workflow recovery for PR #{PR_NUMBER}",
          "headRefName": "{BRANCH}",
          "headRefOid": "{HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "isDraft": false,
          "labels": [
            {{ "name": "run-observe-readiness" }}
          ],
          "reviewDecision": "APPROVED",
          "latestReviews": [
            {{
              "state": "APPROVED",
              "commit": {{ "oid": "{HEAD}" }},
              "author": {{ "login": "reviewer" }}
            }}
          ],
          "statusCheckRollup": [
            {{
              "__typename": "CheckRun",
              "name": "quality-gates",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": "{HEAD}"
            }},
            {{
                "__typename": "StatusContext",
                "context": "docs",
                "state": "SUCCESS"
            }},
            {{
              "__typename": "CheckRun",
              "name": "manual real Alice launch smoke",
              "status": "COMPLETED",
              "conclusion": "SKIPPED",
              "headSha": "{HEAD}"
            }}
          ],
          "files": [
            {{ "path": "docs/default-workflow-pr-readiness.md" }}
          ]
        }}"#
    )
}

fn live_pr_status_rollup_json() -> String {
    format!(
        r#"{{
          "number": {PR_NUMBER},
          "title": "Recover run/observe readiness evidence",
          "body": "Default-workflow recovery for PR #{PR_NUMBER}",
          "headRefName": "{BRANCH}",
          "headRefOid": "{HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "isDraft": false,
          "labels": [
            {{ "name": "run-observe-readiness" }}
          ],
          "reviewDecision": "REVIEW_REQUIRED",
          "latestReviews": [],
          "statusCheckRollup": [
            {{
              "name": "Build MkDocs site",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": null,
              "workflowName": "Documentation Site"
            }},
            {{
              "name": "fmt, clippy, module size",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": null,
              "workflowName": "Quality Gates"
            }},
            {{
              "name": "Deploy to GitHub Pages",
              "status": "COMPLETED",
              "conclusion": "SKIPPED",
              "headSha": null,
              "workflowName": "Documentation Site"
            }},
            {{
              "name": "manual real Alice launch smoke",
              "status": "COMPLETED",
              "conclusion": "SKIPPED",
              "headSha": null,
              "workflowName": "Quality Gates"
            }},
            {{
              "name": "GitGuardian Security Checks",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": null,
              "workflowName": ""
            }}
          ],
          "files": [
            {{ "path": "docs/default-workflow-pr-readiness.md" }}
          ]
        }}"#
    )
}

fn owner_free_review_required_json() -> String {
    format!(
        r#"{{
          "number": {PR_NUMBER},
          "title": "Recover run/observe readiness evidence",
          "body": "Default-workflow recovery for PR #{PR_NUMBER}",
          "headRefName": "{BRANCH}",
          "headRefOid": "{HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "isDraft": false,
          "labels": [],
          "reviewDecision": "REVIEW_REQUIRED",
          "latestReviews": [],
          "statusCheckRollup": [
            {{
              "name": "quality-gates",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": "{HEAD}"
            }}
          ],
          "files": [
            {{ "path": "docs/default-workflow-pr-readiness.md" }}
          ]
        }}"#
    )
}

fn draft_changes_requested_json() -> String {
    format!(
        r#"{{
          "number": {PR_NUMBER},
          "title": "Recover run/observe readiness evidence",
          "body": "Default-workflow recovery for PR #{PR_NUMBER}",
          "headRefName": "{BRANCH}",
          "headRefOid": "{HEAD}",
          "mergeStateStatus": "CLEAN",
          "mergeable": "MERGEABLE",
          "isDraft": true,
          "labels": [
            {{ "name": "run-observe-readiness" }},
            {{ "name": "do-not-merge" }}
          ],
          "reviewDecision": "CHANGES_REQUESTED",
          "latestReviews": [
            {{
              "state": "CHANGES_REQUESTED",
              "commit": {{ "oid": "{HEAD}" }},
              "author": {{ "login": "reviewer" }}
            }}
          ],
          "statusCheckRollup": [
            {{
              "name": "quality-gates",
              "status": "COMPLETED",
              "conclusion": "SUCCESS",
              "headSha": "{HEAD}"
            }}
          ],
          "files": [
            {{ "path": "docs/default-workflow-pr-readiness.md" }}
          ]
        }}"#
    )
}
