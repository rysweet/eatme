use super::LaunchSmokeOptions;
use super::evidence::artifact_info;
use crate::deps::DependencyReport;
use crate::discover::AliceDiscovery;
use crate::launch_ui_action_contract::write_ui_action_contract;
use crate::launch_ui_actions::{
    probe_place_object_preconditions, record_preflight_ui_action_blockers,
    record_ui_action_artifact,
};
use crate::package::PackageResult;
use anyhow::Result;
use eatme_core::{ArtifactInfo, AssertionResult, LaunchSmokeManifest};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub(super) fn write_blocked_manifest(
    options: &LaunchSmokeOptions,
    run_dir: &Path,
    deps: DependencyReport,
    eatme_commit: &str,
    discovery: Option<&AliceDiscovery>,
    package: Option<&PackageResult>,
    display: Option<&str>,
    xvfb_pid: Option<u32>,
    category: &str,
    diagnostic: impl Into<String>,
    mut assertions: BTreeMap<String, AssertionResult>,
) -> Result<LaunchSmokeManifest> {
    let diagnostic = diagnostic.into();
    fs::write(run_dir.join("alice.log"), format!("{diagnostic}\n"))?;
    assertions.insert(
        "real_alice_execution_evidence".into(),
        AssertionResult::fail(diagnostic),
    );
    let log = artifact_info(&run_dir.join("alice.log")).ok();
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let place_object_probe =
            probe_place_object_preconditions(false, false, log.is_some(), None);
        record_preflight_ui_action_blockers(&mut assertions, &place_object_probe);
        let artifact = write_ui_action_contract(
            run_dir,
            false,
            false,
            log.is_some(),
            None,
            None,
            None,
            Some(&place_object_probe),
            None,
            None,
            None,
            None,
        )?;
        record_ui_action_artifact(&mut assertions, &artifact);
        Some(artifact)
    } else {
        None
    };
    let manifest = build_manifest(
        options,
        deps,
        eatme_commit,
        discovery,
        package,
        String::new(),
        display.unwrap_or("").to_string(),
        xvfb_pid,
        None,
        None,
        None,
        None,
        None,
        ui_action_contract,
        log,
        None,
        Vec::new(),
        assertions,
        Some(category.to_string()),
    );
    write_manifest(run_dir, &manifest)?;
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_manifest(
    options: &LaunchSmokeOptions,
    deps: DependencyReport,
    eatme_commit: &str,
    discovery: Option<&AliceDiscovery>,
    package: Option<&PackageResult>,
    launch_command: String,
    display: String,
    xvfb_pid: Option<u32>,
    alice_pid: Option<u32>,
    window_list: Option<ArtifactInfo>,
    window_list_error: Option<String>,
    screenshot: Option<ArtifactInfo>,
    screenshot_error: Option<String>,
    ui_action_contract: Option<ArtifactInfo>,
    log: Option<ArtifactInfo>,
    log_error: Option<String>,
    fatal_log_scan: Vec<String>,
    assertions: BTreeMap<String, AssertionResult>,
    failure_category: Option<String>,
) -> LaunchSmokeManifest {
    LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: discovery
            .map(|value| value.alice_home.clone())
            .unwrap_or_else(|| options.alice_home.display().to_string()),
        alice_git_commit: discovery
            .map(|value| value.git_commit.clone())
            .unwrap_or_else(|| "unknown".into()),
        eatme_git_commit: eatme_commit.to_string(),
        java_version: discovery
            .map(|value| value.java_version.clone())
            .unwrap_or_else(|| "unknown".into()),
        maven_version: discovery
            .map(|value| value.maven_version.clone())
            .unwrap_or_else(|| "unknown".into()),
        dependency_checks: deps.tools,
        build_command: package
            .map(|value| value.command.clone())
            .unwrap_or_default(),
        build_exit_status: package.and_then(|value| value.exit_status),
        launch_command,
        display,
        xvfb_pid,
        alice_pid,
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
    }
}

pub(super) fn write_manifest(run_dir: &Path, manifest: &LaunchSmokeManifest) -> Result<()> {
    let path = run_dir.join("manifest.json");
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json)?;
    Ok(())
}
