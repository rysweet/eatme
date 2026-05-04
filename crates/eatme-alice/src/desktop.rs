use crate::discover::first_non_empty;
use anyhow::{Context, Result, bail};
use eatme_core::{ArtifactInfo, CommandRunner, CommandSpec, file_size, sha256_file};
use std::fs::{self, File};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) fn choose_display() -> String {
    for display in 90..130 {
        let socket = format!("/tmp/.X11-unix/X{display}");
        if !Path::new(&socket).exists() {
            return format!(":{display}");
        }
    }
    ":99".into()
}

pub(crate) fn start_xvfb(display: &str, run_dir: &Path) -> Result<Child> {
    let log = File::create(run_dir.join("xvfb.log"))?;
    Command::new("Xvfb")
        .args([
            display,
            "-screen",
            "0",
            "1280x900x24",
            "+extension",
            "GLX",
            "+render",
            "-noreset",
        ])
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
        .with_context(|| format!("starting Xvfb {display}"))
}

pub(crate) fn start_alice(
    alice_home: &Path,
    display: &str,
    run_dir: &Path,
    log_path: &Path,
    args: &[String],
) -> Result<Child> {
    let log = File::create(log_path)?;
    let mut command = Command::new("java");
    command
        .current_dir(alice_home)
        .env("DISPLAY", display)
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("HOME", run_dir.join("home"))
        .env("TMPDIR", run_dir.join("tmp"))
        .arg(format!("-Duser.home={}", run_dir.join("home").display()))
        .arg(format!(
            "-Djava.util.prefs.userRoot={}",
            run_dir.join("prefs").display()
        ))
        .arg(format!(
            "-Djava.io.tmpdir={}",
            run_dir.join("tmp").display()
        ))
        .args(args)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log));
    command.spawn().context("starting Alice")
}

pub(crate) fn wait_for_start(child: &mut Child, seconds: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds.clamp(5, 60));
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return false;
        }
        thread::sleep(Duration::from_millis(500));
    }
    true
}

pub(crate) fn wait_for_display(
    runner: &impl CommandRunner,
    display: &str,
    timeout: Duration,
) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if command_ok(
            runner,
            CommandSpec::new("xdpyinfo")
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(2))
                .retries(2, Duration::from_millis(100)),
        ) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

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

fn command_ok(runner: &impl CommandRunner, spec: CommandSpec) -> bool {
    runner
        .run(&spec)
        .map(|output| output.exit_status == Some(0))
        .unwrap_or(false)
}

pub(crate) fn artifact_info(path: &Path) -> Result<ArtifactInfo> {
    Ok(ArtifactInfo {
        path: path.display().to_string(),
        size_bytes: file_size(path)?,
        sha256: sha256_file(path)?,
    })
}

pub(crate) fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
