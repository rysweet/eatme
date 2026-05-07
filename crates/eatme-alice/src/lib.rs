pub mod compare;
pub mod deps;
pub mod discover;
pub mod launch;
mod launch_artifacts;
mod launch_desktop_controls;
mod launch_desktop_execution;
mod launch_edit_procedure;
mod launch_license;
mod launch_object_placement;
mod launch_options;
mod launch_run_window;
mod launch_run_world;
mod launch_save_project;
mod launch_ui_action_contract;
mod launch_ui_actions;
mod launch_window_activation;
mod launch_window_targeting;
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
pub use package::{PackageOptions, package_alice};
pub use scenario::LaunchSmokeScenario;
