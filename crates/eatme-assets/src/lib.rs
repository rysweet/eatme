use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize)]
pub struct AssetValidationReport {
    pub schema_version: String,
    pub asset_path: String,
    pub passed: bool,
    pub instructor_count: usize,
    pub student_count: usize,
    pub core_scenario_count: usize,
    pub creative_scenario_count: usize,
    pub scenario_asset_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioAssetValidationReport {
    pub schema_version: String,
    pub asset_path: String,
    pub asset_kind: String,
    pub id: String,
    pub passed: bool,
    pub step_count: usize,
    pub assertion_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate_assets(root: &Path) -> Result<AssetValidationReport> {
    let persona_path = root.join("assets/personas/alice-user-crew.yaml");
    let mut report = validate_persona_crew(&persona_path)?;
    report.schema_version = "eatme.assets/validation/v1".into();
    report.asset_path = root.display().to_string();

    for scenario_path in scenario_asset_paths(&root.join("assets/scenarios"))? {
        let scenario_report = validate_scenario_asset(&scenario_path)?;
        report.scenario_asset_count += 1;
        report.errors.extend(
            scenario_report
                .errors
                .into_iter()
                .map(|error| format!("{}: {error}", scenario_path.display())),
        );
        report.warnings.extend(
            scenario_report
                .warnings
                .into_iter()
                .map(|warning| format!("{}: {warning}", scenario_path.display())),
        );
    }

    report.passed = report.errors.is_empty();
    Ok(report)
}

pub fn validate_persona_crew(path: &Path) -> Result<AssetValidationReport> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading persona crew asset {}", path.display()))?;
    let crew: CrewAsset = serde_yaml::from_str(&content)
        .with_context(|| format!("parsing persona crew YAML {}", path.display()))?;
    Ok(validate_crew(path, &crew))
}

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

fn validate_crew(path: &Path, crew: &CrewAsset) -> AssetValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    require_nonempty(&crew.workstream, "workstream", &mut errors);
    require_nonempty(&crew.title, "title", &mut errors);
    require_nonempty(&crew.purpose, "purpose", &mut errors);

