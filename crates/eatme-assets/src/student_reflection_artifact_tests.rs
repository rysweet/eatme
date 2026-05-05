use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "student-reflection-artifact-review";
const REQUIRED_CONTRACT_TEXT: &[&str] = &[
    "student learning artifact",
    "reflection evidence card",
    "explain one Alice action",
    "instructor review note",
    "student revision prompt",
    "artifact behavior and learner explanation",
    "not full user interface automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
    "not a deployed service",
];
const REQUIRED_OUTPUTS: &[&str] = &[
    "student_reflection_evidence_card",
    "instructor_review_note",
    "student_revision_prompt",
];

#[test]
fn student_reflection_artifact_scenario_names_reviewable_learning_evidence() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(&text).unwrap();
    let expected_outputs = scenario
        .agentic_flow
        .as_ref()
        .expect("student reflection scenario must define agentic_flow")
        .expected_outputs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_contains_all(
        "student reflection artifact scenario",
        &text,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "student reflection artifact outputs",
        &expected_outputs.join("\n"),
        REQUIRED_OUTPUTS,
    );
    for artifact in REQUIRED_OUTPUTS {
        assert!(
            scenario.artifacts.contains_key(*artifact),
            "student reflection scenario must define artifacts.{artifact}"
        );
    }
}

#[test]
fn generated_adapter_preserves_student_reflection_artifact_contract() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the editable student reflection scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated student reflection adapter contract",
        &generated,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "generated student reflection adapter outputs",
        &generated,
        REQUIRED_OUTPUTS,
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, scenario_collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(scenario_collection)
        .join(format!("{SCENARIO_ID}.yaml"))
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize_whitespace(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required evidence language: {missing:?}"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
