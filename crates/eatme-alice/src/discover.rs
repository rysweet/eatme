use anyhow::{Result, bail};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, Serialize)]
pub struct AliceDiscovery {
    pub alice_home: String,
    pub git_commit: String,
    pub java_version: String,
    pub maven_version: String,
    pub alice_ide_jar_exists: bool,
    pub target_lib_exists: bool,
    pub starter_project_exists: bool,
}

pub fn discover_alice(alice_home: &Path, runner: &impl CommandRunner) -> Result<AliceDiscovery> {
    if !alice_home.join("pom.xml").exists() {
        bail!(
            "{} does not look like an Alice Maven repo",
            alice_home.display()
        );
    }

    let git_commit_output = runner.run(
        &CommandSpec::new("git")
            .args(["rev-parse", "HEAD"])
            .cwd(alice_home)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    ensure_success(&git_commit_output, "reading Alice git commit")?;
    let git_commit = git_commit_output.stdout.trim().to_string();
    if git_commit.is_empty() {
        bail!("reading Alice git commit returned empty output");
    }
    let java_version = runner.run(
        &CommandSpec::new("java")
            .args(["-version"])
            .timeout(Duration::from_secs(10))
            .retries(2, Duration::from_millis(100)),
    )?;
    let maven_version = runner.run(
        &CommandSpec::new("mvn")
            .args(["-version"])
            .timeout(Duration::from_secs(10))
            .retries(2, Duration::from_millis(100)),
    )?;
    ensure_success(&java_version, "reading Java version")?;
    ensure_success(&maven_version, "reading Maven version")?;

    Ok(AliceDiscovery {
        alice_home: alice_home.display().to_string(),
        git_commit,
        java_version: first_non_empty(&java_version.stderr, &java_version.stdout),
        maven_version: first_non_empty(&maven_version.stdout, &maven_version.stderr),
        alice_ide_jar_exists: alice_ide_jar_exists(alice_home),
        target_lib_exists: alice_home.join("alice-ide/target/lib").is_dir(),
        starter_project_exists: alice_home
            .join("core/resources/target/distribution/application/starter-projects/africa.a3p")
            .exists(),
    })
}

pub fn first_non_empty(primary: &str, fallback: &str) -> String {
    primary
        .lines()
        .find(|line| !line.trim().is_empty())
        .or_else(|| fallback.lines().find(|line| !line.trim().is_empty()))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn alice_ide_jar_exists(alice_home: &Path) -> bool {
    let target = alice_home.join("alice-ide/target");
    fs::read_dir(target)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .any(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    name.starts_with("alice-ide-")
                        && name.ends_with(".jar")
                        && !name.contains("-sources")
                        && !name.contains("-javadoc")
                })
                .unwrap_or(false)
        })
}

fn ensure_success(output: &CommandOutput, action: &str) -> Result<()> {
    if output.exit_status == Some(0) {
        return Ok(());
    }
    bail!(
        "{action} failed with {:?}\n{}{}",
        output.exit_status,
        output.stdout,
        output.stderr
    )
}
