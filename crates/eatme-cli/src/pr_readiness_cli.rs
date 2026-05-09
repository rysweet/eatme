use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use eatme_core::CommandRunner;
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::pr_readiness;

#[derive(Subcommand)]
pub enum PrReadinessCommand {
    RecoveryEvaluate(PrReadinessRecoveryEvaluateArgs),
    GithubSnapshot(PrReadinessGithubSnapshotArgs),
}

#[derive(Args)]
pub struct PrReadinessRecoveryEvaluateArgs {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub struct PrReadinessGithubSnapshotArgs {
    #[arg(long)]
    owner: String,
    #[arg(long)]
    repo: String,
    #[arg(long)]
    pr_number: u64,
    #[arg(long)]
    local_head_sha: String,
    #[arg(long = "required-check", required = true)]
    required_checks: Vec<String>,
    #[arg(long)]
    json: bool,
}

pub fn run(command: PrReadinessCommand, runner: &impl CommandRunner) -> Result<()> {
    match command {
        PrReadinessCommand::RecoveryEvaluate(args) => run_recovery_evaluate(args),
        PrReadinessCommand::GithubSnapshot(args) => run_github_snapshot(args, runner),
    }
}

fn run_recovery_evaluate(args: PrReadinessRecoveryEvaluateArgs) -> Result<()> {
    let input_file = File::open(&args.input)?;
    let input: pr_readiness::RecoveryReadinessInput = serde_json::from_reader(input_file)?;
    let report = pr_readiness::evaluate_recovery_readiness(&input);
    if args.json {
        write_json_pretty(&report)?;
    } else {
        println!("{}", pr_readiness::render_final_report(&report));
    }
    if report.status == pr_readiness::RecoveryReadinessStatus::NotMergeReady {
        bail!("recovery readiness blocked");
    }
    Ok(())
}

fn run_github_snapshot(
    args: PrReadinessGithubSnapshotArgs,
    runner: &impl CommandRunner,
) -> Result<()> {
    let request = pr_readiness::GitHubPrSnapshotRequest {
        owner: args.owner,
        repo: args.repo,
        pr_number: args.pr_number,
        local_head_sha: args.local_head_sha,
        required_checks: args.required_checks,
    };
    let snapshot = pr_readiness::fetch_github_pr_snapshot(&request, runner)?;
    if args.json {
        write_json_pretty(&snapshot)?;
    } else {
        println!(
            "PR #{} {} at {} with {} checks",
            snapshot.pr_number,
            snapshot.branch,
            snapshot.pr_head_sha,
            snapshot.checks.len()
        );
    }
    Ok(())
}

fn write_json_pretty(value: &impl Serialize) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}
