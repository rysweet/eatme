use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use eatme_alice::{
    LaunchSmokeOptions, LaunchSmokeScenario, PackageOptions, check_dependencies, discover_alice,
    package_alice, run_launch_smoke,
};
use eatme_core::RealCommandRunner;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runner = RealCommandRunner;
    match cli.command {
        Commands::Assets {
            command: AssetsCommand::Validate(args),
        } => match args.path {
            Some(path) if is_scenario_asset_path(&path) => {
                print_result(args.json, &eatme_assets::validate_scenario_asset(&path)?)?
            }
            Some(path) => print_result(args.json, &eatme_assets::validate_persona_crew(&path)?)?,
            None => print_result(args.json, &eatme_assets::validate_assets(Path::new("."))?)?,
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
        },
    }
    Ok(())
}

fn print_result<T: serde::Serialize>(_json: bool, value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
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
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn baseline_launch_smoke_keeps_compatibility_without_gate() {
        let _gate = EnvOverride::remove("EATME_REAL_ALICE");

        let result = ensure_real_alice_gate("real-alice-launch-smoke");

        assert!(result.is_ok());
    }

    #[test]
    fn lesson_launch_smoke_requires_real_alice_gate() {
        let _gate = EnvOverride::remove("EATME_REAL_ALICE");

        let result = ensure_real_alice_gate("building-a-scene-first-world");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("EATME_REAL_ALICE=1")
        );
    }

    #[test]
    fn next_lesson_launch_smoke_requires_real_alice_gate() {
        let _gate = EnvOverride::remove("EATME_REAL_ALICE");

        let result = ensure_real_alice_gate("code-editor-first-run");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("EATME_REAL_ALICE=1")
        );
    }

    #[test]
    fn hour_of_code_launch_smoke_requires_real_alice_gate() {
        let _gate = EnvOverride::remove("EATME_REAL_ALICE");

        let result = ensure_real_alice_gate("hour-of-code-studio-kickoff");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("EATME_REAL_ALICE=1")
        );
    }

    #[test]
    fn lesson_launch_smoke_accepts_explicit_real_alice_gate() {
        let _gate = EnvOverride::set("EATME_REAL_ALICE", "1");

        let result = ensure_real_alice_gate("building-a-scene-first-world");

        assert!(result.is_ok());
    }

    struct EnvOverride<'a> {
        _guard: MutexGuard<'a, ()>,
        key: &'static str,
        old_value: Option<OsString>,
    }

    impl<'a> EnvOverride<'a> {
        fn set(key: &'static str, value: &str) -> Self {
            let guard = ENV_LOCK.lock().unwrap();
            let old_value = env::var_os(key);
            unsafe {
                // SAFETY: environment mutation is process-global. ENV_LOCK keeps
                // these tests serial until Drop restores the original value.
                env::set_var(key, value);
            }
            Self {
                _guard: guard,
                key,
                old_value,
            }
        }

        fn remove(key: &'static str) -> Self {
            let guard = ENV_LOCK.lock().unwrap();
            let old_value = env::var_os(key);
            unsafe {
                // SAFETY: environment mutation is process-global. ENV_LOCK keeps
                // these tests serial until Drop restores the original value.
                env::remove_var(key);
            }
            Self {
                _guard: guard,
                key,
                old_value,
            }
        }
    }

    impl Drop for EnvOverride<'_> {
        fn drop(&mut self) {
            unsafe {
                // SAFETY: ENV_LOCK is still held until this drop completes.
                if let Some(value) = &self.old_value {
                    env::set_var(self.key, value);
                } else {
                    env::remove_var(self.key);
                }
            }
        }
    }
}
