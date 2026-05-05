use crate::schema::EatmeScenarioAsset;
use crate::{generate_gadugi_adapter_yaml, validate_assets, validate_scenario_asset};
use std::fs;
use std::path::{Path, PathBuf};

const WORKSHOP_SCENARIO_ID: &str = "workshop-facilitator-live-studio";
const EXPECTED_SCENARIO_ASSET_COUNT_AFTER_WORKSHOP: usize = 55;
const EXPECTED_INSTRUCTORS: &[&str] = &["workshop-facilitator", "studio-facilitator"];
const EXPECTED_STUDENTS: &[&str] = &[
    "creative-storyteller",
    "collaborative-peer-mentor",
    "reflective-debugger",
];
const REQUIRED_EDITABLE_FIELDS: &[&str] = &[
    "resource_basis",
    "purpose",
    "agentic_test_prompt",
    "acceptance_criteria",
    "acceptance_probes",
    "rubric",
    "avoid",
];
const FORBIDDEN_PORTABILITY_OR_OVERCLAIM_PHRASES: &[&str] = &[
    concat!("la", "ne"),
    concat!("lesson", "-path"),
    concat!("full UI ", "automation"),
    concat!("automated creative ", "assessment"),
    concat!("learner-world ", "grading"),
    concat!("complete Alice ", "coverage"),
];

