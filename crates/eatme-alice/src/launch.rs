use crate::deps::check_dependencies;
use crate::discover::discover_alice;
use crate::launch_artifacts::{artifact_info, write_manifest};
use crate::launch_preflight::write_preflight_blocked_manifest;
use crate::launch_process::{alice_launch_args, start_alice, start_xvfb, wait_for_start};
use crate::launch_ui_actions::{record_ui_action_blockers, write_ui_action_contract};
use crate::launch_window::{
    capture_screenshot, capture_window_list, choose_display, specific_alice_window_detected,
    wait_for_display,
};
use crate::package::{PackageOptions, package_alice};
use anyhow::{Context, Result, bail};
use eatme_core::{
    ArtifactInfo, AssertionResult, CommandRunner, CommandSpec, LaunchSmokeManifest,
    RealCommandRunner,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct LaunchSmokeScenario {
    pub id: String,
    pub run_dir_name: String,
}

impl LaunchSmokeScenario {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            run_dir_name: id.clone(),
            id,
        }
    }

    pub fn real_alice_launch_smoke() -> Self {
        Self::new("real-alice-launch-smoke")
    }

    pub fn accepts_window_evidence(&self) -> bool {
        true
    }

    pub fn requires_real_ui_actions(&self) -> bool {
        self.id == "first-lessons-real-ui-actions"
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
    if !deps.all_required_available {
        return write_preflight_blocked_manifest(
            options,
            &run_dir,
            deps.tools,
            eatme_commit,
            "missing_dependency",
            "preflight blocked: one or more required desktop dependencies are unavailable",
            assertions,
        );
    }

    let discovery = discover_alice(&options.alice_home, &runner)?;
    let package = package_alice(
        PackageOptions {
            alice_home: &options.alice_home,
            offline: options.offline_package,
        },
        &runner,
    )?;

    let display = choose_display();
    let mut xvfb = start_xvfb(&display, &run_dir)?;
    let display_responsive = wait_for_display(&runner, &display, Duration::from_secs(5));
    assertions.insert(
        "display_responsive".into(),
        bool_assert(
            display_responsive,
            format!("{display} responds to xdpyinfo"),
        ),
    );
    if !display_responsive {
        failure_category = Some("display_unresponsive".into());
    }

    let log_path = run_dir.join("alice.log");
    let launch_args = alice_launch_args(&options.alice_home)?;
    let mut alice = start_alice(
        &options.alice_home,
        &display,
        &run_dir,
        &log_path,
        &launch_args,
    )?;
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

    let (window_text, window_list_error) =
        capture_text_or_error(capture_window_list(&runner, &display, &run_dir));
    let (window_list, window_info_error) = artifact_or_error(&run_dir.join("window-list.txt"));
    let window_list_error = combine_errors([window_list_error, window_info_error]);
    let window_evidence_ok = specific_alice_window_detected(&window_text);
    let specific_alice_window_ok = window_evidence_ok;
    let (screenshot, screenshot_error) =
        capture_artifact_or_error(capture_screenshot(&runner, &display, &run_dir));
    let screenshot_ok = screenshot
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let smoke_ready_visual_evidence =
        screenshot_ok || (options.scenario.accepts_window_evidence() && window_evidence_ok);
    let visual_evidence_detail = visual_evidence_detail(
        screenshot_ok,
        options.scenario.accepts_window_evidence() && window_evidence_ok,
        screenshot_error.as_deref(),
        window_list_error.as_deref(),
    );
    assertions.insert(
        "startup_screenshot".into(),
        bool_assert(smoke_ready_visual_evidence, visual_evidence_detail.clone()),
    );
    assertions.insert(
        "startup_window_or_screenshot".into(),
        bool_assert(smoke_ready_visual_evidence, visual_evidence_detail),
    );
    if !smoke_ready_visual_evidence && failure_category.is_none() {
        failure_category = Some("screenshot_missing".into());
    }
    if options.scenario.requires_real_ui_actions() {
        assertions.insert(
            "specific_alice_window_detected".into(),
            bool_assert(
                specific_alice_window_ok,
                "wmctrl window list contains an Alice Stage IDE window",
            ),
        );
        if !specific_alice_window_ok && failure_category.is_none() {
            failure_category = Some("alice_window_not_detected".into());
        }
    }

    let (fatal_log_scan, log_scan_error) = capture_text_or_error(scan_fatal_logs(&log_path));
    assertions.insert(
        "no_fatal_logs".into(),
        bool_assert(
            log_scan_error.is_none() && fatal_log_scan.is_empty(),
            fatal_log_detail(&fatal_log_scan, log_scan_error.as_deref()),
        ),
    );
    if log_scan_error.is_some() && failure_category.is_none() {
        failure_category = Some("log_unreadable".into());
    } else if !fatal_log_scan.is_empty() && failure_category.is_none() {
        failure_category = Some("fatal_log".into());
    }

    let (log, log_artifact_error) = artifact_or_error(&log_path);
    let log_error = combine_errors([log_scan_error, log_artifact_error]);
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let artifact = write_ui_action_contract(
            &run_dir,
            specific_alice_window_ok,
            smoke_ready_visual_evidence,
            log.as_ref()
                .map(|artifact| artifact.size_bytes > 0)
                .unwrap_or(false),
        )?;
        record_ui_action_blockers(&mut assertions, &artifact);
        if failure_category.is_none() {
            failure_category = Some("ui_action_automation_unimplemented".into());
        }
        Some(artifact)
    } else {
        None
    };
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
        display: display.clone(),
        xvfb_pid: Some(xvfb.id()),
        alice_pid: Some(alice.id()),
        timeout_seconds: options.timeout_seconds,
        window_list,
        window_list_error,
        screenshot,
        screenshot_error,
        ui_action_contract,
        log,
        log_error,
        fatal_log_scan,
        assertions,
        failure_category,
    };

    write_manifest(&run_dir, &manifest)?;
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    Ok(manifest)
}

