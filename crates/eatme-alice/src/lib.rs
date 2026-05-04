pub mod deps;
mod desktop;
pub mod discover;
pub mod launch;
pub mod package;

pub use deps::{DependencyReport, check_dependencies};
pub use discover::{AliceDiscovery, discover_alice};
pub use launch::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
pub use package::{PackageOptions, package_alice};
