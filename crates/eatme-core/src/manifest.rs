use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssertionResult {
    pub passed: bool,
    pub detail: String,
}

impl AssertionResult {
    pub fn pass(detail: impl Into<String>) -> Self {
        Self {
            passed: true,
            detail: detail.into(),
        }
    }

    pub fn fail(detail: impl Into<String>) -> Self {
        Self {
            passed: false,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArtifactInfo {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LaunchSmokeManifest {
    pub schema_version: String,
    pub scenario_id: String,
    pub run_id: String,
    pub alice_home: String,
    pub alice_git_commit: String,
    pub eatme_git_commit: String,
    pub java_version: String,
    pub maven_version: String,
    pub dependency_checks: BTreeMap<String, bool>,
    pub build_command: String,
    pub build_exit_status: Option<i32>,
    pub launch_command: String,
    pub display: String,
    pub xvfb_pid: Option<u32>,
    pub alice_pid: Option<u32>,
    pub timeout_seconds: u64,
    pub window_list: Option<ArtifactInfo>,
    pub window_list_error: Option<String>,
    pub screenshot: Option<ArtifactInfo>,
    pub screenshot_error: Option<String>,
    #[serde(default)]
    pub post_focus_screenshot: Option<ArtifactInfo>,
    #[serde(default)]
    pub post_focus_screenshot_error: Option<String>,
    pub ui_action_contract: Option<ArtifactInfo>,
    pub log: Option<ArtifactInfo>,
    pub log_error: Option<String>,
    pub fatal_log_scan: Vec<String>,
    pub assertions: BTreeMap<String, AssertionResult>,
    pub failure_category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn assertion_result_helpers_set_expected_flags() {
        let pass = AssertionResult::pass("ready");
        let fail = AssertionResult::fail("blocked");

        assert!(pass.passed);
        assert_eq!(pass.detail, "ready");
        assert!(!fail.passed);
        assert_eq!(fail.detail, "blocked");
    }

    #[test]
    fn launch_smoke_manifest_defaults_post_focus_fields_when_absent() {
        let manifest: LaunchSmokeManifest = serde_json::from_value(json!({
            "schema_version": "eatme.launch/v1",
            "scenario_id": "first-world",
            "run_id": "run-1",
            "alice_home": "/alice",
            "alice_git_commit": "abc123",
            "eatme_git_commit": "def456",
            "java_version": "21",
            "maven_version": "3.9.9",
            "dependency_checks": {"java": true},
            "build_command": "mvn verify",
            "build_exit_status": 0,
            "launch_command": "alice",
            "display": ":99",
            "xvfb_pid": 42,
            "alice_pid": 43,
            "timeout_seconds": 120,
            "window_list": null,
            "window_list_error": null,
            "screenshot": null,
            "screenshot_error": null,
            "ui_action_contract": null,
            "log": null,
            "log_error": null,
            "fatal_log_scan": [],
            "assertions": {"launch": {"passed": true, "detail": "ok"}},
            "failure_category": null
        }))
        .unwrap();

        assert!(manifest.post_focus_screenshot.is_none());
        assert!(manifest.post_focus_screenshot_error.is_none());
        assert_eq!(manifest.assertions["launch"].detail, "ok");
    }
}
