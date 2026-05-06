use crate::launch_ui_actions::UiActionProbe;
use eatme_core::{AssertionResult, CommandRunner};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub(crate) const DESKTOP_RUN_EXECUTION_SENTINEL: &str =
    "run-window-evidence/desktop-run-execution.json";
const DESKTOP_RUN_RUNTIME_LOG: &str = "desktop-run-runtime.log";
const DESKTOP_RUN_EXECUTION_WAIT: Duration = Duration::from_secs(20);

pub(crate) fn probe_toolbar_run_and_execution(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    activation_probe: Option<&UiActionProbe>,
    run_window_probe: &UiActionProbe,
    assertions: &mut BTreeMap<String, AssertionResult>,
) -> (UiActionProbe, UiActionProbe, UiActionProbe) {
    let (desktop_run_toolbar_probe, run_window_after_toolbar_probe) =
        crate::launch_run_window::probe_run_toolbar_sequence(
            runner,
            display,
            run_dir,
            activation_probe,
            run_window_probe,
        );
    if desktop_run_toolbar_probe.status == "passed" {
        assertions.insert(
            "run_world_desktop_toolbar_dispatch".into(),
            AssertionResult::pass(desktop_run_toolbar_probe.detail.clone()),
        );
        assertions.insert(
            "run_world_desktop_toolbar_window_observed".into(),
            assertion_from_probe(&run_window_after_toolbar_probe),
        );
    }
    let desktop_run_execution_probe =
        probe_desktop_run_execution_after_toolbar(run_dir, &run_window_after_toolbar_probe);
    if run_window_after_toolbar_probe.status == "passed" {
        assertions.insert(
            "run_world_desktop_execution_observed".into(),
            assertion_from_probe(&desktop_run_execution_probe),
        );
    }
    (
        desktop_run_toolbar_probe,
        run_window_after_toolbar_probe,
        desktop_run_execution_probe,
    )
}

fn assertion_from_probe(probe: &UiActionProbe) -> AssertionResult {
    if probe.status == "passed" {
        AssertionResult::pass(probe.detail.clone())
    } else {
        AssertionResult::fail(probe.detail.clone())
    }
}

