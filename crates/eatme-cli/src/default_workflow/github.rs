use super::EVIDENCE_SCHEMA_VERSION;
use super::types::{
    CheckEvidence, ExternalServiceCall, GithubEvidenceReport, LocalEvidence, PrEvidence,
};
use anyhow::{Context, Result, bail};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

const EXTERNAL_ATTEMPTS: usize = 3;
const EXTERNAL_RETRY_DELAY_MS: u64 = 500;

pub(crate) struct GithubEvidenceOptions<'a> {
    pub(crate) pr_number: u64,
    pub(crate) remote: &'a str,
    pub(crate) checkout: bool,
}

pub(crate) fn collect_github_evidence(
    options: &GithubEvidenceOptions<'_>,
    runner: &impl CommandRunner,
) -> Result<GithubEvidenceReport> {
    validate_pr_number(options.pr_number)?;
    validate_remote_name(options.remote)?;

    let mut calls = Vec::with_capacity(8);
    let pr_output = run_external(
        runner,
        "github",
        CommandSpec::new("gh").args([
            "pr".to_string(),
            "view".to_string(),
            options.pr_number.to_string(),
            "--json".to_string(),
            "state,isDraft,mergeable,mergeStateStatus,headRefOid,headRefName,statusCheckRollup,body"
                .to_string(),
        ]),
        &mut calls,
    )?;
    let pr_view = parse_json::<GhPrView>(&pr_output, "GitHub PR metadata")?;
    validate_object_id("GitHub PR headRefOid", &pr_view.head_ref_oid)?;
    validate_branch_name(&pr_view.head_ref_name)?;

    run_external(
        runner,
        "git",
        CommandSpec::new("git").args([
            "fetch".to_string(),
            options.remote.to_string(),
            format!("pull/{}/head", options.pr_number),
        ]),
        &mut calls,
    )?;

    if options.checkout {
        run_external(
            runner,
            "git",
            CommandSpec::new("git").args([
                "switch".to_string(),
                "--detach".to_string(),
                pr_view.head_ref_oid.clone(),
            ]),
            &mut calls,
        )?;
    }

    let local_head = run_external(
        runner,
        "git",
        CommandSpec::new("git").args(["rev-parse", "HEAD"]),
        &mut calls,
    )?;
    let checkout_mode = checkout_mode(runner, &mut calls)?;
    let repository_changes = repository_changes(runner, &mut calls)?;
    let diff_files = pr_diff_files(runner, options.pr_number, &mut calls)?;
    let checks = match checks_from_status_rollup(&pr_view)? {
        Some(checks) => checks,
        None => workflow_run_checks(runner, &pr_view.head_ref_name, &mut calls)?,
    };

    Ok(GithubEvidenceReport {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
        pr: PrEvidence {
            number: options.pr_number,
            state: pr_view.state,
            is_draft: pr_view.is_draft,
            mergeable: pr_view.mergeable,
            merge_state_status: pr_view.merge_state_status,
            head_ref_oid: pr_view.head_ref_oid,
            head_ref_name: pr_view.head_ref_name,
        },
        local: LocalEvidence {
            head: single_line_stdout(&local_head, "git rev-parse HEAD")?,
            checkout_mode,
            manual_merge_performed: false,
            repository_changes,
        },
        checks,
        diff_files,
        pr_body: pr_view.body.unwrap_or_default(),
        service_calls: calls,
    })
}

fn run_external(
    runner: &impl CommandRunner,
    service: &str,
    mut spec: CommandSpec,
    calls: &mut Vec<ExternalServiceCall>,
) -> Result<CommandOutput> {
    spec = spec.retries(
        EXTERNAL_ATTEMPTS,
        Duration::from_millis(EXTERNAL_RETRY_DELAY_MS),
    );
    let command = spec.shell_display();
    let output = runner
        .run(&spec)
        .with_context(|| format!("calling external service command `{command}`"))?;
    calls.push(ExternalServiceCall {
        service: service.to_string(),
        command,
        attempts: EXTERNAL_ATTEMPTS,
        exit_status: output.exit_status,
    });
    if output.exit_status != Some(0) {
        bail!(
            "external service command `{}` failed with status {:?}: {}",
            output.command,
            output.exit_status,
            output.stderr.trim()
        );
    }
    Ok(output)
}

fn parse_json<T: for<'de> Deserialize<'de>>(output: &CommandOutput, label: &str) -> Result<T> {
    serde_json::from_str(&output.stdout)
        .with_context(|| format!("parsing {label} JSON from `{}`", output.command))
}

fn checkout_mode(
    runner: &impl CommandRunner,
    calls: &mut Vec<ExternalServiceCall>,
) -> Result<String> {
    let output = run_external(
        runner,
        "git",
        CommandSpec::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]),
        calls,
    )?;
    let branch = single_line_stdout(&output, "git rev-parse --abbrev-ref HEAD")?;
    Ok(if branch == "HEAD" {
        "detached".to_string()
    } else {
        branch
    })
}

fn repository_changes(
    runner: &impl CommandRunner,
    calls: &mut Vec<ExternalServiceCall>,
) -> Result<Vec<String>> {
    let output = run_external(
        runner,
        "git",
        CommandSpec::new("git").args(["status", "--short"]),
        calls,
    )?;
    Ok(non_empty_lines(&output.stdout))
}

fn pr_diff_files(
    runner: &impl CommandRunner,
    pr_number: u64,
    calls: &mut Vec<ExternalServiceCall>,
) -> Result<Vec<String>> {
    let output = run_external(
        runner,
        "github",
        CommandSpec::new("gh").args([
            "pr".to_string(),
            "diff".to_string(),
            pr_number.to_string(),
            "--name-only".to_string(),
        ]),
        calls,
    )?;
    Ok(non_empty_lines(&output.stdout))
}

