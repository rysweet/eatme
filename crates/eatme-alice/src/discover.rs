use anyhow::{Result, bail};
use eatme_core::{CommandRunner, CommandSpec};
use serde::Serialize;
use std::path::Path;

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

    let git_commit = runner
        .run(
            &CommandSpec::new("git")
                .args(["rev-parse", "HEAD"])
                .cwd(alice_home),
        )?
        .stdout
        .trim()
        .to_string();
    let java_version = runner.run(&CommandSpec::new("java").args(["-version"]))?;
    let maven_version = runner.run(&CommandSpec::new("mvn").args(["-version"]))?;

    Ok(AliceDiscovery {
        alice_home: alice_home.display().to_string(),
        git_commit,
        java_version: first_non_empty(&java_version.stderr, &java_version.stdout),
        maven_version: first_non_empty(&maven_version.stdout, &maven_version.stderr),
        alice_ide_jar_exists: alice_home
            .join("alice-ide/target/alice-ide-9.1.0-SNAPSHOT.jar")
            .exists(),
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
