use super::{
    PersonaReferenceIndex, contextualize_scenario_errors, require_list, require_nonempty,
    validate_id,
};
use crate::report::ScenarioAssetValidationReport;
use crate::schema::{EatmeScenarioAsset, GadugiScenarioAsset, ScenarioPersonas};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn validate_scenario_asset(path: &Path) -> Result<ScenarioAssetValidationReport> {
    let persona_index = scenario_persona_index(path)?;
    validate_scenario_asset_inner(path, persona_index.as_ref())
}

pub(crate) fn validate_scenario_asset_with_personas(
    path: &Path,
    persona_index: &PersonaReferenceIndex,
) -> Result<ScenarioAssetValidationReport> {
    validate_scenario_asset_inner(path, Some(persona_index))
}

fn validate_scenario_asset_inner(
    path: &Path,
    persona_index: Option<&PersonaReferenceIndex>,
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
        Ok(validate_eatme_scenario(path, &scenario, persona_index))
    }
}

fn scenario_persona_index(path: &Path) -> Result<Option<PersonaReferenceIndex>> {
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) != Some("scenarios") {
            continue;
        }
        let Some(assets_dir) = ancestor.parent() else {
            continue;
        };
        if assets_dir.file_name().and_then(|name| name.to_str()) != Some("assets") {
            continue;
        }
        let persona_path = assets_dir.join("personas/alice-user-crew.yaml");
        if persona_path.is_file() {
            return super::persona_reference_index(&persona_path).map(Some);
        }
    }
    Ok(None)
}

fn validate_eatme_scenario(
    path: &Path,
    scenario: &EatmeScenarioAsset,
    persona_index: Option<&PersonaReferenceIndex>,
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

    if scenario.steps.is_empty() && scenario.launcher.is_none() {
        errors.push("scenario must define launcher or steps".into());
    }
    for step in &scenario.steps {
        validate_id(&step.id, "step", &mut errors);
        require_nonempty(&step.command, &format!("{}.command", step.id), &mut errors);
        require_list(
            &step.evidence,
            &format!("{}.evidence", step.id),
            &mut errors,
        );
    }

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
    if scenario.timeouts.is_empty() {
        errors.push("timeouts must define at least one timeout".into());
    }
    require_nonempty(
        &scenario.unsupported_policy,
        "unsupported_policy",
        &mut errors,
    );

    let is_known_lesson_smoke = matches!(
        scenario.id.as_str(),
        "building-a-scene-first-world" | "code-editor-first-run"
    );
    if is_known_lesson_smoke && scenario.kind != "alice_lesson_smoke" {
        errors.push("kind must be alice_lesson_smoke".into());
    }

    if scenario.kind == "alice_lesson_smoke" || is_known_lesson_smoke {
        validate_lesson_smoke_fields(scenario, persona_index, &mut errors);
    } else if let Some(personas) = &scenario.personas {
        validate_scenario_personas(&scenario.id, personas, persona_index, &mut errors);
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

fn validate_eatme_doc_fields(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
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
    }
}

fn validate_lesson_smoke_fields(
    scenario: &EatmeScenarioAsset,
    persona_index: Option<&PersonaReferenceIndex>,
    errors: &mut Vec<String>,
) {
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
        Some(personas) => validate_scenario_personas(&scenario.id, personas, persona_index, errors),
        None => errors.push("personas.instructors and personas.students must be defined".into()),
    }
    if scenario.acceptance_criteria.is_empty() {
        errors.push("acceptance_criteria must contain at least one criterion".into());
    }
    for (index, criterion) in scenario.acceptance_criteria.iter().enumerate() {
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

fn validate_scenario_personas(
    scenario_id: &str,
    personas: &ScenarioPersonas,
    persona_index: Option<&PersonaReferenceIndex>,
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
    }
}

fn validate_reference_list(
    scenario_id: &str,
    refs: &[String],
    expected_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    role: &str,
    errors: &mut Vec<String>,
) {
    for id in refs {
        if !expected_ids.contains(id) {
            let suffix = if all_ids.contains(id) {
                " with wrong role"
            } else {
                ""
            };
            errors.push(format!(
                "scenario {scenario_id} references missing {role} persona {id}{suffix}"
            ));
        }
    }
}

fn validate_gadugi_scenario(
    path: &Path,
    scenario: &GadugiScenarioAsset,
) -> ScenarioAssetValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    require_nonempty(&scenario.name, "name", &mut errors);
    require_nonempty(&scenario.description, "description", &mut errors);
    require_nonempty(&scenario.version, "version", &mut errors);
    validate_gadugi_doc_fields(scenario, &mut errors);
    if scenario.steps.is_empty() {
        errors.push("steps must contain at least one step".into());
    }
    for step in &scenario.steps {
        require_nonempty(&step.name, "step.name", &mut errors);
        require_nonempty(&step.agent, &format!("{}.agent", step.name), &mut errors);
        require_nonempty(&step.action, &format!("{}.action", step.name), &mut errors);
        if step.action == "execute_command" {
            match step.params.get("command").and_then(|value| value.as_str()) {
                Some(command) => require_nonempty(
                    command,
                    &format!("{}.params.command", step.name),
                    &mut errors,
                ),
                None => errors.push(format!("{}.params.command must be a string", step.name)),
            }
        }
        if let Some(command) = step.params.get("command").and_then(|value| value.as_str()) {
            validate_gadugi_runtime_boundary(&step.name, command, &mut errors);
        }
        match &step.expect {
            Some(expect) => {
                if expect.exit_code.is_none() {
                    errors.push(format!("{}.expect.exit_code must be defined", step.name));
                }
                require_list(
                    &expect.stdout_contains,
                    &format!("{}.expect.stdout_contains", step.name),
                    &mut errors,
                );
            }
            None => errors.push(format!("{}.expect must be defined", step.name)),
        }
        if step.timeout == 0 {
            errors.push(format!("{}.timeout must be greater than zero", step.name));
        }
    }
    if scenario.assertions.is_empty() {
        errors.push("assertions must contain at least one assertion".into());
    }
    for assertion in &scenario.assertions {
        require_nonempty(&assertion.name, "assertion.name", &mut errors);
        require_nonempty(
            &assertion.assertion_type,
            &format!("{}.type", assertion.name),
            &mut errors,
        );
        require_nonempty(
            &assertion.agent,
            &format!("{}.agent", assertion.name),
            &mut errors,
        );
        if assertion.params.is_empty() {
            errors.push(format!("{}.params must be defined", assertion.name));
        }
    }

    let errors = contextualize_scenario_errors(path, &scenario.name, errors);

    ScenarioAssetValidationReport {
        schema_version: "eatme.assets/scenario-validation/v1".into(),
        asset_path: path.display().to_string(),
        asset_kind: "gadugi".into(),
        id: scenario.name.clone(),
        passed: errors.is_empty(),
        step_count: scenario.steps.len(),
        assertion_count: scenario.assertions.len(),
        errors,
        warnings,
    }
}

