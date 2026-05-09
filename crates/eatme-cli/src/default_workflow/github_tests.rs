use super::*;
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use std::cell::RefCell;
use std::collections::VecDeque;

const HEAD_SHA: &str = "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d";

#[test]
fn collects_github_metadata_checks_diff_and_local_state() {
    let runner = RecordingRunner::new([
        output(pr_view_json(), 0),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output(" M docs/default-workflow-pr-readiness.md", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
    ]);

    let report = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap();

    assert_eq!(report.pr.number, 203);
    assert_eq!(report.pr.head_ref_oid, HEAD_SHA);
    assert_eq!(report.local.head, HEAD_SHA);
    assert_eq!(report.local.checkout_mode, "detached");
    assert_eq!(
        report.diff_files,
        vec!["docs/default-workflow-pr-readiness.md"]
    );
    assert_eq!(report.checks[0].name, "quality-gates");
    assert_eq!(report.checks[0].status, "COMPLETED");
    assert_eq!(report.checks[0].conclusion.as_deref(), Some("SUCCESS"));
    let commands = runner.commands();
    assert!(commands.iter().all(|command| command.attempts == 3));
    assert!(
        commands
            .iter()
            .all(|command| !command.display.starts_with("gh run list"))
    );
}

#[test]
fn checkout_option_fetches_then_switches_to_exact_head() {
    let runner = RecordingRunner::new([
        output(pr_view_json(), 0),
        output("", 0),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
    ]);

    collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: true,
        },
        &runner,
    )
    .unwrap();

    let commands = runner.commands();
    assert_eq!(
        commands[1].display,
        "git fetch origin pull/203/head".to_string()
    );
    assert_eq!(
        commands[2].display,
        format!("git switch --detach {HEAD_SHA}")
    );
}

#[test]
fn checkout_option_still_falls_back_to_workflow_runs_when_status_rollup_is_empty() {
    let runner = RecordingRunner::new([
        output(pr_view_without_status_rollup_json(), 0),
        output("", 0),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
        output(run_list_json(), 0),
    ]);

    let report = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: true,
        },
        &runner,
    )
    .unwrap();

    assert_eq!(report.checks[0].name, "quality-gates");
    let commands = runner.commands();
    assert_eq!(commands.len(), 8);
    assert_eq!(
        commands[2].display,
        format!("git switch --detach {HEAD_SHA}")
    );
    assert!(
        commands
            .iter()
            .any(|command| command.display.starts_with("gh run list"))
    );
}

#[test]
fn falls_back_to_workflow_runs_when_status_rollup_is_empty() {
    let runner = RecordingRunner::new([
        output(pr_view_without_status_rollup_json(), 0),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
        output(run_list_json(), 0),
    ]);

    let report = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap();

    assert_eq!(report.checks[0].name, "quality-gates");
    assert_eq!(report.checks[0].status, "COMPLETED");
    assert_eq!(report.checks[0].conclusion.as_deref(), Some("SUCCESS"));
    assert!(
        runner
            .commands()
            .iter()
            .any(|command| command.display.starts_with("gh run list"))
    );
}

#[test]
fn falls_back_to_workflow_runs_when_status_rollup_is_not_complete_evidence() {
    let runner = RecordingRunner::new([
        output(
            &pr_view_with_status_rollup(r#"[{"unknownShape": true}]"#),
            0,
        ),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
        output(run_list_json(), 0),
    ]);

    let report = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap();

    assert_eq!(report.checks[0].name, "quality-gates");
    assert!(
        runner
            .commands()
            .iter()
            .any(|command| command.display.starts_with("gh run list"))
    );
}

#[test]
fn external_failures_include_command_context() {
    let runner = RecordingRunner::new([output("{}", 1).with_stderr("network unavailable")]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("gh pr view 203"));
    assert!(error.contains("network unavailable"));
}

#[test]
fn rejects_invalid_remote_name_before_running_commands() {
    let runner = RecordingRunner::new([]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "--upload-pack=sh",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("invalid git remote name"));
    assert!(runner.commands().is_empty());
}

#[test]
fn rejects_invalid_pr_head_sha_before_git_fetch() {
    let runner = RecordingRunner::new([output(&pr_view_with_head("not-a-sha"), 0)]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("GitHub PR headRefOid"));
    assert_eq!(runner.commands().len(), 1);
}

#[test]
fn rejects_invalid_pr_branch_name_before_git_fetch() {
    let runner = RecordingRunner::new([output(&pr_view_with_branch("--bad-branch"), 0)]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("invalid GitHub branch name"));
    assert_eq!(runner.commands().len(), 1);
}

#[test]
fn rejects_invalid_workflow_run_head_sha() {
    let runner = RecordingRunner::new([
        output(pr_view_without_status_rollup_json(), 0),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
        output(invalid_run_list_json(), 0),
    ]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("GitHub workflow run headSha"));
}

#[test]
fn rejects_invalid_status_rollup_head_sha() {
    let runner = RecordingRunner::new([
        output(
            &pr_view_with_status_rollup(
                r#"[{
                "name": "quality-gates",
                "headSha": "not-a-sha",
                "status": "COMPLETED",
                "conclusion": "SUCCESS"
            }]"#,
            ),
            0,
        ),
        output("", 0),
        output(HEAD_SHA, 0),
        output("HEAD", 0),
        output("", 0),
        output("docs/default-workflow-pr-readiness.md", 0),
    ]);

    let error = collect_github_evidence(
        &GithubEvidenceOptions {
            pr_number: 203,
            remote: "origin",
            checkout: false,
        },
        &runner,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("GitHub status rollup headSha"));
    assert!(
        runner
            .commands()
            .iter()
            .all(|command| !command.display.starts_with("gh run list"))
    );
}

