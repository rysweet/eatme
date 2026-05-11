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
