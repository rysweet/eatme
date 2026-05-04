use super::{
    contextualize_scenario_errors, portability, require_list, require_nonempty, validate_id,
};
use crate::report::ScenarioAssetValidationReport;
use crate::schema::{EatmeScenarioAsset, GadugiScenarioAsset};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn validate_scenario_asset(path: &Path) -> Result<ScenarioAssetValidationReport> {
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
        Ok(validate_eatme_scenario(path, &scenario))
    }
}

fn validate_eatme_scenario(
    path: &Path,
    scenario: &EatmeScenarioAsset,
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

    let is_known_lesson_smoke = matches!(
        scenario.id.as_str(),
        "hour-of-code-studio-kickoff"
            | "building-a-scene-first-world"
            | "code-editor-first-run"
            | "events-collision-proximity-game"
            | "functions-as-questions-about-the-world"
            | "loops-and-conditionals-mini-challenge"
            | "reusable-methods-and-parameters"
    );
    if is_known_lesson_smoke && scenario.kind != "alice_lesson_smoke" {
        errors.push("kind must be alice_lesson_smoke".into());
    }

    match scenario.kind.as_str() {
        "alice_lesson_smoke" => validate_lesson_smoke(scenario, &mut errors),
        "alice_class_portability_smoke" => {
            portability::validate_class_portability_scenario(scenario, &mut errors);
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

fn validate_lesson_smoke(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
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
        errors.push(
            "launch-smoke step evidence must inspect manifest assertions.real_alice_execution_evidence"
                .into(),
        );
    }
}

fn validate_instructor_agentic_flow(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.launcher.is_some() || scenario.real_alice.is_some() {
        errors.push(
            "instructor_agentic_flow must use agentic steps, not a real-Alice launcher".into(),
        );
    }
    validate_resource_basis(scenario, errors);
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
    let has_agentic_step = scenario
        .steps
        .iter()
        .any(|step| step.command.contains("agentic"));
    if !has_agentic_step {
        errors.push("instructor_agentic_flow steps must include an agentic evaluation step".into());
    }
    for step in &scenario.steps {
        validate_instructor_flow_boundary(&step.id, &step.command, errors);
    }
}

fn require_timeout_and_policy(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.timeouts.is_empty() {
        errors.push("timeouts must define at least one timeout".into());
    }
    require_nonempty(&scenario.unsupported_policy, "unsupported_policy", errors);
}

fn validate_resource_basis(scenario: &EatmeScenarioAsset, errors: &mut Vec<String>) {
    if scenario.resource_basis.is_empty() {
        errors.push("resource_basis must cite existing Alice.org resources".into());
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

fn validate_gadugi_scenario(
    path: &Path,
    scenario: &GadugiScenarioAsset,
) -> ScenarioAssetValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    require_nonempty(&scenario.name, "name", &mut errors);
    require_nonempty(&scenario.description, "description", &mut errors);
    require_nonempty(&scenario.version, "version", &mut errors);
    if scenario.steps.is_empty() {
        errors.push("steps must contain at least one step".into());
    }
    for step in &scenario.steps {
        require_nonempty(&step.name, "step.name", &mut errors);
        require_nonempty(&step.agent, &format!("{}.agent", step.name), &mut errors);
        require_nonempty(&step.action, &format!("{}.action", step.name), &mut errors);
        if step.action == "execute_command" {
            match step.params.get("command") {
                Some(command) => require_nonempty(
                    command,
                    &format!("{}.params.command", step.name),
                    &mut errors,
                ),
                None => errors.push(format!("{}.params.command must be defined", step.name)),
            }
        }
        if let Some(command) = step.params.get("command") {
            validate_gadugi_runtime_boundary(&step.name, command, &mut errors);
            if command.contains("alice launch-smoke") {
                validate_gadugi_real_evidence_expectation(&step.name, step, &mut errors);
            }
        }
        if step.action == "agentic_test" {
            validate_gadugi_agentic_step(&step.name, &step.params, &mut errors);
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

fn validate_gadugi_agentic_step(
    step_name: &str,
    params: &std::collections::BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    for key in ["asset", "prompt", "acceptance_probes"] {
        match params.get(key) {
            Some(value) => require_nonempty(value, &format!("{step_name}.params.{key}"), errors),
            None => errors.push(format!("{step_name}.params.{key} must be defined")),
        }
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

fn validate_gadugi_real_evidence_expectation(
    step_name: &str,
    step: &crate::schema::GadugiScenarioStep,
    errors: &mut Vec<String>,
) {
    if !step
        .expect
        .stdout_contains
        .iter()
        .any(|expected| expected.contains("real_alice_execution_evidence"))
    {
        errors.push(format!(
            "{step_name}: gadugi launch-smoke step must assert manifest real_alice_execution_evidence"
        ));
    }
}

#[cfg(test)]
#[path = "scenario_tests.rs"]
mod scenario_tests;
