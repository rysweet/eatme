use crate::launch_artifacts::artifact_info;
use anyhow::Result;
use eatme_core::{ArtifactInfo, AssertionResult, CommandRunner, CommandSpec};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

const ALICE_WINDOW_MARKERS: [&str; 4] = [
    "org.alice.stageide.entrypoint",
    "org.alice.stageide",
    "org.alice.ide",
    "alice 3",
];

#[derive(Clone, Debug, Serialize)]
pub struct UiActionProbe {
    pub id: String,
    pub status: String,
    pub detail: String,
    pub window_id: Option<String>,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn record_ui_action_blockers(
    assertions: &mut BTreeMap<String, AssertionResult>,
    artifact: &ArtifactInfo,
) {
    assertions.insert(
        "place_object_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can add/place an object yet",
        ),
    );
    assertions.insert(
        "edit_procedure_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can edit a procedure or code block yet",
        ),
    );
    assertions.insert(
        "run_world_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can run the world yet",
        ),
    );
    assertions.insert(
        "save_project_ui_action".into(),
        AssertionResult::fail(
            "blocked: no supported Alice desktop automation can save the project yet",
        ),
    );
    record_ui_action_artifact(assertions, artifact);
}

pub fn record_preflight_ui_action_blockers(assertions: &mut BTreeMap<String, AssertionResult>) {
    assertions.insert(
        "specific_alice_window_detected".into(),
        AssertionResult::fail("preflight blocked before an Alice window could be verified"),
    );
    assertions.insert(
        "activate_alice_window_ui_action".into(),
        AssertionResult::fail("preflight blocked before an Alice window could be activated"),
    );
    assertions.insert(
        "place_object_ui_action".into(),
        AssertionResult::fail("preflight blocked before add/place object automation could run"),
    );
    assertions.insert(
        "edit_procedure_ui_action".into(),
        AssertionResult::fail("preflight blocked before procedure/code-block editing could run"),
    );
    assertions.insert(
        "run_world_ui_action".into(),
        AssertionResult::fail("preflight blocked before world execution could run"),
    );
    assertions.insert(
        "save_project_ui_action".into(),
        AssertionResult::fail("preflight blocked before project save could run"),
    );
}

pub fn record_ui_action_artifact(
    assertions: &mut BTreeMap<String, AssertionResult>,
    artifact: &ArtifactInfo,
) {
    assertions.insert(
        "ui_action_artifact_captured".into(),
        bool_assert(
            artifact.size_bytes > 0,
            "ui action contract artifact exists and is non-empty",
        ),
    );
}

pub fn write_ui_action_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
    activation_probe: Option<&UiActionProbe>,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("ui-action-contract.json");
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet.",
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "executed_action_probes": activation_probe.into_iter().collect::<Vec<_>>(),
        "required_actions": [
            {
                "id": "verify-specific-alice-window",
                "required_evidence": "wmctrl output identifies an Alice Stage IDE window"
            },
            {
                "id": "activate-specific-alice-window",
                "required_evidence": "wmctrl -ia succeeds against the detected Alice window id"
            },
            {
                "id": "place-object",
                "required_evidence": "artifact proves an object was added to the scene and placed"
            },
            {
                "id": "edit-procedure-or-code-block",
                "required_evidence": "artifact proves a procedure or code block was edited"
            },
            {
                "id": "run-world",
                "required_evidence": "artifact proves the world run control was invoked"
            },
            {
                "id": "save-project",
                "required_evidence": "saved .a3p project artifact exists and is non-empty"
            }
        ]
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    artifact_info(&path)
}

pub fn probe_alice_window_activation(
    runner: &impl CommandRunner,
    display: &str,
    window_list: &str,
) -> UiActionProbe {
    let Some(window_id) = alice_window_id(window_list) else {
        return UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "blocked".into(),
            detail: "blocked: wmctrl -lx output did not identify an Alice window id".into(),
            window_id: None,
            command: None,
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        };
    };

    let output = runner.run(
        &CommandSpec::new("wmctrl")
            .args(["-ia".to_string(), window_id.clone()])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );

    match output {
        Ok(output) if output.exit_status == Some(0) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "passed".into(),
            detail: format!("wmctrl activated Alice window {window_id}"),
            window_id: Some(window_id),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Ok(output) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "failed".into(),
            detail: format!(
                "wmctrl could not activate Alice window {window_id}; exit_status={:?}",
                output.exit_status
            ),
            window_id: Some(window_id),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Err(error) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "failed".into(),
            detail: format!(
                "wmctrl activation probe failed for Alice window {window_id}: {error:#}"
            ),
            command: Some(format!("wmctrl -ia {window_id}")),
            window_id: Some(window_id),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

pub fn record_alice_window_activation(
    assertions: &mut BTreeMap<String, AssertionResult>,
    probe: &UiActionProbe,
) {
    assertions.insert(
        "activate_alice_window_ui_action".into(),
        bool_assert(probe.status == "passed", probe.detail.clone()),
    );
}

pub fn alice_window_id(window_list: &str) -> Option<String> {
    window_list.lines().find_map(|line| {
        let normalized = line.to_ascii_lowercase();
        if !ALICE_WINDOW_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
        {
            return None;
        }
        line.split_whitespace()
            .next()
            .filter(|id| id.starts_with("0x"))
            .map(str::to_string)
    })
}

fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eatme_core::CommandOutput;
    use eatme_test_support::FakeCommandRunner;

    #[test]
    fn finds_alice_window_id_from_wmctrl_output() {
        let window_list = "0x001  0 host org.alice.stageide.EntryPoint Alice 3";

        assert_eq!(alice_window_id(window_list).as_deref(), Some("0x001"));
    }

    #[test]
    fn activation_probe_runs_wmctrl_against_detected_window() {
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "wmctrl -ia 0x001".into(),
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        });

        let probe = probe_alice_window_activation(
            &runner,
            ":99",
            "0x001  0 host org.alice.stageide.EntryPoint Alice 3",
        );

        assert_eq!(probe.status, "passed");
        assert_eq!(probe.window_id.as_deref(), Some("0x001"));
        assert_eq!(runner.commands(), vec!["wmctrl -ia 0x001"]);
    }

    #[test]
    fn activation_probe_blocks_without_specific_alice_window() {
        let runner = FakeCommandRunner::default();

        let probe =
            probe_alice_window_activation(&runner, ":99", "0x002  0 host firefox.Firefox Firefox");

        assert_eq!(probe.status, "blocked");
        assert!(runner.commands().is_empty());
    }
}
