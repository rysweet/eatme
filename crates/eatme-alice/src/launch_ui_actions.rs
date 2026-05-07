use crate::launch_edit_procedure::{
    UiActionEditProcedureProbe, probe_edit_procedure_preconditions,
};
use crate::launch_object_placement::{
    UiActionObjectPlacementProbe, missing_object_placement_affordance,
};
use crate::launch_run_world::{UiActionRunWorldProbe, probe_run_world_preconditions};
use crate::launch_save_project::{UiActionSaveProjectProbe, probe_project_save_preconditions};
use crate::launch_window_activation::activation_failure_detail;
use crate::launch_window_targeting::{AliceWindowSearch, alice_window_search};
use eatme_core::{ArtifactInfo, AssertionResult, CommandRunner, CommandSpec};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

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
    edit_procedure_probe: &UiActionEditProcedureProbe,
    run_world_probe: &UiActionRunWorldProbe,
    save_project_probe: &UiActionSaveProjectProbe,
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
        bool_assert(
            edit_procedure_probe.proves_edit(),
            edit_procedure_probe.detail.clone(),
        ),
    );
    record_edit_procedure_probe(assertions, edit_procedure_probe);
    if object_placement_probe.proves_placement() && !edit_procedure_probe.proves_edit() {
        let edit_precondition_probe = probe_edit_procedure_preconditions(object_placement_probe);
        record_edit_procedure_precondition_no_go(assertions, &edit_precondition_probe);
    }
    record_run_world_probe(assertions, run_world_probe);
    if edit_procedure_probe.proves_edit() && !run_world_probe.proves_run() {
        let run_world_probe = probe_run_world_preconditions(edit_procedure_probe);
        record_run_world_precondition_no_go(assertions, &run_world_probe);
    }
    assertions.insert(
        "run_world_ui_action".into(),
        bool_assert(run_world_probe.proves_run(), run_world_probe.detail.clone()),
    );
    record_save_project_probe(assertions, save_project_probe);
    if run_world_probe.proves_run() && !save_project_probe.proves_save() {
        let save_project_probe = probe_project_save_preconditions(run_world_probe);
        record_save_project_precondition_no_go(assertions, &save_project_probe);
    }
    assertions.insert(
        "save_project_ui_action".into(),
        bool_assert(
            save_project_probe.proves_save(),
            save_project_probe.detail.clone(),
        ),
    );
    record_ui_action_artifact(assertions, artifact);
}

fn record_save_project_probe(
    assertions: &mut BTreeMap<String, AssertionResult>,
    probe: &UiActionSaveProjectProbe,
) {
    assertions.insert(
        "save_project_candidate_hook_probe".into(),
        bool_assert(probe.proves_save(), probe.detail.clone()),
    );
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
                detail: "wmctrl or xwininfo output identifies the Alice main window".into(),
            },
            UiActionPrecondition {
                id: "activate-specific-alice-window".into(),
                passed: activation_passed,
                detail: "wmctrl -ia or xdotool windowfocus succeeds against the detected Alice window id".into(),
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
    let window_id = match alice_window_search(window_list) {
        AliceWindowSearch::Found { window_id, .. } => window_id,
        AliceWindowSearch::WrongAliceLikeWindow { detail }
        | AliceWindowSearch::NoAliceWindow { detail } => {
            return UiActionProbe {
                id: "activate-specific-alice-window".into(),
                status: "blocked".into(),
                detail: format!("blocked: {detail}"),
                window_id: None,
                command: None,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
            };
        }
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
        Ok(output) => focus_alice_window_with_xdotool(runner, display, &window_id, Some(output)),
        Err(_) => focus_alice_window_with_xdotool(runner, display, &window_id, None),
    }
}

