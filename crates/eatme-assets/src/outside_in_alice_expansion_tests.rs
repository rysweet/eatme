use crate::schema::EatmeScenarioAsset;
use crate::validation::validate_persona_crew_against_scenario_assets;
use crate::{generate_gadugi_adapter_yaml, validate_assets, validate_scenario_asset};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SCENARIO_ASSET_COUNT: usize = 53;

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
    let mut failures = Vec::new();

    if report.scenario_asset_count != EXPECTED_SCENARIO_ASSET_COUNT {
        failures.push(format!(
            "expected {EXPECTED_SCENARIO_ASSET_COUNT} scenario YAML assets after adding four canonical eatme scenarios and four generated Gadugi adapters, got {}",
            report.scenario_asset_count
        ));
    }
    if !report.passed {
        failures.push(format!(
            "expanded asset inventory must validate cleanly: {:?}",
            report.errors
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
            continue;
        }

        for path in [&eatme_path, &gadugi_path] {
            let scenario_report = validate_scenario_asset(path).unwrap();
            if !scenario_report.passed {
                failures.push(format!(
                    "{} must validate: {:?}",
                    path.display(),
                    scenario_report.errors
                ));
            }
        }

        let generated = generate_gadugi_adapter_yaml(&root, &eatme_path).unwrap();
        let committed = fs::read_to_string(&gadugi_path).unwrap();
        if generated != committed {
            failures.push(format!(
                "{} must match the generated adapter for {}",
                gadugi_path.display(),
                eatme_path.display()
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
fn docs_describe_expanded_inventory_as_committed_not_planned() {
    let root = repository_root();
    let docs = [
        root.join("docs/student-missions.md"),
        root.join("docs/instructor-missions.md"),
        root.join("docs/persona-assets.md"),
        root.join("docs/alice-lesson-smoke.md"),
        root.join("docs/generated-asset-consistency.md"),
    ];
    let mut combined = String::new();
    for path in docs {
        combined.push_str(&format!("\n--- {} ---\n", path.display()));
        combined.push_str(&fs::read_to_string(&path).unwrap());
    }

    for target in TARGET_SCENARIOS {
        assert!(
            combined.contains(target.id),
            "student/instructor/persona docs must mention {}",
            target.id
        );
    }
    for persona in [
        "setup-support-specialist",
        "alice-2-migration-mentor",
        "vr-player-tester",
        "model-texture-importer",
        "data-detective",
        "immersive-camera-director",
        "game-narrative-designer",
    ] {
        assert!(
            combined.contains(persona),
            "student/instructor/persona docs must mention persona {persona}"
        );
    }

    assert!(
        combined.contains("53 scenario YAML files"),
        "docs must describe the expanded committed 53-file scenario inventory"
    );
    assert!(
        combined.contains("26 canonical"),
        "docs must describe the expanded committed 26 canonical eatme scenarios"
    );
    assert!(
        combined.contains("26 generated"),
        "docs must describe the expanded committed 26 generated Gadugi adapters"
    );
    assert!(
        !combined.contains("Target expansion lanes")
            && !combined.contains("planned expansion")
            && !combined.contains("planned expanded inventory")
            && !combined.contains("target expansion"),
        "docs must be updated from planned-language to committed inventory language once the scenarios land"
    );
}

#[test]
fn lesson_path_evidence_contracts_stay_explicit_and_honest() {
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

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, lane: &str, id: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(lane)
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

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