fn validate_gadugi_doc_fields(scenario: &GadugiScenarioAsset, errors: &mut Vec<String>) {
    match &scenario.config {
        Some(config) if config.timeout > 0 => {
            if config.retries != 0 {
                errors.push("config.retries must be 0 for deterministic validation".into());
            }
            if config.parallel {
                errors.push("config.parallel must be false for deterministic validation".into());
            }
        }
        Some(_) => errors.push("config.timeout must be greater than zero".into()),
        None => errors.push("config must be defined".into()),
    }
    match &scenario.environment {
        Some(environment) => {
            require_list(&environment.requires, "environment.requires", errors);
            if environment
                .optional
                .iter()
                .any(|value| value.trim().is_empty())
            {
                errors.push("environment.optional must contain non-empty values".into());
            }
        }
        None => errors.push("environment.requires must be defined".into()),
    }
    if scenario.agents.is_empty() {
        errors.push("agents must contain at least one agent".into());
    }
    for agent in &scenario.agents {
        require_nonempty(&agent.name, "agent.name", errors);
        require_nonempty(&agent.agent_type, &format!("{}.type", agent.name), errors);
        require_nonempty(
            &agent.config.shell,
            &format!("{}.config.shell", agent.name),
            errors,
        );
        require_nonempty(
            &agent.config.cwd,
            &format!("{}.config.cwd", agent.name),
            errors,
        );
        if agent.config.timeout == 0 {
            errors.push(format!(
                "{}.config.timeout must be greater than zero",
                agent.name
            ));
        }
        if !agent.config.capture_output {
            errors.push(format!("{}.config.capture_output must be true", agent.name));
        }
    }
    match &scenario.metadata {
        Some(metadata) => {
            require_list(&metadata.tags, "metadata.tags", errors);
            require_nonempty(&metadata.priority, "metadata.priority", errors);
            require_nonempty(&metadata.author, "metadata.author", errors);
            require_nonempty(&metadata.test_type, "metadata.test_type", errors);
        }
        None => errors.push("metadata must be defined".into()),
    }
}

fn validate_gadugi_runtime_boundary(step_name: &str, command: &str, errors: &mut Vec<String>) {
    let direct_runtime_markers = [
        "Xvfb",
        "org.alice.stageide",
        "java org.alice",
        "java -",
        "scrot",
        "import -window",
        "wmctrl",
        "xwd",
    ];
    if command.contains("alice launch-smoke") {
        return;
    }
    if direct_runtime_markers
        .iter()
        .any(|marker| command.contains(marker))
    {
        errors.push(format!(
            "{step_name}: gadugi scenario must invoke eatme CLI alice launch-smoke and inspect manifest evidence only; it must not own Alice runtime concerns"
        ));
    }
}

#[cfg(test)]
#[path = "scenario_tests.rs"]
mod scenario_tests;
