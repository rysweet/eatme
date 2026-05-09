use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use eatme_alice::{
    AliceComparisonOptions, FIRST_LESSON_SCENARIO_ID, FirstLessonReadinessOptions,
    LaunchSmokeOptions, LaunchSmokeScenario, PackageOptions, check_dependencies, discover_alice,
    package_alice, run_first_lesson_readiness_sequence, run_launch_smoke,
    run_launch_smoke_comparison,
};
use eatme_core::RealCommandRunner;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod first_lesson;
use first_lesson::{RunFirstLessonReadinessArgs, print_first_lesson_readiness_result};
mod default_workflow;
mod lesson_contract;
use lesson_contract::{print_lesson_readiness_check, print_lesson_session_check};

#[derive(Parser)]
#[command(name = "eatme")]
#[command(about = "Agentic Alice outside-in QA harness")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Assets {
        #[command(subcommand)]
        command: AssetsCommand,
    },
    Deps {
        #[command(subcommand)]
        command: DepsCommand,
    },
    Alice {
        #[command(subcommand)]
        command: AliceCommand,
    },
    DefaultWorkflow {
        #[command(subcommand)]
        command: DefaultWorkflowCommand,
    },
}

#[derive(Subcommand)]
enum AssetsCommand {
    Validate(AssetsValidateArgs),
    GenerateGadugi(AssetsGenerateGadugiArgs),
}

#[derive(Subcommand)]
enum DepsCommand {
    Check(JsonFlag),
}

#[derive(Subcommand)]
enum AliceCommand {
    Discover(AliceHomeArgs),
    Package(PackageArgs),
    LaunchSmoke(LaunchSmokeArgs),
    CompareLaunchSmoke(CompareLaunchSmokeArgs),
    CheckLessonSession(CheckLessonSessionArgs),
    CheckLessonReadiness(CheckLessonSessionArgs),
    RunFirstLessonReadiness(RunFirstLessonReadinessArgs),
}

#[derive(Subcommand)]
enum DefaultWorkflowCommand {
    PrReadiness(DefaultWorkflowPrReadinessArgs),
    CollectGithubEvidence(DefaultWorkflowCollectGithubEvidenceArgs),
}

