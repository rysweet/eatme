use eatme_core::{CommandRunner, CommandSpec, RealCommandRunner};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn retries_return_last_failed_attempt_output() {
    let counter_path = unique_artifact_path("eatme-command-final-retry");
    fs::create_dir_all(counter_path.parent().unwrap()).unwrap();

    let output = RealCommandRunner
        .run(
            &CommandSpec::new("sh")
                .args([
                    "-c",
                    "count=$(cat \"$1\" 2>/dev/null || echo 0); count=$((count + 1)); echo \"$count\" > \"$1\"; printf 'attempt:%s' \"$count\"; printf 'warn:%s' \"$count\" >&2; exit 9",
                    "sh",
                    counter_path.to_str().unwrap(),
                ])
                .retries(3, Duration::from_millis(10)),
        )
        .unwrap();

    assert_eq!(output.exit_status, Some(9));
    assert_eq!(output.stdout, "attempt:3");
    assert_eq!(output.stderr, "warn:3");
    assert_eq!(fs::read_to_string(&counter_path).unwrap().trim(), "3");

    let _ = fs::remove_file(counter_path);
}

#[test]
fn timed_out_commands_preserve_stdout_before_timeout_suffix() {
    let output = RealCommandRunner
        .run(
            &CommandSpec::new("sh")
                .args(["-c", "printf 'hello'; sleep 2"])
                .timeout(Duration::from_millis(100)),
        )
        .unwrap();

    assert_eq!(output.exit_status, None);
    assert_eq!(output.stdout, "hello");
    assert!(output.stderr.contains("command timed out"));
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
