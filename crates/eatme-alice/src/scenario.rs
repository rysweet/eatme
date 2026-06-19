use std::path::PathBuf;

const DEFAULT_STARTER_PROJECT: &str =
    "core/resources/target/distribution/application/starter-projects/africa.a3p";

#[derive(Clone, Debug)]
pub struct LaunchSmokeScenario {
    pub id: String,
    pub starter_project: PathBuf,
}

impl LaunchSmokeScenario {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            starter_project: PathBuf::from(DEFAULT_STARTER_PROJECT),
            id: id.into(),
        }
    }

    pub fn real_alice_launch_smoke() -> Self {
        Self::new("real-alice-launch-smoke")
    }

    pub fn accepts_window_evidence(&self) -> bool {
        self.id != "real-alice-launch-smoke"
    }

    pub fn requires_real_ui_actions(&self) -> bool {
        self.id == "first-lessons-real-ui-actions" || self.id == "code-editor-first-run"
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_matches_real_launch_smoke() {
        let scenario = LaunchSmokeScenario::default();

        assert_eq!(scenario.id, "real-alice-launch-smoke");
        assert_eq!(
            scenario.starter_project,
            PathBuf::from(DEFAULT_STARTER_PROJECT)
        );
        assert!(!scenario.accepts_window_evidence());
    }

    #[test]
    fn real_ui_action_scenarios_are_strictly_enumerated() {
        assert!(
            LaunchSmokeScenario::new("first-lessons-real-ui-actions").requires_real_ui_actions()
        );
        assert!(LaunchSmokeScenario::new("code-editor-first-run").requires_real_ui_actions());
        assert!(!LaunchSmokeScenario::new("student-progression").requires_real_ui_actions());
        assert!(LaunchSmokeScenario::new("student-progression").accepts_window_evidence());
    }

    #[test]
    fn with_starter_project_replaces_only_the_project_path() {
        let scenario =
            LaunchSmokeScenario::new("custom").with_starter_project("fixtures/custom.a3p");

        assert_eq!(scenario.id, "custom");
        assert_eq!(
            scenario.starter_project,
            PathBuf::from("fixtures/custom.a3p")
        );
    }
}
