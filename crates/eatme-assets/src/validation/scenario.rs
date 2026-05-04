use super::{
    PersonaDiscovery, PersonaReferenceIndex, contextualize_scenario_errors,
    discover_scenario_personas, portability, require_list, require_nonempty, validate_id,
    validate_reference_list,
};
use crate::report::ScenarioAssetValidationReport;
use crate::schema::{EatmeScenarioAsset, GadugiScenarioAsset, ScenarioPersonas};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn validate_scenario_asset(path: &Path) -> Result<ScenarioAssetValidationReport> {
    let persona_discovery = discover_scenario_personas(path)?;
    validate_scenario_asset_inner(path, persona_discovery)
}

pub(crate) fn validate_scenario_asset_with_personas(
    path: &Path,
    persona_index: &PersonaReferenceIndex,
) -> Result<ScenarioAssetValidationReport> {
    validate_scenario_asset_inner(
        path,
        PersonaDiscovery {
            index: Some(persona_index.clone()),
            diagnostics: Vec::new(),
        },
    )
}

fn validate_scenario_asset_inner(
    path: &Path,
    persona_discovery: PersonaDiscovery,
) -> Result<ScenarioAssetValidationReport> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading scenario asset {}", path.display()))?;
    if path
        .components()
        .any(|component| component.as_os_str().to_string_lossy() == "gadugi")
    {
        let scenario: GadugiScenarioAsset = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing gadugi scenario YAML {}", path.display()))?;
        Ok(validate_gadugi_scenario(path, &scenario))
    } else {
        let scenario: EatmeScenarioAsset = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing eatme scenario YAML {}", path.display()))?;
        Ok(validate_eatme_scenario(
            path,
            &scenario,
            persona_discovery.index.as_ref(),
            &persona_discovery.diagnostics,
        ))
    }
}

fn validate_eatme_scenario(
    path: &Path,
    scenario: &EatmeScenarioAsset,
    persona_index: Option<&PersonaReferenceIndex>,
    persona_diagnostics: &[String],
) -> ScenarioAssetValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    if scenario.schema_version != "eatme.scenario/v1" {
        errors.push(format!(
            "schema_version must be eatme.scenario/v1, got {}",
            scenario.schema_version
        ));
    }
    validate_id(&scenario.id, "scenario", &mut errors);
    require_nonempty(&scenario.title, "title", &mut errors);
    require_nonempty(&scenario.purpose, "purpose", &mut errors);
    validate_eatme_doc_fields(scenario, &mut errors);
    if !scenario.owner.is_empty() && scenario.owner != "eatme" {
        errors.push(format!("owner must be eatme, got {}", scenario.owner));
    }
    if let Some(launcher) = &scenario.launcher {
        require_nonempty(&launcher.command, "launcher.command", &mut errors);
        if launcher.scenario != scenario.id {
            errors.push(format!(
                "launcher.scenario must match id {}, got {}",
                scenario.id, launcher.scenario
            ));
        }
    }
    validate_eatme_steps(scenario, &mut errors);

    let is_known_lesson_smoke = known_lesson_smoke(&scenario.id);
    if is_known_lesson_smoke && scenario.kind != "alice_lesson_smoke" {
        errors.push("kind must be alice_lesson_smoke".into());
    }

    match scenario.kind.as_str() {
        "alice_lesson_smoke" => {
            validate_lesson_smoke(scenario, persona_index, persona_diagnostics, &mut errors)
        }
        "alice_class_portability_smoke" => {
            portability::validate_class_portability_scenario(scenario, &mut errors)
        }
        "instructor_agentic_flow" => validate_instructor_agentic_flow(scenario, &mut errors),
        "" if scenario.id == "real-alice-launch-smoke" => {
            validate_legacy_launch_smoke(scenario, &mut errors)
        }
        "" => errors.push("kind must be defined".into()),
        other => errors.push(format!(
            "kind must be alice_lesson_smoke, alice_class_portability_smoke, or instructor_agentic_flow, got {other}"
        )),
    }
    if is_known_lesson_smoke && scenario.kind != "alice_lesson_smoke" {
        validate_lesson_smoke(scenario, persona_index, persona_diagnostics, &mut errors);
    } else if !matches!(
        scenario.kind.as_str(),
        "alice_lesson_smoke" | "instructor_agentic_flow"
    ) && let Some(personas) = &scenario.personas
    {
        validate_scenario_personas(
            &scenario.id,
            personas,
            persona_index,
            persona_diagnostics,
            &mut errors,
        );
    }
    if portability::is_class_portability_scenario(scenario)
        && scenario.kind != "alice_class_portability_smoke"
    {
        portability::validate_class_portability_scenario(scenario, &mut errors);
    }

    let errors = contextualize_scenario_errors(path, &scenario.id, errors);
    ScenarioAssetValidationReport {
        schema_version: "eatme.assets/scenario-validation/v1".into(),
        asset_path: path.display().to_string(),
        asset_kind: "eatme".into(),
        id: scenario.id.clone(),
        passed: errors.is_empty(),
        step_count: scenario.steps.len(),
        assertion_count: scenario.acceptance_criteria.len(),
        errors,
        warnings,
    }
}

