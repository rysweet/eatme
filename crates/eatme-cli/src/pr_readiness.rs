use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use eatme_core::default_workflow_pr_readiness::{
    Decision, FinalizationDecision, FinalizationEvidence, HandoffOptions, evaluate_finalization,
    render_handoff,
};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum PrReadinessCommand {
    Finalize(FinalizePrReadinessArgs),
}

#[derive(Args)]
pub struct FinalizePrReadinessArgs {
    #[arg(long)]
    pr: u64,
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(command: PrReadinessCommand) -> Result<()> {
    match command {
        PrReadinessCommand::Finalize(args) => finalize(args),
    }
}

fn finalize(args: FinalizePrReadinessArgs) -> Result<()> {
    let evidence_text = fs::read_to_string(&args.evidence)?;
    let evidence = FinalizationEvidence::from_offline_json(&evidence_text)?;
    if evidence.pr_number != args.pr {
        bail!(
            "evidence PR number {} does not match requested PR {}",
            evidence.pr_number,
            args.pr
        );
    }
    let decision = evaluate_finalization(evidence.clone());
    let handoff = if decision.decision == Decision::MergeReady {
        Some(render_handoff(
            &evidence,
            &decision,
            HandoffOptions::owner_free(),
        )?)
    } else {
        None
    };
    let report = PrReadinessReport::new(&decision, handoff, args.dry_run);
    print_report(args.json, &report)?;
    if decision.decision != Decision::MergeReady {
        bail!("PR readiness finalization did not reach MERGE_READY");
    }
    Ok(())
}

fn print_report(_json: bool, report: &PrReadinessReport) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

#[derive(Serialize)]
struct PrReadinessReport {
    decision: Decision,
    no_op: bool,
    dry_run: bool,
    blockers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    no_op_justification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handoff: Option<String>,
}

impl PrReadinessReport {
    fn new(
        decision: &FinalizationDecision,
        handoff: Option<String>,
        dry_run: bool,
    ) -> PrReadinessReport {
        Self {
            decision: decision.decision.clone(),
            no_op: decision.no_op_justification.is_some(),
            dry_run,
            blockers: decision.blockers.clone(),
            no_op_justification: decision.no_op_justification.clone(),
            handoff,
        }
    }
}
