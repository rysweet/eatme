mod alice_cmd;
mod display;

use self::alice_cmd::{DEFAULT_STARTER_PROJECT, alice_launch_args, start_alice};
use self::display::{
    artifact_info, capture_screenshot, capture_window_list, reserve_display, start_xvfb,
    wait_for_display,
};
use crate::deps::check_dependencies;
use crate::discover::discover_alice;
use crate::package::{PackageOptions, package_alice};
use anyhow::{Context, Result, bail};
use eatme_core::{
    AssertionResult, CommandRunner, CommandSpec, LaunchSmokeManifest, RealCommandRunner,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct LaunchSmokeScenario {
    pub id: String,
    pub run_dir_name: String,
    pub starter_project: PathBuf,
}

impl LaunchSmokeScenario {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            run_dir_name: id.clone(),
            starter_project: PathBuf::from(DEFAULT_STARTER_PROJECT),
            id,
        }
    }

    pub fn real_alice_launch_smoke() -> Self {
        Self::new("real-alice-launch-smoke")
    }

    pub fn accepts_window_evidence(&self) -> bool {
        self.id != "real-alice-launch-smoke"
    }

    pub fn with_starter_project(mut self, starter_project: impl Into<PathBuf>) -> Self {
        self.starter_project = starter_project.into();
        self
    }
}

impl Default for LaunchSmokeScenario {
    fn default() -> Self {
        Self::real_alice_launch_smoke()
    }
}

#[derive(Clone, Debug)]
pub struct LaunchSmokeOptions {
    pub alice_home: PathBuf,
    pub run_id: String,
    pub runs_dir: PathBuf,
    pub timeout_seconds: u64,
    pub json: bool,
    pub no_memory: bool,
    pub offline_package: bool,
    pub scenario: LaunchSmokeScenario,
}

pub fn run_launch_smoke(options: &LaunchSmokeOptions) -> Result<LaunchSmokeManifest> {
    validate_scenario_name(&options.scenario.id)?;
    validate_scenario_name(&options.scenario.run_dir_name)?;

    let runner = RealCommandRunner;
    let deps = check_dependencies(&runner)?;
    let discovery = discover_alice(&options.alice_home, &runner)?;
    let package = package_alice(
        PackageOptions {
            alice_home: &options.alice_home,
            offline: options.offline_package,
        },
        &runner,
    )?;
    let eatme_commit = git_commit(Path::new("."), &runner).unwrap_or_else(|_| "unknown".into());
    let run_dir = options
        .runs_dir
        .join(&options.scenario.run_dir_name)
        .join(&options.run_id);
    prepare_run_dir(&run_dir)?;

    let mut assertions = BTreeMap::new();
    assertions.insert(
        "dependencies_available".into(),
        bool_assert(
            deps.all_required_available,
            "required desktop tools detected",
        ),
    );
    let mut failure_category = if deps.all_required_available {
        None
    } else {
        Some("missing_dependency".to_string())
    };

    let display = reserve_display(&options.runs_dir)?;
    let mut xvfb = start_xvfb(display.name(), &run_dir)?;
    let display_responsive = wait_for_display(&runner, display.name(), Duration::from_secs(5));
    assertions.insert(
        "display_responsive".into(),
        bool_assert(
            display_responsive,
            format!("{} responds to xdpyinfo", display.name()),
        ),
    );
    if !display_responsive {
        failure_category = Some("display_unresponsive".into());
    }

    let log_path = run_dir.join("alice.log");
    let launch_args = alice_launch_args(&options.alice_home, &options.scenario.starter_project)?;
    let mut alice = match start_alice(
        &options.alice_home,
        display.name(),
        &run_dir,
        &log_path,
        &launch_args,
    ) {
        Ok(child) => child,
        Err(error) => {
            shutdown(&mut xvfb);
            return Err(error);
        }
    };
    let process_started = wait_for_start(&mut alice, options.timeout_seconds.min(60));
    assertions.insert(
        "process_started".into(),
        bool_assert(
            process_started,
            "Alice process stayed alive through startup wait",
        ),
    );
    if !process_started {
        failure_category = Some("alice_process_exited".into());
    }

    let window_list = capture_window_list(&runner, display.name(), &run_dir).unwrap_or_default();
    let window_evidence_ok = !window_list.trim().is_empty();
    let screenshot = capture_screenshot(&runner, display.name(), &run_dir).ok();
    let screenshot_ok = screenshot
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let smoke_ready_visual_evidence =
        screenshot_ok || (options.scenario.accepts_window_evidence() && window_evidence_ok);
    assertions.insert(
        "startup_screenshot".into(),
        bool_assert(
            smoke_ready_visual_evidence,
            if options.scenario.accepts_window_evidence() {
                "startup screenshot exists or window evidence was captured"
            } else {
                "startup screenshot exists and is non-empty"
            },
        ),
    );
    if options.scenario.accepts_window_evidence() {
        assertions.insert(
            "startup_window_or_screenshot".into(),
            bool_assert(
                smoke_ready_visual_evidence,
                "startup screenshot or window evidence exists",
            ),
        );
    }
    if !smoke_ready_visual_evidence && failure_category.is_none() {
        failure_category = Some("screenshot_missing".into());
    }

    let fatal_log_scan = scan_fatal_logs(&log_path);
    assertions.insert(
        "no_fatal_logs".into(),
        bool_assert(
            fatal_log_scan.is_empty(),
            format!("{} fatal log lines found", fatal_log_scan.len()),
        ),
    );
    if !fatal_log_scan.is_empty() && failure_category.is_none() {
        failure_category = Some("fatal_log".into());
    }

    let log = artifact_info(&log_path).ok();
    let launch_command = format!("java {}", launch_args.join(" "));
    let manifest = LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: discovery.alice_home,
        alice_git_commit: discovery.git_commit,
        eatme_git_commit: eatme_commit,
        java_version: discovery.java_version,
        maven_version: discovery.maven_version,
        dependency_checks: deps.tools,
        build_command: package.command,
        build_exit_status: package.exit_status,
        launch_command,
        display: display.name().to_string(),
        xvfb_pid: Some(xvfb.id()),
        alice_pid: Some(alice.id()),
        timeout_seconds: options.timeout_seconds,
        screenshot,
        log,
        fatal_log_scan,
        assertions,
        failure_category,
    };

    let manifest_write = write_manifest(&run_dir, &manifest);
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    manifest_write?;
    Ok(manifest)
}