fn focus_alice_window_with_xdotool(
    runner: &impl CommandRunner,
    display: &str,
    window_id: &str,
    wmctrl_output: Option<eatme_core::CommandOutput>,
) -> UiActionProbe {
    let output = runner.run(
        &CommandSpec::new("xdotool")
            .args(["windowfocus".to_string(), window_id.to_string()])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );

    match output {
        Ok(output) if output.exit_status == Some(0) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "passed".into(),
            detail: format!("xdotool focused Alice window {window_id}"),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Ok(output) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "failed".into(),
            detail: activation_failure_detail(window_id, wmctrl_output.as_ref(), &output),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: combined_probe_output(wmctrl_output.as_ref(), &output.stdout),
            stderr: combined_probe_output(wmctrl_output.as_ref(), &output.stderr),
        },
        Err(error) => UiActionProbe {
            id: "activate-specific-alice-window".into(),
            status: "failed".into(),
            detail: format!(
                "wmctrl and xdotool could not focus Alice window {window_id}: {error:#}"
            ),
            command: Some(format!("xdotool windowfocus {window_id}")),
            window_id: Some(window_id.into()),
            exit_status: None,
            stdout: wmctrl_output
                .as_ref()
                .map(|output| output.stdout.clone())
                .unwrap_or_default(),
            stderr: wmctrl_output
                .as_ref()
                .map(|output| output.stderr.clone())
                .unwrap_or_default(),
        },
    }
}

fn combined_probe_output(
    wmctrl_output: Option<&eatme_core::CommandOutput>,
    xdotool_output: &str,
) -> String {
    match wmctrl_output {
        Some(output) if !output.stderr.is_empty() || !output.stdout.is_empty() => {
            format!(
                "wmctrl stdout:\n{}wmctrl stderr:\n{}xdotool:\n{}",
                output.stdout, output.stderr, xdotool_output
            )
        }
        _ => xdotool_output.into(),
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

fn record_edit_procedure_probe(
    assertions: &mut BTreeMap<String, AssertionResult>,
    edit_procedure_probe: &UiActionEditProcedureProbe,
) {
    assertions.insert(
        "edit_procedure_candidate_hook_probe".into(),
        bool_assert(
            edit_procedure_probe.action_id == "edit-procedure-or-code-block"
                && edit_procedure_probe.id == "alice-side-procedure-edit-command-hook"
                && ["passed", "blocked", "failed"].contains(&edit_procedure_probe.status.as_str()),
            edit_procedure_probe.detail.clone(),
        ),
    );
}

fn record_edit_procedure_precondition_no_go(
    assertions: &mut BTreeMap<String, AssertionResult>,
    precondition_probe: &UiActionNoGoProbe,
) {
    assertions.insert(
        "edit_procedure_precondition_no_go_probe".into(),
        bool_assert(
            precondition_probe.action_id == "edit-procedure-or-code-block"
                && precondition_probe.status == "blocked"
                && precondition_probe.decision == "no_go",
            precondition_probe.blocking_reason.clone(),
        ),
    );
}

fn record_run_world_precondition_no_go(
    assertions: &mut BTreeMap<String, AssertionResult>,
    precondition_probe: &UiActionNoGoProbe,
) {
    assertions.insert(
        "run_world_precondition_no_go_probe".into(),
        bool_assert(
            precondition_probe.action_id == "run-world"
                && precondition_probe.status == "blocked"
                && precondition_probe.decision == "no_go",
            precondition_probe.blocking_reason.clone(),
        ),
    );
}

fn record_run_world_probe(
    assertions: &mut BTreeMap<String, AssertionResult>,
    run_world_probe: &UiActionRunWorldProbe,
) {
    assertions.insert(
        "run_world_candidate_hook_probe".into(),
        bool_assert(
            run_world_probe.action_id == "run-world"
                && run_world_probe.id == "alice-side-world-run-command-hook"
                && ["passed", "blocked", "failed"].contains(&run_world_probe.status.as_str()),
            run_world_probe.detail.clone(),
        ),
    );
}

fn record_save_project_precondition_no_go(
    assertions: &mut BTreeMap<String, AssertionResult>,
    precondition_probe: &UiActionNoGoProbe,
) {
    assertions.insert(
        "save_project_precondition_no_go_probe".into(),
        bool_assert(
            precondition_probe.action_id == "save-project"
                && precondition_probe.status == "blocked"
                && precondition_probe.decision == "no_go",
            precondition_probe.blocking_reason.clone(),
        ),
    );
}

fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}

#[cfg(test)]
mod tests;
