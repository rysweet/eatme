use crate::schema::EatmeScenarioAsset;
use crate::validation::validate_persona_crew_against_scenario_assets;
use crate::{generate_gadugi_adapters, validate_assets};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SCENARIO_ASSET_COUNT: usize = 83;

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

#[test]
fn alice_outside_in_expansion_assets_exist_validate_and_have_fresh_gadugi_adapters() {
    let root = repository_root();
    let report = validate_assets(&root).unwrap();
    let gadugi_report = generate_gadugi_adapters(&root, true).unwrap();
    let mut failures = Vec::new();

    if report.scenario_asset_count != EXPECTED_SCENARIO_ASSET_COUNT {
        failures.push(format!(
            "expected {EXPECTED_SCENARIO_ASSET_COUNT} scenario YAML assets after adding outside-in expansion and workshop coverage assets, got {}",
            report.scenario_asset_count
        ));
    }
    if !report.passed {
        failures.push(format!(
            "expanded asset inventory must validate cleanly: {:?}",
            report.errors
        ));
    }
    if !gadugi_report.passed {
        failures.push(format!(
            "expanded Gadugi adapters must be fresh: {:?}",
            gadugi_report.errors
        ));
    }

    for target in TARGET_SCENARIOS {
        let eatme_path = scenario_path(&root, "eatme", target.id);
        let gadugi_path = scenario_path(&root, "gadugi", target.id);

        if !eatme_path.is_file() {
            failures.push(format!(
                "{} must exist as the canonical outside-in Alice scenario",
                eatme_path.display()
            ));
            continue;
        }
        if !gadugi_path.is_file() {
            failures.push(format!(
                "{} must exist as the generated Gadugi adapter",
                gadugi_path.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "outside-in Alice expansion asset contract failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn target_scenarios_use_required_personas_and_real_alice_gate_without_ci_auto_run() {
    let root = repository_root();
    let mut failures = Vec::new();

    for target in TARGET_SCENARIOS {
        let eatme_path = scenario_path(&root, "eatme", target.id);
        if !eatme_path.is_file() {
            failures.push(format!("{} is missing", eatme_path.display()));
            continue;
        }

        let scenario = read_eatme_scenario(&eatme_path);
        if scenario.kind != "alice_lesson_smoke" {
            failures.push(format!(
                "{} kind must be alice_lesson_smoke, got {}",
                target.id, scenario.kind
            ));
        }
        if scenario
            .launcher
            .as_ref()
            .map(|launcher| launcher.scenario.as_str())
            != Some(target.id)
        {
            failures.push(format!(
                "{} launcher.scenario must match the scenario id",
                target.id
            ));
        }
        if scenario
            .real_alice
            .as_ref()
            .map(|real_alice| real_alice.gated_by.as_str())
            != Some("EATME_REAL_ALICE=1")
        {
            failures.push(format!(
                "{} must keep real Alice execution behind EATME_REAL_ALICE=1",
                target.id
            ));
        }
        if !scenario
            .steps
            .iter()
            .any(|step| step.command.contains("EATME_REAL_ALICE=1"))
        {
            failures.push(format!(
                "{} must document the explicit manual real-Alice gate in a smoke step",
                target.id
            ));
        }
        if !scenario.steps.iter().any(|step| {
            step.command.contains("alice launch-smoke")
                && step
                    .evidence
                    .iter()
                    .any(|evidence| evidence.contains("real_alice_execution_evidence"))
        }) {
            failures.push(format!(
                "{} launch smoke evidence must inspect manifest assertions.real_alice_execution_evidence",
                target.id
            ));
        }

        let Some(personas) = scenario.personas.as_ref() else {
            failures.push(format!(
                "{} must declare instructor/student personas",
                target.id
            ));
            continue;
        };
        for instructor in target.instructors {
            if !personas
                .instructors
                .iter()
                .any(|actual| actual == instructor)
            {
                failures.push(format!(
                    "{} must include instructor persona {}",
                    target.id, instructor
                ));
            }
        }
        for student in target.students {
            if !personas.students.iter().any(|actual| actual == student) {
                failures.push(format!(
                    "{} must include student persona {}",
                    target.id, student
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "outside-in Alice expansion scenario contracts failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn missing_expansion_prompt_card_scenario_assets_fail_loudly() {
    let root = repository_root();
    let scenario_asset_ids = BTreeSet::from(["building-a-scene-first-world".to_string()]);
    let report = validate_persona_crew_against_scenario_assets(
        &root.join("assets/personas/alice-user-crew.yaml"),
        Some(&scenario_asset_ids),
    )
    .unwrap();

    assert!(!report.passed);
    for target in TARGET_SCENARIOS {
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("prompt card")
                    && error.contains(target.id)
                    && error.contains("missing scenario asset")),
            "missing expansion prompt-card scenario asset {} must be reported explicitly; got {:?}",
            target.id,
            report.errors
        );
    }
}

#[test]
fn starter_project_preflight_contract_names_real_action_evidence_without_overclaiming() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "starter-project-open-save-export-preflight",
    ))
    .unwrap();

    assert_contains_all(
        "starter-project-open-save-export-preflight contract",
        &contract,
        &[
            "real Alice action evidence",
            "opened starter project",
            "manifest/log/window/screenshot evidence",
            "inspectable action evidence",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
    assert_not_contains_any(
        "starter-project-open-save-export-preflight contract",
        &contract,
        &forbidden_internal_shorthand(),
    );
}

#[test]
fn first_lesson_evidence_contracts_stay_explicit_and_honest() {
    let root = repository_root();
    let student_contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "first-lessons-real-ui-actions",
    ))
    .unwrap();
    let instructor_contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "instructor-lesson-materials-remix",
    ))
    .unwrap();
    let launch_contract =
        fs::read_to_string(scenario_path(&root, "eatme", "real-alice-launch-smoke")).unwrap();
    let docs = [
        root.join("docs/alice-lesson-smoke.md"),
        root.join("docs/student-missions.md"),
        root.join("docs/instructor-missions.md"),
        root.join("docs/persona-assets.md"),
        root.join("docs/index.md"),
    ]
    .into_iter()
    .map(|path| fs::read_to_string(path).unwrap())
    .collect::<Vec<_>>()
    .join("\n");

    assert_contains_all(
        "first-lessons-real-ui-actions contract",
        &student_contract,
        &[
            "scenario-labeled real Alice launch path",
            "manifest, Alice log, window list, and startup screenshot evidence",
            "Alice window detection",
            "ui-action-contract.json",
            "This is launch/action-contract evidence only.",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
        ],
    );
    assert_contains_all(
        "instructor-lesson-materials-remix contract",
        &instructor_contract,
        &[
            "lesson-material remix path",
            "scenario-labeled assets",
            "agentic probes",
            "does not grade learner worlds",
            "assess creativity automatically",
            "automated creative grading",
            "learner-world assessment",
        ],
    );
    assert_contains_all(
        "real-alice-launch-smoke contract",
        &launch_contract,
        &[
            "scenario-labeled launch path",
            "manifest/log/window/screenshot evidence",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
        ],
    );
    assert_contains_all(
        "lesson evidence docs",
        &docs,
        &[
            "first-lessons-real-ui-actions",
            "instructor-lesson-materials-remix",
            "real-alice-launch-smoke",
            "launch/action-contract evidence only",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "does not grade learner worlds or assess creativity automatically",
        ],
    );
}

#[test]
fn teacher_community_sharing_loop_contract_names_handoff_and_honest_boundaries() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "teacher-community-sharing-loop",
    ))
    .unwrap();

    assert_contains_all(
        "teacher-community-sharing-loop contract",
        &contract,
        &[
            "teacher-community share card",
            "classroom handoff note",
            "editable scenario and persona links",
            "attribution",
            "classroom constraints",
            "student evidence",
            "remix feedback prompts",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
            "not a deployed community platform",
        ],
    );
}

