use crate::discovery::scenario_asset_paths;
use crate::report::GadugiAdapterGenerationReport;
use crate::schema::EatmeScenarioAsset;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const GENERATED_BY: &str = "eatme-assets gadugi adapter generator";

pub fn generate_gadugi_adapters(root: &Path, check: bool) -> Result<GadugiAdapterGenerationReport> {
    let eatme_root = root.join("assets/scenarios/eatme");
    let gadugi_root = root.join("assets/scenarios/gadugi");
    let mut report = GadugiAdapterGenerationReport {
        schema_version: "eatme.assets/gadugi-adapter-generation/v1".into(),
        root: root.display().to_string(),
        generated_count: 0,
        checked_count: 0,
        changed: Vec::new(),
        passed: true,
        errors: Vec::new(),
    };

    if !check {
        fs::create_dir_all(&gadugi_root)
            .with_context(|| format!("creating {}", gadugi_root.display()))?;
    }

    for source_path in scenario_asset_paths(&eatme_root)? {
        let yaml = generate_gadugi_adapter_yaml(root, &source_path)?;
        let scenario = read_eatme_scenario(&source_path)?;
        let target_path = gadugi_root.join(format!("{}.yaml", scenario.id));
        report.generated_count += 1;
        if check {
            report.checked_count += 1;
            match fs::read_to_string(&target_path) {
                Ok(existing) if existing == yaml => {}
                Ok(_) => {
                    report.passed = false;
                    report.changed.push(target_path.display().to_string());
                    report.errors.push(format!(
                        "{} is stale; regenerate with `eatme assets generate-gadugi`",
                        target_path.display()
                    ));
                }
                Err(error) => {
                    report.passed = false;
                    report.changed.push(target_path.display().to_string());
                    report.errors.push(format!(
                        "{} is missing or unreadable: {error}",
                        target_path.display()
                    ));
                }
            }
        } else {
            let changed = fs::read_to_string(&target_path)
                .map(|existing| existing != yaml)
                .unwrap_or(true);
            if changed {
                fs::write(&target_path, yaml)
                    .with_context(|| format!("writing {}", target_path.display()))?;
                report.changed.push(target_path.display().to_string());
            }
        }
    }

    Ok(report)
}

pub fn generate_gadugi_adapter_yaml(root: &Path, source_path: &Path) -> Result<String> {
    let scenario = read_eatme_scenario(source_path)?;
    let source_asset = source_path
        .strip_prefix(root)
        .with_context(|| {
            format!(
                "scenario asset {} is not under root {}",
                source_path.display(),
                root.display()
            )
        })?
        .to_string_lossy()
        .replace('\\', "/");
    if scenario.id.is_empty() {
        bail!("{} must define id", source_path.display());
    }

    let timeout_ms = scenario
        .timeouts
        .get("scenario_seconds")
        .copied()
        .unwrap_or(1800)
        * 1000;
    let launch_timeout = scenario
        .timeouts
        .get("launch_seconds")
        .copied()
        .unwrap_or(900);
    let run_id = format!("gadugi-{}", scenario.id);
    let steps = scenario
        .steps
        .iter()
        .map(|step| generated_step(&scenario, step, &run_id, launch_timeout))
        .collect::<Vec<_>>();
    let assertions = scenario
        .steps
        .iter()
        .map(|step| GeneratedAssertion {
            name: format!("{} succeeds", step.id),
            assertion_type: "command_success".into(),
            agent: "eatme-cli-agent".into(),
            params: BTreeMap::from([("step".into(), step_title(&step.id))]),
        })
        .collect::<Vec<_>>();

    let adapter = GeneratedGadugiAdapter {
        name: format!("Eatme {}", scenario.title),
        description: format!(
            "Gadugi-compatible CLI scenario generated from {source_asset}. Alice desktop launch behavior remains owned by eatme; gadugi invokes eatme commands and checks manifest-level evidence only."
        ),
        version: "1.0.0".into(),
        config: GeneratedConfig {
            timeout: timeout_ms,
            retries: 0,
            parallel: false,
        },
        environment: GeneratedEnvironment {
            requires: required_environment(&scenario),
            optional: vec!["RUN_ID".into(), "EATME_REPO".into()],
        },
        agents: vec![GeneratedAgent {
            name: "eatme-cli-agent".into(),
            agent_type: "system".into(),
            config: GeneratedAgentConfig {
                shell: "bash".into(),
                cwd: ".".into(),
                timeout: timeout_ms,
                capture_output: true,
            },
        }],
        steps,
        assertions,
        metadata: GeneratedMetadata {
            source_eatme_asset: source_asset,
            generated_by: GENERATED_BY.into(),
            tags: vec![
                "alice".into(),
                "eatme".into(),
                "gadugi".into(),
                "outside-in".into(),
                scenario.id.clone(),
            ],
            priority: "critical".into(),
            author: "eatme".into(),
            test_type: "launch-smoke".into(),
        },
    };

    let mut yaml = serde_yaml::to_string(&adapter).context("serializing gadugi adapter YAML")?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    Ok(yaml)
}