#[test]
fn workshop_facilitator_live_studio_assets_exist_validate_and_have_fresh_gadugi_adapter() {
    let root = repository_root();
    let eatme_path = scenario_path(&root, "eatme", WORKSHOP_SCENARIO_ID);
    let gadugi_path = scenario_path(&root, "gadugi", WORKSHOP_SCENARIO_ID);
    let mut failures = Vec::new();

    if !eatme_path.is_file() {
        failures.push(format!(
            "{} must exist as the canonical editable Alice workshop scenario",
            eatme_path.display()
        ));
    }
    if !gadugi_path.is_file() {
        failures.push(format!(
            "{} must exist as the generated Gadugi adapter",
            gadugi_path.display()
        ));
    }

    if eatme_path.is_file() {
        let report = validate_scenario_asset(&eatme_path).unwrap();
        if !report.passed {
            failures.push(format!(
                "{} must validate: {:?}",
                eatme_path.display(),
                report.errors
            ));
        }
    }
    if gadugi_path.is_file() {
        let report = validate_scenario_asset(&gadugi_path).unwrap();
        if !report.passed {
            failures.push(format!(
                "{} must validate: {:?}",
                gadugi_path.display(),
                report.errors
            ));
        }
    }
    if eatme_path.is_file() && gadugi_path.is_file() {
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

    let report = validate_assets(&root).unwrap();
    if report.scenario_asset_count != EXPECTED_SCENARIO_ASSET_COUNT_AFTER_WORKSHOP {
        failures.push(format!(
            "expected {EXPECTED_SCENARIO_ASSET_COUNT_AFTER_WORKSHOP} scenario YAML assets after adding the workshop canonical scenario and generated Gadugi adapter, got {}",
            report.scenario_asset_count
        ));
    }
    if !report.passed {
        failures.push(format!(
            "asset inventory must validate cleanly after workshop coverage lands: {:?}",
            report.errors
        ));
    }

    assert!(
        failures.is_empty(),
        "workshop-facilitator-live-studio asset workflow contract failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn workshop_facilitator_live_studio_connects_the_alice_crew_personas_to_a_scenario() {
    let root = repository_root();
    let crew = fs::read_to_string(root.join("assets/personas/alice-user-crew.yaml")).unwrap();
    let scenario = read_required_workshop_scenario(&root);
    let mut failures = Vec::new();

    assert_contains_all(
        "alice-user-crew workshop marker",
        &crew,
        &[
            "id: workshop-facilitator-live-studio",
            "instructors: [workshop-facilitator, studio-facilitator]",
            "students: [creative-storyteller, collaborative-peer-mentor, reflective-debugger]",
            "visible participant progress and reflection",
        ],
    );

    if scenario.id != WORKSHOP_SCENARIO_ID {
        failures.push(format!(
            "scenario id must be {WORKSHOP_SCENARIO_ID}, got {}",
            scenario.id
        ));
    }
    if scenario.kind != "instructor_agentic_flow" {
        failures.push(format!(
            "{WORKSHOP_SCENARIO_ID} kind must be instructor_agentic_flow, got {}",
            scenario.kind
        ));
    }
    let Some(personas) = scenario.personas.as_ref() else {
        panic!("{WORKSHOP_SCENARIO_ID} must declare instructor and student personas");
    };
    for instructor in EXPECTED_INSTRUCTORS {
        if !personas
            .instructors
            .iter()
            .any(|actual| actual == instructor)
        {
            failures.push(format!(
                "{WORKSHOP_SCENARIO_ID} must include instructor persona {instructor}"
            ));
        }
    }
    for student in EXPECTED_STUDENTS {
        if !personas.students.iter().any(|actual| actual == student) {
            failures.push(format!(
                "{WORKSHOP_SCENARIO_ID} must include student persona {student}"
            ));
        }
    }
    let Some(flow) = scenario.agentic_flow.as_ref() else {
        panic!("{WORKSHOP_SCENARIO_ID} must define agentic_flow");
    };
    if flow.focus != "facilitating-live-studio-workshops" {
        failures.push(format!(
            "{WORKSHOP_SCENARIO_ID} agentic_flow.focus must be facilitating-live-studio-workshops, got {}",
            flow.focus
        ));
    }
    if flow.prompt_source
        != "assets/scenarios/eatme/workshop-facilitator-live-studio.yaml#agentic_test_prompt"
    {
        failures.push(format!(
            "{WORKSHOP_SCENARIO_ID} prompt_source must point at the canonical editable scenario asset"
        ));
    }
    for field in REQUIRED_EDITABLE_FIELDS {
        if !flow.non_coder_editable.iter().any(|actual| actual == field) {
            failures.push(format!(
                "{WORKSHOP_SCENARIO_ID} non_coder_editable must include {field}"
            ));
        }
    }
    for output in [
        "facilitation_plan",
        "participant_checkpoint_board",
        "showcase_notes",
    ] {
        if !flow.expected_outputs.iter().any(|actual| actual == output) {
            failures.push(format!(
                "{WORKSHOP_SCENARIO_ID} expected_outputs must include {output}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "workshop persona-to-scenario coverage contract failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn workshop_facilitator_live_studio_defines_reviewable_student_progress_evidence() {
    let root = repository_root();
    let scenario_path = scenario_path(&root, "eatme", WORKSHOP_SCENARIO_ID);
    let scenario = read_required_workshop_scenario(&root);
    let scenario_yaml = fs::read_to_string(&scenario_path).unwrap();

    assert_contains_all(
        "workshop scenario YAML",
        &scenario_yaml,
        &[
            "short live-studio flow",
            "timeboxed",
            "checkpoint",
            "helper",
            "recovery",
            "showcase",
            "visible participant progress",
            "peer feedback",
            "non-coder",
        ],
    );
    assert!(
        scenario.acceptance_criteria.len() >= 3,
        "{WORKSHOP_SCENARIO_ID} must include at least three Given/When/Then acceptance criteria"
    );
    assert!(
        scenario.acceptance_probes.len() >= 4,
        "{WORKSHOP_SCENARIO_ID} must include review probes for workshop facilitation evidence"
    );
    assert!(
        scenario.rubric.len() >= 3,
        "{WORKSHOP_SCENARIO_ID} must include a small rubric for facilitation, student progress, and recovery evidence"
    );
    assert!(
        scenario.steps.iter().any(|step| {
            step.id == "agentic-instructor-review"
                && step.command.contains("agentic")
                && step.command.contains(WORKSHOP_SCENARIO_ID)
        }),
        "{WORKSHOP_SCENARIO_ID} must include an agentic instructor review step using the editable scenario asset"
    );
}

#[test]
fn workshop_facilitator_live_studio_stays_portable_and_honest_about_coverage_limits() {
    let root = repository_root();
    let scenario_paths = [
        scenario_path(&root, "eatme", WORKSHOP_SCENARIO_ID),
        scenario_path(&root, "gadugi", WORKSHOP_SCENARIO_ID),
    ];

    for path in scenario_paths {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("reading {} failed: {error}", path.display()));
        let normalized = normalize_whitespace(&text).to_ascii_lowercase();
        for phrase in FORBIDDEN_PORTABILITY_OR_OVERCLAIM_PHRASES {
            assert!(
                !normalized.contains(&phrase.to_ascii_lowercase()),
                "{} must not use internal shorthand or overclaim phrase {phrase:?}",
                path.display()
            );
        }
        let runtime_markers = [
            "alice launch-smoke",
            "xvfb",
            "wmctrl",
            concat!("automated creative ", "grading"),
            concat!("automated learner-world ", "grading"),
            concat!("learner-world ", "assessment"),
        ];
        for runtime_marker in runtime_markers {
            assert!(
                !normalized.contains(runtime_marker),
                "{} must stay at the editable asset and agentic review boundary, not claim runtime automation or grading: found {runtime_marker:?}",
                path.display()
            );
        }
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, namespace: &str, id: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(namespace)
        .join(format!("{id}.yaml"))
}

fn read_required_workshop_scenario(root: &Path) -> EatmeScenarioAsset {
    let path = scenario_path(root, "eatme", WORKSHOP_SCENARIO_ID);
    let content = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "{} must exist before workshop persona-to-scenario coverage can pass: {error}",
            path.display()
        )
    });
    serde_yaml::from_str(&content).unwrap_or_else(|error| {
        panic!(
            "{} must parse as Eatme scenario YAML: {error}",
            path.display()
        )
    })
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
        "{label} is missing required contract language: {missing:?}"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
