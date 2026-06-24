use super::*;
use crate::schema::{
    GadugiScenarioAgent, GadugiScenarioAgentConfig, GadugiScenarioAssertion, GadugiScenarioAsset,
    GadugiScenarioStep, GadugiStepExpect,
};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::Path;

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
                Value::String("Xvfb :99 & java org.alice.stageide.EntryPoint".into()),
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

#[test]
fn gadugi_launch_smoke_requires_real_execution_evidence_assertion() {
    let scenario = GadugiScenarioAsset {
        name: "Metadata Only Smoke".into(),
        description: "Launches through eatme but only checks scenario metadata.".into(),
        version: "1.0.0".into(),
        steps: vec![GadugiScenarioStep {
            name: "Launch Alice".into(),
            agent: "eatme-cli-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                Value::String(
                    "cargo run -q -p eatme-cli -- alice launch-smoke --scenario code-editor-first-run"
                        .into(),
                ),
            )]),
            expect: Some(GadugiStepExpect {
                exit_code: Some(0),
                stdout_contains: vec!["\"scenario_id\": \"code-editor-first-run\"".into()],
                ..GadugiStepExpect::default()
            }),
            timeout: 1,
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Launch succeeded".into(),
            assertion_type: "command_success".into(),
            agent: "eatme-cli-agent".into(),
            params: BTreeMap::from([("step".into(), Value::String("Launch Alice".into()))]),
        }],
        ..GadugiScenarioAsset::default()
    };

    let report = validate_gadugi_scenario(
        Path::new("assets/scenarios/gadugi/metadata-only.yaml"),
        &scenario,
    );

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("real_alice_execution_evidence")),
        "gadugi launch-smoke validation should reject metadata-only checks: {:?}",
        report.errors
    );
}

#[test]
fn gadugi_scenario_rejects_hardcoded_repo_paths() {
    let hardcoded_checkout = Path::new("/")
        .join("home")
        .join("runner")
        .join("work")
        .join("eatme");
    let scenario = GadugiScenarioAsset {
        name: "Hard-coded repo path".into(),
        description: "Uses an environment-specific checkout path.".into(),
        version: "1.0.0".into(),
        agents: vec![valid_system_agent(".")],
        steps: vec![GadugiScenarioStep {
            name: "Validate assets".into(),
            agent: "eatme-cli-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                Value::String(format!(
                    "cd {} && cargo run -q -p eatme-cli -- assets validate --json",
                    hardcoded_checkout.display()
                )),
            )]),
            expect: Some(GadugiStepExpect {
                exit_code: Some(0),
                stdout_contains: vec!["\"passed\": true".into()],
                ..GadugiStepExpect::default()
            }),
            timeout: 1,
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Validation succeeded".into(),
            assertion_type: "command_success".into(),
            agent: "eatme-cli-agent".into(),
            params: BTreeMap::from([("step".into(), Value::String("Validate assets".into()))]),
        }],
        metadata: valid_gadugi_metadata("assets/scenarios/eatme/real-alice-launch-smoke.yaml"),
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
            .any(|error| error.contains("must not hard-code")),
        "{:?}",
        report.errors
    );
}

#[test]
fn gadugi_scenario_rejects_hardcoded_cwd_paths() {
    let scenario = GadugiScenarioAsset {
        name: "Hard-coded cwd".into(),
        description: "Uses an environment-specific agent cwd.".into(),
        version: "1.0.0".into(),
        agents: vec![valid_system_agent("/home/alice/src/eatme")],
        steps: vec![GadugiScenarioStep {
            name: "Validate assets".into(),
            agent: "eatme-cli-agent".into(),
            action: "execute_command".into(),
            params: BTreeMap::from([(
                "command".into(),
                Value::String(
                    "cd \"${EATME_REPO:-.}\" && cargo run -q -p eatme-cli -- assets validate --json"
                        .into(),
                ),
            )]),
            expect: Some(GadugiStepExpect {
                exit_code: Some(0),
                stdout_contains: vec!["\"passed\": true".into()],
                ..GadugiStepExpect::default()
            }),
            timeout: 1,
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Validation succeeded".into(),
            assertion_type: "command_success".into(),
            agent: "eatme-cli-agent".into(),
            params: BTreeMap::from([("step".into(), Value::String("Validate assets".into()))]),
        }],
        metadata: valid_gadugi_metadata("assets/scenarios/eatme/real-alice-launch-smoke.yaml"),
        ..GadugiScenarioAsset::default()
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
            params: BTreeMap::from([("asset".into(), Value::String("".into()))]),
            ..GadugiScenarioStep::default()
        }],
        assertions: vec![GadugiScenarioAssertion {
            name: "Instructor review completed".into(),
            assertion_type: "agentic_acceptance".into(),
            ..GadugiScenarioAssertion::default()
        }],
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
            .any(|error| error
                .contains("Run instructor review.action agentic_test is not supported")),
        "{:?}",
        report.errors
    );
}

fn valid_system_agent(cwd: &str) -> GadugiScenarioAgent {
    GadugiScenarioAgent {
        name: "eatme-cli-agent".into(),
        agent_type: "system".into(),
        config: GadugiScenarioAgentConfig {
            shell: "bash".into(),
            cwd: cwd.into(),
            timeout: 1,
            capture_output: true,
            ..GadugiScenarioAgentConfig::default()
        },
    }
}

fn valid_gadugi_metadata(source: &str) -> crate::schema::GadugiMetadata {
    crate::schema::GadugiMetadata {
        source_eatme_asset: source.into(),
        generated_by: "test".into(),
        tags: vec!["alice".into(), "eatme".into()],
        priority: "critical".into(),
        author: "eatme".into(),
        test_type: "launch-smoke".into(),
    }
}
