use super::*;
use crate::schema::{
    EatmeScenarioAcceptanceCriterion, EatmeScenarioAgenticFlow, EatmeScenarioLauncher,
    EatmeScenarioRealAlice, EatmeScenarioResource, EatmeScenarioRubricCriterion,
    EatmeScenarioSmokeReady, EatmeScenarioStep, GadugiScenarioAgent, GadugiScenarioAgentConfig,
    GadugiScenarioAssertion, GadugiScenarioStep, ScenarioPersonas,
};
use std::collections::BTreeMap;

#[test]
fn rejects_malformed_eatme_scenario_asset() {
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "not valid".into(),
        title: "".into(),
        ..EatmeScenarioAsset::default()
    };
    let report = validate_eatme_scenario(Path::new("bad.yaml"), &scenario);
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
    let report = validate_eatme_scenario(path, &scenario);

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
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "code-editor-first-run".into(),
        title: "Code Editor First Run".into(),
        kind: "alice_lesson_smoke".into(),
        owner: "eatme".into(),
        purpose: "launches through the real Alice smoke harness".into(),
        launcher: Some(EatmeScenarioLauncher {
            command: "alice launch-smoke".into(),
            scenario: "code-editor-first-run".into(),
        }),
        steps: vec![EatmeScenarioStep {
            id: "launch-smoke".into(),
            command: "eatme alice launch-smoke --scenario code-editor-first-run".into(),
            evidence: vec!["manifest scenario_id matches".into()],
        }],
        timeouts: BTreeMap::from([("launch_seconds".into(), 120)]),
        artifacts: BTreeMap::from([
            ("manifest".into(), "runs/building/manifest.json".into()),
            (
                "screenshot".into(),
                "runs/building/screenshots/startup.png".into(),
            ),
            ("log".into(), "runs/building/alice.log".into()),
        ]),
        unsupported_policy: "fail loudly when prerequisites are unavailable".into(),
        ..EatmeScenarioAsset::default()
    };
    let report = validate_eatme_scenario(Path::new("building.yaml"), &scenario);
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
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "code-editor-first-run".into(),
        title: "Code Editor First Run".into(),
        owner: "eatme".into(),
        purpose: "launches through the real Alice smoke harness".into(),
        launcher: Some(EatmeScenarioLauncher {
            command: "alice launch-smoke".into(),
            scenario: "code-editor-first-run".into(),
        }),
        real_alice: Some(EatmeScenarioRealAlice {
            gated_by: "EATME_REAL_ALICE=1".into(),
        }),
        smoke_ready: Some(EatmeScenarioSmokeReady {
            evidence: vec!["manifest assertions".into()],
        }),
        acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
            given: "dependencies are available".into(),
            when: "the lane launches".into(),
            then: "the manifest records the scenario id".into(),
        }],
        steps: vec![EatmeScenarioStep {
            id: "launch-smoke".into(),
            command: "eatme alice launch-smoke --scenario code-editor-first-run".into(),
            evidence: vec!["manifest scenario_id matches".into()],
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
    };
    let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario);

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("kind must be alice_lesson_smoke"))
    );
}

#[test]
fn accepts_instructor_agentic_flow_asset() {
    let scenario = instructor_agentic_scenario("instructor-exercise-builder");

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/instructor-exercise-builder.yaml"),
        &scenario,
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

    let report = validate_eatme_scenario(Path::new("bad-instructor-flow.yaml"), &scenario);

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
fn gadugi_scenario_rejects_direct_alice_runtime_commands() {
    let scenario = GadugiScenarioAsset {
        name: "Bad Gadugi Alice Runtime Owner".into(),
        description: "Attempts to own Xvfb and Java launch directly instead of using eatme CLI."
            .into(),
        version: "1.0.0".into(),
        steps: vec![GadugiScenarioStep {
            name: "Launch Alice directly".into(),
            agent: "gadugi-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                "Xvfb :99 & java org.alice.stageide.EntryPoint".into(),
            )]),
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Direct runtime command succeeded".into(),
            assertion_type: "command_success".into(),
        }],
        metadata: crate::schema::GadugiScenarioMetadata {
            source_eatme_asset: "assets/scenarios/eatme/real-alice-launch-smoke.yaml".into(),
            generated_by: "test".into(),
        },
        ..GadugiScenarioAsset::default()
    };

    let report = validate_gadugi_scenario(
        Path::new("assets/scenarios/gadugi/bad-direct-runtime.yaml"),
        &scenario,
    );
    assert!(
        !report.passed,
        "gadugi assets must not own Alice runtime details: {:?}",
        report.errors
    );
    assert!(
        report.errors.iter().any(|error| {
            error.contains("gadugi")
                && error.contains("alice launch-smoke")
                && error.contains("runtime")
        }),
        "boundary error should direct gadugi scenarios to the eatme launch-smoke CLI: {:?}",
        report.errors
    );
}

