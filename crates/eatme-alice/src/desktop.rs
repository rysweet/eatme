use anyhow::{Context, Result};
use eatme_core::{CommandRunner, CommandSpec};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) fn choose_display() -> String {
    let x11_unix_dir = env::var_os("X11_UNIX_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(".X11-unix"));
    for display in 90..130 {
        let socket = x11_unix_dir.join(format!("X{display}"));
        if !socket.exists() {
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

fn command_ok(runner: &impl CommandRunner, spec: CommandSpec) -> bool {
    runner
        .run(&spec)
        .map(|output| output.exit_status == Some(0))
        .unwrap_or(false)
}

pub(crate) fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
