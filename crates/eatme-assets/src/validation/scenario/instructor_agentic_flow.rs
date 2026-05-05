use super::{
    honest_boundary::scenario_has_unqualified_automated_grading_claim, require_timeout_and_policy,
    validate_acceptance_criteria, validate_rubric,
};
use crate::schema::EatmeScenarioAsset;
use crate::validation::{require_list, require_nonempty};

pub(super) const LIVE_STUDIO_REQUIRED_EVIDENCE: &[&str] = &[
    "setup checklist",
    "timing plan",
    "observation points",
    "intervention cues",
    "checkpoint artifacts",
    "share-out support",
    "instructor-facing acceptance probes",
    "student prompt cards",
    "student-owned Alice action evidence",
    "add or adjust one visible behavior",
    "run it",
    "record the observed result",
    "revise one small choice",
    "help signals",
    "peer feedback",
    "revision behavior",
    "reflection",
    "share-out artifacts",
    "not full Alice user interface automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
];

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
    validate_instructor_grading_boundary(scenario, errors);
    validate_live_studio_classroom_evidence_contract(scenario, errors);
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

fn validate_live_studio_classroom_evidence_contract(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if scenario.id != "workshop-facilitator-live-studio" {
        return;
    }

    let evidence_text = normalized_scenario_evidence_text(scenario);
    for required in LIVE_STUDIO_REQUIRED_EVIDENCE {
        if !evidence_text.contains(required) {
            errors.push(format!(
                "workshop-facilitator-live-studio must include {required}"
            ));
        }
    }
}

fn normalized_scenario_evidence_text(scenario: &EatmeScenarioAsset) -> String {
    let mut text = String::new();
    push_normalized_part(&mut text, &scenario.purpose);
    push_normalized_part(&mut text, &scenario.agentic_test_prompt);
    push_normalized_part(&mut text, &scenario.unsupported_policy);

    for resource in &scenario.resource_basis {
        push_normalized_part(&mut text, &resource.use_note);
    }
    if let Some(flow) = &scenario.agentic_flow {
        push_normalized_part(&mut text, &flow.focus);
        push_normalized_part(&mut text, &flow.instructor_goal);
        push_normalized_part(&mut text, &flow.prompt_source);
        for item in &flow.non_coder_editable {
            push_normalized_part(&mut text, item);
        }
        for item in &flow.expected_outputs {
            push_normalized_part(&mut text, item);
        }
    }
    for criterion in &scenario.acceptance_criteria {
        push_normalized_part(&mut text, &criterion.given);
        push_normalized_part(&mut text, &criterion.when);
        push_normalized_part(&mut text, &criterion.then);
    }
    for probe in &scenario.acceptance_probes {
        push_normalized_part(&mut text, probe);
    }
    for rubric in &scenario.rubric {
        push_normalized_part(&mut text, &rubric.criterion);
        for evidence in &rubric.evidence {
            push_normalized_part(&mut text, evidence);
        }
    }
    for item in &scenario.avoid {
        push_normalized_part(&mut text, item);
    }
    for step in &scenario.steps {
        push_normalized_part(&mut text, &step.id);
        push_normalized_part(&mut text, &step.command);
        for evidence in &step.evidence {
            push_normalized_part(&mut text, evidence);
        }
    }
    for (name, uri) in &scenario.artifacts {
        push_normalized_part(&mut text, name);
        push_normalized_part(&mut text, uri);
    }

    text
}

fn push_normalized_part(buffer: &mut String, text: &str) {
    for word in text.split_whitespace() {
        if !buffer.is_empty() {
            buffer.push(' ');
        }
        buffer.push_str(word);
    }
}

fn validate_instructor_grading_boundary(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario_has_unqualified_automated_grading_claim(scenario) {
        errors.push(
            "instructor_agentic_flow must not claim automated creative grading or learner-world assessment; keep those as instructor review tasks"
                .into(),
        );
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
