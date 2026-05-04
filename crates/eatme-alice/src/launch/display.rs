use anyhow::{Context, Result};
use eatme_core::{CommandRunner, CommandSpec};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const X11_SOCKET_DIR_ENV: &str = "EATME_X11_SOCKET_DIR";

pub(super) fn choose_display() -> String {
    for display in 90..130 {
        if !x11_socket_path(display).exists() {
            return format!(":{display}");
        }
    }
    ":99".into()
}

fn x11_socket_path(display: u16) -> PathBuf {
    x11_socket_dir().join(format!("X{display}"))
}

fn x11_socket_dir() -> PathBuf {
    env::var_os(X11_SOCKET_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(".X11-unix"))
}

pub(super) fn start_xvfb(display: &str, run_dir: &Path) -> Result<Child> {
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

pub(super) fn wait_for_display(
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
