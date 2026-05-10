use super::{
    MissingAffordanceContract, NoGoPrecondition, NoGoProbeContract, candidate_affordance_probes,
    has_no_go_probe, non_empty_artifact, string_field,
};
use crate::launch_save_project::DEFAULT_SAVE_SELECTOR;

const SAVE_PROJECT_NO_GO: NoGoProbeContract = NoGoProbeContract {
    probe_id: "project-save-precondition",
    action_id: "save-project",
    missing_affordance: MissingAffordanceContract {
        id: "deterministic-alice-project-save-affordance",
        required_capability_contains: &["save the project", "saved .a3p is readable"],
        missing_contract_contains: &["No Alice-side command", "returns project-save proof"],
        next_implementation_contains: &["save-project command", "named save control"],
    },
    preconditions: &[
        NoGoPrecondition {
            id: "run-world",
            passed: true,
        },
        NoGoPrecondition {
            id: "deterministic-alice-project-save-affordance",
            passed: false,
        },
    ],
};

pub(super) fn has_save_project_no_go_probe(contract: &serde_json::Value) -> bool {
    has_no_go_probe(contract, &SAVE_PROJECT_NO_GO)
}

pub(super) fn has_passed_save_project_candidate_affordance_probe(
    contract: &serde_json::Value,
) -> bool {
    candidate_affordance_probes(contract).any(|probe| {
        string_field(probe, "id") == Some("alice-side-project-save-command-hook")
            && string_field(probe, "action_id") == Some("save-project")
            && string_field(probe, "status") == Some("passed")
            && string_field(probe, "save_selector") == Some(DEFAULT_SAVE_SELECTOR)
            && string_field(probe, "candidate_hook_path")
                .is_some_and(|value| value.ends_with("tools/eatme-save-project"))
            && probe
                .get("saved_project_artifact")
                .is_some_and(non_empty_artifact)
            && probe.get("save_artifact").is_some_and(non_empty_artifact)
            && probe
                .get("validation_errors")
                .and_then(serde_json::Value::as_array)
                .is_some_and(Vec::is_empty)
    })
}
