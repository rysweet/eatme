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
use crate::launch_class_portability::{
    desktop_class_portability_assertion, desktop_class_portability_failure_category,
    probe_desktop_class_portability_hook, write_desktop_class_portability_contract,
};
use crate::launch_desktop_controls::{probe_desktop_run_shortcut, probe_desktop_save_shortcut};
use crate::launch_desktop_execution::probe_toolbar_run_and_execution;
use crate::launch_edit_procedure::{
    DEFAULT_PROCEDURE_EDIT_HOOK, OBJECTS_FIRST_PROCEDURE_SELECTOR, probe_edit_procedure_hook,
    probe_movement_procedure_hook,
};
use crate::launch_license::seed_license_preferences_if_requested;
use crate::launch_object_placement::{
    DEFAULT_OBJECT_PLACEMENT_HOOK, default_object_identifier, probe_object_placement_hook,
};
use crate::launch_object_transform::{DEFAULT_OBJECT_TRANSFORM_HOOK, probe_object_transform_hook};
use crate::launch_objects_first_full_path::{
    FullPathPhaseProbes, FullPathVisualEvidence, write_objects_first_full_path_contract,
};
use crate::launch_options::LaunchSmokeOptions;
use crate::launch_reopen_project::{
    DEFAULT_PROJECT_REOPEN_HOOK, probe_project_reopen_hook_with_selector,
};
use crate::launch_run_window::probe_run_window_after_shortcut;
use crate::launch_run_world::{
    DEFAULT_WORLD_RUN_HOOK, probe_run_world_hook, probe_run_world_hook_with_selector,
};
use crate::launch_save_project::{
    DEFAULT_PROJECT_SAVE_HOOK, OBJECTS_FIRST_SAVE_SELECTOR, probe_project_save_hook_with_selector,
};
use crate::launch_ui_action_contract::write_ui_action_contract;
use crate::launch_ui_actions::{
    probe_alice_window_activation, probe_place_object_preconditions, probe_specific_alice_window,
    record_alice_window_activation, record_ui_action_blockers, ui_action_failure_category,
};
use crate::launch_window_activation::ui_action_activation_failure_category;
use crate::launch_window_targeting::alice_window_search;
use crate::objects_first_workflow::{
    create_or_open_project_assertion, is_objects_first_scenario, persisted_state_assertion,
    record_evidence_summary,
};
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
    if options.scenario.is_objects_first_full_path() {
        let missing_hooks = missing_objects_first_full_path_hooks(&options.alice_home);
        let hooks_available = missing_hooks.is_empty();
        let detail = if hooks_available {
            "all Alice-side objects-first full-path hooks are present".to_string()
        } else {
            format!(
                "preflight blocked: Alice checkout is missing required objects-first full-path hooks: {}",
                missing_hooks.join(", ")
            )
        };
        assertions.insert(
            "alice_objects_first_required_hooks_available".into(),
            bool_assert(hooks_available, detail.clone()),
        );
        if !hooks_available {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                None,
                None,
                None,
                "alice_required_hook_missing",
                detail,
                assertions,
            );
        }
    }
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
    if is_objects_first_scenario(&options.scenario.id) {
        assertions.insert(
            "create_or_open_project_ui_action".into(),
            create_or_open_project_assertion(
                process_started,
                &options.alice_home.join(&options.scenario.starter_project),
            ),
        );
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
    let (ui_action_contract, objects_first_full_path_evidence) = if options
        .scenario
        .requires_real_ui_actions()
    {
        if options.scenario.is_modified_class_portability() {
            let probe = probe_desktop_class_portability_hook(
                &runner,
                &options.alice_home,
                &run_dir,
                &options.scenario.starter_project,
                display.name(),
                specific_alice_window_ok && smoke_ready_visual_evidence && log_ok,
            );
            assertions.insert(
                "desktop_class_portability_evidence".into(),
                desktop_class_portability_assertion(&probe),
            );
            if !probe.proves_portability() && failure_category.is_none() {
                failure_category = Some(desktop_class_portability_failure_category(&probe).into());
            }
            let artifact = write_desktop_class_portability_contract(
                &run_dir,
                specific_alice_window_ok,
                smoke_ready_visual_evidence,
                log_ok,
                &probe,
            )?;
            (Some(artifact), None)
        } else {
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
            if options.scenario.is_objects_first_full_path() {
                let object_transform_probe = probe_object_transform_hook(
                    &runner,
                    &options.alice_home,
                    &run_dir,
                    &object_placement_probe,
                    &options.scenario.starter_project,
                    display.name(),
                );
                let edit_procedure_probe = probe_movement_procedure_hook(
                    &runner,
                    &options.alice_home,
                    &run_dir,
                    object_transform_probe.proves_transform(),
                    &object_transform_probe.object_id,
                    display.name(),
                );
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
                let run_world_probe = probe_run_world_hook_with_selector(
                    &runner,
                    &options.alice_home,
                    &run_dir,
                    &edit_procedure_probe,
                    OBJECTS_FIRST_PROCEDURE_SELECTOR,
                    display.name(),
                );
                let save_project_probe = probe_project_save_hook_with_selector(
                    &runner,
                    &options.alice_home,
                    &run_dir,
                    &run_world_probe,
                    OBJECTS_FIRST_SAVE_SELECTOR,
                    display.name(),
                );
                let reopen_project_probe = probe_project_reopen_hook_with_selector(
                    &runner,
                    &options.alice_home,
                    &run_dir,
                    &save_project_probe,
                    OBJECTS_FIRST_SAVE_SELECTOR,
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
                    Some(&object_transform_probe),
                    Some(&edit_procedure_probe),
                    Some(&run_world_probe),
                    Some(&save_project_probe),
                    Some(&reopen_project_probe),
                )?;
                record_ui_action_blockers(
                    &mut assertions,
                    &artifact,
                    &place_object_probe,
                    &object_placement_probe,
                    &object_transform_probe,
                    &edit_procedure_probe,
                    &run_world_probe,
                    &save_project_probe,
                    &reopen_project_probe,
                );
                let full_path_evidence = write_objects_first_full_path_contract(
                    &run_dir,
                    FullPathVisualEvidence {
                        screenshot: screenshot.as_ref(),
                        screenshot_error: screenshot_error.as_deref(),
                        ui_action_contract: Some(&artifact),
                    },
                    FullPathPhaseProbes {
                        object_placement: &object_placement_probe,
                        object_transform: &object_transform_probe,
                        edit_procedure: &edit_procedure_probe,
                        run_world: &run_world_probe,
                        save_project: &save_project_probe,
                        reopen_project: &reopen_project_probe,
                    },
                )?;
                for (id, assertion) in &full_path_evidence.assertions {
                    assertions.insert(id.clone(), assertion.clone());
                }
                if failure_category.is_none() {
                    failure_category = full_path_evidence.failure_category.clone();
                }
                (Some(artifact), Some(full_path_evidence))
            } else {
                let object_transform_probe =
                    is_objects_first_scenario(&options.scenario.id).then(|| {
                        probe_object_transform_hook(
                            &runner,
                            &options.alice_home,
                            &run_dir,
                            &object_placement_probe,
                            &options.scenario.starter_project,
                            display.name(),
                        )
                    });
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
                let reopen_project_probe =
                    is_objects_first_scenario(&options.scenario.id).then(|| {
                        probe_project_reopen_hook_with_selector(
                            &runner,
                            &options.alice_home,
                            &run_dir,
                            &save_project_probe,
                            crate::launch_save_project::DEFAULT_SAVE_SELECTOR,
                            display.name(),
                        )
                    });
                let persisted_state = is_objects_first_scenario(&options.scenario.id).then(|| {
                    reopen_project_probe
                        .as_ref()
                        .map(|probe| persisted_state_assertion(&run_dir, probe))
                        .unwrap_or_else(|| {
                            eatme_core::AssertionResult::fail(
                                "project reopen proof is required before persisted state can be trusted",
                            )
                        })
                });
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
                    object_transform_probe.as_ref(),
                    Some(&edit_procedure_probe),
                    Some(&run_world_probe),
                    Some(&save_project_probe),
                    reopen_project_probe.as_ref(),
                )?;
                if let (Some(object_transform_probe), Some(reopen_project_probe)) = (
                    object_transform_probe.as_ref(),
                    reopen_project_probe.as_ref(),
                ) {
                    record_ui_action_blockers(
                        &mut assertions,
                        &artifact,
                        &place_object_probe,
                        &object_placement_probe,
                        object_transform_probe,
                        &edit_procedure_probe,
                        &run_world_probe,
                        &save_project_probe,
                        reopen_project_probe,
                    );
                } else {
                    crate::launch_ui_actions::record_legacy_ui_action_blockers(
                        &mut assertions,
                        &artifact,
                        &place_object_probe,
                        &object_placement_probe,
                        &edit_procedure_probe,
                        &run_world_probe,
                        &save_project_probe,
                    );
                }
                if let Some(persisted_state) = persisted_state {
                    assertions.insert("persisted_state_verified".into(), persisted_state.clone());
                    let all_required_proof = object_placement_probe.proves_placement()
                        && object_transform_probe
                            .as_ref()
                            .is_some_and(|probe| probe.proves_transform())
                        && edit_procedure_probe.proves_edit()
                        && run_world_probe.proves_run()
                        && save_project_probe.proves_save()
                        && reopen_project_probe
                            .as_ref()
                            .is_some_and(|probe| probe.proves_reopen());
                    let evidence_summary =
                        record_evidence_summary(&run_dir, all_required_proof, &persisted_state)?;
                    assertions.insert(
                    "objects_first_evidence_recorded".into(),
                    bool_assert(
                        all_required_proof
                            && persisted_state.passed
                            && evidence_summary.size_bytes > 0,
                        "objects-first evidence summary records every major learner workflow step",
                    ),
                );
                    if failure_category.is_none() && !(all_required_proof && persisted_state.passed)
                    {
                        failure_category = Some("objects_first_workflow_incomplete".into());
                    }
                } else if failure_category.is_none() {
                    failure_category =
                        Some(ui_action_failure_category(&object_placement_probe).into());
                }
                (Some(artifact), None)
            }
        }
    } else {
        (None, None)
    };
    let launch_command = format!("java {}", launch_args.join(" "));
    let mut manifest = build_manifest(
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
    if let Some(full_path_evidence) = objects_first_full_path_evidence {
        manifest.command = Some(full_path_evidence.command);
        manifest.scenario = Some(full_path_evidence.scenario);
        manifest.evidence = Some(full_path_evidence.evidence);
        manifest.persistence_assertions = Some(full_path_evidence.persistence_assertions);
    }
    let manifest_write = write_manifest(&run_dir, &manifest);
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    manifest_write?;
    Ok(manifest)
}

fn missing_objects_first_full_path_hooks(alice_home: &Path) -> Vec<&'static str> {
    [
        DEFAULT_OBJECT_PLACEMENT_HOOK,
        DEFAULT_OBJECT_TRANSFORM_HOOK,
        DEFAULT_PROCEDURE_EDIT_HOOK,
        DEFAULT_WORLD_RUN_HOOK,
        DEFAULT_PROJECT_SAVE_HOOK,
        DEFAULT_PROJECT_REOPEN_HOOK,
    ]
    .into_iter()
    .filter(|relative_path| !alice_home.join(relative_path).is_file())
    .collect()
}

#[cfg(test)]
mod tests;
