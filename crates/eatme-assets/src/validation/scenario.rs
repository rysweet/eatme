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

mod gadugi_scenario;
use self::gadugi_scenario::validate_gadugi_scenario;
mod instructor_agentic_flow;
use self::instructor_agentic_flow::validate_instructor_agentic_flow;
mod real_ui_action_contract;
use self::real_ui_action_contract::validate_real_ui_action_contract;

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
        let scenario: GadugiScenarioAsset = match serde_yaml::from_str(&content) {
            Ok(scenario) => scenario,
            Err(error) => return Ok(parse_failure_report(path, "gadugi", error)),
        };
        Ok(validate_gadugi_scenario(path, &scenario))
    } else {
        let scenario: EatmeScenarioAsset = match serde_yaml::from_str(&content) {
            Ok(scenario) => scenario,
            Err(error) => return Ok(parse_failure_report(path, "eatme", error)),
        };
        Ok(validate_eatme_scenario(
            path,
            &scenario,
            persona_discovery.index.as_ref(),
            &persona_discovery.diagnostics,
        ))
    }
}

fn parse_failure_report(
    path: &Path,
    asset_kind: &str,
    error: serde_yaml::Error,
) -> ScenarioAssetValidationReport {
    ScenarioAssetValidationReport {
        schema_version: "eatme.assets/scenario-validation/v1".into(),
        asset_path: path.display().to_string(),
        asset_kind: asset_kind.into(),
        id: String::new(),
        passed: false,
        step_count: 0,
        assertion_count: 0,
        errors: vec![format!(
            "parsing {asset_kind} scenario YAML {}: {error}",
            path.display()
        )],
        warnings: Vec::new(),
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
        "alice_real_ui_action_contract" => {
            validate_lesson_smoke(scenario, persona_index, persona_diagnostics, &mut errors);
            validate_real_ui_action_contract(scenario, &mut errors);
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
            "kind must be alice_lesson_smoke, alice_real_ui_action_contract, alice_class_portability_smoke, or instructor_agentic_flow, got {other}"
        )),
    }
    if is_known_lesson_smoke && scenario.kind != "alice_lesson_smoke" {
        validate_lesson_smoke(scenario, persona_index, persona_diagnostics, &mut errors);
    } else if !matches!(
        scenario.kind.as_str(),
        "alice_lesson_smoke" | "alice_real_ui_action_contract" | "instructor_agentic_flow"
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
    const KNOWN: &str = "hour-of-code-studio-kickoff building-a-scene-first-world code-editor-first-run events-collision-proximity-game functions-as-questions-about-the-world loops-and-conditionals-mini-challenge reusable-methods-and-parameters";
    KNOWN.split_whitespace().any(|known| known == id)
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
    validate_launch_smoke_real_evidence(scenario, errors);
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

fn validate_launch_smoke_real_evidence(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    let launch_step_mentions_real_evidence = scenario.steps.iter().any(|step| {
        step.command.contains("alice launch-smoke")
            && step
                .evidence
                .iter()
                .any(|evidence| evidence.contains("real_alice_execution_evidence"))
    });
    if !launch_step_mentions_real_evidence {
        errors.push("launch-smoke step evidence must inspect manifest assertions.real_alice_execution_evidence".into());
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

#[cfg(test)]
#[path = "scenario_tests.rs"]
mod scenario_tests;
