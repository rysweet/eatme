use super::*;
use crate::schema::{
    EatmeScenarioAcceptanceCriterion, EatmeScenarioAgenticFlow, EatmeScenarioLauncher,
    EatmeScenarioRealAlice, EatmeScenarioResource, EatmeScenarioRubricCriterion,
    EatmeScenarioSmokeReady, EatmeScenarioStep, ScenarioAdapter, ScenarioCapabilities,
    ScenarioPersonas,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[test]
fn rejects_malformed_eatme_scenario_asset() {
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "not valid".into(),
        title: "".into(),
        ..EatmeScenarioAsset::default()
    };
    let report = validate_eatme_scenario(Path::new("bad.yaml"), &scenario, None, &[]);
    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("must be kebab-case"))
    );
}

#[test]
fn scenario_validation_errors_include_path_or_scenario_id() {
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "building-a-scene-first-world".into(),
        kind: "alice_lesson_smoke".into(),
        owner: "eatme".into(),
        launcher: Some(EatmeScenarioLauncher {
            command: "alice launch-smoke".into(),
            scenario: "building-a-scene-first-world".into(),
        }),
        ..EatmeScenarioAsset::default()
    };
    let path = Path::new("assets/scenarios/eatme/building-a-scene-first-world.yaml");
    let report = validate_eatme_scenario(path, &scenario, None, &[]);

    assert!(!report.passed);
    assert!(
        report.errors.iter().all(|error| {
            error.contains("building-a-scene-first-world")
                || error.contains("assets/scenarios/eatme/building-a-scene-first-world.yaml")
        }),
        "each schema error should identify the scenario id or asset path: {:?}",
        report.errors
    );
}

#[test]
fn lesson_smoke_requires_real_alice_gate() {
    let mut scenario = valid_lesson_smoke("code-editor-first-run");
    scenario.real_alice = None;

    let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario, None, &[]);
    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("real_alice.gated_by"))
    );
}

#[test]
fn known_lesson_smoke_requires_lesson_kind() {
    let mut scenario = valid_lesson_smoke("code-editor-first-run");
    scenario.kind.clear();

    let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario, None, &[]);

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("kind must be alice_lesson_smoke"))
    );
}

#[test]
fn lesson_smoke_rejects_missing_persona_references() {
    let mut scenario = valid_lesson_smoke("code-editor-first-run");
    scenario.personas = Some(ScenarioPersonas {
        instructors: vec!["missing-instructor".into(), "curious-novice".into()],
        students: vec!["missing-student".into(), "debug-coach".into()],
    });
    let persona_index = PersonaReferenceIndex {
        instructors: BTreeSet::from(["debug-coach".into()]),
        students: BTreeSet::from(["curious-novice".into()]),
        all: BTreeSet::from(["debug-coach".into(), "curious-novice".into()]),
    };

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/code-editor-first-run.yaml"),
        &scenario,
        Some(&persona_index),
        &[],
    );

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("missing instructor persona missing-instructor"))
    );
    assert!(
        report.errors.iter().any(
            |error| error.contains("missing instructor persona curious-novice with wrong role")
        )
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("missing student persona missing-student"))
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("missing student persona debug-coach with wrong role"))
    );
}

#[test]
fn lesson_smoke_rejects_personas_without_crew_index() {
    let scenario = valid_lesson_smoke("code-editor-first-run");

    let report = validate_eatme_scenario(
        Path::new("scenarios/eatme/code-editor-first-run.yaml"),
        &scenario,
        None,
        &[],
    );

    assert!(!report.passed);
    assert!(report.errors.iter().any(|error| {
        error.contains("declares personas but no persona crew asset could be located")
    }));
}