fn known_lesson_smoke(id: &str) -> bool {
    matches!(
        id,
        "hour-of-code-studio-kickoff"
            | "building-a-scene-first-world"
            | "code-editor-first-run"
            | "events-collision-proximity-game"
            | "functions-as-questions-about-the-world"
            | "loops-and-conditionals-mini-challenge"
            | "reusable-methods-and-parameters"
    )
}

fn validate_eatme_doc_fields(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    validate_resource_basis_names(scenario, errors);
    if scenario.kind != "instructor_agentic_flow" {
        match &scenario.capabilities {
            Some(capabilities) => {
                require_list(&capabilities.required, "capabilities.required", errors);
                if capabilities
                    .optional
                    .iter()
                    .any(|value| value.trim().is_empty())
                {
                    errors.push("capabilities.optional must contain non-empty values".into());
                }
            }
            None => errors.push("capabilities.required must be defined".into()),
        }
        match &scenario.adapter {
            Some(adapter) => require_list(&adapter.targets, "adapter.targets", errors),
            None => errors.push("adapter.targets must be defined".into()),
        }
    }
    if let Some(follow_on) = &scenario.agentic_follow_on {
        require_nonempty(
            &follow_on.prompt_source,
            "agentic_follow_on.prompt_source",
            errors,
        );
        if follow_on
            .personality_assets
            .iter()
            .any(|value| value.trim().is_empty())
        {
            errors
                .push("agentic_follow_on.personality_assets must contain non-empty values".into());
        }
        require_nonempty(
            &follow_on.deterministic_gate,
            "agentic_follow_on.deterministic_gate",
            errors,
        );
        if follow_on
            .required_observables
            .iter()
            .any(|value| value.trim().is_empty())
        {
            errors.push(
                "agentic_follow_on.required_observables must contain non-empty values".into(),
            );
        }
    }
}

fn validate_resource_basis_names(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.resource_basis.is_empty() {
        errors.push("resource_basis must contain at least one named resource".into());
    }
    for (index, resource) in scenario.resource_basis.iter().enumerate() {
        require_nonempty(
            &resource.name,
            &format!("resource_basis[{index}].name"),
            errors,
        );
        require_nonempty(
            &resource.url,
            &format!("resource_basis[{index}].url"),
            errors,
        );
    }
}

fn validate_eatme_steps(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.steps.is_empty() && scenario.launcher.is_none() {
        errors.push("scenario must define launcher or steps".into());
    }
    for step in &scenario.steps {
        validate_id(&step.id, "step", errors);
        require_nonempty(&step.command, &format!("{}.command", step.id), errors);
        require_list(&step.evidence, &format!("{}.evidence", step.id), errors);
    }
}

fn validate_lesson_smoke(
    scenario: &EatmeScenarioAsset,
    persona_index: Option<&PersonaReferenceIndex>,
    persona_diagnostics: &[String],
    errors: &mut Vec<String>,
) {
    validate_launch_smoke_contract(scenario, errors);
    if scenario.owner != "eatme" {
        errors.push("owner must be eatme".into());
    }
    match &scenario.real_alice {
        Some(real_alice) if real_alice.gated_by == "EATME_REAL_ALICE=1" => {}
        Some(real_alice) => errors.push(format!(
            "real_alice.gated_by must be EATME_REAL_ALICE=1, got {}",
            real_alice.gated_by
        )),
        None => errors.push("real_alice.gated_by must be EATME_REAL_ALICE=1".into()),
    }
    match &scenario.smoke_ready {
        Some(smoke_ready) => require_list(&smoke_ready.evidence, "smoke_ready.evidence", errors),
        None => errors.push("smoke_ready.evidence must be defined".into()),
    }
    match &scenario.personas {
        Some(personas) => validate_scenario_personas(
            &scenario.id,
            personas,
            persona_index,
            persona_diagnostics,
            errors,
        ),
        None => errors.push("personas.instructors and personas.students must be defined".into()),
    }
    validate_acceptance_criteria(&scenario.acceptance_criteria, errors);
}

