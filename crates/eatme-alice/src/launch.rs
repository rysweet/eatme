mod alice_cmd;
mod assertions;
mod display;
mod evidence;
mod manifest;
mod run_dir;

use self::alice_cmd::{alice_launch_args, start_alice};
use self::assertions::{bool_assert, fatal_log_detail, visual_evidence_detail};
use self::display::{reserve_display, start_xvfb, wait_for_display};
use self::evidence::{
    artifact_info, capture_screenshot, capture_window_list, has_alice_window_evidence,
    scan_fatal_logs,
};
use self::manifest::{build_manifest, write_blocked_manifest, write_manifest};
use self::run_dir::prepare_run_dir;
use crate::deps::check_dependencies;
use crate::discover::discover_alice;
use crate::launch_edit_procedure::probe_edit_procedure_hook;
use crate::launch_object_placement::{default_object_identifier, probe_object_placement_hook};
use crate::launch_run_world::probe_run_world_hook;
use crate::launch_ui_action_contract::write_ui_action_contract;
use crate::launch_ui_actions::{
    probe_alice_window_activation, probe_place_object_preconditions,
    record_alice_window_activation, record_ui_action_blockers, ui_action_failure_category,
};
use crate::package::{PackageOptions, package_alice};
use crate::scenario::LaunchSmokeScenario;
use anyhow::{Result, bail};
use eatme_core::{
    ArtifactInfo, CommandRunner, CommandSpec, LaunchSmokeManifest, RealCommandRunner,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};
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
    let runner = RealCommandRunner;
    let deps = check_dependencies(&runner)?;
    let eatme_commit = git_commit(Path::new("."), &runner).unwrap_or_else(|_| "unknown".into());
    let run_dir = options
        .runs_dir
        .join(&options.scenario.id)
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
        return write_blocked_manifest(
            options,
            &run_dir,
            deps,
            &eatme_commit,
            None,
            None,
            None,
            None,
            "missing_dependency",
            "preflight blocked: one or more required desktop dependencies are unavailable",
            assertions,
        );
    }

    let discovery = match discover_alice(&options.alice_home, &runner) {
        Ok(discovery) => discovery,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                None,
                None,
                None,
                None,
                "alice_discovery_failed",
                format!("preflight blocked: Alice discovery failed: {error:#}"),
                assertions,
            );
        }
    };
    let package = match package_alice(
        PackageOptions {
            alice_home: &options.alice_home,
            offline: options.offline_package,
        },
        &runner,
    ) {
        Ok(package) => package,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                None,
                None,
                None,
                "alice_package_failed",
                format!("preflight blocked: Alice package failed: {error:#}"),
                assertions,
            );
        }
    };

    let display = match reserve_display(&options.runs_dir) {
        Ok(display) => display,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                Some(&package),
                None,
                None,
                "display_reservation_failed",
                format!("preflight blocked: X display could not be reserved: {error:#}"),
                assertions,
            );
        }
    };
    let mut xvfb = match start_xvfb(display.name(), &run_dir) {
        Ok(xvfb) => xvfb,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                Some(&package),
                Some(display.name()),
                None,
                "xvfb_start_failed",
                format!("preflight blocked: Xvfb could not start: {error:#}"),
                assertions,
            );
        }
    };
    let display_responsive = wait_for_display(&runner, display.name(), Duration::from_secs(5));
    assertions.insert(
        "display_responsive".into(),
        bool_assert(
            display_responsive,
            format!("{} responds to xdpyinfo", display.name()),
        ),
    );
    if !display_responsive {
        shutdown(&mut xvfb);
        return write_blocked_manifest(
            options,
            &run_dir,
            deps,
            &eatme_commit,
            Some(&discovery),
            Some(&package),
            Some(display.name()),
            Some(xvfb.id()),
            "display_unresponsive",
            format!(
                "preflight blocked: {} did not respond to xdpyinfo",
                display.name()
            ),
            assertions,
        );
    }

    let log_path = run_dir.join("alice.log");
    let launch_args =
        match alice_launch_args(&options.alice_home, &options.scenario.starter_project) {
            Ok(args) => args,
            Err(error) => {
                shutdown(&mut xvfb);
                return write_blocked_manifest(
                    options,
                    &run_dir,
                    deps,
                    &eatme_commit,
                    Some(&discovery),
                    Some(&package),
                    Some(display.name()),
                    Some(xvfb.id()),
                    "alice_launch_args_failed",
                    format!("preflight blocked: Alice launch arguments failed: {error:#}"),
                    assertions,
                );
            }
        };
    let mut alice = match start_alice(
        &options.alice_home,
        display.name(),
        &run_dir,
        &log_path,
        &launch_args,
    ) {
        Ok(alice) => alice,
        Err(error) => {
            shutdown(&mut xvfb);
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                Some(&package),
                Some(display.name()),
                Some(xvfb.id()),
                "alice_start_failed",
                format!("preflight blocked: Alice process could not start: {error:#}"),
                assertions,
            );
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

    let (window_text, window_list_error) =
        capture_text_or_error(capture_window_list(&runner, display.name(), &run_dir));
    let (window_list, window_info_error) = artifact_or_error(&run_dir.join("window-list.txt"));
    let window_list_error = combine_errors([window_list_error, window_info_error]);
    let specific_alice_window_ok = has_alice_window_evidence(&window_text);
    let window_evidence_ok = options.scenario.accepts_window_evidence() && specific_alice_window_ok;
    let (screenshot, screenshot_error) =
        capture_artifact_or_error(capture_screenshot(&runner, display.name(), &run_dir));
    let screenshot_ok = screenshot
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let smoke_ready_visual_evidence = screenshot_ok || window_evidence_ok;
    let visual_evidence_detail = visual_evidence_detail(
        screenshot_ok,
        window_evidence_ok,
        screenshot_error.as_deref(),
        window_list_error.as_deref(),
    );
    assertions.insert(
        "startup_screenshot".into(),
        bool_assert(smoke_ready_visual_evidence, visual_evidence_detail.clone()),
    );
    if options.scenario.accepts_window_evidence() {
        assertions.insert(
            "startup_window_or_screenshot".into(),
            bool_assert(smoke_ready_visual_evidence, visual_evidence_detail),
        );
    }
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
    let alice_window_activation_probe = if options.scenario.requires_real_ui_actions() {
        let probe = probe_alice_window_activation(&runner, display.name(), &window_text);
        record_alice_window_activation(&mut assertions, &probe);
        if probe.status != "passed" && failure_category.is_none() {
            failure_category = Some("alice_window_activation_failed".into());
        }
        Some(probe)
    } else {
        None
    };

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
    let log_ok = log
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let real_alice_execution_evidence =
        process_started && display_responsive && smoke_ready_visual_evidence && log_ok;
    assertions.insert(
        "real_alice_execution_evidence".into(),
        bool_assert(
            real_alice_execution_evidence,
            "real Alice process, responsive virtual display, visual evidence, and launch log were captured",
        ),
    );
    if !real_alice_execution_evidence && failure_category.is_none() {
        failure_category = Some("real_alice_evidence_missing".into());
    }
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let place_object_probe = probe_place_object_preconditions(
            specific_alice_window_ok,
            smoke_ready_visual_evidence,
            log_ok,
            alice_window_activation_probe.as_ref(),
        );
        let object_placement_probe = probe_object_placement_hook(
            &runner,
            &options.alice_home,
            &run_dir,
            &options.scenario.starter_project,
            default_object_identifier(),
            display.name(),
        );
        let edit_procedure_probe = probe_edit_procedure_hook(
            &runner,
            &options.alice_home,
            &run_dir,
            &object_placement_probe,
            display.name(),
        );
        let run_world_probe = probe_run_world_hook(
            &runner,
            &options.alice_home,
            &run_dir,
            &edit_procedure_probe,
            display.name(),
        );
        let save_project_probe = crate::launch_save_project::probe_project_save_hook(
            &runner,
            &options.alice_home,
            &run_dir,
            &run_world_probe,
            display.name(),
        );
        let artifact = write_ui_action_contract(
            &run_dir,
            specific_alice_window_ok,
            smoke_ready_visual_evidence,
            log_ok,
            alice_window_activation_probe.as_ref(),
            Some(&place_object_probe),
            Some(&object_placement_probe),
            Some(&edit_procedure_probe),
            Some(&run_world_probe),
            Some(&save_project_probe),
        )?;
        record_ui_action_blockers(
            &mut assertions,
            &artifact,
            &place_object_probe,
            &object_placement_probe,
            &edit_procedure_probe,
            &run_world_probe,
            &save_project_probe,
        );
        if failure_category.is_none() {
            failure_category = Some(ui_action_failure_category(&object_placement_probe).into());
        }
        Some(artifact)
    } else {
        None
    };
    let launch_command = format!("java {}", launch_args.join(" "));
    let manifest = build_manifest(
        options,
        deps,
        &eatme_commit,
        Some(&discovery),
        Some(&package),
        launch_command,
        display.name().to_string(),
        Some(xvfb.id()),
        Some(alice.id()),
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
    );
    let manifest_write = write_manifest(&run_dir, &manifest);
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    manifest_write?;
    Ok(manifest)
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
        std::thread::sleep(Duration::from_millis(500));
    }
    true
}
fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
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
#[cfg(test)]
mod tests;
