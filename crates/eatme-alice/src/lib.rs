pub mod compare;
pub mod deps;
pub mod discover;
pub mod launch;
mod launch_artifacts;
mod launch_class_portability;
mod launch_desktop_controls;
mod launch_desktop_execution;
mod launch_edit_procedure;
mod launch_license;
mod launch_object_placement;
mod launch_object_transform;
mod launch_objects_first_full_path;
mod launch_options;
mod launch_path_validation;
mod launch_preflight;
mod launch_reopen_project;
mod launch_run_window;
mod launch_run_window_poll;
mod launch_run_world;
mod launch_save_project;
#[cfg(test)]
mod launch_save_reopen_contract_tests;
mod launch_ui_action_contract;
mod launch_ui_actions;
mod launch_window_activation;
mod launch_window_targeting;
mod objects_first_workflow;
pub mod package;
pub mod scenario;

pub use compare::{
    AliceComparisonOptions, FIRST_LESSON_SCENARIO_ID, FirstLessonReadinessOptions,
    check_lesson_session_contract, check_lesson_session_readiness,
    run_first_lesson_readiness_sequence, run_launch_smoke_comparison,
};
pub use deps::{DependencyReport, check_dependencies};
pub use discover::{AliceDiscovery, discover_alice};
pub use launch::run_launch_smoke;
pub use launch_options::LaunchSmokeOptions;
pub use launch_preflight::write_preflight_blocked_manifest;
pub use package::{PackageOptions, package_alice};
pub use scenario::{LaunchSmokeScenario, OBJECTS_FIRST_FULL_PATH_SCENARIO_ID};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn root_reexports_cover_option_and_report_types() {
        let scenario =
            LaunchSmokeScenario::new(FIRST_LESSON_SCENARIO_ID).with_starter_project("starter.a3p");
        let launch = LaunchSmokeOptions {
            alice_home: PathBuf::from("/alice"),
            run_id: "run-1".into(),
            runs_dir: PathBuf::from("runs"),
            timeout_seconds: 120,
            json: true,
            no_memory: false,
            offline_package: true,
            scenario: scenario.clone(),
        };
        let package = PackageOptions {
            alice_home: Path::new("/alice"),
            offline: true,
        };
        let dependency_report = DependencyReport {
            tools: BTreeMap::from([("java".into(), true)]),
            screenshot_available: false,
            all_required_available: false,
        };
        let discovery = AliceDiscovery {
            alice_home: "/alice".into(),
            git_commit: "abc123".into(),
            java_version: "21".into(),
            maven_version: "3.9.9".into(),
            alice_ide_jar_exists: true,
            target_lib_exists: true,
            starter_project_exists: true,
        };
        let comparison = AliceComparisonOptions {
            registry_path: PathBuf::from("registry.yaml"),
            baseline_target: "baseline".into(),
            modernized_target: "modernized".into(),
            baseline_home_override: None,
            modernized_home_override: None,
            scenario,
            run_id: "run-1".into(),
            runs_dir: PathBuf::from("runs"),
            timeout_seconds: 120,
            json: true,
            no_memory: false,
            offline_package: true,
            execute: false,
        };

        assert_eq!(launch.scenario.id, FIRST_LESSON_SCENARIO_ID);
        assert!(launch.scenario.requires_real_ui_actions());
        assert!(package.offline);
        assert_eq!(dependency_report.tools.get("java"), Some(&true));
        assert!(discovery.starter_project_exists);
        assert_eq!(comparison.baseline_target, "baseline");
    }

    #[test]
    fn root_reexports_preserve_scenario_defaults() {
        let default_scenario = LaunchSmokeScenario::default();

        assert_eq!(default_scenario.id, "real-alice-launch-smoke");
        assert!(!default_scenario.accepts_window_evidence());
    }
}