#[test]
fn malformed_discovered_persona_yaml_is_returned_as_parse_error() {
    let case_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-assets-tests/malformed-discovered-persona");
    let scenario_dir = case_dir.join("assets/scenarios/eatme");
    let persona_dir = scenario_dir.join("personas");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&persona_dir).unwrap();
    fs::write(persona_dir.join("bad.yaml"), "personas: [\n").unwrap();
    let scenario_path = scenario_dir.join("code-editor-first-run.yaml");
    fs::write(
        &scenario_path,
        "schema_version: eatme.scenario/v1\nid: code-editor-first-run\ntitle: Code Editor\npurpose: Test\n",
    )
    .unwrap();

    let error = validate_scenario_asset(&scenario_path).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("parsing persona crew YAML"), "{message}");
    assert!(message.contains("bad.yaml"), "{message}");
    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn accepts_instructor_agentic_flow_asset() {
    let scenario = instructor_agentic_scenario("instructor-exercise-builder");

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/instructor-exercise-builder.yaml"),
        &scenario,
        None,
        &[],
    );

    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(report.assertion_count, 1);
}

#[test]
fn instructor_agentic_flow_rejects_desktop_runtime_ownership() {
    let mut scenario = instructor_agentic_scenario("instructor-classroom-setup");
    scenario.steps.push(EatmeScenarioStep {
        id: "launch-directly".into(),
        command: "Xvfb :99 && alice launch-smoke --scenario building".into(),
        evidence: vec!["desktop opened".into()],
    });

    let report =
        validate_eatme_scenario(Path::new("bad-instructor-flow.yaml"), &scenario, None, &[]);

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("instructor_agentic_flow") && error.contains("Alice desktop runtime")
        }),
        "{:?}",
        report.errors
    );
}

#[test]
fn lesson_smoke_rejects_overclaimed_ui_and_assessment_evidence() {
    let mut scenario = valid_lesson_smoke("real-alice-launch-smoke");
    scenario.purpose =
        "Proves full UI automation, creative assessment, and learner-world grading.".into();
    scenario.smoke_ready = Some(EatmeScenarioSmokeReady {
        evidence: vec![
            "full UI automation passed".into(),
            "creative assessment completed".into(),
            "learner-world grading completed".into(),
        ],
    });
    scenario
        .acceptance_criteria
        .push(EatmeScenarioAcceptanceCriterion {
            given: "a real Alice launch smoke run".into(),
            when: "reviewers inspect the manifest".into(),
            then: "they may treat launch smoke as full UI automation and learner-world grading"
                .into(),
        });
    scenario.steps[0].evidence.push(
        "manifest assertions prove full UI automation, creative assessment, and learner-world grading"
            .into(),
    );

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/real-alice-launch-smoke.yaml"),
        &scenario,
        Some(&persona_index_for(&scenario)),
        &[],
    );

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("launch smoke")
                && error.contains("full UI automation")
                && error.contains("creative assessment")
                && error.contains("learner-world grading")
        }),
        "launch-smoke validation must fail loudly when evidence overclaims the boundary: {:?}",
        report.errors
    );
}

#[test]
fn lesson_smoke_rejects_overclaim_even_with_unrelated_honest_marker() {
    let mut scenario = valid_lesson_smoke("real-alice-launch-smoke");
    scenario.purpose =
        "Do not claim creative assessment without proof. This run proves full UI automation."
            .into();

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/real-alice-launch-smoke.yaml"),
        &scenario,
        Some(&persona_index_for(&scenario)),
        &[],
    );

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("launch smoke") && error.contains("full UI automation")
        }),
        "unrelated honest markers must not hide overclaimed launch-smoke evidence: {:?}",
        report.errors
    );
}

