use crate::launch::LaunchSmokeOptions;
use crate::launch_artifacts::{artifact_info, write_manifest};
use crate::launch_ui_actions::{
    record_preflight_ui_action_blockers, record_ui_action_artifact, write_ui_action_contract,
};
use anyhow::Result;
use eatme_core::{AssertionResult, LaunchSmokeManifest};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn write_preflight_blocked_manifest(
    options: &LaunchSmokeOptions,
    run_dir: &Path,
    dependency_checks: BTreeMap<String, bool>,
    eatme_commit: String,
    failure_category: &str,
    detail: &str,
    mut assertions: BTreeMap<String, AssertionResult>,
) -> Result<LaunchSmokeManifest> {
    let log_path = run_dir.join("alice.log");
    fs::write(&log_path, format!("{detail}\n"))?;
    if options.scenario.requires_real_ui_actions() {
        record_preflight_ui_action_blockers(&mut assertions);
    }
    let log = artifact_info(&log_path).ok();
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let artifact = write_ui_action_contract(run_dir, false, false, log.is_some())?;
        record_ui_action_artifact(&mut assertions, &artifact);
        Some(artifact)
    } else {
        None
    };
    let manifest = LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: options.alice_home.display().to_string(),
        alice_git_commit: "unknown".into(),
        eatme_git_commit: eatme_commit,
        java_version: "unknown".into(),
        maven_version: "unknown".into(),
        dependency_checks,
        build_command: String::new(),
        build_exit_status: None,
        launch_command: String::new(),
        display: String::new(),
        xvfb_pid: None,
        alice_pid: None,
        timeout_seconds: options.timeout_seconds,
        window_list: None,
        window_list_error: None,
        screenshot: None,
        screenshot_error: None,
        ui_action_contract,
        log,
        log_error: None,
        fatal_log_scan: vec![detail.to_string()],
        assertions,
        failure_category: Some(failure_category.to_string()),
    };
    write_manifest(run_dir, &manifest)?;
    Ok(manifest)
}
