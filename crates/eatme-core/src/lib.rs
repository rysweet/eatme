pub mod ast;
pub mod collaboration;
pub mod command;
pub mod fs_hash;
pub mod manifest;
pub mod pr199_recovery;

pub use ast::{Procedure, Program, Statement};
pub use command::{CommandOutput, CommandRunner, CommandSpec, RealCommandRunner};
pub use fs_hash::{file_size, sha256_file};
pub use manifest::{ArtifactInfo, AssertionResult, LaunchSmokeManifest};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::time::Duration;

    fn accepts_runner(_: &dyn CommandRunner) {}

    #[test]
    fn root_reexports_cover_ast_command_and_manifest_surfaces() {
        let program = Program::new(vec![Procedure {
            name: "main".into(),
            parameters: vec![],
            body: vec![Statement::Comment {
                text: "hello".into(),
            }],
        }]);
        let spec = CommandSpec::new("echo")
            .args(["hello"])
            .timeout(Duration::from_secs(1));
        let manifest = LaunchSmokeManifest {
            schema_version: "eatme.launch/v1".into(),
            scenario_id: "starter".into(),
            run_id: "run-1".into(),
            alice_home: "/alice".into(),
            alice_git_commit: "abc123".into(),
            eatme_git_commit: "def456".into(),
            java_version: "21".into(),
            maven_version: "3.9.9".into(),
            dependency_checks: BTreeMap::from([("java".into(), true)]),
            build_command: "mvn package".into(),
            build_exit_status: Some(0),
            launch_command: "alice".into(),
            display: ":99".into(),
            xvfb_pid: Some(1),
            alice_pid: Some(2),
            timeout_seconds: 120,
            window_list: Some(ArtifactInfo {
                path: "window-list.txt".into(),
                size_bytes: 12,
                sha256: "deadbeef".into(),
            }),
            window_list_error: None,
            screenshot: None,
            screenshot_error: None,
            post_focus_screenshot: None,
            post_focus_screenshot_error: None,
            ui_action_contract: None,
            log: None,
            log_error: None,
            fatal_log_scan: vec![],
            assertions: BTreeMap::from([("launch".into(), AssertionResult::pass("ok"))]),
            failure_category: None,
        };
        let runner = RealCommandRunner;

        accepts_runner(&runner);
        assert_eq!(program.procedures[0].name, "main");
        assert_eq!(spec.shell_display(), "echo hello");
        assert_eq!(manifest.assertions["launch"].detail, "ok");
    }

    #[test]
    fn root_reexported_hash_helpers_remain_callable() {
        let size_fn: fn(&Path) -> anyhow::Result<u64> = file_size;
        let hash_fn: fn(&Path) -> anyhow::Result<String> = sha256_file;

        let _ = (size_fn, hash_fn);
    }
}
