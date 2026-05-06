use crate::launch_edit_procedure::UiActionEditProcedureProbe;
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};

pub(crate) const DEFAULT_WORLD_RUN_HOOK: &str = "tools/eatme-run-world";

pub(crate) fn probe_run_world_preconditions(
    edit_procedure_probe: &UiActionEditProcedureProbe,
) -> UiActionNoGoProbe {
    let edit_ready = edit_procedure_probe.proves_edit();
    let blocking_reason = if edit_ready {
        "blocked: missing deterministic-alice-world-run-affordance"
    } else {
        "blocked: procedure/code-block edit proof is required before world run would be safe"
    };

    UiActionNoGoProbe {
        id: "run-world-precondition".into(),
        action_id: "run-world".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence: "artifact proves the world run control or equivalent runtime entry point executed after the first-lesson edit".into(),
        missing_affordance: missing_world_run_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "edit-procedure-or-code-block".into(),
                passed: edit_ready,
                detail: "procedure/code-block edit hook returned a non-empty edited project and procedure/code diff".into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-world-run-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, run control contract, or runtime hook for proving the edited world was run".into(),
            },
        ],
    }
}

fn missing_world_run_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-world-run-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given an edited Alice project, deterministically run the world or equivalent runtime entry point and return proof that execution reached the edited world.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_WORLD_RUN_HOOK}, accessibility target, run control contract, or runtime verification hook currently accepts an edited project and returns world-run proof."),
        next_implementation: "Add one stable affordance: either an Alice-side run-world command hook defined by this contract, or a desktop automation contract with named run control plus runtime/log evidence.".into(),
    }
}