#[test]
fn real_ui_action_contract_requires_explicit_unimplemented_boundary_language() {
    let mut scenario = valid_lesson_smoke("first-lessons-real-ui-actions");
    scenario.kind = "alice_real_ui_action_contract".into();
    scenario.purpose =
        "Launches Alice and verifies a student first-lesson object/code/run/save path.".into();
    scenario.artifacts.insert(
        "ui_action_contract".into(),
        "runs/first-lessons/${RUN_ID}/ui-action-contract.json".into(),
    );
    scenario.steps[0].id = "launch-real-ui-action-contract".into();
    scenario.steps[0].evidence = vec![
        "manifest assertions.real_alice_execution_evidence is present".into(),
        "manifest assertions include specific_alice_window_detected".into(),
        "manifest assertions include place_object_ui_action".into(),
        "manifest assertions include edit_procedure_ui_action".into(),
        "manifest assertions include run_world_ui_action".into(),
        "manifest assertions include save_project_ui_action".into(),
        "manifest assertions include ui_action_artifact_captured".into(),
        "ui-action-contract.json exists and is non-empty".into(),
    ];
    scenario.unsupported_policy = "Fail loudly when prerequisites are unavailable.".into();

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/first-lessons-real-ui-actions.yaml"),
        &scenario,
        Some(&persona_index_for(&scenario)),
        &[],
    );

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("ui_action_automation_unimplemented")
                && error.contains("not full UI automation")
                && error.contains("not creative assessment")
                && error.contains("not learner-world grading")
        }),
        "real UI action contracts must state the current unimplemented boundary explicitly: {:?}",
        report.errors
    );
}

#[test]
fn instructor_agentic_flow_rejects_automated_creative_or_learner_world_grading_claims() {
    let mut scenario = instructor_agentic_scenario("instructor-lesson-materials-remix");
    scenario.purpose =
        "Automatically grades learner worlds and creative quality for instructor materials.".into();
    scenario.agentic_test_prompt =
        "Produce a lesson packet and assign automated creative grades to learner worlds.".into();
    scenario.acceptance_probes = vec![
        "Output includes an automated creative assessment score.".into(),
        "Output grades learner worlds without instructor review.".into(),
    ];
    scenario.rubric[0].evidence = vec![
        "Automated learner-world grading is present.".into(),
        "Creative assessment is scored by the agent.".into(),
    ];
    scenario.avoid = vec!["Avoid missing the automated grade.".into()];

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/instructor-lesson-materials-remix.yaml"),
        &scenario,
        None,
        &[],
    );

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("instructor_agentic_flow")
                && error.contains("automated creative grading")
                && error.contains("learner-world assessment")
        }),
        "instructor lesson-material flows must reject automated grading claims: {:?}",
        report.errors
    );
}

#[path = "scenario/gadugi_tests.rs"]
mod gadugi_tests;

fn valid_lesson_smoke(id: &str) -> EatmeScenarioAsset {
    EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: id.into(),
        title: "Code Editor First Run".into(),
        kind: "alice_lesson_smoke".into(),
        owner: "eatme".into(),
        resource_basis: vec![EatmeScenarioResource {
            name: "Alice lesson".into(),
            url: "https://www.alice.org/resources/".into(),
            ..EatmeScenarioResource::default()
        }],
        purpose: "launches through the real Alice smoke harness".into(),
        launcher: Some(EatmeScenarioLauncher {
            command: "alice launch-smoke".into(),
            scenario: id.into(),
        }),
        real_alice: Some(EatmeScenarioRealAlice {
            gated_by: "EATME_REAL_ALICE=1".into(),
        }),
        personas: Some(ScenarioPersonas {
            instructors: vec!["debug-coach".into()],
            students: vec!["curious-novice".into()],
        }),
        capabilities: Some(ScenarioCapabilities {
            required: vec!["rust-cli".into()],
            optional: vec!["glxinfo".into()],
        }),
        adapter: Some(ScenarioAdapter {
            targets: vec!["eatme-cli".into()],
        }),
        smoke_ready: Some(EatmeScenarioSmokeReady {
            evidence: vec!["manifest assertions".into()],
        }),
        acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
            given: "dependencies are available".into(),
            when: "the scenario launches".into(),
            then: "the manifest records the scenario id".into(),
        }],
        steps: vec![EatmeScenarioStep {
            id: "launch-smoke".into(),
            command: format!("eatme alice launch-smoke --scenario {id}"),
            evidence: vec![
                "manifest scenario_id matches".into(),
                "manifest assertions.real_alice_execution_evidence exists".into(),
            ],
        }],
        timeouts: BTreeMap::from([("launch_seconds".into(), 120)]),
        artifacts: BTreeMap::from([
            ("manifest".into(), "runs/code/manifest.json".into()),
            (
                "screenshot".into(),
                "runs/code/screenshots/startup.png".into(),
            ),
            ("log".into(), "runs/code/alice.log".into()),
        ]),
        unsupported_policy: "fail loudly when prerequisites are unavailable".into(),
        ..EatmeScenarioAsset::default()
    }
}

