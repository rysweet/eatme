use super::{require_timeout_and_policy, validate_acceptance_criteria, validate_rubric};
use crate::schema::EatmeScenarioAsset;
use crate::validation::{require_list, require_nonempty};

pub(super) fn validate_instructor_agentic_flow(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if scenario.launcher.is_some() || scenario.real_alice.is_some() {
        errors.push(
            "instructor_agentic_flow must use agentic steps, not a real-Alice launcher".into(),
        );
    }
    validate_instructor_resource_basis(scenario, errors);
    validate_agentic_personas(scenario, errors);
    match &scenario.agentic_flow {
        Some(flow) => {
            require_nonempty(&flow.focus, "agentic_flow.focus", errors);
            require_nonempty(
                &flow.instructor_goal,
                "agentic_flow.instructor_goal",
                errors,
            );
            require_nonempty(&flow.prompt_source, "agentic_flow.prompt_source", errors);
            require_list(
                &flow.non_coder_editable,
                "agentic_flow.non_coder_editable",
                errors,
            );
            require_list(
                &flow.expected_outputs,
                "agentic_flow.expected_outputs",
                errors,
            );
        }
        None => errors.push("agentic_flow must be defined".into()),
    }
    require_nonempty(&scenario.agentic_test_prompt, "agentic_test_prompt", errors);
    require_list(&scenario.acceptance_probes, "acceptance_probes", errors);
    require_list(&scenario.avoid, "avoid", errors);
    validate_acceptance_criteria(&scenario.acceptance_criteria, errors);
    validate_rubric(&scenario.rubric, errors);
    if scenario.artifacts.is_empty() {
        errors.push("artifacts must name the instructor-maintainable outputs".into());
    }
    require_timeout_and_policy(scenario, errors);
    if !scenario
        .steps
        .iter()
        .any(|step| step.command.contains("agentic"))
    {
        errors.push("instructor_agentic_flow steps must include an agentic evaluation step".into());
    }
    for step in &scenario.steps {
        validate_instructor_flow_boundary(&step.id, &step.command, errors);
    }
}

fn validate_instructor_resource_basis(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    for (index, resource) in scenario.resource_basis.iter().enumerate() {
        require_nonempty(
            &resource.use_note,
            &format!("resource_basis[{index}].use"),
            errors,
        );
    }
}

fn validate_agentic_personas(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    match &scenario.personas {
        Some(personas) => {
            require_list(&personas.instructors, "personas.instructors", errors);
            require_list(&personas.students, "personas.students", errors);
        }
        None => errors.push("personas must define instructor and student viewpoints".into()),
    }
}

fn validate_instructor_flow_boundary(step_id: &str, command: &str, errors: &mut Vec<String>) {
    let direct_runtime_markers = [
        "Xvfb",
        "org.alice.stageide",
        "java org.alice",
        "wmctrl",
        "xwd",
    ];
    if command.contains("alice launch-smoke")
        || direct_runtime_markers
            .iter()
            .any(|marker| command.contains(marker))
    {
        errors.push(format!(
            "{step_id}: instructor_agentic_flow must stay at the editable asset/agentic evidence boundary, not own Alice desktop runtime"
        ));
    }
}