#[test]
fn gadugi_scenario_rejects_hardcoded_repo_paths() {
    let scenario = GadugiScenarioAsset {
        name: "Hard-coded repo path".into(),
        description: "Uses an environment-specific checkout path.".into(),
        version: "1.0.0".into(),
        steps: vec![GadugiScenarioStep {
            name: "Validate assets".into(),
            agent: "eatme-cli-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                "cd /home/runner/work/eatme && cargo run -q -p eatme-cli -- assets validate --json"
                    .into(),
            )]),
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Validation succeeded".into(),
            assertion_type: "command_success".into(),
        }],
        metadata: crate::schema::GadugiScenarioMetadata {
            source_eatme_asset: "assets/scenarios/eatme/real-alice-launch-smoke.yaml".into(),
            generated_by: "test".into(),
        },
        ..GadugiScenarioAsset::default()
    };

    let report = validate_gadugi_scenario(
        Path::new("assets/scenarios/gadugi/hard-coded.yaml"),
        &scenario,
    );

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("must not hard-code"))
    );
}

#[test]
fn gadugi_scenario_rejects_hardcoded_cwd_paths() {
    let scenario = GadugiScenarioAsset {
        name: "Hard-coded cwd".into(),
        description: "Uses an environment-specific agent cwd.".into(),
        version: "1.0.0".into(),
        agents: vec![GadugiScenarioAgent {
            name: "eatme-cli-agent".into(),
            agent_type: "system".into(),
            config: GadugiScenarioAgentConfig {
                cwd: "/home/alice/src/eatme".into(),
            },
        }],
        steps: vec![GadugiScenarioStep {
            name: "Validate assets".into(),
            agent: "eatme-cli-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                "cd \"${EATME_REPO:-.}\" && cargo run -q -p eatme-cli -- assets validate --json"
                    .into(),
            )]),
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Validation succeeded".into(),
            assertion_type: "command_success".into(),
        }],
        metadata: crate::schema::GadugiScenarioMetadata {
            source_eatme_asset: "assets/scenarios/eatme/real-alice-launch-smoke.yaml".into(),
            generated_by: "test".into(),
        },
    };

    let report = validate_gadugi_scenario(
        Path::new("assets/scenarios/gadugi/hard-coded-cwd.yaml"),
        &scenario,
    );

    assert!(!report.passed);
    assert!(report.errors.iter().any(|error| {
        error.contains("eatme-cli-agent.config.cwd") && error.contains("must not hard-code")
    }));
}

#[test]
fn gadugi_agentic_steps_require_editable_asset_contract() {
    let scenario = GadugiScenarioAsset {
        name: "Incomplete Instructor Agentic Adapter".into(),
        description: "Forgets to name the editable prompt asset.".into(),
        version: "1.0.0".into(),
        steps: vec![GadugiScenarioStep {
            name: "Run instructor review".into(),
            agent: "instructor-qa-agent".into(),
            action: "agentic_test".into(),
            params: BTreeMap::from([("asset".into(), "".into())]),
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Instructor review completed".into(),
            assertion_type: "agentic_acceptance".into(),
        }],
        metadata: crate::schema::GadugiScenarioMetadata {
            source_eatme_asset: "assets/scenarios/eatme/instructor-exercise-builder.yaml".into(),
            generated_by: "test".into(),
        },
        ..GadugiScenarioAsset::default()
    };

    let report = validate_gadugi_scenario(
        Path::new("assets/scenarios/gadugi/incomplete-instructor.yaml"),
        &scenario,
    );

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("acceptance_probes")),
        "{:?}",
        report.errors
    );
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
