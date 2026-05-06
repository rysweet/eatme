use crate::launch_artifacts::artifact_info;
use crate::launch_object_placement::{
    DEFAULT_OBJECT_PLACEMENT_HOOK, UiActionObjectPlacementProbe,
    missing_object_placement_affordance,
};
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

#[derive(Clone, Debug, Serialize)]
pub struct UiActionPrecondition {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiActionNoGoProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub decision: String,
    pub blocking_reason: String,
    pub required_evidence: String,
    pub missing_affordance: UiActionMissingAffordance,
    pub preconditions: Vec<UiActionPrecondition>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiActionMissingAffordance {
    pub id: String,
    pub kind: String,
    pub required_capability: String,
    pub missing_contract: String,
    pub next_implementation: String,
}

pub fn record_ui_action_blockers(
    assertions: &mut BTreeMap<String, AssertionResult>,
    artifact: &ArtifactInfo,
    place_object_precondition_probe: &UiActionNoGoProbe,
    object_placement_probe: &UiActionObjectPlacementProbe,
) {
    record_place_object_probe(
        assertions,
        place_object_precondition_probe,
        object_placement_probe,
    );
    assertions.insert(
        "place_object_ui_action".into(),
        bool_assert(
            object_placement_probe.proves_placement(),
            object_placement_probe.detail.clone(),
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

pub fn record_preflight_ui_action_blockers(
    assertions: &mut BTreeMap<String, AssertionResult>,
    place_object_probe: &UiActionNoGoProbe,
) {
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
    record_place_object_precondition_no_go(assertions, place_object_probe);
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

pub fn ui_action_failure_category(
    object_placement_probe: &UiActionObjectPlacementProbe,
) -> &'static str {
    if object_placement_probe.proves_placement() {
        "ui_action_remaining_steps_unimplemented"
    } else {
        "ui_action_automation_unimplemented"
    }
}

pub fn write_ui_action_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
    activation_probe: Option<&UiActionProbe>,
    place_object_precondition_probe: Option<&UiActionNoGoProbe>,
    object_placement_probe: Option<&UiActionObjectPlacementProbe>,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("ui-action-contract.json");
    let placement_status = object_placement_probe
        .map(|probe| probe.status.as_str())
        .unwrap_or("blocked");
    let action_precondition_probes = place_object_precondition_probe
        .into_iter()
        .filter(|_| placement_status != "passed")
        .collect::<Vec<_>>();
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": ui_action_blocking_reason(placement_status),
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "executed_action_probes": activation_probe.into_iter().collect::<Vec<_>>(),
        "action_precondition_probes": action_precondition_probes,
        "candidate_affordance_probes": object_placement_probe.into_iter().collect::<Vec<_>>(),
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
                "required_evidence": "artifact proves a named object was added to the scene and placed without coordinate guessing",
                "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
                "contract_required": {
                    "candidate_backend": DEFAULT_OBJECT_PLACEMENT_HOOK,
                    "inputs": ["open_project", "object_identifier", "evidence_dir"],
                    "outputs": ["placement_artifact", "scene_or_project_diff"],
                    "unsafe_until_available": placement_status != "passed"
                }
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

pub fn probe_place_object_preconditions(
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
    activation_probe: Option<&UiActionProbe>,
) -> UiActionNoGoProbe {
    let activation_passed = activation_probe
        .map(|probe| probe.status == "passed")
        .unwrap_or(false);
    let window_targeting_ready = specific_alice_window_detected && activation_passed;
    let missing_affordance = missing_object_placement_affordance();
    let blocking_reason = if window_targeting_ready {
        "blocked: missing deterministic-alice-object-gallery-placement-affordance"
    } else {
        "blocked: Alice window targeting preconditions are incomplete, so object placement would be unsafe"
    };

    UiActionNoGoProbe {
        id: "place-object-precondition".into(),
        action_id: "place-object".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence: "artifact proves a named object was added to the Alice scene and placed without coordinate guessing".into(),
        missing_affordance,
        preconditions: vec![
            UiActionPrecondition {
                id: "specific-alice-window-detected".into(),
                passed: specific_alice_window_detected,
                detail: "wmctrl output identifies an Alice Stage IDE window".into(),
            },
            UiActionPrecondition {
                id: "activate-specific-alice-window".into(),
                passed: activation_passed,
                detail: "wmctrl -ia succeeds against the detected Alice window id".into(),
            },
            UiActionPrecondition {
                id: "visual-evidence-captured".into(),
                passed: visual_evidence_captured,
                detail: "startup screenshot or window evidence exists".into(),
            },
            UiActionPrecondition {
                id: "log-captured".into(),
                passed: log_captured,
                detail: "Alice launch log exists and is non-empty".into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-object-gallery-placement-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, menu action, or scene-graph verification hook for named object placement".into(),
            },
        ],
    }
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

fn record_place_object_probe(
    assertions: &mut BTreeMap<String, AssertionResult>,
    precondition_probe: &UiActionNoGoProbe,
    object_placement_probe: &UiActionObjectPlacementProbe,
) {
    if !object_placement_probe.proves_placement() {
        record_place_object_precondition_no_go(assertions, precondition_probe);
    }
    assertions.insert(
        "place_object_candidate_hook_probe".into(),
        bool_assert(
            object_placement_probe.action_id == "place-object"
                && object_placement_probe.id == "alice-side-object-placement-command-hook"
                && ["passed", "blocked", "failed"]
                    .contains(&object_placement_probe.status.as_str()),
            object_placement_probe.detail.clone(),
        ),
    );
}

fn record_place_object_precondition_no_go(
    assertions: &mut BTreeMap<String, AssertionResult>,
    precondition_probe: &UiActionNoGoProbe,
) {
    assertions.insert(
        "place_object_precondition_no_go_probe".into(),
        bool_assert(
            precondition_probe.action_id == "place-object"
                && precondition_probe.status == "blocked"
                && precondition_probe.decision == "no_go",
            precondition_probe.blocking_reason.clone(),
        ),
    );
}

fn ui_action_blocking_reason(placement_status: &str) -> &'static str {
    if placement_status == "passed" {
        "Deterministic object placement evidence exists, but procedure editing, world run, and project save automation are not wired yet."
    } else {
        "The harness can activate a detected Alice window when present, but deterministic object placement, procedure editing, world run, and project save automation are not wired yet."
    }
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
fn object_placement_probe_with_status(status: &str) -> UiActionObjectPlacementProbe {
    UiActionObjectPlacementProbe {
        id: "alice-side-object-placement-command-hook".into(),
        action_id: "place-object".into(),
        status: status.into(),
        detail: "probe detail".into(),
        object_identifier: "alice-gallery://animals/bunny".into(),
        candidate_hook_path: "tools/eatme-place-object".into(),
        command: Some("tools/eatme-place-object --json".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        placement_artifact: if status == "passed" {
            Some(ArtifactInfo {
                path: "object-placement/placement.json".into(),
                size_bytes: 2,
                sha256: "placement-sha".into(),
            })
        } else {
            None
        },
        scene_or_project_diff: if status == "passed" {
            Some(ArtifactInfo {
                path: "object-placement/scene.diff.json".into(),
                size_bytes: 2,
                sha256: "diff-sha".into(),
            })
        } else {
            None
        },
        validation_errors: Vec::new(),
        missing_affordance: None,
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

    #[test]
    fn place_object_precondition_probe_records_no_go_after_window_activation() {
        let activation_probe = UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "passed".into(),
            detail: "wmctrl activated Alice window 0x001".into(),
            window_id: Some("0x001".into()),
            command: Some("wmctrl -ia 0x001".into()),
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        };

        let probe = probe_place_object_preconditions(true, true, true, Some(&activation_probe));

        assert_eq!(probe.id, "place-object-precondition");
        assert_eq!(probe.action_id, "place-object");
        assert_eq!(probe.status, "blocked");
        assert_eq!(probe.decision, "no_go");
        assert_eq!(
            probe.missing_affordance.id,
            "deterministic-alice-object-gallery-placement-affordance"
        );
        assert!(
            probe
                .missing_affordance
                .required_capability
                .contains("named object identifier")
        );
        assert!(
            probe
                .missing_affordance
                .next_implementation
                .contains("named gallery selector")
        );
        assert!(probe.preconditions.iter().any(|precondition| {
            precondition.id == "deterministic-alice-object-gallery-placement-affordance"
                && !precondition.passed
        }));
    }

    #[test]
    fn ui_action_failure_category_advances_after_object_placement_proof() {
        let placed = object_placement_probe_with_status("passed");
        let blocked = object_placement_probe_with_status("blocked");

        assert_eq!(
            ui_action_failure_category(&placed),
            "ui_action_remaining_steps_unimplemented"
        );
        assert_eq!(
            ui_action_failure_category(&blocked),
            "ui_action_automation_unimplemented"
        );
    }
}