fn validate_legacy_launch_smoke(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    validate_launch_smoke_contract(scenario, errors);
}

fn validate_launch_smoke_contract(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    let launch_command = scenario
        .launcher
        .as_ref()
        .map(|launcher| launcher.command.as_str())
        .unwrap_or("");
    let has_launch_smoke = launch_command.contains("alice launch-smoke")
        || scenario
            .steps
            .iter()
            .any(|step| step.command.contains("alice launch-smoke"));
    if !has_launch_smoke {
        errors.push("scenario must route runtime through alice launch-smoke".into());
    }
    for artifact in ["manifest", "screenshot", "log"] {
        if !scenario.artifacts.contains_key(artifact) {
            errors.push(format!("artifacts.{artifact} must be defined"));
        }
    }
    require_timeout_and_policy(scenario, errors);
}

fn validate_instructor_agentic_flow(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
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

fn validate_acceptance_criteria(
    criteria: &[crate::schema::EatmeScenarioAcceptanceCriterion],
    errors: &mut Vec<String>,
) {
    if criteria.is_empty() {
        errors.push("acceptance_criteria must contain at least one criterion".into());
    }
    for (index, criterion) in criteria.iter().enumerate() {
        require_nonempty(
            &criterion.given,
            &format!("acceptance_criteria[{index}].given"),
            errors,
        );
        require_nonempty(
            &criterion.when,
            &format!("acceptance_criteria[{index}].when"),
            errors,
        );
        require_nonempty(
            &criterion.then,
            &format!("acceptance_criteria[{index}].then"),
            errors,
        );
    }
}

fn validate_rubric(
    rubric: &[crate::schema::EatmeScenarioRubricCriterion],
    errors: &mut Vec<String>,
) {
    if rubric.is_empty() {
        errors.push("rubric must contain at least one criterion".into());
    }
    for (index, item) in rubric.iter().enumerate() {
        require_nonempty(
            &item.criterion,
            &format!("rubric[{index}].criterion"),
            errors,
        );
        require_list(&item.evidence, &format!("rubric[{index}].evidence"), errors);
    }
}

fn validate_scenario_personas(
    scenario_id: &str,
    personas: &ScenarioPersonas,
    persona_index: Option<&PersonaReferenceIndex>,
    persona_diagnostics: &[String],
    errors: &mut Vec<String>,
) {
    require_list(
        &personas.instructors,
        &format!("{scenario_id}.personas.instructors"),
        errors,
    );
    require_list(
        &personas.students,
        &format!("{scenario_id}.personas.students"),
        errors,
    );
    if !personas.instructors.is_empty() || !personas.students.is_empty() {
        errors.extend(persona_diagnostics.iter().cloned());
    }
    if let Some(index) = persona_index {
        validate_reference_list(
            scenario_id,
            &personas.instructors,
            &index.instructors,
            &index.all,
            "instructor",
            errors,
        );
        validate_reference_list(
            scenario_id,
            &personas.students,
            &index.students,
            &index.all,
            "student",
            errors,
        );
    } else if !personas.instructors.is_empty() || !personas.students.is_empty() {
        errors.push(format!(
            "scenario {scenario_id} declares personas but no persona crew asset could be located"
        ));
    }
}

fn require_timeout_and_policy(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.timeouts.is_empty() {
        errors.push("timeouts must define at least one timeout".into());
    }
    require_nonempty(&scenario.unsupported_policy, "unsupported_policy", errors);
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

#[path = "gadugi.rs"]
mod gadugi;
use gadugi::validate_gadugi_scenario;
#[cfg(test)]
#[path = "scenario_tests.rs"]
mod scenario_tests;
