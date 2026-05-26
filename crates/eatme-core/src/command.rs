use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub timeout: Option<Duration>,
    pub attempts: usize,
    pub retry_delay: Duration,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            timeout: None,
            attempts: 1,
            retry_delay: Duration::from_millis(250),
        }
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn retries(mut self, attempts: usize, retry_delay: Duration) -> Self {
        self.attempts = attempts.max(1);
        self.retry_delay = retry_delay;
        self
    }

    pub fn shell_display(&self) -> String {
        let args = self.args.join(" ");
        if args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, args)
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandOutput {
    pub command: String,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput>;
}

#[derive(Clone, Debug, Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        let attempts = spec.attempts.max(1);
        let mut output = self.run_once(spec)?;
        for _ in 1..attempts {
            if output.exit_status == Some(0) {
                break;
            }
            thread::sleep(spec.retry_delay);
            output = self.run_once(spec)?;
        }
        Ok(output)
    }
}

impl RealCommandRunner {
    fn run_once(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        let mut command = Command::new(&spec.program);
        command.args(&spec.args);
        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }
        for (key, value) in &spec.env {
            command.env(key, value);
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = run_with_timeout(command, spec.timeout)
            .with_context(|| format!("running {}", spec.shell_display()))?;

        Ok(CommandOutput {
            command: spec.shell_display(),
            exit_status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

fn run_with_timeout(mut command: Command, timeout: Option<Duration>) -> Result<Output> {
    let Some(timeout) = timeout else {
        return Ok(command.output()?);
    };
    let mut child = command.spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            return Ok(child.wait_with_output()?);
        }
        if Instant::now() >= deadline {
            return finish_timed_out_child(child, timeout);
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn finish_timed_out_child(mut child: Child, timeout: Duration) -> Result<Output> {
    if child.try_wait()?.is_none() {
        match child.kill() {
            Ok(()) => {}
            Err(kill_error) => match child.try_wait()? {
                Some(_) => {}
                None => return Err(kill_error).context("killing timed out command"),
            },
        }
    }

    let mut output = child.wait_with_output()?;
    let timeout_message = format!("\ncommand timed out after {}s", timeout.as_secs().max(1));
    output.stderr.extend_from_slice(timeout_message.as_bytes());
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn real_runner_times_out_long_running_command() {
        let output = RealCommandRunner
            .run(
                &CommandSpec::new("sh")
                    .args(["-c", "sleep 2"])
                    .timeout(Duration::from_millis(100)),
            )
            .unwrap();

        assert_eq!(output.exit_status, None);
        assert!(output.stderr.contains("command timed out"));
    }

    #[test]
    fn real_runner_retries_until_success() {
        let counter_path = unique_temp_path("eatme-command-retry");
        let output = RealCommandRunner
            .run(
                &CommandSpec::new("sh")
                    .args([
                        "-c",
                        "count=$(cat \"$1\" 2>/dev/null || echo 0); count=$((count + 1)); echo \"$count\" > \"$1\"; [ \"$count\" -ge 2 ]",
                        "sh",
                        counter_path.to_str().unwrap(),
                    ])
                    .retries(2, Duration::from_millis(10)),
            )
            .unwrap();

        assert_eq!(output.exit_status, Some(0));
        assert_eq!(fs::read_to_string(&counter_path).unwrap().trim(), "2");
        let _ = fs::remove_file(counter_path);
    }

    #[test]
    fn real_runner_stops_retrying_after_first_success() {
        let counter_path = unique_artifact_path("eatme-command-single-success");
        fs::create_dir_all(counter_path.parent().unwrap()).unwrap();
        let output = RealCommandRunner
            .run(
                &CommandSpec::new("sh")
                    .args([
                        "-c",
                        "count=$(cat \"$1\" 2>/dev/null || echo 0); count=$((count + 1)); echo \"$count\" > \"$1\"; exit 0",
                        "sh",
                        counter_path.to_str().unwrap(),
                    ])
                    .retries(3, Duration::from_millis(10)),
            )
            .unwrap();

        assert_eq!(output.exit_status, Some(0));
        assert_eq!(fs::read_to_string(&counter_path).unwrap().trim(), "1");
        let _ = fs::remove_file(counter_path);
    }

    #[test]
    fn retries_clamps_zero_attempts_to_one() {
        let spec = CommandSpec::new("sh").retries(0, Duration::from_millis(10));

        assert_eq!(spec.attempts, 1);
        assert_eq!(spec.retry_delay, Duration::from_millis(10));
    }

    #[test]
    fn shell_display_joins_program_and_args_without_trailing_spaces() {
        assert_eq!(CommandSpec::new("cargo").shell_display(), "cargo");
        assert_eq!(
            CommandSpec::new("cargo")
                .args(["test", "--workspace"])
                .shell_display(),
            "cargo test --workspace"
        );
    }

    #[test]
    fn real_runner_honors_working_directory_and_environment() {
        let work_dir = unique_artifact_path("eatme-command-cwd");
        fs::create_dir_all(&work_dir).unwrap();
        let output = RealCommandRunner
            .run(
                &CommandSpec::new("sh")
                    .args(["-c", "printf '%s|%s' \"$PWD\" \"$ALICE_TEST_FLAG\""])
                    .cwd(&work_dir)
                    .env("ALICE_TEST_FLAG", "enabled"),
            )
            .unwrap();

        assert_eq!(output.exit_status, Some(0));
        assert_eq!(output.stdout, format!("{}|enabled", work_dir.display()));

        let _ = fs::remove_dir_all(work_dir);
    }

    #[test]
    fn real_runner_preserves_failed_exit_status_stdout_and_stderr() {
        let output = RealCommandRunner
            .run(&CommandSpec::new("sh").args(["-c", "printf 'hello'; printf 'warn' >&2; exit 7"]))
            .unwrap();

        assert_eq!(output.exit_status, Some(7));
        assert_eq!(output.stdout, "hello");
        assert_eq!(output.stderr, "warn");
    }

    #[test]
    fn timed_out_commands_keep_existing_stderr_before_timeout_suffix() {
        let output = RealCommandRunner
            .run(
                &CommandSpec::new("sh")
                    .args(["-c", "printf 'warn' >&2; sleep 2"])
                    .timeout(Duration::from_millis(100)),
            )
            .unwrap();

        assert_eq!(output.exit_status, None);
        assert!(output.stderr.contains("warn"));
        assert!(output.stderr.contains("command timed out"));
    }

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{prefix}-{nonce}"))
    }

    fn unique_artifact_path(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::current_dir()
            .unwrap()
            .join("target/test-artifacts")
            .join(format!("{prefix}-{nonce}"))
    }
}
