use crate::schema::EatmeScenarioAsset;
use crate::schema::EatmeScenarioStep;
use std::sync::LazyLock;

const PREFLIGHT_TEMPLATE_YAML: &str =
    include_str!("../../../assets/scenarios/gadugi/step-blocks/alice-preflight.yaml");
const LAUNCH_SMOKE_TEMPLATE_YAML: &str =
    include_str!("../../../assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml");

#[derive(serde::Deserialize)]
struct StepBlockTemplate {
    steps: Vec<StepBlockEntry>,
}

#[derive(serde::Deserialize)]
struct StepBlockEntry {
    id: String,
    expected_stdout: Vec<String>,
}

fn parse_step_block_template(yaml: &str) -> Vec<StepBlockEntry> {
    serde_yaml::from_str::<StepBlockTemplate>(yaml)
        .expect("step-block template must be valid YAML")
        .steps
}

// Parse each template exactly once, regardless of how many scenarios reference them.
static PREFLIGHT_STEPS: LazyLock<Vec<StepBlockEntry>> =
    LazyLock::new(|| parse_step_block_template(PREFLIGHT_TEMPLATE_YAML));
static LAUNCH_SMOKE_STEPS: LazyLock<Vec<StepBlockEntry>> =
    LazyLock::new(|| parse_step_block_template(LAUNCH_SMOKE_TEMPLATE_YAML));

fn step_block_patterns(steps: &[StepBlockEntry], step_id: &str) -> Vec<String> {
    steps
        .iter()
        .find(|entry| entry.id == step_id)
        .unwrap_or_else(|| panic!("step-block template must contain step '{step_id}'"))
        .expected_stdout
        .clone()
}

pub(super) fn preflight_validate_assets_patterns(
    expected_scenario_asset_count: usize,
) -> Vec<String> {
    step_block_patterns(&PREFLIGHT_STEPS, "validate-assets")
        .into_iter()
        .map(|pattern| {
            pattern.replace(
                "{{scenario-asset-count}}",
                &expected_scenario_asset_count.to_string(),
            )
        })
        .collect()
}

pub(super) fn preflight_check_dependencies_patterns() -> Vec<String> {
    step_block_patterns(&PREFLIGHT_STEPS, "check-dependencies")
}

pub(super) fn launch_expected_stdout(
    scenario: &EatmeScenarioAsset,
    step: &EatmeScenarioStep,
) -> Vec<String> {
    let evidence = step.evidence.join("\n").to_lowercase();
    let template_patterns = step_block_patterns(&LAUNCH_SMOKE_STEPS, "launch-smoke");
    let mut expected = Vec::new();
    for pattern in &template_patterns {
        let substituted = pattern.replace("{{scenario-id}}", &scenario.id);
        expected.push(substituted);
        // failure_category varies by scenario kind — insert after scenario_id
        if pattern.contains("\"scenario_id\"") {
            if scenario.kind == "alice_real_ui_action_contract" {
                expected.push("\"failure_category\":".into());
            } else {
                expected.push("\"failure_category\": null".into());
            }
        }
    }

    if evidence.contains("screenshot or window") {
        expected.push("\"startup_window_or_screenshot\": {".into());
    } else if evidence.contains("screenshot") {
        expected.push("\"startup_screenshot\": {".into());
    }

    for assertion in [
        "specific_alice_window_detected",
        "activate_alice_window_ui_action",
        "save_project_desktop_shortcut_dispatch",
        "place_object_precondition_no_go_probe",
        "place_object_ui_action",
        "edit_procedure_ui_action",
        "run_world_ui_action",
        "save_project_ui_action",
        "ui_action_artifact_captured",
    ] {
        if evidence.contains(assertion) {
            expected.push(format!("\"{assertion}\": {{"));
        }
    }
    if evidence.contains("ui-action-contract.json") {
        expected.push("\"ui_action_contract\": {".into());
    }
    if evidence.contains("africa.a3p") {
        expected.push("africa.a3p".into());
    }
    if evidence.contains("assertions all pass") || evidence.contains("passed=true") {
        expected.push("\"passed\": true".into());
    }
    expected
}
