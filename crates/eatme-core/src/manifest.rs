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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistence_assertions: Option<serde_json::Value>,
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

    #[test]
    fn launch_smoke_manifest_round_trips_optional_artifacts() {
        let manifest = LaunchSmokeManifest {
            schema_version: "eatme.launch/v1".into(),
            scenario_id: "creative-world".into(),
            run_id: "run-42".into(),
            alice_home: "/alice".into(),
            alice_git_commit: "abc123".into(),
            eatme_git_commit: "def456".into(),
            java_version: "21".into(),
            maven_version: "3.9.9".into(),
            dependency_checks: BTreeMap::from([("java".into(), true), ("maven".into(), true)]),
            build_command: "mvn -q test".into(),
            build_exit_status: Some(0),
            launch_command: "alice --headless".into(),
            display: ":99".into(),
            xvfb_pid: Some(41),
            alice_pid: Some(42),
            timeout_seconds: 180,
            window_list: Some(ArtifactInfo {
                path: "artifacts/window-list.txt".into(),
                size_bytes: 128,
                sha256: "window-sha".into(),
            }),
            window_list_error: None,
            screenshot: Some(ArtifactInfo {
                path: "artifacts/before.png".into(),
                size_bytes: 256,
                sha256: "before-sha".into(),
            }),
            screenshot_error: None,
            post_focus_screenshot: Some(ArtifactInfo {
                path: "artifacts/after.png".into(),
                size_bytes: 512,
                sha256: "after-sha".into(),
            }),
            post_focus_screenshot_error: Some("focus retry used".into()),
            ui_action_contract: Some(ArtifactInfo {
                path: "artifacts/ui-actions.json".into(),
                size_bytes: 64,
                sha256: "contract-sha".into(),
            }),
            log: Some(ArtifactInfo {
                path: "artifacts/launch.log".into(),
                size_bytes: 1024,
                sha256: "log-sha".into(),
            }),
            log_error: None,
            fatal_log_scan: vec!["SEVERE missing texture".into()],
            assertions: BTreeMap::from([("launch".into(), AssertionResult::pass("ok"))]),
            command: None,
            scenario: None,
            evidence: None,
            persistence_assertions: None,
            failure_category: Some("screenshot-mismatch".into()),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let restored: LaunchSmokeManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(
            restored.post_focus_screenshot.unwrap().path,
            "artifacts/after.png"
        );
        assert_eq!(
            restored.post_focus_screenshot_error.as_deref(),
            Some("focus retry used")
        );
        assert_eq!(
            restored.failure_category.as_deref(),
            Some("screenshot-mismatch")
        );
        assert_eq!(restored.assertions["launch"].detail, "ok");
    }

    #[test]
    fn launch_smoke_manifest_accepts_empty_dependency_checks() {
        let manifest: LaunchSmokeManifest = serde_json::from_value(json!({
            "schema_version": "eatme.launch/v1",
            "scenario_id": "baseline",
            "run_id": "run-empty",
            "alice_home": "/alice",
            "alice_git_commit": "abc123",
            "eatme_git_commit": "def456",
            "java_version": "21",
            "maven_version": "3.9.9",
            "dependency_checks": {},
            "build_command": "mvn verify",
            "build_exit_status": null,
            "launch_command": "alice",
            "display": ":99",
            "xvfb_pid": null,
            "alice_pid": null,
            "timeout_seconds": 120,
            "window_list": null,
            "window_list_error": null,
            "screenshot": null,
            "screenshot_error": null,
            "post_focus_screenshot": null,
            "post_focus_screenshot_error": null,
            "ui_action_contract": null,
            "log": null,
            "log_error": null,
            "fatal_log_scan": [],
            "assertions": {},
            "failure_category": null
        }))
        .unwrap();

        assert!(manifest.dependency_checks.is_empty());
        assert!(manifest.assertions.is_empty());
    }
}
