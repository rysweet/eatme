use anyhow::Result;
use clap::Args;
use eatme_alice::compare::FirstLessonReadinessSequenceReport;
use std::path::PathBuf;

#[derive(Args)]
pub struct RunFirstLessonReadinessArgs {
    #[arg(long, default_value = "assets/alice-comparison-targets.yaml")]
    pub registry: PathBuf,
    #[arg(long, default_value = "baseline")]
    pub baseline_target: String,
    #[arg(long, default_value = "modernized")]
    pub modernized_target: String,
    #[arg(long)]
    pub baseline_home: Option<PathBuf>,
    #[arg(long)]
    pub modernized_home: Option<PathBuf>,
    #[arg(long)]
    pub run_id: String,
    #[arg(long, default_value = "runs")]
    pub runs_dir: PathBuf,
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub no_memory: bool,
    #[arg(long)]
    pub offline_package: bool,
    #[arg(long)]
    pub starter_project: Option<PathBuf>,
    #[arg(long)]
    pub execute: bool,
}

pub fn print_first_lesson_readiness_result(
    json: bool,
    report: &FirstLessonReadinessSequenceReport,
) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("First-lesson readiness: {}", report.readiness_status);
    println!("Evidence progress: {}", report.evidence_progress.summary);
    println!("Required evidence:");
    for item in &report.evidence_progress.items {
        println!("- {}: {} ({})", item.state, item.evidence, item.detail);
    }
    if !report.limitations.is_empty() {
        println!("Limits:");
        for limitation in &report.limitations {
            println!("- {limitation}");
        }
    }
    if !report.issues.is_empty() {
        println!("Still missing or blocked:");
        for issue in &report.issues {
            println!("- {issue}");
        }
    }
    Ok(())
}
