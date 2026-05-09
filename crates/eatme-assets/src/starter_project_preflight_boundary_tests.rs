use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "starter-project-open-save-export-preflight";

const REQUIRED_SOURCE_BOUNDARIES: &[&str] = &[
    "plain automation scenario for instructors and students",
    "opened starter project",
    "small editable starter-world change",
    "attempt to run or observe",
    "save/reopen/export/readiness gaps",
    "not proof of visible rendering correctness",
    "without claiming full Save completion or full UI automation",
    "without claiming first-lesson completion",
    "not grading",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
];

const REQUIRED_ADAPTER_BOUNDARIES: &[&str] = &[
    "opened starter project",
    "manifest/log/window/screenshot evidence",
    "bounded starter-world and readiness-gap artifacts",
    "without claiming save/reopen/export coverage",
    "not full UI automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
    "not visible rendering correctness proof",
    "not first-lesson completion",
    "not full Save completion",
];

const INTERNAL_OR_OVERBROAD_LANGUAGE: &[&str] = &[
    "action evidence",
    "source boundary",
    "manifest-level evidence only",
    "proves visible rendering correctness",
    "proves save/reopen/export",
    "first lesson is complete",
    "grades learner work",
    "assesses creativity",
];

#[test]
fn source_starter_project_preflight_uses_plain_bounded_user_facing_language() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(&text).unwrap();

    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "alice_lesson_smoke");
    assert_contains_all(
        "starter-project preflight source",
        &text,
        REQUIRED_SOURCE_BOUNDARIES,
    );
    assert_contains_none(
        "starter-project preflight source",
        &text,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
}

#[test]
fn generated_starter_project_preflight_adapter_uses_same_plain_boundaries() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the canonical starter-project scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated starter-project preflight adapter",
        &generated,
        REQUIRED_ADAPTER_BOUNDARIES,
    );
    assert_contains_none(
        "generated starter-project preflight adapter",
        &generated,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(collection)
        .join(format!("{SCENARIO_ID}.yaml"))
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required bounded language: {missing:?}"
    );
}

fn assert_contains_none(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "{label} contains internal or overbroad language: {present:?}"
    );
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
