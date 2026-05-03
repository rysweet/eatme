use anyhow::{Result, bail};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PackageOptions<'a> {
    pub alice_home: &'a Path,
    pub offline: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PackageResult {
    pub command: String,
    pub exit_status: Option<i32>,
}

pub fn package_alice(
    options: PackageOptions<'_>,
    runner: &impl CommandRunner,
) -> Result<PackageResult> {
    let output = run_package_command(options, runner)?;
    if output.exit_status != Some(0) {
        bail!(
            "Alice package failed with {:?}\n{}{}",
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    Ok(PackageResult {
        command: output.command,
        exit_status: output.exit_status,
    })
}

pub fn run_package_command(
    options: PackageOptions<'_>,
    runner: &impl CommandRunner,
) -> Result<CommandOutput> {
    let mut args = Vec::new();
    if options.offline {
        args.push("-o".to_string());
    }
    args.extend([
        "-DskipTests".to_string(),
        "-DincludeSims=false".to_string(),
        "-Dinstall4j.skip".to_string(),
        "-pl".to_string(),
        "alice-ide".to_string(),
        "-am".to_string(),
        "package".to_string(),
    ]);
    runner.run(
        &CommandSpec::new("mvn")
            .args(args)
            .cwd(options.alice_home)
            .timeout(Duration::from_secs(30 * 60)),
    )
}
