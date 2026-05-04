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
            Some(smoke_ready) => {
                require_list(&smoke_ready.evidence, "smoke_ready.evidence", &mut errors)
            }
            None => errors.push("smoke_ready.evidence must be defined".into()),
        }
        if scenario.acceptance_criteria.is_empty() {
            errors.push("acceptance_criteria must contain at least one criterion".into());
        }
        for (index, criterion) in scenario.acceptance_criteria.iter().enumerate() {
            require_nonempty(
                &criterion.given,
                &format!("acceptance_criteria[{index}].given"),
                &mut errors,
            );
            require_nonempty(
                &criterion.when,
                &format!("acceptance_criteria[{index}].when"),
                &mut errors,
            );
            require_nonempty(
                &criterion.then,
                &format!("acceptance_criteria[{index}].then"),
                &mut errors,
            );
        }
    }

    if portability::is_class_portability_scenario(scenario) {
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
    for agent in &scenario.agents {
        require_nonempty(&agent.name, "agent.name", &mut errors);
        require_nonempty(
            &agent.agent_type,
            &format!("{}.type", agent.name),
            &mut errors,
        );
        validate_no_hardcoded_repo_path(
            &format!("{}.config.cwd", agent.name),
            &agent.config.cwd,
            &mut errors,
        );
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
            validate_no_hardcoded_repo_path(
                &format!("{}.params.command", step.name),
                command,
                &mut errors,
            );
            validate_gadugi_runtime_boundary(&step.name, command, &mut errors);
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
mod tests;
