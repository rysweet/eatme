use anyhow::{Result, bail};
use clap::Args;
use eatme_alice::check_dependencies;
use eatme_core::CommandRunner;
use std::path::Path;
use std::path::PathBuf;

#[derive(Args)]
pub struct AssetsGradingReportArgs {
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
}

pub fn run_grading_report(
    args: &AssetsGradingReportArgs,
    runner: &impl CommandRunner,
) -> Result<()> {
    let ar = eatme_assets::validate_assets(Path::new(&args.path))?;
    let dr = check_dependencies(runner)?;
    let asset_reason = if ar.passed {
        format!(
            "All {} scenario assets passed validation",
            ar.scenario_asset_count
        )
    } else {
        format!("{} errors found", ar.errors.len())
    };
    let missing: Vec<_> = dr
        .tools
        .iter()
        .filter(|(_, v)| !*v)
        .map(|(k, _)| k.as_str())
        .collect();
    let deps_reason: String = if dr.all_required_available {
        "All required tools available".into()
    } else {
        format!("Missing required tools: {}", missing.join(", "))
    };
    let report = eatme_assets::grade_first_lesson_readiness(eatme_assets::GradingInput {
        assets_valid: ar.passed,
        asset_reason,
        deps_available: dr.all_required_available,
        deps_reason,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.passed {
        bail!("first-lesson grading report: not all steps ready");
    }
    Ok(())
}
