use crate::schema::EatmeScenarioAsset;
use crate::validate_scenario_asset;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn ide_performance_parity_contract_covers_large_projects_and_animation_load() {
    let root = repository_root();
    let path = scenario_path(&root, "ide-performance-parity");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);

    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_contains_all(
        "ide-performance-parity contract",
        &contract,
        &[
            "50+ entities",
            "load time",
            "under threshold",
            "10 concurrent animations",
            "frame-rate target",
            "first user-visible degradation",
            "not full user interface automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

#[test]
fn ide_accessibility_parity_contract_covers_labels_keyboard_contrast_and_zoom() {
    let root = repository_root();
    let path = scenario_path(&root, "ide-accessibility-parity");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);

    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_contains_all(
        "ide-accessibility-parity contract",
        &contract,
        &[
            "all interactive elements have accessible labels",
            "all IDE features are reachable via keyboard",
            "UI remains usable in high contrast",
            "UI elements scale correctly at 200%",
            "keyboard-only",
            "screen reader",
            "not full user interface automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

#[test]
fn new_performance_and_accessibility_scenarios_validate_successfully() {
    let root = repository_root();
    for id in ["ide-performance-parity", "ide-accessibility-parity"] {
        let report = validate_scenario_asset(&scenario_path(&root, id)).unwrap();
        assert!(report.passed, "{id}: {:?}", report.errors);
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn scenario_path(root: &Path, id: &str) -> PathBuf {
    root.join("assets/scenarios/eatme")
        .join(format!("{id}.yaml"))
}

fn read_eatme_scenario(path: &Path) -> EatmeScenarioAsset {
    let content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let missing: Vec<_> = needles
        .iter()
        .filter(|needle| {
            let compact = needle.split_whitespace().collect::<Vec<_>>().join(" ");
            !normalized.contains(&compact)
        })
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{label} is missing required evidence language: {missing:?}"
    );
}