fn read_eatme_scenario(path: &Path) -> Result<EatmeScenarioAsset> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading eatme scenario asset {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("parsing eatme scenario YAML {}", path.display()))
}

fn generated_step(
    scenario: &EatmeScenarioAsset,
    step: &crate::schema::EatmeScenarioStep,
    run_id: &str,
    launch_timeout: u64,
) -> GeneratedStep {
    let command = repository_command(step.command.trim(), run_id);
    GeneratedStep {
        name: step_title(&step.id),
        agent: "eatme-cli-agent".into(),
        action: "execute_command".into(),
        params: BTreeMap::from([("command".into(), command)]),
        expect: GeneratedExpect {
            exit_code: 0,
            stdout_contains: expected_stdout(scenario, &step.id),
        },
        timeout: step_timeout_ms(&step.id, launch_timeout),
    }
}

fn repository_command(command: &str, run_id: &str) -> String {
    format!("cd \"${{EATME_REPO:-.}}\"\nexport RUN_ID=\"${{RUN_ID:-{run_id}}}\"\n{command}")
}

fn step_title(id: &str) -> String {
    id.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn step_timeout_ms(step_id: &str, launch_timeout: u64) -> u64 {
    if step_id.contains("launch") {
        launch_timeout * 1000
    } else {
        60_000
    }
}

fn expected_stdout(scenario: &EatmeScenarioAsset, step_id: &str) -> Vec<String> {
    if step_id.contains("validate") {
        return vec!["\"passed\": true".into()];
    }
    if step_id.contains("dependencies") {
        return vec!["\"all_required_available\": true".into()];
    }
    if step_id.contains("discover") {
        return vec!["\"alice_ide_jar_exists\": true".into()];
    }
    if step_id.contains("launch") || step_id.contains("smoke") {
        return vec![
            format!("\"scenario_id\": \"{}\"", scenario.id),
            "\"failure_category\": null".into(),
            "\"passed\": true".into(),
        ];
    }
    Vec::new()
}

fn required_environment(scenario: &EatmeScenarioAsset) -> Vec<String> {
    let mut required = vec!["ALICE_HOME".into()];
    if scenario
        .real_alice
        .as_ref()
        .map(|real_alice| real_alice.gated_by == "EATME_REAL_ALICE=1")
        .unwrap_or(false)
    {
        required.push("EATME_REAL_ALICE".into());
    }
    required
}

#[derive(Serialize)]
struct GeneratedGadugiAdapter {
    name: String,
    description: String,
    version: String,
    config: GeneratedConfig,
    environment: GeneratedEnvironment,
    agents: Vec<GeneratedAgent>,
    steps: Vec<GeneratedStep>,
    assertions: Vec<GeneratedAssertion>,
    metadata: GeneratedMetadata,
}

#[derive(Serialize)]
struct GeneratedConfig {
    timeout: u64,
    retries: u64,
    parallel: bool,
}

#[derive(Serialize)]
struct GeneratedEnvironment {
    requires: Vec<String>,
    optional: Vec<String>,
}

#[derive(Serialize)]
struct GeneratedAgent {
    name: String,
    #[serde(rename = "type")]
    agent_type: String,
    config: GeneratedAgentConfig,
}

#[derive(Serialize)]
struct GeneratedAgentConfig {
    shell: String,
    cwd: String,
    timeout: u64,
    capture_output: bool,
}

#[derive(Serialize)]
struct GeneratedStep {
    name: String,
    agent: String,
    action: String,
    params: BTreeMap<String, String>,
    expect: GeneratedExpect,
    timeout: u64,
}

#[derive(Serialize)]
struct GeneratedExpect {
    exit_code: u64,
    stdout_contains: Vec<String>,
}

#[derive(Serialize)]
struct GeneratedAssertion {
    name: String,
    #[serde(rename = "type")]
    assertion_type: String,
    agent: String,
    params: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct GeneratedMetadata {
    source_eatme_asset: String,
    generated_by: String,
    tags: Vec<String>,
    priority: String,
    author: String,
    test_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_scenario_asset;

    #[test]
    fn generated_gadugi_adapters_match_committed_assets_and_validate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for source_path in scenario_asset_paths(&root.join("assets/scenarios/eatme")).unwrap() {
            let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
            let scenario = read_eatme_scenario(&source_path).unwrap();
            let target_path = root
                .join("assets/scenarios/gadugi")
                .join(format!("{}.yaml", scenario.id));
            let committed = fs::read_to_string(&target_path).unwrap();

            assert_eq!(committed, generated, "{} is stale", target_path.display());
            let report = validate_scenario_asset(&target_path).unwrap();
            assert!(
                report.passed,
                "{}: {:?}",
                target_path.display(),
                report.errors
            );
        }
    }
}
