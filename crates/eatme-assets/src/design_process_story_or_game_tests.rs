use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn design_process_story_or_game_covers_creative_storyteller_and_systems_puzzle_solver() {
    let root = repository_root();
    let path = scenario_path(&root, "design-process-story-or-game");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario.personas.as_ref().expect("must define personas");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    for instructor in &["studio-facilitator", "assessment-curator"] {
        assert!(
            personas.instructors.iter().any(|p| p == instructor),
            "must include instructor persona {instructor}"
        );
    }
    for student in &[
        "creative-storyteller",
        "systems-puzzle-solver",
        "reflective-debugger",
    ] {
        assert!(
            personas.students.iter().any(|p| p == student),
            "must include student persona {student}"
        );
    }
    assert_contains_all(
        "design-process-story-or-game contract",
        &contract,
        &[
            "design-process guide",
            "scene-sketch card",
            "design-to-code bridge card",
            "story-vs-game framing",
            "event loop",
            "design brief",
            "Alice concept",
            "narrated text overlays",
            "optional extension",
            "revision note",
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
