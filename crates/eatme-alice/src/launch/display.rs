use anyhow::{Context, Result, bail};
use eatme_core::{CommandRunner, CommandSpec};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const EATME_X11_SOCKET_DIR_ENV: &str = "EATME_X11_SOCKET_DIR";
const X11_UNIX_DIR_ENV: &str = "X11_UNIX_DIR";

pub(super) struct DisplayAllocation {
    name: String,
    lock_path: PathBuf,
}

impl DisplayAllocation {
    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for DisplayAllocation {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

pub(super) fn reserve_display(runs_dir: &Path) -> Result<DisplayAllocation> {
    reserve_display_with_socket_check(runs_dir, display_socket_exists)
}

fn reserve_display_with_socket_check(
    runs_dir: &Path,
    mut socket_exists: impl FnMut(u16) -> bool,
) -> Result<DisplayAllocation> {
    let lock_dir = runs_dir.join(".display-locks");
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("creating display lock directory {}", lock_dir.display()))?;

    for display in 90..130 {
        if socket_exists(display) {
            continue;
        }

        let lock_path = lock_dir.join(format!("X{display}.lock"));
        let mut lock = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(lock) => lock,
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("creating display lock {}", lock_path.display()));
            }
        };
        writeln!(lock, "pid={}", std::process::id())
            .with_context(|| format!("writing display lock {}", lock_path.display()))?;

        return Ok(DisplayAllocation {
            name: format!(":{display}"),
            lock_path,
        });
    }

    bail!(
        "no free X display found between :90 and :129 with locks in {}",
        lock_dir.display()
    )
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

fn display_socket_exists(display: u16) -> bool {
    x11_socket_dir().join(format!("X{display}")).exists()
}

fn x11_socket_dir() -> PathBuf {
    env::var_os(EATME_X11_SOCKET_DIR_ENV)
        .filter(|value| !value.is_empty())
        .or_else(|| env::var_os(X11_UNIX_DIR_ENV).filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join(".X11-unix"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reserve_display_uses_lock_file_and_releases_it_on_drop() {
        let root = unique_test_dir("display-lock");
        let allocation = reserve_display(&root).unwrap();
        let lock_path = root.join(".display-locks").join(format!(
            "X{}.lock",
            allocation.name().trim_start_matches(':')
        ));

        assert!(allocation.name().starts_with(':'));
        assert!(lock_path.is_file());
        drop(allocation);
        assert!(!lock_path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reserve_display_keeps_lock_after_socket_check_passes() {
        let root = unique_test_dir("display-lock-race");
        let mut checks = Vec::new();
        let allocation = reserve_display_with_socket_check(&root, |display| {
            checks.push(display);
            false
        })
        .unwrap();
        let lock_path = root.join(".display-locks").join("X90.lock");

        assert_eq!(allocation.name(), ":90");
        assert_eq!(checks, vec![90]);
        assert!(lock_path.is_file());
        drop(allocation);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reserve_display_skips_preexisting_lock_and_preserves_it() {
        let root = unique_test_dir("display-lock-preexisting");
        let lock_dir = root.join(".display-locks");
        fs::create_dir_all(&lock_dir).unwrap();

        let preexisting_display = 90;
        let expected_display = 91;
        let preexisting_lock = lock_dir.join(format!("X{preexisting_display}.lock"));
        fs::write(&preexisting_lock, "preexisting\n").unwrap();

        let mut checks = Vec::new();
        let allocation = reserve_display_with_socket_check(&root, |display| {
            checks.push(display);
            false
        })
        .unwrap();
        let reserved_display: u16 = allocation.name().trim_start_matches(':').parse().unwrap();
        let reserved_lock = lock_dir.join(format!("X{reserved_display}.lock"));

        assert_eq!(reserved_display, expected_display);
        assert_eq!(checks, vec![preexisting_display, expected_display]);
        assert_ne!(reserved_lock, preexisting_lock);
        assert_eq!(
            fs::read_to_string(&preexisting_lock).unwrap(),
            "preexisting\n"
        );
        assert!(reserved_lock.is_file());

        drop(allocation);

        assert!(preexisting_lock.is_file());
        assert_eq!(
            fs::read_to_string(&preexisting_lock).unwrap(),
            "preexisting\n"
        );
        assert!(!reserved_lock.exists());
        let _ = fs::remove_dir_all(root);
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("eatme-alice-tests")
            .join(format!("{prefix}-{nonce}"))
    }
}
