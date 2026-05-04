use crate::discover::first_non_empty;
use anyhow::{Context, Result, bail};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec, file_size, sha256_file};
use std::fs;
use std::path::Path;
use std::time::Duration;

const ALICE_WINDOW_MARKERS: [&str; 4] = [
    "org.alice.stageide.entrypoint",
    "org.alice.stageide",
    "org.alice.ide",
    "alice 3",
];

pub(crate) fn capture_window_list(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<String> {
    let output = runner.run(
        &CommandSpec::new("wmctrl")
            .args(["-lx"])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if output.exit_status != Some(0) {
        bail!(
            "capturing window list failed with {:?}\n{}{}",
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    let combined = first_non_empty(&output.stdout, &output.stderr);
    fs::write(run_dir.join("window-list.txt"), &combined)?;
    Ok(combined)
}

pub(crate) fn has_alice_window_evidence(window_list: &str) -> bool {
    window_list.lines().any(|line| {
        let normalized = line.to_ascii_lowercase();
        ALICE_WINDOW_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
    })
}

pub(crate) fn capture_screenshot(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("screenshots/startup.png");
    let scrot = CommandSpec::new("scrot")
        .args([path.display().to_string()])
        .env("DISPLAY", display)
        .timeout(Duration::from_secs(10))
        .retries(2, Duration::from_millis(100));
    let output = runner.run(&scrot)?;
    if output.exit_status != Some(0) {
        let fallback = runner.run(
            &CommandSpec::new("import")
                .args(["-window".into(), "root".into(), path.display().to_string()])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(10))
                .retries(2, Duration::from_millis(100)),
        )?;
        if fallback.exit_status != Some(0) {
            bail!(
                "capturing startup screenshot failed: scrot={:?}, import={:?}\nscrot stdout:\n{}scrot stderr:\n{}import stdout:\n{}import stderr:\n{}",
                output.exit_status,
                fallback.exit_status,
                output.stdout,
                output.stderr,
                fallback.stdout,
                fallback.stderr
            );
        }
    }
    artifact_info(&path).with_context(|| format!("capturing screenshot {}", path.display()))
}

pub(crate) fn artifact_info(path: &Path) -> Result<ArtifactInfo> {
    Ok(ArtifactInfo {
        path: path.display().to_string(),
        size_bytes: file_size(path)?,
        sha256: sha256_file(path)?,
    })
}

pub(crate) fn scan_fatal_logs(log_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(log_path)
        .with_context(|| format!("reading Alice log {}", log_path.display()))?;
    let patterns = [
        "Unable to open DISPLAY",
        "No X11 DISPLAY",
        "SEVERE",
        "Exception in thread",
        "HeadlessException",
        "GLException",
    ];
    Ok(content
        .lines()
        .filter(|line| patterns.iter().any(|pattern| line.contains(pattern)))
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_alice_window_identity() {
        assert!(has_alice_window_evidence(
            "0x001  0 host org.alice.stageide.EntryPoint Alice 3"
        ));
    }

    #[test]
    fn rejects_unrelated_windows() {
        assert!(!has_alice_window_evidence(
            "0x001  0 host firefox.Firefox Firefox"
        ));
    }
}
