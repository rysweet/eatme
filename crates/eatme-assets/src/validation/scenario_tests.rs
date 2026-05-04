use super::*;
use crate::schema::{
    EatmeScenarioAcceptanceCriterion, EatmeScenarioLauncher, EatmeScenarioRealAlice,
    EatmeScenarioSmokeReady, EatmeScenarioStep, GadugiScenarioAssertion, GadugiScenarioStep,
    ScenarioAdapter, ScenarioCapabilities, ScenarioPersonas, ScenarioResourceBasis,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn rejects_malformed_eatme_scenario_asset() {
    let scenario = EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: "not valid".into(),
        title: "".into(),
        ..EatmeScenarioAsset::default()
    };
    let report = validate_eatme_scenario(Path::new("bad.yaml"), &scenario, None);
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
    let report = validate_eatme_scenario(path, &scenario, None);

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

    let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario, None);
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

    let report = validate_eatme_scenario(Path::new("code-editor.yaml"), &scenario, None);

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
    );

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains("missing instructor persona missing-instructor") })
    );
    assert!(report.errors.iter().any(|error| {
        error.contains("missing instructor persona curious-novice with wrong role")
    }));
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains("missing student persona missing-student") })
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| { error.contains("missing student persona debug-coach with wrong role") })
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
                serde_yaml::Value::String("Xvfb :99 & java org.alice.stageide.EntryPoint".into()),
            )]),
            ..GadugiScenarioStep::default()
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Direct runtime command succeeded".into(),
            assertion_type: "command_success".into(),
            agent: "gadugi-agent".into(),
            ..GadugiScenarioAssertion::default()
        }],
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

fn valid_lesson_smoke(id: &str) -> EatmeScenarioAsset {
    EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: id.into(),
        title: "Code Editor First Run".into(),
        kind: "alice_lesson_smoke".into(),
        owner: "eatme".into(),
        resource_basis: vec![ScenarioResourceBasis {
            name: "Alice lesson".into(),
            url: "https://www.alice.org/resources/".into(),
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
            when: "the lane launches".into(),
            then: "the manifest records the scenario id".into(),
        }],
        steps: vec![EatmeScenarioStep {
            id: "launch-smoke".into(),
            command: format!("eatme alice launch-smoke --scenario {id}"),
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
    }
}