fn prepare_run_dir(run_dir: &Path) -> Result<()> {
    if run_dir.exists() {
        fs::remove_dir_all(run_dir).with_context(|| format!("removing {}", run_dir.display()))?;
    }
    fs::create_dir_all(run_dir.join("screenshots"))
        .with_context(|| format!("creating {}", run_dir.display()))?;
    fs::create_dir_all(run_dir.join("home"))?;
    fs::create_dir_all(run_dir.join("prefs"))?;
    fs::create_dir_all(run_dir.join("tmp"))?;
    Ok(())
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

fn scan_fatal_logs(log_path: &Path) -> Result<Vec<String>> {
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

fn capture_text_or_error<T: Default>(result: Result<T>) -> (T, Option<String>) {
    match result {
        Ok(value) => (value, None),
        Err(error) => (T::default(), Some(format!("{error:#}"))),
    }
}

fn capture_artifact_or_error<T>(result: Result<T>) -> (Option<T>, Option<String>) {
    match result {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(format!("{error:#}"))),
    }
}

fn artifact_or_error(path: &Path) -> (Option<ArtifactInfo>, Option<String>) {
    capture_artifact_or_error(artifact_info(path))
}

fn combine_errors(errors: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    let errors = errors.into_iter().flatten().collect::<Vec<_>>();
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("\n"))
    }
}

fn visual_evidence_detail(
    screenshot_ok: bool,
    window_evidence_ok: bool,
    screenshot_error: Option<&str>,
    window_list_error: Option<&str>,
) -> String {
    if screenshot_ok {
        return "startup screenshot exists and is non-empty".into();
    }
    if window_evidence_ok {
        return "Alice-specific window identity was captured".into();
    }

    let mut details = vec![
        "startup requires a non-empty screenshot or Alice-specific window identity".to_string(),
    ];
    match screenshot_error {
        Some(error) => details.push(format!("screenshot error: {error}")),
        None => details.push("startup screenshot is missing or empty".into()),
    }
    match window_list_error {
        Some(error) => details.push(format!("window list error: {error}")),
        None => details.push("no Alice-specific window identity found".into()),
    }
    details.join("; ")
}

fn fatal_log_detail(fatal_lines: &[String], log_error: Option<&str>) -> String {
    if let Some(error) = log_error {
        return format!("Alice log could not be read: {error}");
    }
    format!("{} fatal log lines found", fatal_lines.len())
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

fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests;