pub(crate) fn probe_desktop_run_execution_after_toolbar(
    run_dir: &Path,
    run_window_after_toolbar_probe: &UiActionProbe,
) -> UiActionProbe {
    if run_window_after_toolbar_probe.status != "passed" {
        return blocked_probe(
            "observe-desktop-run-execution-after-toolbar-button",
            "blocked: Run window must be observed before desktop Run execution evidence",
        );
    }
    let path = run_dir.join(DESKTOP_RUN_EXECUTION_SENTINEL);
    let mut last_content = String::new();
    let mut last_rejection = "artifact was not present".to_string();
    let evidence_dir = path.parent().unwrap_or(run_dir);
    let deadline = Instant::now() + DESKTOP_RUN_EXECUTION_WAIT;
    while Instant::now() <= deadline {
        if let Ok(content) = fs::read_to_string(&path) {
            match desktop_execution_artifact_indicates_statement_execution(&content, evidence_dir) {
                Ok(()) => {
                    return UiActionProbe {
                        id: "observe-desktop-run-execution-after-toolbar-button".into(),
                        status: "passed".into(),
                        detail: "observed RabbitHole desktop Run execution artifact with VM statement events; this proves desktop execution started, not rendering correctness or lesson completion".into(),
                        window_id: run_window_after_toolbar_probe.window_id.clone(),
                        command: Some(format!("read {}", path.display())),
                        exit_status: Some(0),
                        stdout: content,
                        stderr: String::new(),
                    };
                }
                Err(reason) => {
                    if !is_transient_execution_artifact_rejection(&content, &reason) {
                        return failed_present_probe(
                            &path,
                            run_window_after_toolbar_probe,
                            content,
                            reason,
                        );
                    }
                    last_content = content;
                    last_rejection = reason;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if last_content.is_empty() {
        return UiActionProbe {
            id: "observe-desktop-run-execution-after-toolbar-button".into(),
            status: "failed".into(),
            detail:
                "Run window opened, but no RabbitHole desktop Run execution artifact was observed"
                    .into(),
            window_id: run_window_after_toolbar_probe.window_id.clone(),
            command: Some(format!("read {}", path.display())),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    failed_present_probe(
        &path,
        run_window_after_toolbar_probe,
        last_content,
        last_rejection,
    )
}

fn failed_present_probe(
    path: &Path,
    run_window_after_toolbar_probe: &UiActionProbe,
    content: String,
    reason: String,
) -> UiActionProbe {
    UiActionProbe {
        id: "observe-desktop-run-execution-after-toolbar-button".into(),
        status: "failed".into(),
        detail: format!(
            "RabbitHole desktop Run execution artifact was present but did not prove VM statement execution: {reason}"
        ),
        window_id: run_window_after_toolbar_probe.window_id.clone(),
        command: Some(format!("read {}", path.display())),
        exit_status: Some(0),
        stdout: content,
        stderr: String::new(),
    }
}

fn desktop_execution_artifact_indicates_statement_execution(
    content: &str,
    evidence_dir: &Path,
) -> Result<(), String> {
    let value = serde_json::from_str::<serde_json::Value>(content)
        .map_err(|error| format!("invalid json: {error}"))?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.alice-desktop-run-execution/v1")
    {
        return Err("schema_version mismatch".into());
    }
    if value.get("status").and_then(serde_json::Value::as_str)
        != Some("statement_execution_observed")
    {
        return Err("status is not statement_execution_observed".into());
    }
    if value
        .get("active_scene_invoke_started")
        .and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        return Err("active scene invocation did not start".into());
    }
    if value
        .get("executing_statement_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        return Err("executing_statement_count is zero".into());
    }
    let runtime_log = value
        .get("runtime_log")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "runtime_log is missing".to_string())?;
    if runtime_log != DESKTOP_RUN_RUNTIME_LOG {
        return Err("runtime_log name mismatch".into());
    }
    let metadata = fs::metadata(evidence_dir.join(DESKTOP_RUN_RUNTIME_LOG))
        .map_err(|error| format!("runtime_log artifact is not readable: {error}"))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("runtime_log artifact is empty".into());
    }
    Ok(())
}

fn is_transient_execution_artifact_rejection(content: &str, reason: &str) -> bool {
    content.contains(r#""status":"preparing""#)
        || content.contains(r#""status": "preparing""#)
        || reason.starts_with("invalid json:")
        || reason.starts_with("runtime_log artifact is not readable:")
}

fn blocked_probe(id: &str, detail: &str) -> UiActionProbe {
    UiActionProbe {
        id: id.into(),
        status: "blocked".into(),
        detail: detail.into(),
        window_id: None,
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn observes_desktop_run_execution_artifact_with_statement_count() {
        let run_dir = unique_test_dir("desktop-run-execution");
        write_desktop_execution_artifact(&run_dir, 1);

        let probe = probe_desktop_run_execution_after_toolbar(
            &run_dir,
            &run_window_after_toolbar_probe("passed"),
        );

        assert_eq!(probe.status, "passed");
        assert_eq!(
            probe.id,
            "observe-desktop-run-execution-after-toolbar-button"
        );
        assert!(probe.detail.contains("VM statement events"));
        assert!(
            probe
                .stdout
                .contains("eatme.alice-desktop-run-execution/v1")
        );
        let _ = std::fs::remove_dir_all(run_dir);
    }

    #[test]
    fn rejects_desktop_run_execution_artifact_without_statement_count() {
        let run_dir = unique_test_dir("desktop-run-execution-zero");
        write_desktop_execution_artifact(&run_dir, 0);

        let probe = probe_desktop_run_execution_after_toolbar(
            &run_dir,
            &run_window_after_toolbar_probe("passed"),
        );

        assert_eq!(probe.status, "failed");
        assert!(probe.detail.contains("executing_statement_count is zero"));
        let _ = std::fs::remove_dir_all(run_dir);
    }

    #[test]
    fn blocks_desktop_run_execution_probe_until_run_window_is_proven() {
        let run_dir = unique_test_dir("desktop-run-execution-blocked");

        let probe = probe_desktop_run_execution_after_toolbar(
            &run_dir,
            &run_window_after_toolbar_probe("failed"),
        );

        assert_eq!(probe.status, "blocked");
        assert!(probe.detail.contains("Run window must be observed"));
        let _ = std::fs::remove_dir_all(run_dir);
    }

    fn run_window_after_toolbar_probe(status: &str) -> UiActionProbe {
        UiActionProbe {
            id: "observe-run-window-after-toolbar-button".into(),
            status: status.into(),
            detail: "Run window observed".into(),
            window_id: Some("0x002".into()),
            command: Some("read run-window-created.json".into()),
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn write_desktop_execution_artifact(run_dir: &Path, executing_statement_count: u64) {
        let sentinel = run_dir.join(DESKTOP_RUN_EXECUTION_SENTINEL);
        let evidence_dir = sentinel.parent().unwrap();
        std::fs::create_dir_all(evidence_dir).unwrap();
        std::fs::write(
            evidence_dir.join(DESKTOP_RUN_RUNTIME_LOG),
            "executing:Comment\n",
        )
        .unwrap();
        std::fs::write(
            sentinel,
            format!(
                r#"{{
                    "schema_version":"eatme.alice-desktop-run-execution/v1",
                    "status":"statement_execution_observed",
                    "active_scene_invoke_started":true,
                    "executing_statement_count":{executing_statement_count},
                    "runtime_log":"desktop-run-runtime.log"
                }}"#
            ),
        )
        .unwrap();
    }

    fn unique_test_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("eatme-{prefix}-{nonce}"))
    }
}
