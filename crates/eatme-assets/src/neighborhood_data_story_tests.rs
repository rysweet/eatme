use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn neighborhood_data_story_covers_accessibility_advocate_and_creative_storyteller() {
    let root = repository_root();
    let path = scenario_path(&root, "neighborhood-data-story");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario.personas.as_ref().expect("must define personas");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    for instructor in &["assessment-curator", "studio-facilitator"] {
        assert!(
            personas.instructors.iter().any(|p| p == instructor),
            "must include instructor persona {instructor}"
        );
    }
    for student in &[
        "accessibility-advocate",
        "creative-storyteller",
        "systems-puzzle-solver",
    ] {
        assert!(
            personas.students.iter().any(|p| p == student),
            "must include student persona {student}"
        );
    }
    assert_contains_all(
        "neighborhood-data-story contract",
        &contract,
        &[
            "student data-mapping card",
            "representation-limits reflection",
            "fictional",
            "privacy-safe",
            "interpretation limits",
            "accessibility-advocate",
            "text labels",
            "audio narration",
            "optional extension",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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
        .filter(|n| {
            let nw = n.split_whitespace().collect::<Vec<_>>().join(" ");
            !normalized.contains(&nw)
        })
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{label} is missing required evidence language: {missing:?}"
    );
}
