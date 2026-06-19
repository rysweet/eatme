use crate::launch_artifacts::artifact_info;
use crate::launch_options::LaunchSmokeOptions;
use crate::launch_ui_action_contract::write_ui_action_contract;
use crate::launch_ui_actions::{
    probe_place_object_preconditions, record_preflight_ui_action_blockers,
    record_ui_action_artifact,
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
    let log = artifact_info(&log_path).ok();
    if options.scenario.requires_real_ui_actions() {
        let place_object_probe =
            probe_place_object_preconditions(false, false, log.is_some(), None);
        record_preflight_ui_action_blockers(&mut assertions, &place_object_probe);
    }
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let place_object_probe =
            probe_place_object_preconditions(false, false, log.is_some(), None);
        let artifact = write_ui_action_contract(
            run_dir,
            false,
            false,
            log.is_some(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&place_object_probe),
            None,
            None,
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
        post_focus_screenshot: None,
        post_focus_screenshot_error: None,
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

fn write_manifest(run_dir: &Path, manifest: &LaunchSmokeManifest) -> Result<()> {
    let path = run_dir.join("manifest.json");
    fs::write(path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::LaunchSmokeScenario;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_run_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/launch-preflight-tests")
            .join(format!("{nonce}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn options_for(scenario_id: &str) -> LaunchSmokeOptions {
        LaunchSmokeOptions {
            alice_home: PathBuf::from("/alice"),
            run_id: "run-1".into(),
            runs_dir: PathBuf::from("runs"),
            timeout_seconds: 120,
            json: true,
            no_memory: false,
            offline_package: true,
            scenario: LaunchSmokeScenario::new(scenario_id),
        }
    }

    #[test]
    fn writes_ui_action_contract_for_real_ui_preflight_failures() {
        let run_dir = test_run_dir();
        let manifest = write_preflight_blocked_manifest(
            &options_for("first-lessons-real-ui-actions"),
            &run_dir,
            BTreeMap::from([("java".into(), true)]),
            "commit123".into(),
            "missing_dependency",
            "preflight blocked",
            BTreeMap::new(),
        )
        .unwrap();

        assert_eq!(
            manifest.failure_category.as_deref(),
            Some("missing_dependency")
        );
        assert_eq!(
            manifest.fatal_log_scan,
            vec!["preflight blocked".to_string()]
        );
        assert!(manifest.ui_action_contract.is_some());
        assert!(
            manifest.assertions["specific_alice_window_detected"]
                .detail
                .contains("preflight blocked")
        );
        assert!(manifest.assertions["ui_action_artifact_captured"].passed);
        assert_eq!(
            fs::read_to_string(run_dir.join("alice.log")).unwrap(),
            "preflight blocked\n"
        );
        assert!(run_dir.join("manifest.json").exists());
        assert!(run_dir.join("ui-action-contract.json").exists());

        let _ = fs::remove_dir_all(run_dir);
    }

    #[test]
    fn keeps_non_ui_preflight_manifests_minimal() {
        let run_dir = test_run_dir();
        let manifest = write_preflight_blocked_manifest(
            &options_for("real-alice-launch-smoke"),
            &run_dir,
            BTreeMap::from([("java".into(), false)]),
            "commit456".into(),
            "launch_failed",
            "startup failed",
            BTreeMap::from([(
                "dependencies_available".into(),
                AssertionResult::pass("checked"),
            )]),
        )
        .unwrap();

        assert!(manifest.ui_action_contract.is_none());
        assert_eq!(manifest.assertions.len(), 1);
        assert_eq!(
            manifest.assertions["dependencies_available"].detail,
            "checked"
        );
        assert!(!run_dir.join("ui-action-contract.json").exists());

        let _ = fs::remove_dir_all(run_dir);
    }
}