fn workflow_run_checks(
    runner: &impl CommandRunner,
    branch: &str,
    calls: &mut Vec<ExternalServiceCall>,
) -> Result<Vec<CheckEvidence>> {
    validate_branch_name(branch)?;

    let output = run_external(
        runner,
        "github",
        CommandSpec::new("gh").args([
            "run".to_string(),
            "list".to_string(),
            "--branch".to_string(),
            branch.to_string(),
            "--json".to_string(),
            "databaseId,headSha,status,conclusion,workflowName".to_string(),
        ]),
        calls,
    )?;
    let runs = parse_json::<Vec<GhWorkflowRun>>(&output, "GitHub workflow run list")?;
    runs.into_iter()
        .map(|run| {
            validate_object_id("GitHub workflow run headSha", &run.head_sha)?;
            Ok(CheckEvidence {
                name: run.workflow_name,
                head_sha: run.head_sha,
                status: normalize_status(&run.status),
                conclusion: run.conclusion.map(|value| normalize_status(&value)),
                required: true,
            })
        })
        .collect()
}

fn checks_from_status_rollup(pr: &GhPrView) -> Result<Option<Vec<CheckEvidence>>> {
    if pr.status_check_rollup.is_empty() {
        return Ok(None);
    }

    let mut checks = Vec::with_capacity(pr.status_check_rollup.len());
    for item in &pr.status_check_rollup {
        match check_from_rollup_item(item, &pr.head_ref_oid)? {
            Some(check) => checks.push(check),
            None => return Ok(None),
        }
    }

    Ok(Some(checks))
}

fn check_from_rollup_item(item: &Value, head_ref_oid: &str) -> Result<Option<CheckEvidence>> {
    let Some(name) = item
        .get("name")
        .or_else(|| item.get("context"))
        .or_else(|| item.get("workflowName"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
    else {
        return Ok(None);
    };
    let status = if let Some(status) = item.get("status").and_then(Value::as_str) {
        normalize_status(status)
    } else if let Some(state) = item.get("state").and_then(Value::as_str) {
        status_from_status_context_state(state)
    } else {
        "UNKNOWN".to_string()
    };
    let conclusion = item
        .get("conclusion")
        .and_then(Value::as_str)
        .map(normalize_status)
        .or_else(|| {
            item.get("state")
                .and_then(Value::as_str)
                .and_then(conclusion_from_status_context_state)
        });
    let head_sha = match item.get("headSha").and_then(Value::as_str) {
        Some(head_sha) => {
            validate_object_id("GitHub status rollup headSha", head_sha)?;
            head_sha.to_string()
        }
        None => head_ref_oid.to_string(),
    };

    Ok(Some(CheckEvidence {
        name,
        head_sha,
        status,
        conclusion,
        required: true,
    }))
}

fn normalize_status(value: &str) -> String {
    value.replace('-', "_").to_ascii_uppercase()
}

fn status_from_status_context_state(value: &str) -> String {
    match normalize_status(value).as_str() {
        "SUCCESS" | "FAILURE" | "ERROR" => "COMPLETED".to_string(),
        other => other.to_string(),
    }
}

fn conclusion_from_status_context_state(value: &str) -> Option<String> {
    match normalize_status(value).as_str() {
        "SUCCESS" => Some("SUCCESS".to_string()),
        "FAILURE" | "ERROR" => Some("FAILURE".to_string()),
        _ => None,
    }
}

fn single_line_stdout(output: &CommandOutput, label: &str) -> Result<String> {
    let value = output.stdout.trim();
    if value.is_empty() {
        bail!("{label} returned empty stdout");
    }
    Ok(value.lines().next().unwrap_or_default().to_string())
}

fn non_empty_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn validate_pr_number(pr_number: u64) -> Result<()> {
    if pr_number == 0 {
        bail!("PR number must be greater than zero");
    }
    Ok(())
}

fn validate_remote_name(remote: &str) -> Result<()> {
    let valid = !remote.is_empty()
        && remote.len() <= 100
        && !remote.starts_with('-')
        && !remote.starts_with('.')
        && !remote.ends_with('.')
        && !remote.contains("..")
        && remote
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));

    if !valid {
        bail!(
            "invalid git remote name; expected a configured remote name using ASCII letters, digits, '.', '_' or '-'"
        );
    }
    Ok(())
}

fn validate_object_id(label: &str, oid: &str) -> Result<()> {
    if matches!(oid.len(), 40 | 64) && oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }

    bail!("{label} must be a 40- or 64-character hexadecimal object ID");
}

fn validate_branch_name(branch: &str) -> Result<()> {
    let valid = !branch.is_empty()
        && branch.len() <= 255
        && !branch.starts_with('-')
        && !branch.starts_with('/')
        && !branch.ends_with('/')
        && !branch.ends_with('.')
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.contains("@{")
        && !branch.split('/').any(|component| {
            component.is_empty() || component.starts_with('.') || component.ends_with(".lock")
        })
        && branch
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'));

    if !valid {
        bail!(
            "invalid GitHub branch name from PR metadata; expected a safe ref name using ASCII letters, digits, '/', '.', '_' or '-'"
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrView {
    state: String,
    is_draft: bool,
    mergeable: String,
    merge_state_status: String,
    head_ref_oid: String,
    head_ref_name: String,
    body: Option<String>,
    #[serde(default)]
    status_check_rollup: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhWorkflowRun {
    head_sha: String,
    status: String,
    conclusion: Option<String>,
    workflow_name: String,
}

#[cfg(test)]
#[path = "github_tests.rs"]
mod github_tests;
