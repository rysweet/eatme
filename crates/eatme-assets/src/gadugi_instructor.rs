use super::*;

pub(super) fn generate_instructor_agentic_adapter(
    scenario: &EatmeScenarioAsset,
    source_asset: String,
    timeout_ms: u64,
) -> Result<String> {
    let agentic_timeout_ms = scenario
        .timeouts
        .get("agentic_seconds")
        .copied()
        .unwrap_or(900)
        * 1000;
    let agentic_flow = scenario.agentic_flow.as_ref();
    let focus = agentic_flow
        .map(|flow| flow.focus.as_str())
        .filter(|focus| !focus.is_empty())
        .unwrap_or(&scenario.id)
        .to_owned();
    let expected_outputs = agentic_flow
        .map(|flow| flow.expected_outputs.clone())
        .unwrap_or_default();
    let validate_step = "Validate editable Alice instructor assets";
    let agentic_step = "Run instructor agentic acceptance review";
    let adapter = GeneratedGadugiAdapter {
        name: format!("Eatme {} Agentic Flow", scenario.title),
        description: format!(
            "Gadugi-compatible instructor acceptance adapter generated from {source_asset}. It keeps the scenario at the editable asset and agentic evidence boundary so non-coders can maintain prompts, probes, and rubrics without changing Rust."
        ),
        version: "1.0.0".into(),
        config: GeneratedConfig {
            timeout: timeout_ms,
            retries: 0,
            parallel: false,
        },
        environment: GeneratedEnvironment {
            requires: Vec::new(),
            optional: vec!["RUN_ID".into(), "EATME_REPO".into()],
        },
        agents: vec![
            GeneratedAgent {
                name: "eatme-cli-agent".into(),
                agent_type: "system".into(),
                config: GeneratedAgentConfig {
                    shell: Some("bash".into()),
                    cwd: Some(".".into()),
                    timeout: 60_000,
                    capture_output: Some(true),
                    persona_asset: None,
                    scenario_asset: None,
                },
            },
            GeneratedAgent {
                name: "instructor-acceptance-agent".into(),
                agent_type: "agentic".into(),
                config: GeneratedAgentConfig {
                    shell: None,
                    cwd: None,
                    timeout: agentic_timeout_ms,
                    capture_output: None,
                    persona_asset: Some("assets/personas/alice-user-crew.yaml".into()),
                    scenario_asset: Some(source_asset.clone()),
                },
            },
        ],
        steps: vec![
            GeneratedStep {
                name: validate_step.into(),
                agent: "eatme-cli-agent".into(),
                action: "execute_command".into(),
                params: BTreeMap::from([(
                    "command".into(),
                    repository_command(
                        "cargo run -q -p eatme-cli -- assets validate --json",
                        &format!("gadugi-{}", scenario.id),
                    ),
                )]),
                expect: GeneratedExpect {
                    exit_code: Some(0),
                    stdout_contains: Some(vec![
                        "\"passed\": true".into(),
                        format!("\"{}\"", scenario.id),
                    ]),
                    output_contains: None,
                },
                timeout: 60_000,
            },
            GeneratedStep {
                name: agentic_step.into(),
                agent: "instructor-acceptance-agent".into(),
                action: "agentic_test".into(),
                params: BTreeMap::from([
                    ("asset".into(), source_asset.clone()),
                    ("prompt".into(), scenario.agentic_test_prompt.clone()),
                    (
                        "acceptance_probes".into(),
                        scenario.acceptance_probes.join("\n"),
                    ),
                ]),
                expect: GeneratedExpect {
                    exit_code: None,
                    stdout_contains: None,
                    output_contains: Some(expected_outputs),
                },
                timeout: agentic_timeout_ms,
            },
        ],
        assertions: vec![
            GeneratedAssertion {
                name: "Assets Validate".into(),
                assertion_type: "command_success".into(),
                agent: "eatme-cli-agent".into(),
                params: BTreeMap::from([("step".into(), validate_step.into())]),
            },
            GeneratedAssertion {
                name: "Instructor Agentic Acceptance Review Covers Probes".into(),
                assertion_type: "agentic_acceptance".into(),
                agent: "instructor-acceptance-agent".into(),
                params: BTreeMap::from([
                    ("step".into(), agentic_step.into()),
                    ("asset".into(), source_asset.clone()),
                ]),
            },
        ],
        metadata: GeneratedMetadata {
            source_eatme_asset: source_asset,
            generated_by: GENERATED_BY.into(),
            tags: vec![
                "alice".into(),
                "eatme".into(),
                "gadugi".into(),
                "outside-in".into(),
                "instructor".into(),
                "agentic".into(),
                focus,
                scenario.id.clone(),
            ],
            priority: "high".into(),
            author: "eatme".into(),
            test_type: "instructor-agentic-flow".into(),
        },
    };
    render_yaml(adapter)
}
