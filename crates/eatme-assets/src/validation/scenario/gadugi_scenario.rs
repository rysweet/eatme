use super::{contextualize_scenario_errors, require_list, require_nonempty};
use crate::report::ScenarioAssetValidationReport;
use crate::schema::GadugiScenarioAsset;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn validate_gadugi_scenario(
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
            match string_param(&step.params, "command") {
                Some(command) => require_nonempty(
                    command,
                    &format!("{}.params.command", step.name),
                    &mut errors,
                ),
                None => errors.push(format!("{}.params.command must be a string", step.name)),
            }
            validate_execute_expect(&step.name, step.expect.as_ref(), &mut errors);
        }
        if let Some(command) = string_param(&step.params, "command") {
            validate_no_hardcoded_repo_path(
                &format!("{}.params.command", step.name),
                command,
                &mut errors,
            );
            validate_gadugi_runtime_boundary(&step.name, command, &mut errors);
            if command.contains("alice launch-smoke") {
                validate_gadugi_real_evidence_expectation(
                    &step.name,
                    step.expect.as_ref(),
                    &mut errors,
                );
            }
        }
        if step.action == "agentic_test" {
            validate_gadugi_agentic_step(&step.name, &step.params, &mut errors);
            validate_agentic_expect(&step.name, step.expect.as_ref(), &mut errors);
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
    validate_gadugi_source_reference(path, scenario, &mut errors);

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
    if let Some(config) = &scenario.config {
        if config.timeout == 0 {
            errors.push("config.timeout must be greater than zero".into());
        }
        if config.retries != 0 {
            errors.push("config.retries must be 0 for deterministic validation".into());
        }
        if config.parallel {
            errors.push("config.parallel must be false for deterministic validation".into());
        }
    }
    if let Some(environment) = &scenario.environment {
        if environment
            .requires
            .iter()
            .any(|value| value.trim().is_empty())
        {
            errors.push("environment.requires must contain non-empty values".into());
        }
        if environment
            .optional
            .iter()
            .any(|value| value.trim().is_empty())
        {
            errors.push("environment.optional must contain non-empty values".into());
        }
    }
    if scenario.agents.is_empty() {
        errors.push("agents must contain at least one agent".into());
    }
    for agent in &scenario.agents {
        require_nonempty(&agent.name, "agent.name", errors);
        require_nonempty(&agent.agent_type, &format!("{}.type", agent.name), errors);
        validate_no_hardcoded_repo_path(
            &format!("{}.config.cwd", agent.name),
            &agent.config.cwd,
            errors,
        );
        if agent.config.timeout == 0 {
            errors.push(format!(
                "{}.config.timeout must be greater than zero",
                agent.name
            ));
        }
        match agent.agent_type.as_str() {
            "system" => {
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
                if !agent.config.capture_output {
                    errors.push(format!("{}.config.capture_output must be true", agent.name));
                }
            }
            "agentic" => {
                require_nonempty(
                    &agent.config.persona_asset,
                    &format!("{}.config.persona_asset", agent.name),
                    errors,
                );
                require_nonempty(
                    &agent.config.scenario_asset,
                    &format!("{}.config.scenario_asset", agent.name),
                    errors,
                );
            }
            _ => {}
        }
    }
    require_list(&scenario.metadata.tags, "metadata.tags", errors);
    require_nonempty(&scenario.metadata.priority, "metadata.priority", errors);
    require_nonempty(&scenario.metadata.author, "metadata.author", errors);
    require_nonempty(&scenario.metadata.test_type, "metadata.test_type", errors);
}

fn validate_execute_expect(
    step_name: &str,
    expect: Option<&crate::schema::GadugiStepExpect>,
    errors: &mut Vec<String>,
) {
    match expect {
        Some(expect) => {
            if expect.exit_code.is_none() {
                errors.push(format!("{step_name}.expect.exit_code must be defined"));
            }
            if expect
                .stdout_contains
                .iter()
                .any(|value| value.trim().is_empty())
            {
                errors.push(format!(
                    "{step_name}.expect.stdout_contains must contain non-empty values"
                ));
            }
        }
        None => errors.push(format!("{step_name}.expect must be defined")),
    }
}

fn validate_agentic_expect(
    step_name: &str,
    expect: Option<&crate::schema::GadugiStepExpect>,
    errors: &mut Vec<String>,
) {
    match expect {
        Some(expect) => require_list(
            &expect.output_contains,
            &format!("{step_name}.expect.output_contains"),
            errors,
        ),
        None => errors.push(format!("{step_name}.expect must be defined")),
    }
}

fn validate_no_hardcoded_repo_path(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.contains("/home/") {
        errors.push(format!(
            "{field} must not hard-code an absolute home directory path; use relative paths or EATME_REPO"
        ));
    }
}

fn validate_gadugi_source_reference(
    path: &Path,
    scenario: &GadugiScenarioAsset,
    errors: &mut Vec<String>,
) {
    let source = scenario.metadata.source_eatme_asset.trim();
    require_nonempty(
        &scenario.metadata.generated_by,
        "metadata.generated_by",
        errors,
    );
    if source.is_empty() {
        errors.push(
            "metadata.source_eatme_asset must reference the canonical eatme scenario asset".into(),
        );
        return;
    }
    let source_path = Path::new(source);
    if source_path.is_absolute() || source.contains("..") {
        errors.push("metadata.source_eatme_asset must be a repo-relative path".into());
        return;
    }
    if !source.starts_with("assets/scenarios/eatme/") {
        errors.push("metadata.source_eatme_asset must point at assets/scenarios/eatme".into());
        return;
    }
    let Some(root) = repo_root_from_asset_path(path) else {
        return;
    };
    if !root.join(source_path).is_file() {
        errors.push(format!(
            "metadata.source_eatme_asset references missing asset {source}"
        ));
    }
}

fn repo_root_from_asset_path(path: &Path) -> Option<&Path> {
    for ancestor in path.ancestors() {
        if ancestor
            .file_name()
            .map(|name| name == "assets")
            .unwrap_or(false)
        {
            return ancestor.parent();
        }
    }
    None
}

fn validate_gadugi_real_evidence_expectation(
    step_name: &str,
    expect: Option<&crate::schema::GadugiStepExpect>,
    errors: &mut Vec<String>,
) {
    let checks_real_evidence = expect
        .map(|expect| {
            expect
                .stdout_contains
                .iter()
                .any(|expected| expected.contains("real_alice_execution_evidence"))
        })
        .unwrap_or(false);
    if !checks_real_evidence {
        errors.push(format!(
            "{step_name}: gadugi launch-smoke step must assert manifest real_alice_execution_evidence"
        ));
    }
}

fn validate_gadugi_agentic_step(
    step_name: &str,
    params: &BTreeMap<String, Value>,
    errors: &mut Vec<String>,
) {
    for key in ["asset", "prompt", "acceptance_probes"] {
        match string_param(params, key) {
            Some(value) => require_nonempty(value, &format!("{step_name}.params.{key}"), errors),
            None => errors.push(format!("{step_name}.params.{key} must be defined")),
        }
    }
}

fn string_param<'a>(params: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str)
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
