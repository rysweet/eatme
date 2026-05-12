mod alice_cmd;
pub(crate) mod assertions;
mod display;
mod evidence;
mod manifest;
mod run_dir;
mod util;

use self::alice_cmd::{alice_launch_args, start_alice};
use self::assertions::{bool_assert, fatal_log_detail, visual_evidence_detail};
use self::display::{reserve_display, start_xvfb, wait_for_display};
use self::evidence::{capture_screenshot, capture_window_list, record_post_focus, scan_fatal_logs};
use self::manifest::{build_manifest, write_blocked_manifest, write_manifest};
use self::run_dir::{launch_run_dir, prepare_run_dir};
use self::util::{
    artifact_or_error, capture_artifact_or_error, capture_text_or_error, combine_errors,
    git_commit, shutdown, validate_scenario_name, wait_for_start,
};
use crate::deps::check_dependencies;
use crate::discover::discover_alice;
use crate::launch_desktop_controls::{probe_desktop_run_shortcut, probe_desktop_save_shortcut};
use crate::launch_desktop_execution::probe_toolbar_run_and_execution;
use crate::launch_edit_procedure::probe_edit_procedure_hook;
use crate::launch_license::seed_license_preferences_if_requested;
use crate::launch_object_placement::{default_object_identifier, probe_object_placement_hook};
use crate::launch_options::LaunchSmokeOptions;
use crate::launch_run_window::probe_run_window_after_shortcut;
use crate::launch_run_world::probe_run_world_hook;
use crate::launch_ui_action_contract::write_ui_action_contract;
use crate::launch_ui_actions::{
    probe_alice_window_activation, probe_place_object_preconditions, probe_specific_alice_window,
    record_alice_window_activation, record_ui_action_blockers, ui_action_failure_category,
};
use crate::launch_window_activation::ui_action_activation_failure_category;
use crate::launch_window_targeting::alice_window_search;
use crate::package::{PackageOptions, package_alice};
use anyhow::Result;
use eatme_core::{LaunchSmokeManifest, RealCommandRunner};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;
pub fn run_launch_smoke(options: &LaunchSmokeOptions) -> Result<LaunchSmokeManifest> {
    validate_scenario_name(&options.scenario.id)?;
    let runner = RealCommandRunner;
    let deps = check_dependencies(&runner)?;
    let eatme_commit = git_commit(Path::new("."), &runner).unwrap_or_else(|_| "unknown".into());
    let run_dir = launch_run_dir(&options.runs_dir, &options.scenario.id, &options.run_id)?;
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
    if let Some(detail) = seed_license_preferences_if_requested(&run_dir)? {
        assertions.insert(
            "alice_license_preferences_seeded".into(),
            bool_assert(true, detail),
        );
    }
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
    let window_search = alice_window_search(&window_text);
    let specific_alice_window_ok = window_search.detected();
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
            bool_assert(specific_alice_window_ok, window_search.detail().to_string()),
        );
        if !specific_alice_window_ok && failure_category.is_none() {
            failure_category = window_search.failure_category().map(str::to_string);
        }
    }
    let alice_window_verification_probe = options
        .scenario
        .requires_real_ui_actions()
        .then(|| probe_specific_alice_window(&window_search));
    let (alice_window_activation_probe, post_focus_screenshot, post_focus_screenshot_error) =
        if options.scenario.requires_real_ui_actions() {
            let probe = probe_alice_window_activation(&runner, display.name(), &window_text);
            record_alice_window_activation(&mut assertions, &probe);
            if probe.status != "passed" && failure_category.is_none() {
                failure_category = Some(ui_action_activation_failure_category(&probe).into());
            }
            let (pfs, err) = record_post_focus(
                &runner,
                display.name(),
                &run_dir,
                probe.status == "passed",
                &mut assertions,
            );
            (Some(probe), pfs, err)
        } else {
            (None, None, None)
        };
    let desktop_save_shortcut_probe = options.scenario.requires_real_ui_actions().then(|| {
        probe_desktop_save_shortcut(
            &runner,
            display.name(),
            alice_window_activation_probe.as_ref(),
        )
    });
    if let Some(probe) = &desktop_save_shortcut_probe {
        assertions.insert(
            "save_project_desktop_shortcut_dispatch".into(),
            bool_assert(probe.status == "passed", probe.detail.clone()),
        );
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
        )
        .with_proof_artifact_check(&run_dir);
        let desktop_run_shortcut_probe = probe_desktop_run_shortcut(
            &runner,
            display.name(),
            alice_window_activation_probe.as_ref(),
            edit_procedure_probe.proves_edit(),
        );
        if desktop_run_shortcut_probe.status == "passed" {
            assertions.insert(
                "run_world_desktop_shortcut_dispatch".into(),
                bool_assert(true, desktop_run_shortcut_probe.detail.clone()),
            );
        }
        let run_window_probe = probe_run_window_after_shortcut(
            &runner,
            display.name(),
            &run_dir,
            &desktop_run_shortcut_probe,
        );
        if desktop_run_shortcut_probe.status == "passed" {
            assertions.insert(
                "run_world_desktop_window_observed".into(),
                bool_assert(
                    run_window_probe.status == "passed",
                    run_window_probe.detail.clone(),
                ),
            );
        }
        let (
            desktop_run_toolbar_probe,
            run_window_after_toolbar_probe,
            desktop_run_execution_probe,
        ) = probe_toolbar_run_and_execution(
            &runner,
            display.name(),
            &run_dir,
            alice_window_activation_probe.as_ref(),
            &run_window_probe,
            &mut assertions,
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
            alice_window_verification_probe.as_ref(),
            alice_window_activation_probe.as_ref(),
            desktop_save_shortcut_probe.as_ref(),
            Some(&desktop_run_shortcut_probe),
            Some(&run_window_probe),
            Some(&desktop_run_toolbar_probe),
            Some(&run_window_after_toolbar_probe),
            Some(&desktop_run_execution_probe),
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
        post_focus_screenshot,
        post_focus_screenshot_error,
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
#[cfg(test)]
mod tests;