    let mut instructor_ids = BTreeSet::new();
    let mut student_ids = BTreeSet::new();
    validate_personas(
        &crew.personas.instructors,
        "instructor",
        &mut instructor_ids,
        &mut errors,
    );
    validate_personas(
        &crew.personas.students,
        "student",
        &mut student_ids,
        &mut errors,
    );
    let all_ids = instructor_ids
        .iter()
        .chain(student_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    if crew.core_scenarios_from_existing_alice_resources.len() < 5 {
        errors
            .push("core scenarios must contain at least five Alice.org-grounded scenarios".into());
    }
    if crew.creative_new_teaching_learning_scenarios.len() < 10 {
        errors.push("creative scenarios must contain at least ten scenarios".into());
    }

    let mut scenario_ids = BTreeSet::new();
    validate_scenarios(
        &crew.core_scenarios_from_existing_alice_resources,
        "existing-alice-resource",
        &instructor_ids,
        &student_ids,
        &all_ids,
        &mut scenario_ids,
        &mut errors,
    );
    validate_scenarios(
        &crew.creative_new_teaching_learning_scenarios,
        "creative-new",
        &instructor_ids,
        &student_ids,
        &all_ids,
        &mut scenario_ids,
        &mut errors,
    );

    AssetValidationReport {
        schema_version: "eatme.assets/persona-crew-validation/v1".into(),
        asset_path: path.display().to_string(),
        passed: errors.is_empty(),
        instructor_count: crew.personas.instructors.len(),
        student_count: crew.personas.students.len(),
        core_scenario_count: crew.core_scenarios_from_existing_alice_resources.len(),
        creative_scenario_count: crew.creative_new_teaching_learning_scenarios.len(),
        scenario_asset_count: 0,
        errors,
        warnings,
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

fn validate_personas(
    personas: &[Persona],
    expected_role: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for persona in personas {
        validate_id(&persona.id, "persona", errors);
        if !ids.insert(persona.id.clone()) {
            errors.push(format!("duplicate persona id {}", persona.id));
        }
        if persona.role != expected_role {
            errors.push(format!(
                "persona {} has role {}, expected {expected_role}",
                persona.id, persona.role
            ));
        }
        require_nonempty(
            &persona.archetype,
            &format!("{}.archetype", persona.id),
            errors,
        );
        require_list(&persona.goals, &format!("{}.goals", persona.id), errors);
        require_list(
            &persona.constraints,
            &format!("{}.constraints", persona.id),
            errors,
        );
        require_list(
            &persona.educational_intent,
            &format!("{}.educational_intent", persona.id),
            errors,
        );
        require_list(
            &persona.observable_behaviors,
            &format!("{}.observable_behaviors", persona.id),
            errors,
        );
        require_list(
            &persona.anti_behaviors,
            &format!("{}.anti_behaviors", persona.id),
            errors,
        );
        require_list(
            &persona.evidence,
            &format!("{}.evidence", persona.id),
            errors,
        );
    }
}

fn validate_scenarios(
    scenarios: &[Scenario],
    expected_origin: &str,
    instructor_ids: &BTreeSet<String>,
    student_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    scenario_ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for scenario in scenarios {
        validate_id(&scenario.id, "scenario", errors);
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("duplicate scenario id {}", scenario.id));
        }
        if scenario.origin != expected_origin {
            errors.push(format!(
                "scenario {} has origin {}, expected {expected_origin}",
                scenario.id, scenario.origin
            ));
        }
        require_list(
            &scenario.coverage,
            &format!("{}.coverage", scenario.id),
            errors,
        );
        require_nonempty(
            &scenario.user_story,
            &format!("{}.user_story", scenario.id),
            errors,
        );
        if !scenario.user_story.starts_with("As a ") && !scenario.user_story.starts_with("As an ") {
            errors.push(format!(
                "scenario {} user_story must start with 'As a ' or 'As an '",
                scenario.id
            ));
        }
        require_list(
            &scenario.educational_intent.concepts,
            &format!("{}.educational_intent.concepts", scenario.id),
            errors,
        );
        require_list(
            &scenario.educational_intent.habits,
            &format!("{}.educational_intent.habits", scenario.id),
            errors,
        );
        require_list(
            &scenario.constraints,
            &format!("{}.constraints", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.instructor,
            &format!("{}.observable_behaviors.instructor", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.student,
            &format!("{}.observable_behaviors.student", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.system_or_artifact,
            &format!("{}.observable_behaviors.system_or_artifact", scenario.id),
            errors,
        );
        require_nonempty(
            &scenario.agentic_test_prompt,
            &format!("{}.agentic_test_prompt", scenario.id),
            errors,
        );
        require_list(
            &scenario.acceptance_probes,
            &format!("{}.acceptance_probes", scenario.id),
            errors,
        );
        require_list(&scenario.avoid, &format!("{}.avoid", scenario.id), errors);
        validate_references(
            &scenario.id,
            &scenario.personas.instructors,
            instructor_ids,
            all_ids,
            "instructor",
            errors,
        );
        validate_references(
            &scenario.id,
            &scenario.personas.students,
            student_ids,
            all_ids,
            "student",
            errors,
        );
    }
}

fn validate_references(
    scenario_id: &str,
    refs: &[String],
    expected_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    role: &str,
    errors: &mut Vec<String>,
) {
    require_list(refs, &format!("{scenario_id}.personas.{role}s"), errors);
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

fn validate_id(id: &str, kind: &str, errors: &mut Vec<String>) {
    if id.is_empty()
        || id.starts_with('-')
        || id.ends_with('-')
        || !id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        errors.push(format!("{kind} id {id:?} must be kebab-case"));
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

fn contextualize_scenario_errors(
    path: &Path,
    scenario_id: &str,
    errors: Vec<String>,
) -> Vec<String> {
    if errors.is_empty() {
        return errors;
    }
    let path_display = path.display().to_string();
    let context = if scenario_id.trim().is_empty() {
        path_display.clone()
    } else {
        format!("{path_display} ({scenario_id})")
    };
    errors
        .into_iter()
        .map(|error| {
            if (!scenario_id.trim().is_empty() && error.contains(scenario_id))
                || error.contains(&path_display)
            {
                error
            } else {
                format!("{context}: {error}")
            }
        })
        .collect()
}

fn require_nonempty(value: &str, field: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
    }
}

fn require_list(values: &[String], field: &str, errors: &mut Vec<String>) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        errors.push(format!("{field} must contain non-empty values"));
    }
}

fn scenario_asset_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_yaml_paths(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_yaml_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_yaml_paths(&path, paths)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension == "yaml" || extension == "yml")
            .unwrap_or(false)
        {
            paths.push(path);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
struct CrewAsset {
    workstream: String,
    title: String,
    purpose: String,
    personas: PersonaGroups,
    core_scenarios_from_existing_alice_resources: Vec<Scenario>,
    creative_new_teaching_learning_scenarios: Vec<Scenario>,
}

#[derive(Clone, Debug, Deserialize)]
struct PersonaGroups {
    instructors: Vec<Persona>,
    students: Vec<Persona>,
}

#[derive(Clone, Debug, Deserialize)]
struct Persona {
    id: String,
    role: String,
    archetype: String,
    goals: Vec<String>,
    constraints: Vec<String>,
    educational_intent: Vec<String>,
    observable_behaviors: Vec<String>,
    anti_behaviors: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Scenario {
    id: String,
    origin: String,
    coverage: Vec<String>,
    user_story: String,
    personas: ScenarioPersonas,
    educational_intent: ScenarioIntent,
    constraints: Vec<String>,
    observable_behaviors: ScenarioObservables,
    agentic_test_prompt: String,
    acceptance_probes: Vec<String>,
    avoid: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ScenarioPersonas {
    instructors: Vec<String>,
    students: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ScenarioIntent {
    concepts: Vec<String>,
    habits: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ScenarioObservables {
    instructor: Vec<String>,
    student: Vec<String>,
    system_or_artifact: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EatmeScenarioAsset {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    launcher: Option<EatmeScenarioLauncher>,
    #[serde(default)]
    real_alice: Option<EatmeScenarioRealAlice>,
    #[serde(default)]
    smoke_ready: Option<EatmeScenarioSmokeReady>,
    #[serde(default)]
    acceptance_criteria: Vec<EatmeScenarioAcceptanceCriterion>,
    #[serde(default)]
    steps: Vec<EatmeScenarioStep>,
    #[serde(default)]
    timeouts: BTreeMap<String, u64>,
    #[serde(default)]
    artifacts: BTreeMap<String, String>,
    #[serde(default)]
    unsupported_policy: String,
}

#[derive(Clone, Debug, Deserialize)]
struct EatmeScenarioLauncher {
    #[serde(default)]
    command: String,
    #[serde(default)]
    scenario: String,
}

#[derive(Clone, Debug, Deserialize)]
struct EatmeScenarioRealAlice {
    #[serde(default)]
    gated_by: String,
}

#[derive(Clone, Debug, Deserialize)]
struct EatmeScenarioSmokeReady {
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct EatmeScenarioAcceptanceCriterion {
    #[serde(default)]
    given: String,
    #[serde(default)]
    when: String,
    #[serde(default)]
    then: String,
}

#[derive(Clone, Debug, Deserialize)]
struct EatmeScenarioStep {
    #[serde(default)]
    id: String,
    #[serde(default)]
    command: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GadugiScenarioAsset {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    steps: Vec<GadugiScenarioStep>,
    #[serde(default)]
    assertions: Vec<GadugiScenarioAssertion>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GadugiScenarioStep {
    #[serde(default)]
    name: String,
    #[serde(default)]
    agent: String,
    #[serde(default)]
    action: String,
    #[serde(default)]
    params: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GadugiScenarioAssertion {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    assertion_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_committed_persona_crew_asset() {
        let asset = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/personas/alice-user-crew.yaml");
        let report = validate_persona_crew(&asset).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert_eq!(report.instructor_count, 6);
        assert_eq!(report.student_count, 7);
        assert_eq!(report.creative_scenario_count, 10);
    }

    #[test]
    fn validates_committed_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = validate_assets(&root).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert!(report.scenario_asset_count >= 2);
    }

    #[test]
    fn validates_committed_lesson_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for asset in [
            "assets/scenarios/eatme/building-a-scene-first-world.yaml",
            "assets/scenarios/gadugi/building-a-scene-first-world.yaml",
            "assets/scenarios/eatme/code-editor-first-run.yaml",
            "assets/scenarios/gadugi/code-editor-first-run.yaml",
        ] {
            let report = validate_scenario_asset(&root.join(asset)).unwrap();
            assert!(report.passed, "{asset}: {:?}", report.errors);
        }
    }

    #[test]
    fn rejects_malformed_eatme_scenario_asset() {
        let scenario = EatmeScenarioAsset {
            schema_version: "eatme.scenario/v1".into(),
            id: "not valid".into(),
            title: "".into(),
            ..EatmeScenarioAsset::default()
        };
        let report = validate_eatme_scenario(Path::new("bad.yaml"), &scenario);
        assert!(!report.passed);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("must be kebab-case"))
        );
    }

    #[test]
    fn scenario_validation_errors_include_path_or_scenario_id() {
        let scenario = EatmeScenarioAsset {
            schema_version: "eatme.scenario/v1".into(),
            id: "building-a-scene-first-world".into(),
            kind: "alice_lesson_smoke".into(),
            owner: "eatme".into(),
            launcher: Some(EatmeScenarioLauncher {
                command: "alice launch-smoke".into(),
                scenario: "building-a-scene-first-world".into(),
            }),
            ..EatmeScenarioAsset::default()
        };
        let path = Path::new("assets/scenarios/eatme/building-a-scene-first-world.yaml");
        let report = validate_eatme_scenario(path, &scenario);

        assert!(!report.passed);
        assert!(
            report.errors.iter().all(|error| {
                error.contains("building-a-scene-first-world")
                    || error.contains("assets/scenarios/eatme/building-a-scene-first-world.yaml")
            }),
            "each schema error should identify the scenario id or asset path: {:?}",
            report.errors
        );
    }

    #[test]
    fn lesson_smoke_requires_real_alice_gate() {
        let scenario = EatmeScenarioAsset {
            schema_version: "eatme.scenario/v1".into(),
            id: "code-editor-first-run".into(),
            title: "Code Editor First Run".into(),
            kind: "alice_lesson_smoke".into(),
            owner: "eatme".into(),
            purpose: "launches through the real Alice smoke harness".into(),
            launcher: Some(EatmeScenarioLauncher {
                command: "alice launch-smoke".into(),
                scenario: "code-editor-first-run".into(),
            }),
            steps: vec![EatmeScenarioStep {
                id: "launch-smoke".into(),
                command: "eatme alice launch-smoke --scenario code-editor-first-run".into(),
                evidence: vec!["manifest scenario_id matches".into()],
            }],
            timeouts: BTreeMap::from([("launch_seconds".into(), 120)]),
            artifacts: BTreeMap::from([
                ("manifest".into(), "runs/building/manifest.json".into()),
                (
                    "screenshot".into(),
                    "runs/building/screenshots/startup.png".into(),
                ),
                ("log".into(), "runs/building/alice.log".into()),
            ]),
            unsupported_policy: "fail loudly when prerequisites are unavailable".into(),
            ..EatmeScenarioAsset::default()
        };
        let report = validate_eatme_scenario(Path::new("building.yaml"), &scenario);
        assert!(!report.passed);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("real_alice.gated_by"))
        );
    }

    #[test]
    fn known_lesson_smoke_requires_lesson_kind() {
        let scenario = EatmeScenarioAsset {
            schema_version: "eatme.scenario/v1".into(),
            id: "code-editor-first-run".into(),
            title: "Code Editor First Run".into(),
            owner: "eatme".into(),
            purpose: "launches through the real Alice smoke harness".into(),
            launcher: Some(EatmeScenarioLauncher {
                command: "alice launch-smoke".into(),
                scenario: "code-editor-first-run".into(),
            }),
            real_alice: Some(EatmeScenarioRealAlice {
                gated_by: "EATME_REAL_ALICE=1".into(),
            }),
            smoke_ready: Some(EatmeScenarioSmokeReady {
                evidence: vec!["manifest assertions".into()],
            }),
            acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
                given: "dependencies are available".into(),
                when: "the lane launches".into(),
                then: "the manifest records the scenario id".into(),
            }],
            steps: vec![EatmeScenarioStep {
                id: "launch-smoke".into(),
                command: "eatme alice launch-smoke --scenario code-editor-first-run".into(),
                evidence: vec!["manifest scenario_id matches".into()],
            }],
            timeouts: BTreeMap::from([("launch_seconds".into(), 120)]),
            artifacts: BTreeMap::from([
                ("manifest".into(), "runs/code/manifest.json".into()),
                (
                    "screenshot".into(),
                    "runs/code/screenshots/startup.png".into(),
                ),
                ("log".into(), "runs/code/alice.log".into()),
            ]),
            unsupported_policy: "fail loudly when prerequisites are unavailable".into(),
            ..EatmeScenarioAsset::default()
        };
        let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario);

        assert!(!report.passed);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("kind must be alice_lesson_smoke"))
        );
    }

    #[test]
    fn gadugi_scenario_rejects_direct_alice_runtime_commands() {
        let scenario = GadugiScenarioAsset {
            name: "Bad Gadugi Alice Runtime Owner".into(),
            description:
                "Attempts to own Xvfb and Java launch directly instead of using eatme CLI.".into(),
            version: "1.0.0".into(),
            steps: vec![GadugiScenarioStep {
                name: "Launch Alice directly".into(),
                agent: "gadugi-agent".into(),
                action: "execute_command".into(),
                params: BTreeMap::from([(
                    "command".into(),
                    "Xvfb :99 & java org.alice.stageide.EntryPoint".into(),
                )]),
            }],
            assertions: vec![GadugiScenarioAssertion {
                name: "Direct runtime command succeeded".into(),
                assertion_type: "command_success".into(),
            }],
        };

        let report = validate_gadugi_scenario(
            Path::new("assets/scenarios/gadugi/bad-direct-runtime.yaml"),
            &scenario,
        );
        assert!(
            !report.passed,
            "gadugi assets must not own Alice runtime details: {:?}",
            report.errors
        );
        assert!(
            report.errors.iter().any(|error| {
                error.contains("gadugi")
                    && error.contains("alice launch-smoke")
                    && error.contains("runtime")
            }),
            "boundary error should direct gadugi scenarios to the eatme launch-smoke CLI: {:?}",
            report.errors
        );
    }
}
