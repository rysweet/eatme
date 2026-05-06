use anyhow::{Context, Result, bail};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec, file_size, sha256_file};
use std::fs;
use std::path::Path;
use std::time::Duration;

const ALICE_WINDOW_MARKERS: [&str; 4] = [
    "org.alice.stageide.entrypoint",
    "org.alice.stageide",
    "org.alice.ide",
    "\"alice 3",
];

pub(super) fn capture_window_list(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<String> {
    let wmctrl = runner.run(
        &CommandSpec::new("wmctrl")
            .args(["-lx"])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if wmctrl.exit_status == Some(0) {
        let combined = command_text(&wmctrl.stdout, &wmctrl.stderr);
        if !combined.trim().is_empty() {
            fs::write(run_dir.join("window-list.txt"), &combined)?;
            return Ok(combined);
        }
    }

    let xwininfo = runner.run(
        &CommandSpec::new("xwininfo")
            .args(["-root", "-tree"])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if xwininfo.exit_status != Some(0) {
        bail!(
            "capturing window list failed: wmctrl={:?}, xwininfo={:?}\nwmctrl stdout:\n{}wmctrl stderr:\n{}xwininfo stdout:\n{}xwininfo stderr:\n{}",
            wmctrl.exit_status,
            xwininfo.exit_status,
            wmctrl.stdout,
            wmctrl.stderr,
            xwininfo.stdout,
            xwininfo.stderr
        );
    }
    let combined = command_text(&xwininfo.stdout, &xwininfo.stderr);
    fs::write(run_dir.join("window-list.txt"), &combined)?;
    Ok(combined)
}

fn command_text(stdout: &str, stderr: &str) -> String {
    if stdout.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    }
}

pub(super) fn has_alice_window_evidence(window_list: &str) -> bool {
    window_list.lines().any(|line| {
        let normalized = line.to_ascii_lowercase();
        ALICE_WINDOW_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
    })
}

pub(super) fn capture_screenshot(
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

pub(super) fn artifact_info(path: &Path) -> Result<ArtifactInfo> {
    Ok(ArtifactInfo {
        path: path.display().to_string(),
        size_bytes: file_size(path)?,
        sha256: sha256_file(path)?,
    })
}

pub(super) fn scan_fatal_logs(log_path: &Path) -> Result<Vec<String>> {
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
    fn recognizes_main_alice_window_from_xwininfo_tree() {
        assert!(has_alice_window_evidence(
            r#"0x600007 "Alice 3 ": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")"#
        ));
    }

    #[test]
    fn rejects_alice_license_dialog_as_main_window() {
        assert!(!has_alice_window_evidence(
            r#"0x60002a "License Agreement (Part 1 of 2): Alice 3": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")"#
        ));
    }

    #[test]
    fn rejects_unrelated_windows() {
        assert!(!has_alice_window_evidence(
            "0x001  0 host firefox.Firefox Firefox"
        ));
    }
}
