use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

mod asset_validation;
mod first_lesson_evidence;
mod gadugi_adapters;
mod module_size_contracts;
mod scenario_contracts;

const EXPECTED_SCENARIO_ASSET_COUNT: usize = 95;

struct TargetScenario {
    id: &'static str,
    instructors: &'static [&'static str],
    students: &'static [&'static str],
}

const TARGET_SCENARIOS: &[TargetScenario] = &[
    TargetScenario {
        id: "setup-support-lab-readiness",
        instructors: &["setup-support-specialist", "classroom-orchestrator"],
        students: &["collaborative-peer-mentor", "curious-novice"],
    },
    TargetScenario {
        id: "alice-2-migration-bridge",
        instructors: &["alice-2-migration-mentor", "curriculum-pathway-designer"],
        students: &["curious-novice", "creative-storyteller"],
    },
    TargetScenario {
        id: "vr-player-comfort-playtest",
        instructors: &["studio-facilitator", "assessment-curator"],
        students: &[
            "vr-player-tester",
            "accessibility-advocate",
            "systems-puzzle-solver",
        ],
    },
    TargetScenario {
        id: "model-texture-import-checkpoint",
        instructors: &["setup-support-specialist", "exercise-forger"],
        students: &[
            "model-texture-importer",
            "reflective-debugger",
            "creative-storyteller",
        ],
    },
];

const FIRST_LESSON_SMOKE_READY_EVIDENCE_COUNT: usize = 14;
const FIRST_LESSON_REQUIRED_SMOKE_READY_EVIDENCE: &[&str] = &[
    "manifest_assertions",
    "scenario_labeled_real_alice_launch_path",
    "manifest_log_window_and_startup_screenshot_evidence",
    "specific_alice_window_detected",
    "activate_alice_window_ui_action",
    "save_project_desktop_shortcut_dispatch",
    "run_world_desktop_shortcut_dispatch",
    "run_world_desktop_window_observed",
    "place_object_precondition_no_go_probe",
    "edit_procedure_precondition_no_go_probe",
    "run_world_precondition_no_go_probe",
    "ui_action_contract_artifact",
    "action_contract_expectations_for_place_edit_run_and_save",
    "explicit_failure_when_ui_actions_are_not_automated",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, scenario_collection: &str, id: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(scenario_collection)
        .join(format!("{id}.yaml"))
}

fn read_eatme_scenario(path: &Path) -> EatmeScenarioAsset {
    let content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
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

fn assert_not_contains_any(label: &str, text: &str, needles: &[String]) {
    let normalized_text = normalize_whitespace(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize_whitespace(needle).to_lowercase()))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "{label} contains non-portable wording: {present:?}"
    );
}

fn forbidden_internal_shorthand() -> Vec<String> {
    vec![
        format!("{}{}", "la", "ne"),
        format!("{}{}", "lesson-", "path"),
    ]
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
