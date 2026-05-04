use std::path::PathBuf;

const DEFAULT_STARTER_PROJECT: &str =
    "core/resources/target/distribution/application/starter-projects/africa.a3p";

#[derive(Clone, Debug)]
pub struct LaunchSmokeScenario {
    pub id: String,
    pub run_dir_name: String,
    pub starter_project: PathBuf,
}

impl LaunchSmokeScenario {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            run_dir_name: id.clone(),
            starter_project: PathBuf::from(DEFAULT_STARTER_PROJECT),
            id,
        }
    }

    pub fn real_alice_launch_smoke() -> Self {
        Self::new("real-alice-launch-smoke")
    }

    pub fn accepts_window_evidence(&self) -> bool {
        self.id != "real-alice-launch-smoke"
    }

    pub fn requires_real_ui_actions(&self) -> bool {
        self.id == "first-lessons-real-ui-actions"
    }

    pub fn with_starter_project(mut self, starter_project: impl Into<PathBuf>) -> Self {
        self.starter_project = starter_project.into();
        self
    }
}

impl Default for LaunchSmokeScenario {
    fn default() -> Self {
        Self::real_alice_launch_smoke()
    }
}
