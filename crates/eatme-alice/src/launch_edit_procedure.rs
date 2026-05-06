use crate::launch_object_placement::UiActionObjectPlacementProbe;
use crate::launch_ui_actions::{
    UiActionMissingAffordance, UiActionNoGoProbe, UiActionPrecondition,
};

pub(crate) const DEFAULT_PROCEDURE_EDIT_HOOK: &str = "tools/eatme-edit-procedure";

pub(crate) fn probe_edit_procedure_preconditions(
    object_placement_probe: &UiActionObjectPlacementProbe,
) -> UiActionNoGoProbe {
    let object_placement_ready = object_placement_probe.proves_placement();
    let blocking_reason = if object_placement_ready {
        "blocked: missing deterministic-alice-procedure-edit-affordance"
    } else {
        "blocked: object placement proof is required before procedure/code-block editing would be safe"
    };

    UiActionNoGoProbe {
        id: "edit-procedure-precondition".into(),
        action_id: "edit-procedure-or-code-block".into(),
        status: "blocked".into(),
        decision: "no_go".into(),
        blocking_reason: blocking_reason.into(),
        required_evidence:
            "artifact proves a procedure or code block was edited in the project after object placement"
                .into(),
        missing_affordance: missing_procedure_edit_affordance(),
        preconditions: vec![
            UiActionPrecondition {
                id: "place-object".into(),
                passed: object_placement_ready,
                detail:
                    "object-placement hook returned a non-empty placement artifact and scene/project diff"
                        .into(),
            },
            UiActionPrecondition {
                id: "deterministic-alice-procedure-edit-affordance".into(),
                passed: false,
                detail: "missing stable backend command, accessibility target, or editor automation contract for editing a named procedure or code block".into(),
            },
        ],
    }
}

fn missing_procedure_edit_affordance() -> UiActionMissingAffordance {
    UiActionMissingAffordance {
        id: "deterministic-alice-procedure-edit-affordance".into(),
        kind: "backend_or_ui_affordance".into(),
        required_capability: "Given a project after object placement plus a named procedure or code-block selector, deterministically edit that procedure or code block and return proof of the edit.".into(),
        missing_contract: format!("No Alice-side command at {DEFAULT_PROCEDURE_EDIT_HOOK}, accessibility target, or editor automation contract currently accepts a procedure/code-block selector and returns an edited project artifact plus a procedure/code diff."),
        next_implementation: "Add one stable affordance: either an Alice-side procedure edit command hook defined by this contract, or a UI automation contract with a named editor target plus saved-project or AST diff verification.".into(),
    }
}
