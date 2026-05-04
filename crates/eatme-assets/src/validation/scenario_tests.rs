use super::*;
use crate::schema::{
    EatmeScenarioAcceptanceCriterion, EatmeScenarioLauncher, EatmeScenarioRealAlice,
    EatmeScenarioSmokeReady, EatmeScenarioStep, GadugiScenarioAssertion, GadugiScenarioStep,
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
                    "cd /home/azureuser/src/eatme && cargo run -q -p eatme-cli -- assets validate --json".into(),
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
