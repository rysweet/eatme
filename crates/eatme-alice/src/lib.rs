pub mod compare;
pub mod deps;
pub mod discover;
pub mod launch;
mod launch_artifacts;
mod launch_object_placement;
mod launch_ui_actions;
pub mod package;
pub mod scenario;

pub use compare::{
    AliceComparisonOptions, FIRST_LESSON_SCENARIO_ID, FirstLessonReadinessOptions,
    check_lesson_session_contract, check_lesson_session_readiness,
    run_first_lesson_readiness_sequence, run_launch_smoke_comparison,
};
pub use deps::{DependencyReport, check_dependencies};
pub use discover::{AliceDiscovery, discover_alice};
pub use launch::{LaunchSmokeOptions, run_launch_smoke};
pub use package::{PackageOptions, package_alice};
pub use scenario::LaunchSmokeScenario;
