use super::*;

pub(super) fn generate_instructor_agentic_adapter(
    scenario: &EatmeScenarioAsset,
    source_asset: String,
    timeout_ms: u64,
    launch_timeout: u64,
    expected_scenario_asset_count: usize,
) -> Result<String> {
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
    let agentic_step = "Validate instructor acceptance review contract";
    let command_steps = scenario
        .steps
        .iter()
        .filter(|step| {
            step.id != "validate-assets"
                && !step
                    .command
                    .trim_start()
                    .starts_with("agentic instructor acceptance review")
        })
        .map(|step| {
            generated_step(
                scenario,
                step,
                &format!("gadugi-{}", scenario.id),
                launch_timeout,
                expected_scenario_asset_count,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let command_assertions = scenario
        .steps
        .iter()
        .filter(|step| {
            step.id != "validate-assets"
                && !step
                    .command
                    .trim_start()
                    .starts_with("agentic instructor acceptance review")
        })
        .map(|step| {
            generated_assertion(
                scenario,
                step,
                "eatme-cli-agent",
                expected_scenario_asset_count,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let mut steps = Vec::new();
    steps.push(GeneratedStep {
        name: validate_step.into(),
        agent: "eatme-cli-agent".into(),
        action: "execute_command".into(),
        params: BTreeMap::from([(
            "command".into(),
            repository_command(
                &format!(
                    "cargo run -q -p eatme-cli -- assets validate --path {source_asset} --json"
                ),
                &format!("gadugi-{}", scenario.id),
            ),
        )]),
        expect: GeneratedExpect {
            exit_code: Some(0),
            stdout_contains: Some(vec![
                "\"passed\": true".into(),
                format!("\"id\": \"{}\"", scenario.id),
            ]),
            output_contains: None,
        },
        timeout: 60_000,
    });
    steps.extend(command_steps);
    steps.push(GeneratedStep {
        name: agentic_step.into(),
        agent: "eatme-cli-agent".into(),
        action: "execute_command".into(),
        params: BTreeMap::from([(
            "command".into(),
            repository_command(
                &instructor_contract_validation_command(
                    &source_asset,
                    &expected_outputs,
                    &scenario.acceptance_probes,
                    &scenario.agentic_test_prompt,
                ),
                &format!("gadugi-{}", scenario.id),
            ),
        )]),
        expect: GeneratedExpect {
            exit_code: Some(0),
            stdout_contains: Some(instructor_contract_expected_stdout(&expected_outputs)),
            output_contains: None,
        },
        timeout: 60_000,
    });
    let mut assertions = vec![GeneratedAssertion {
        name: "Assets Validate".into(),
        assertion_type: "command_success".into(),
        agent: "eatme-cli-agent".into(),
        params: BTreeMap::from([("step".into(), validate_step.into())]),
    }];
    assertions.extend(command_assertions);
    assertions.push(GeneratedAssertion {
        name: "Instructor Acceptance Review Contract Is Runnable".into(),
        assertion_type: "command_success".into(),
        agent: "eatme-cli-agent".into(),
        params: BTreeMap::from([("step".into(), agentic_step.into())]),
    });
    let adapter = GeneratedGadugiAdapter {
        name: format!("Eatme {} Agentic Flow", scenario.title),
        description: format!(
            "Gadugi-compatible instructor acceptance adapter generated from {source_asset}. It validates the editable scenario contract and runnable evidence steps so non-coders can maintain prompts, probes, and rubrics without changing Rust."
        ),
        version: "1.0.0".into(),
        config: GeneratedConfig {
            timeout: timeout_ms,
            retries: 0,
            parallel: false,
        },
        environment: GeneratedEnvironment {
            requires: required_environment(scenario),
            optional: optional_environment(scenario),
        },
        agents: vec![GeneratedAgent {
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
        }],
        steps,
        assertions,
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

fn instructor_contract_validation_command(
    source_asset: &str,
    expected_outputs: &[String],
    acceptance_probes: &[String],
    agentic_test_prompt: &str,
) -> String {
    let mut command = format!(
        "cargo run -q -p eatme-cli -- assets validate --path {} --json\n",
        shell_quote(source_asset)
    );
    command.push_str(&format!(
        "AGENTIC_TEST_PROMPT={}\n",
        shell_quote(agentic_test_prompt)
    ));
    command.push_str("printf '%s\\n' \"$AGENTIC_TEST_PROMPT\" >/dev/null\n");
    command.push_str(&format!("SOURCE_ASSET={}\n", shell_quote(source_asset)));
    for output in expected_outputs {
        command.push_str(&format!(
            "grep -F -- {} \"$SOURCE_ASSET\" >/dev/null\n",
            shell_quote(output)
        ));
    }
    for probe in acceptance_probes {
        command.push_str(&format!(
            "grep -F -- {} \"$SOURCE_ASSET\" >/dev/null\n",
            shell_quote(probe)
        ));
    }
    for phrase in required_prompt_contract_phrases(agentic_test_prompt) {
        command.push_str(&format!(
            "grep -Fi -- {} \"$SOURCE_ASSET\" >/dev/null\n",
            shell_quote(phrase)
        ));
    }
    for output in expected_outputs {
        command.push_str(&format!("printf '%s\\n' {}\n", shell_quote(output)));
    }

    fn required_prompt_contract_phrases(agentic_test_prompt: &str) -> Vec<&'static str> {
        [
            "instructor-facing acceptance probes",
            "student-owned Alice action evidence",
        ]
        .into_iter()
        .filter(|phrase| {
            agentic_test_prompt
                .to_lowercase()
                .contains(&phrase.to_lowercase())
        })
        .collect()
    }
    command.push_str("printf '%s\\n' instructor-acceptance-contract-ok");
    command
}

fn instructor_contract_expected_stdout(expected_outputs: &[String]) -> Vec<String> {
    let mut expected = expected_outputs.to_vec();
    expected.push("instructor-acceptance-contract-ok".into());
    expected
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
