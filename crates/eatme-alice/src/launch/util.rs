use super::evidence::artifact_info;
use anyhow::{Result, bail};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec};
use std::path::Path;
use std::process::Child;
use std::time::{Duration, Instant};

pub(super) fn wait_for_start(child: &mut Child, seconds: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds.clamp(5, 60));
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    true
}

pub(super) fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

pub(super) fn capture_text_or_error<T: Default>(result: Result<T>) -> (T, Option<String>) {
    match result {
        Ok(value) => (value, None),
        Err(error) => (T::default(), Some(format!("{error:#}"))),
    }
}

pub(super) fn capture_artifact_or_error<T>(result: Result<T>) -> (Option<T>, Option<String>) {
    match result {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(format!("{error:#}"))),
    }
}

pub(super) fn artifact_or_error(path: &Path) -> (Option<ArtifactInfo>, Option<String>) {
    capture_artifact_or_error(artifact_info(path))
}

pub(super) fn combine_errors(errors: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    let errors = errors.into_iter().flatten().collect::<Vec<_>>();
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("\n"))
    }
}

pub(super) fn git_commit(path: &Path, runner: &impl CommandRunner) -> Result<String> {
    let output = runner.run(
        &CommandSpec::new("git")
            .args(["rev-parse", "HEAD"])
            .cwd(path)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if output.exit_status != Some(0) {
        bail!(
            "reading git commit in {} failed with {:?}\n{}{}",
            path.display(),
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    Ok(output.stdout.trim().to_string())
}

pub(super) fn validate_scenario_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('-')
        || name.ends_with('-')
        || !name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("launch smoke scenario {name:?} must be kebab-case");
    }
    Ok(())
}