fn instructor_agentic_scenario(id: &str) -> EatmeScenarioAsset {
    EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: id.into(),
        title: "Instructor Exercise Builder".into(),
        kind: "instructor_agentic_flow".into(),
        owner: "eatme".into(),
        purpose: "help instructors create an Alice exercise from existing resources".into(),
        resource_basis: vec![EatmeScenarioResource {
            name: "Alice.org Programming in Alice".into(),
            url: "https://www.alice.org/resources/lessons/programming-in-alice/".into(),
            use_note: "Ground exercise concepts in procedures, parameters, and run/revise.".into(),
        }],
        personas: Some(ScenarioPersonas {
            instructors: vec!["exercise-forger".into()],
            students: vec!["curious-novice".into()],
        }),
        agentic_flow: Some(EatmeScenarioAgenticFlow {
            focus: "creating-exercises".into(),
            instructor_goal: "draft a classroom-ready Alice exercise".into(),
            prompt_source: "assets/scenarios/eatme/instructor-exercise-builder.yaml".into(),
            non_coder_editable: vec!["agentic_test_prompt".into(), "rubric".into()],
            expected_outputs: vec!["exercise brief".into(), "student evidence checklist".into()],
        }),
        agentic_test_prompt: "Act as the instructor QA agent and produce an exercise brief.".into(),
        acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
            given: "an Alice.org lesson concept".into(),
            when: "the instructor agent drafts materials".into(),
            then: "the output names concept evidence and learner choice".into(),
        }],
        acceptance_probes: vec!["Exercise has concept, starter task, and extension.".into()],
        rubric: vec![EatmeScenarioRubricCriterion {
            criterion: "Concept evidence".into(),
            evidence: vec!["Student links a visible world behavior to a concept.".into()],
        }],
        avoid: vec!["Do not require exact coordinates or private implementation details.".into()],
        steps: vec![
            EatmeScenarioStep {
                id: "validate-assets".into(),
                command: "cargo run -q -p eatme-cli -- assets validate --json".into(),
                evidence: vec!["asset validation passes".into()],
            },
            EatmeScenarioStep {
                id: "agentic-instructor-review".into(),
                command: "agentic review using this YAML prompt and acceptance probes".into(),
                evidence: vec!["review returns maintainable lesson materials".into()],
            },
        ],
        timeouts: BTreeMap::from([("agentic_seconds".into(), 900)]),
        artifacts: BTreeMap::from([("lesson_brief".into(), "agentic://lesson-brief".into())]),
        unsupported_policy: "Fail visibly if the agent cannot read this editable asset.".into(),
        ..EatmeScenarioAsset::default()
    }
}

fn persona_index_for(scenario: &EatmeScenarioAsset) -> PersonaReferenceIndex {
    let personas = scenario.personas.as_ref().unwrap();
    let instructors = personas
        .instructors
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let students = personas.students.iter().cloned().collect::<BTreeSet<_>>();
    let all = instructors
        .iter()
        .chain(students.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    PersonaReferenceIndex {
        instructors,
        students,
        all,
    }
}