#[test]
fn media_audio_cue_storyboard_covers_media_audio_student_persona() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "media-audio-cue-storyboard");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario
        .personas
        .as_ref()
        .expect("media-audio scenario must define personas");

    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert!(
        personas
            .students
            .iter()
            .any(|persona| persona == "media-audio-creator"),
        "media-audio-cue-storyboard must cover media-audio-creator"
    );
    assert_contains_all(
        "media-audio-cue-storyboard contract",
        &contract,
        &[
            "media cue storyboard",
            "student prediction prompt",
            "accessibility fallback note",
            "visible or audible result",
            "student-owned revision",
            "media-audio-creator",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

#[test]
fn lost_robot_debug_museum_covers_reflective_debugger_and_debug_coach() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "lost-robot-debug-museum");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario.personas.as_ref().expect("must define personas");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert!(personas.instructors.iter().any(|p| p == "debug-coach"));
    assert!(personas.students.iter().any(|p| p == "reflective-debugger"));
    assert!(
        personas
            .students
            .iter()
            .any(|p| p == "collaborative-peer-mentor")
    );
    assert_contains_all(
        "lost-robot-debug-museum contract",
        &contract,
        &[
            "debug mystery brief",
            "student debug journal",
            "peer question checkpoint",
            "hypothesis",
            "minimal change",
            "reflective-debugger",
            "collaborative-peer-mentor",
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