fn prepare_run_dir(run_dir: &Path) -> Result<()> {
    if run_dir.exists() {
        archive_existing_run_dir(run_dir)?;
    }
    fs::create_dir_all(run_dir.join("screenshots"))
        .with_context(|| format!("creating {}", run_dir.display()))?;
    fs::create_dir_all(run_dir.join("home"))?;
    fs::create_dir_all(run_dir.join("prefs"))?;
    fs::create_dir_all(run_dir.join("tmp"))?;
    Ok(())
}

fn archive_existing_run_dir(run_dir: &Path) -> Result<()> {
    let parent = run_dir
        .parent()
        .with_context(|| format!("{} has no parent directory", run_dir.display()))?;
    let name = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("{} has no valid directory name", run_dir.display()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_nanos();

    for attempt in 0..1000 {
        let archive_name = format!("{name}.previous-{stamp}-{attempt}");
        let archive_path = parent.join(archive_name);
        if archive_path.exists() {
            continue;
        }
        fs::rename(run_dir, &archive_path).with_context(|| {
            format!(
                "archiving existing launch evidence {} to {}",
                run_dir.display(),
                archive_path.display()
            )
        })?;
        return Ok(());
    }

    bail!(
        "could not find a unique archive path for existing launch evidence {}",
        run_dir.display()
    );
}

fn validate_scenario_name(name: &str) -> Result<()> {
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

fn wait_for_start(child: &mut Child, seconds: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds.clamp(5, 60));
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return false;
        }
        thread::sleep(Duration::from_millis(500));
    }
    true
}

fn scan_fatal_logs(log_path: &Path) -> Vec<String> {
    let content = fs::read_to_string(log_path).unwrap_or_default();
    let patterns = [
        "Unable to open DISPLAY",
        "No X11 DISPLAY",
        "SEVERE",
        "Exception in thread",
        "HeadlessException",
        "GLException",
    ];
    content
        .lines()
        .filter(|line| patterns.iter().any(|pattern| line.contains(pattern)))
        .map(str::to_string)
        .collect()
}

fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}

fn git_commit(path: &Path, runner: &impl CommandRunner) -> Result<String> {
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

fn write_manifest(run_dir: &Path, manifest: &LaunchSmokeManifest) -> Result<()> {
    let path = run_dir.join("manifest.json");
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json)?;
    Ok(())
}

fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archives_existing_run_dir_instead_of_deleting_it() {
        let root = unique_test_dir("archive-existing-run");
        let run_dir = root.join("runs/real-alice-launch-smoke/reused-run");
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(run_dir.join("manifest.json"), "old evidence").unwrap();

        prepare_run_dir(&run_dir).unwrap();

        assert!(run_dir.join("screenshots").is_dir());
        let archived_manifest = fs::read_dir(run_dir.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("reused-run.previous-"))
                    .unwrap_or(false)
                    && path.join("manifest.json").is_file()
            });
        assert!(
            archived_manifest.is_some(),
            "existing evidence should be archived next to the new run"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_non_kebab_case_scenario_names() {
        assert!(validate_scenario_name("../bad").is_err());
        assert!(validate_scenario_name("building-a-scene-first-world").is_ok());
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
