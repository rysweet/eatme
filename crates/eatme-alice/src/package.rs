use anyhow::{Result, bail};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PackageOptions<'a> {
    pub alice_home: &'a Path,
    pub offline: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PackageResult {
    pub command: String,
    pub exit_status: Option<i32>,
}

pub fn package_alice(
    options: PackageOptions<'_>,
    runner: &impl CommandRunner,
) -> Result<PackageResult> {
    if should_reuse_existing_build(options.alice_home) {
        return Ok(PackageResult {
            command: "skip-existing-alice-build".into(),
            exit_status: Some(0),
        });
    }

    let output = run_package_command(options, runner)?;
    if output.exit_status != Some(0) {
        bail!(
            "Alice package failed with {:?}\n{}{}",
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    Ok(PackageResult {
        command: output.command,
        exit_status: output.exit_status,
    })
}

fn should_reuse_existing_build(alice_home: &Path) -> bool {
    let configured_home_matches = std::env::var("ALICE_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .is_some_and(|configured| configured == alice_home);
    std::env::var("EATME_REAL_ALICE").as_deref() == Ok("1")
        && configured_home_matches
        && build_artifacts_exist(alice_home)
}

fn build_artifacts_exist(alice_home: &Path) -> bool {
    let target_dir = alice_home.join("alice-ide/target");
    let has_jar = fs::read_dir(&target_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .any(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| {
                    name.starts_with("alice-ide-")
                        && name.ends_with(".jar")
                        && !name.contains("-sources")
                        && !name.contains("-javadoc")
                })
                .unwrap_or(false)
        });
    has_jar
        && alice_home.join("alice-ide/target/lib").is_dir()
        && alice_home
            .join("core/resources/target/distribution/application/starter-projects/africa.a3p")
            .is_file()
}

pub fn run_package_command(
    options: PackageOptions<'_>,
    runner: &impl CommandRunner,
) -> Result<CommandOutput> {
    let mut args = Vec::new();
    if options.offline {
        args.push("-o".to_string());
    }
    args.extend([
        "-DskipTests".to_string(),
        "-DincludeSims=false".to_string(),
        "-Dinstall4j.skip".to_string(),
        "-pl".to_string(),
        "alice-ide".to_string(),
        "-am".to_string(),
        "package".to_string(),
    ]);
    runner.run(
        &CommandSpec::new("mvn")
            .args(args)
            .cwd(options.alice_home)
            .timeout(Duration::from_secs(30 * 60)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct RecordingRunner {
        calls: RefCell<Vec<String>>,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            self.calls.borrow_mut().push(spec.shell_display());
            Ok(CommandOutput {
                command: spec.shell_display(),
                exit_status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, original }
        }

        fn unset(key: &'static str) -> Self {
            let original = std::env::var(key).ok();
            unsafe {
                std::env::remove_var(key);
            }
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(original) = &self.original {
                    std::env::set_var(self.key, original);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    fn test_alice_home() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/package-tests")
            .join(format!("{nonce}"));
        fs::create_dir_all(root.join("alice-ide/target/lib")).unwrap();
        fs::write(root.join("pom.xml"), "<project/>").unwrap();
        fs::write(
            root.join("alice-ide/target/alice-ide-9.1.0-SNAPSHOT.jar"),
            "jar",
        )
        .unwrap();
        fs::create_dir_all(
            root.join("core/resources/target/distribution/application/starter-projects"),
        )
        .unwrap();
        fs::write(
            root.join("core/resources/target/distribution/application/starter-projects/africa.a3p"),
            "project",
        )
        .unwrap();
        root
    }

    #[test]
    fn package_alice_reuses_existing_build_for_real_alice_runs() {
        let alice_home = test_alice_home();
        let runner = RecordingRunner::new();
        let _real_alice = EnvVarGuard::set("EATME_REAL_ALICE", "1");
        let _alice_home = EnvVarGuard::set("ALICE_HOME", &alice_home.display().to_string());

        let result = package_alice(
            PackageOptions {
                alice_home: &alice_home,
                offline: true,
            },
            &runner,
        )
        .unwrap();

        assert_eq!(result.command, "skip-existing-alice-build");
        assert!(runner.calls.borrow().is_empty());
        let _ = fs::remove_dir_all(alice_home);
    }

    #[test]
    fn package_alice_runs_maven_when_real_alice_env_is_unset() {
        let alice_home = test_alice_home();
        let runner = RecordingRunner::new();
        let _real_alice = EnvVarGuard::unset("EATME_REAL_ALICE");

        let result = package_alice(
            PackageOptions {
                alice_home: &alice_home,
                offline: true,
            },
            &runner,
        )
        .unwrap();

        assert!(result.command.starts_with("mvn "));
        assert_eq!(runner.calls.borrow().len(), 1);
        let _ = fs::remove_dir_all(alice_home);
    }
}
