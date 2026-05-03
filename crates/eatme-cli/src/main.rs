use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use eatme_alice::{
    LaunchSmokeOptions, PackageOptions, check_dependencies, discover_alice, package_alice,
    run_launch_smoke,
};
use eatme_core::RealCommandRunner;
use std::path::PathBuf;

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
}

#[derive(Subcommand)]
enum AssetsCommand {
    Validate(AssetsValidateArgs),
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
}

#[derive(Args)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AssetsValidateArgs {
    #[arg(long, default_value = "assets/personas/alice-user-crew.yaml")]
    path: PathBuf,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runner = RealCommandRunner;
    match cli.command {
        Commands::Assets {
            command: AssetsCommand::Validate(args),
        } => print_result(args.json, &eatme_assets::validate_persona_crew(&args.path)?)?,
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
            AliceCommand::LaunchSmoke(args) => print_result(
                args.json,
                &run_launch_smoke(&LaunchSmokeOptions {
                    alice_home: args.alice_home,
                    run_id: args.run_id,
                    runs_dir: args.runs_dir,
                    timeout_seconds: args.timeout,
                    json: args.json,
                    no_memory: args.no_memory,
                    offline_package: args.offline_package,
                })?,
            )?,
        },
    }
    Ok(())
}

fn print_result<T: serde::Serialize>(_json: bool, value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