#[derive(Args)]
struct DefaultWorkflowPrReadinessArgs {
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DefaultWorkflowCollectGithubEvidenceArgs {
    #[arg(long)]
    pr: u64,
    #[arg(long, default_value = "origin")]
    remote: String,
    #[arg(long)]
    checkout: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AssetsValidateArgs {
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AssetsGenerateGadugiArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AliceHomeArgs {
    #[arg(long, env = "ALICE_HOME")]
    alice_home: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct PackageArgs {
    #[arg(long, env = "ALICE_HOME")]
    alice_home: PathBuf,
    #[arg(long)]
    offline: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LaunchSmokeArgs {
    #[arg(long, env = "ALICE_HOME")]
    alice_home: PathBuf,
    #[arg(long)]
    run_id: String,
    #[arg(long, default_value = "runs")]
    runs_dir: PathBuf,
    #[arg(long, default_value_t = 120)]
    timeout: u64,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    no_memory: bool,
    #[arg(long)]
    offline_package: bool,
    #[arg(long, default_value = "real-alice-launch-smoke")]
    scenario: String,
    #[arg(long)]
    starter_project: Option<PathBuf>,
}

#[derive(Args)]
struct CompareLaunchSmokeArgs {
    #[arg(long, default_value = "assets/alice-comparison-targets.yaml")]
    registry: PathBuf,
    #[arg(long, default_value = "baseline")]
    baseline_target: String,
    #[arg(long, default_value = "modernized")]
    modernized_target: String,
    #[arg(long)]
    baseline_home: Option<PathBuf>,
    #[arg(long)]
    modernized_home: Option<PathBuf>,
    #[arg(long)]
    run_id: String,
    #[arg(long, default_value = "runs")]
    runs_dir: PathBuf,
    #[arg(long, default_value_t = 120)]
    timeout: u64,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    no_memory: bool,
    #[arg(long)]
    offline_package: bool,
    #[arg(long, default_value = "real-alice-launch-smoke")]
    scenario: String,
    #[arg(long)]
    starter_project: Option<PathBuf>,
    #[arg(long)]
    execute: bool,
}

#[derive(Args)]
struct CheckLessonSessionArgs {
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runner = RealCommandRunner;
    match cli.command {
        Commands::Assets {
            command: AssetsCommand::Validate(args),
        } => match args.path {
            Some(path) if is_scenario_asset_path(&path) => print_scenario_validation_result(
                args.json,
                &eatme_assets::validate_scenario_asset(&path)?,
            )?,
            Some(path) => print_asset_validation_result(
                args.json,
                &eatme_assets::validate_persona_crew(&path)?,
            )?,
            None => print_asset_validation_result(
                args.json,
                &eatme_assets::validate_assets(Path::new("."))?,
            )?,
        },
        Commands::Assets {
            command: AssetsCommand::GenerateGadugi(args),
        } => {
            let report = eatme_assets::generate_gadugi_adapters(&args.root, args.check)?;
            print_result(args.json, &report)?;
            if !report.passed {
                bail!("gadugi adapter generation check failed");
            }
        }
        Commands::Deps {
            command: DepsCommand::Check(args),
        } => print_result(args.json, &check_dependencies(&runner)?)?,
        Commands::Alice { command } => match command {
            AliceCommand::Discover(args) => {
                print_result(args.json, &discover_alice(&args.alice_home, &runner)?)?
            }
            AliceCommand::Package(args) => print_result(
                args.json,
                &package_alice(
                    PackageOptions {
                        alice_home: &args.alice_home,
                        offline: args.offline,
                    },
                    &runner,
                )?,
            )?,
            AliceCommand::LaunchSmoke(args) => {
                ensure_real_alice_gate(&args.scenario)?;
                let mut scenario = LaunchSmokeScenario::new(args.scenario);
                if let Some(starter_project) = args.starter_project {
                    scenario = scenario.with_starter_project(starter_project);
                }
                let manifest = run_launch_smoke(&LaunchSmokeOptions {
                    alice_home: args.alice_home,
                    run_id: args.run_id,
                    runs_dir: args.runs_dir,
                    timeout_seconds: args.timeout,
                    json: args.json,
                    no_memory: args.no_memory,
                    offline_package: args.offline_package,
                    scenario,
                })?;
                print_result(args.json, &manifest)?;
                if let Some(category) = manifest.failure_category {
                    bail!("launch smoke failed: {category}");
                }
            }
            AliceCommand::CompareLaunchSmoke(args) => {
                if args.execute {
                    ensure_real_alice_gate(&args.scenario)?;
                }
                let mut scenario = LaunchSmokeScenario::new(args.scenario);
                if let Some(starter_project) = args.starter_project {
                    scenario = scenario.with_starter_project(starter_project);
                }
                let manifest = run_launch_smoke_comparison(&AliceComparisonOptions {
                    registry_path: args.registry,
                    baseline_target: args.baseline_target,
                    modernized_target: args.modernized_target,
                    baseline_home_override: args.baseline_home,
                    modernized_home_override: args.modernized_home,
                    scenario,
                    run_id: args.run_id,
                    runs_dir: args.runs_dir,
                    timeout_seconds: args.timeout,
                    json: args.json,
                    no_memory: args.no_memory,
                    offline_package: args.offline_package,
                    execute: args.execute,
                })?;
                print_result(args.json, &manifest)?;
            }
            AliceCommand::CheckLessonSession(args) => {
                print_lesson_session_check(&args.manifest, args.json)?;
            }
            AliceCommand::CheckLessonReadiness(args) => {
                print_lesson_readiness_check(&args.manifest, args.json)?;
            }
            AliceCommand::RunFirstLessonReadiness(args) => {
                if args.execute {
                    ensure_real_alice_gate(FIRST_LESSON_SCENARIO_ID)?;
                }
                let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
                    registry_path: args.registry,
                    baseline_target: args.baseline_target,
                    modernized_target: args.modernized_target,
                    baseline_home_override: args.baseline_home,
                    modernized_home_override: args.modernized_home,
                    run_id: args.run_id,
                    runs_dir: args.runs_dir,
                    timeout_seconds: args.timeout,
                    json: args.json,
                    no_memory: args.no_memory,
                    offline_package: args.offline_package,
                    execute: args.execute,
                    starter_project: args.starter_project,
                })?;
                print_first_lesson_readiness_result(args.json, &report)?;
                if !report.passed {
                    bail!("first-lesson readiness sequence incomplete");
                }
            }
        },
        Commands::DefaultWorkflow {
            command: DefaultWorkflowCommand::PrReadiness(args),
        } => {
            let outcome = default_workflow::evaluate_pr_readiness_evidence(&args.evidence);
            print_result(args.json, &outcome.report)?;
            if outcome.exit_code != 0 {
                std::process::exit(outcome.exit_code);
            }
        }
        Commands::DefaultWorkflow {
            command: DefaultWorkflowCommand::CollectGithubEvidence(args),
        } => {
            let report = default_workflow::github::collect_github_evidence(
                &default_workflow::github::GithubEvidenceOptions {
                    pr_number: args.pr,
                    remote: &args.remote,
                    checkout: args.checkout,
                },
                &runner,
            )?;
            print_result(args.json, &report)?;
        }
    }
    Ok(())
}

fn print_result<T: serde::Serialize>(_json: bool, value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_asset_validation_result(
    json: bool,
    report: &eatme_assets::AssetValidationReport,
) -> Result<()> {
    print_result(json, report)?;
    if !report.passed {
        bail!("asset validation failed");
    }
    Ok(())
}

fn print_scenario_validation_result(
    json: bool,
    report: &eatme_assets::ScenarioAssetValidationReport,
) -> Result<()> {
    print_result(json, report)?;
    if !report.passed {
        bail!("scenario validation failed");
    }
    Ok(())
}

fn is_scenario_asset_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_string_lossy() == "scenarios")
        || file_declares_eatme_scenario(path)
}

fn file_declares_eatme_scenario(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|content| {
            content.lines().any(|line| {
                matches!(
                    line.trim(),
                    "schema_version: eatme.scenario/v1"
                        | "schema_version: \"eatme.scenario/v1\""
                        | "schema_version: 'eatme.scenario/v1'"
                )
            })
        })
        .unwrap_or(false)
}

fn ensure_real_alice_gate(scenario: &str) -> Result<()> {
    if scenario != "real-alice-launch-smoke" && env::var("EATME_REAL_ALICE").as_deref() != Ok("1") {
        bail!("launch smoke scenario {scenario} requires EATME_REAL_ALICE=1");
    }
    Ok(())
}

#[cfg(test)]
mod main_tests;
