use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AliceQaScenario {
    id: String,
    workflow: String,
    #[serde(default)]
    automation_mode: String,
    #[serde(default)]
    supporting_evidence: Vec<String>,
    #[serde(default)]
    evidence: ScenarioEvidence,
    #[serde(default)]
    automation: Option<ScenarioAutomation>,
    #[serde(default)]
    steps: Vec<String>,
    #[serde(default)]
    agents: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct ScenarioEvidence {
    #[serde(default)]
    required: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct ScenarioAutomation {
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    argv: Vec<String>,
}

fn alice_repository_root() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("ALICE_REPO") {
        let path = PathBuf::from(explicit);
        if path.is_dir() {
            return Some(path);
        }
    }

    let sibling = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../alice");
    sibling.is_dir().then_some(sibling)
}

fn scenario_dir() -> Option<PathBuf> {
    alice_repository_root().map(|root| root.join("qa/outside-in/alice-desktop/scenarios"))
}

fn read_scenario(name: &str) -> Option<AliceQaScenario> {
    let path = scenario_dir()?.join(format!("{name}.yaml"));
    let text = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}

fn read_all_scenario_values() -> Option<Vec<Value>> {
    let dir = scenario_dir()?;
    let mut scenarios = Vec::new();
    for entry in fs::read_dir(dir).ok()? {
        let path = entry.ok()?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let text = fs::read_to_string(path).ok()?;
        scenarios.push(serde_yaml::from_str(&text).ok()?);
    }
    Some(scenarios)
}

fn yaml_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn yaml_string_list(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_sequence)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn yaml_required_evidence(value: &Value) -> Vec<String> {
    value
        .get("evidence")
        .and_then(|evidence| evidence.get("required"))
        .and_then(Value::as_sequence)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn require_alice_repo() -> bool {
    if alice_repository_root().is_some() {
        return true;
    }
    eprintln!("skipping Alice QA scenario integration test (set ALICE_REPO or check out ../alice)");
    false
}

fn assert_contains_all(label: &str, values: &[String], required: &[&str]) {
    for needle in required {
        assert!(
            values.iter().any(|value| value.contains(needle)),
            "{label} should contain evidence matching {needle:?}, got {values:?}"
        );
    }
}

#[test]
fn alice_launch_scenario_matches_eatme_launch_smoke_expectations() {
    if !require_alice_repo() {
        return;
    }

    let scenario = read_scenario("launch").expect("launch scenario should parse");
    let automation = scenario
        .automation
        .as_ref()
        .expect("launch scenario should define automation");

    assert_eq!(scenario.id, "alice-desktop-launch");
    assert_eq!(scenario.workflow, "launch");
    assert_eq!(scenario.automation_mode, "xvfb-real-alice");
    assert_eq!(automation.cwd, "alice-ide");
    assert!(automation.argv.iter().any(|arg| arg == "mvn"));
    assert!(automation.argv.iter().any(|arg| arg == "compile"));
    assert!(automation.argv.iter().any(|arg| arg == "exec:java"));
    assert!(automation.argv.iter().any(|arg| arg == "-Dalice-ide"));
    assert_contains_all(
        "launch evidence.required",
        &scenario.evidence.required,
        &[
            "Launch log",
            "Screenshot",
            "x-window-inventory.json",
            "Environment summary",
            "Exit, status, or timeout record",
        ],
    );
    assert_eq!(scenario.steps, vec!["validate"]);
    assert_eq!(scenario.agents, vec!["alice-desktop-qa"]);
}

#[test]
fn alice_run_window_contract_stays_wired_to_eatme_artifacts() {
    if !require_alice_repo() {
        return;
    }

    let scenario =
        read_scenario("run-window-contract").expect("run-window contract scenario should parse");
    let automation = scenario
        .automation
        .as_ref()
        .expect("run-window contract should define automation");

    assert_eq!(scenario.id, "alice-desktop-run-window-contract");
    assert_eq!(scenario.workflow, "run-window-contract");
    assert_eq!(scenario.automation_mode, "gated-command-smoke");
    assert_eq!(automation.cwd, ".");
    assert!(automation.argv.iter().any(|arg| arg == "-pl"));
    assert!(
        automation
            .argv
            .iter()
            .any(|arg| arg == "-Dtest=org.alice.tools.EatmeRunWindowEvidenceTest")
    );
    assert_contains_all(
        "run-window evidence.required",
        &scenario.evidence.required,
        &[
            "run-window-created.json",
            "eatme.alice-run-window-created/v1",
            "contract_scope=run-window-creation-wiring",
            "full_ui_automation_claimed=false",
            "does_not_claim",
        ],
    );
}

#[test]
fn manual_alice_desktop_workflows_keep_launch_prerequisite_explicit() {
    if !require_alice_repo() {
        return;
    }

    for scenario_name in ["save-load", "export"] {
        let scenario = read_scenario(scenario_name)
            .unwrap_or_else(|| panic!("{scenario_name} scenario should parse"));
        assert_eq!(
            scenario.automation_mode, "manual-evidence-required",
            "{scenario_name} should remain manual until eatme can prove the full GUI lane"
        );
        assert!(
            scenario
                .supporting_evidence
                .iter()
                .any(|id| id == "alice-desktop-launch"),
            "{scenario_name} should depend on the shared Alice desktop launch prerequisite"
        );
    }
}

#[test]
fn all_alice_desktop_scenarios_have_core_structure_and_valid_cross_references() {
    if !require_alice_repo() {
        return;
    }

    let scenarios = read_all_scenario_values().expect("scenario directory should parse");
    let known_ids = scenarios
        .iter()
        .map(|scenario| yaml_string(scenario, "id"))
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>();

    assert!(known_ids.contains("alice-desktop-launch"));
    assert!(!known_ids.is_empty());

    for scenario in &scenarios {
        let id = yaml_string(scenario, "id");
        let workflow = yaml_string(scenario, "workflow");
        let required_evidence = yaml_required_evidence(scenario);
        let steps = yaml_string_list(scenario, "steps");
        let agents = yaml_string_list(scenario, "agents");
        let supporting_evidence = yaml_string_list(scenario, "supportingEvidence");
        let automation_mode = yaml_string(scenario, "automationMode");

        assert!(!id.trim().is_empty(), "scenario id must be present");
        assert!(
            !workflow.trim().is_empty(),
            "scenario {id} must declare a workflow"
        );
        assert!(
            !required_evidence.is_empty(),
            "scenario {id} must declare required evidence"
        );
        assert!(
            !steps.is_empty(),
            "scenario {id} must declare at least one step"
        );
        assert!(
            !automation_mode.trim().is_empty() || !agents.is_empty(),
            "scenario {id} must declare an automationMode or at least one agent"
        );
        for supporting_id in &supporting_evidence {
            assert!(
                known_ids.contains(supporting_id),
                "scenario {id} references missing supportingEvidence id {supporting_id}"
            );
        }
    }
}
