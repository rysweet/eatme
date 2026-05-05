use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "instructor-student-outcomes-rubric";
const REQUIRED_CONTRACT_TEXT: &[&str] = &[
    "project discussion guide",
    "student evidence questions",
    "instructor boundary note",
    "student-owned next revision",
    "visible Alice project behavior",
    "learner explanation",
    "without claiming automated creative assessment",
    "not learner-world grading",
    "not full user interface automation",
];
const REQUIRED_OUTPUTS: &[&str] = &[
    "student_outcomes_rubric",
    "feedback_frame",
    "revision_next_step",
    "project_discussion_guide",
];

#[test]
fn instructor_student_outcomes_scenario_names_the_assessment_boundary() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(&text).unwrap();
    let expected_outputs = scenario
        .agentic_flow
        .as_ref()
        .expect("student outcomes scenario must define agentic_flow")
        .expected_outputs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_contains_all(
        "student outcomes boundary scenario",
        &text,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "student outcomes boundary outputs",
        &expected_outputs.join("\n"),
        REQUIRED_OUTPUTS,
    );
    for artifact in REQUIRED_OUTPUTS {
        assert!(
            scenario.artifacts.contains_key(*artifact),
            "student outcomes scenario must define artifacts.{artifact}"
        );
    }
}

#[test]
fn generated_adapter_preserves_the_assessment_boundary_contract() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the editable student outcomes scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated student outcomes boundary adapter",
        &generated,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "generated student outcomes boundary outputs",
        &generated,
        REQUIRED_OUTPUTS,
    );
}

#[test]
fn assessment_curator_persona_keeps_discussion_honest() {
    let root = repository_root();
    let text = fs::read_to_string(root.join("assets/personas/alice-user-crew.yaml")).unwrap();

    assert_contains_all(
        "assessment curator persona",
        &text,
        &[
            "discussion questions that separate visible project behavior, learner explanation, and student-owned revision",
            "does not claim automated creative assessment or learner-world grading",
        ],
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