#[test]
fn maps_status_context_state_to_completed_successful_check() {
    let item = serde_json::json!({
        "context": "branch-protection",
        "state": "SUCCESS"
    });

    let check = check_from_rollup_item(&item, HEAD_SHA).unwrap().unwrap();

    assert_eq!(check.name, "branch-protection");
    assert_eq!(check.status, "COMPLETED");
    assert_eq!(check.conclusion.as_deref(), Some("SUCCESS"));
}

fn pr_view_json() -> &'static str {
    r#"{
        "state": "OPEN",
        "isDraft": false,
        "mergeable": "MERGEABLE",
        "mergeStateStatus": "CLEAN",
        "headRefOid": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
        "headRefName": "feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul",
        "body": "Exact-head evidence recorded.",
        "statusCheckRollup": [
            {
                "name": "quality-gates",
                "headSha": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
                "status": "COMPLETED",
                "conclusion": "SUCCESS"
            }
        ]
    }"#
}

fn pr_view_without_status_rollup_json() -> &'static str {
    r#"{
        "state": "OPEN",
        "isDraft": false,
        "mergeable": "MERGEABLE",
        "mergeStateStatus": "CLEAN",
        "headRefOid": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
        "headRefName": "feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul",
        "body": "Exact-head evidence recorded.",
        "statusCheckRollup": []
    }"#
}

fn pr_view_with_head(head: &str) -> String {
    format!(
        r#"{{
        "state": "OPEN",
        "isDraft": false,
        "mergeable": "MERGEABLE",
        "mergeStateStatus": "CLEAN",
        "headRefOid": "{head}",
        "headRefName": "feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul",
        "body": "Exact-head evidence recorded.",
        "statusCheckRollup": []
    }}"#
    )
}

fn pr_view_with_branch(branch: &str) -> String {
    format!(
        r#"{{
        "state": "OPEN",
        "isDraft": false,
        "mergeable": "MERGEABLE",
        "mergeStateStatus": "CLEAN",
        "headRefOid": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
        "headRefName": "{branch}",
        "body": "Exact-head evidence recorded.",
        "statusCheckRollup": []
    }}"#
    )
}

fn pr_view_with_status_rollup(status_check_rollup: &str) -> String {
    format!(
        r#"{{
        "state": "OPEN",
        "isDraft": false,
        "mergeable": "MERGEABLE",
        "mergeStateStatus": "CLEAN",
        "headRefOid": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
        "headRefName": "feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul",
        "body": "Exact-head evidence recorded.",
        "statusCheckRollup": {status_check_rollup}
    }}"#
    )
}

fn run_list_json() -> &'static str {
    r#"[{
        "databaseId": 123,
        "headSha": "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d",
        "status": "completed",
        "conclusion": "success",
        "workflowName": "quality-gates"
    }]"#
}

fn invalid_run_list_json() -> &'static str {
    r#"[{
        "databaseId": 123,
        "headSha": "not-a-sha",
        "status": "completed",
        "conclusion": "success",
        "workflowName": "quality-gates"
    }]"#
}

fn output(stdout: &str, exit_status: i32) -> CommandOutput {
    CommandOutput {
        command: String::new(),
        exit_status: Some(exit_status),
        stdout: stdout.to_string(),
        stderr: String::new(),
    }
}

trait WithStderr {
    fn with_stderr(self, stderr: &str) -> Self;
}

impl WithStderr for CommandOutput {
    fn with_stderr(mut self, stderr: &str) -> Self {
        self.stderr = stderr.to_string();
        self
    }
}

struct RecordingRunner {
    outputs: RefCell<VecDeque<CommandOutput>>,
    commands: RefCell<Vec<RecordedCommand>>,
}

#[derive(Clone)]
struct RecordedCommand {
    display: String,
    attempts: usize,
}

impl RecordingRunner {
    fn new(outputs: impl IntoIterator<Item = CommandOutput>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into_iter().collect()),
            commands: RefCell::new(Vec::new()),
        }
    }

    fn commands(&self) -> Vec<RecordedCommand> {
        self.commands.borrow().clone()
    }
}

impl CommandRunner for RecordingRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        self.commands.borrow_mut().push(RecordedCommand {
            display: spec.shell_display(),
            attempts: spec.attempts,
        });
        let mut output = self.outputs.borrow_mut().pop_front().unwrap();
        output.command = spec.shell_display();
        Ok(output)
    }
}
