use super::*;
use crate::validate_scenario_asset;
use serde_yaml::Value;

#[test]
fn generated_gadugi_adapter_has_do_not_edit_header() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(generated.starts_with("# DO NOT EDIT:"));
    assert!(generated.contains("assets/scenarios/eatme/"));
}

#[test]
fn generated_gadugi_adapters_match_committed_assets_and_validate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenario_root = root.join("assets/scenarios");
    let eatme_root = scenario_root.join("eatme");
    let gadugi_root = scenario_root.join("gadugi");
    let scenario_paths = scenario_asset_paths(&scenario_root).unwrap();
    let sources = scenario_paths
        .iter()
        .filter(|path| path.starts_with(&eatme_root))
        .map(|source_path| {
            let scenario = read_eatme_scenario(source_path).unwrap();
            let target_path = target_gadugi_path(&gadugi_root, source_path, &scenario).unwrap();
            (source_path, scenario, target_path)
        })
        .collect::<Vec<_>>();
    let expected_scenario_asset_count = scenario_paths.len()
        + missing_generated_target_count(
            &scenario_paths,
            sources.iter().map(|(_, _, target_path)| target_path),
        );

    for (source_path, scenario, target_path) in sources {
        let generated = generate_gadugi_adapter_yaml_for_scenario(
            &root,
            source_path,
            &scenario,
            expected_scenario_asset_count,
        )
        .unwrap();
        let committed = fs::read_to_string(&target_path).unwrap();

        assert_portable_gadugi_yaml(&generated, &root);
        assert_eq!(committed, generated, "{} is stale", target_path.display());
        let report = validate_scenario_asset(&target_path).unwrap();
        assert!(
            report.passed,
            "{}: {:?}",
            target_path.display(),
            report.errors
        );
    }
}

#[test]
fn generated_cli_adapter_counts_all_discovered_scenario_assets() {
    let root = scratch_root("generated-cli-adapter-counts-all-scenarios");
    let source_path = root.join("assets/scenarios/eatme/count-contract.yaml");
    let existing_gadugi_path = root.join("assets/scenarios/gadugi/count-contract.yaml");
    let hand_authored_gadugi_path = root.join("assets/scenarios/gadugi/hand-authored.yaml");
    fs::create_dir_all(existing_gadugi_path.parent().unwrap()).unwrap();
    write_minimal_eatme_scenario(&source_path, "count-contract");
    fs::write(
        &existing_gadugi_path,
        "stale adapter: counted before regeneration\n",
    )
    .unwrap();
    fs::write(
        &hand_authored_gadugi_path,
        "name: Hand Authored Regression\n",
    )
    .unwrap();

    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(
        generated.contains(r#""scenario_asset_count": 3"#),
        "{generated}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generated_cli_adapter_counts_missing_generated_adapter_before_writing() {
    let root = scratch_root("generated-cli-adapter-counts-missing-generated-adapter");
    let source_path = root.join("assets/scenarios/eatme/new-contract.yaml");
    let hand_authored_gadugi_path = root.join("assets/scenarios/gadugi/hand-authored.yaml");
    write_minimal_eatme_scenario(&source_path, "new-contract");
    fs::create_dir_all(hand_authored_gadugi_path.parent().unwrap()).unwrap();
    fs::write(
        &hand_authored_gadugi_path,
        "name: Hand Authored Regression\n",
    )
    .unwrap();

    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(
        generated.contains(r#""scenario_asset_count": 3"#),
        "{generated}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generated_instructor_adapter_validates_the_source_asset_before_checking_id() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = "assets/scenarios/eatme/instructor-classroom-setup-readiness.yaml";
    let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
    let adapter = generated_adapter_value(&generated);
    let validate_step = adapter["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"] == "Validate editable Alice instructor assets")
        .expect("instructor validation step is generated");
    let command = validate_step["params"]["command"].as_str().unwrap();
    let expected_stdout = validate_step["expect"]["stdout_contains"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        command.contains(&format!("assets validate --path {source} --json")),
        "{command}"
    );
    assert!(expected_stdout.contains(r#""passed": true"#));
    assert!(expected_stdout.contains(r#""id": "instructor-classroom-setup-readiness""#));
}

#[test]
fn generated_vr_adapters_declare_real_vr_switches_as_optional_environment() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = "assets/scenarios/eatme/vr-player-comfort-playtest.yaml";
    let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
    let adapter = generated_adapter_value(&generated);
    let optional = adapter["environment"]["optional"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    assert!(optional.contains(&"RUN_ID"), "{optional:?}");
    assert!(optional.contains(&"EATME_REPO"), "{optional:?}");
    assert!(optional.contains(&"EATME_REAL_VR"), "{optional:?}");
    assert!(optional.contains(&"VR_HEADSET_AVAILABLE"), "{optional:?}");
}

#[test]
fn generator_rejects_scenario_ids_that_escape_gadugi_root() {
    let root = scratch_root("generator-rejects-path-traversal-id");
    let source_path = root.join("assets/scenarios/eatme/path-traversal.yaml");
    let escaped_target = root.join("assets/owned.yaml");
    write_minimal_eatme_scenario(&source_path, "../../owned");

    let error = generate_gadugi_adapters(&root, false).unwrap_err();

    assert!(error.to_string().contains("must be kebab-case"), "{error}");
    assert!(
        !escaped_target.exists(),
        "{} must not be written",
        escaped_target.display()
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generated_real_ui_action_contract_preserves_loud_failure_semantics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/first-lessons-real-ui-actions.yaml");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let adapter = generated_adapter_value(&generated);
    let launch_step = adapter["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"] == "Launch Real Ui Action Contract")
        .expect("launch-real-ui-action-contract step is generated");
    let launch_assertion = adapter["assertions"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|assertion| {
            assertion["name"] == "launch-real-ui-action-contract expected failure is explicit"
        })
        .expect("explicit failure assertion is generated");

    assert_eq!(launch_step["expect"]["exit_code"].as_i64(), Some(1));
    let expected_stdout = launch_step["expect"]["stdout_contains"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(expected_stdout.contains(r#""scenario_id": "first-lessons-real-ui-actions""#));
    assert!(expected_stdout.contains(r#""failure_category":"#));
    assert!(expected_stdout.contains(r#""activate_alice_window_ui_action": {"#));
    assert!(expected_stdout.contains(r#""save_project_desktop_shortcut_dispatch": {"#));
    assert!(expected_stdout.contains(r#""ui_action_contract": {"#));
    assert_eq!(
        launch_assertion["type"].as_str(),
        Some("output_contains_all")
    );
}

#[test]
fn generated_first_lesson_adapters_preserve_honest_boundary_language() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let lesson_path_sources = [
        "assets/scenarios/eatme/real-alice-launch-smoke.yaml",
        "assets/scenarios/eatme/first-lessons-real-ui-actions.yaml",
    ];

    for source in lesson_path_sources {
        let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
        let normalized = generated.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("not full UI automation"),
            "{source} generated adapter must preserve the full-UI-automation limitation:\n{generated}"
        );
        assert!(
            normalized.contains("not creative assessment"),
            "{source} generated adapter must preserve the creative-assessment limitation:\n{generated}"
        );
        assert!(
            normalized.contains("not learner-world grading"),
            "{source} generated adapter must preserve the learner-world-grading limitation:\n{generated}"
        );
    }
}

#[test]
fn generated_starter_project_preflight_adapter_preserves_plain_user_facing_boundaries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml";
    let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
    let normalized = generated.split_whitespace().collect::<Vec<_>>().join(" ");

    for required in [
        "opened starter project",
        "manifest/log/window/screenshot evidence",
        "bounded starter-world and readiness-gap artifacts",
        "separate LookingGlass save/reopen/export evidence",
        "not full UI automation",
        "not creative assessment",
        "not learner-world grading",
        "not complete Alice coverage",
        "starter-world-change-note.txt",
        "run-observe-readiness-gaps.txt",
        "not visible rendering correctness proof",
        "not first-lesson completion",
    ] {
        assert!(
            normalized.contains(required),
            "{source} generated adapter must preserve {required:?}:\n{generated}"
        );
    }
    for blocked in [
        format!("{}{}", "la", "ne"),
        format!("{}{}", "lesson-", "path"),
        "source boundary".into(),
        "manifest-level evidence only".into(),
        "action evidence".into(),
    ] {
        assert!(
            !normalized.to_lowercase().contains(&blocked),
            "{source} generated adapter must not use internal {blocked:?} shorthand:\n{generated}"
        );
    }
}

#[test]
fn generated_lookingglass_verification_steps_assert_test_stdout_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let expectations = [
        (
            "assets/scenarios/eatme/alice-2-migration-bridge.yaml",
            "Verify Lookingglass Bounded Alice2 Guidance",
            vec!["test/project-migration.test.ts"],
        ),
        (
            "assets/scenarios/eatme/modified-class-portability.yaml",
            "Verify Lookingglass Class Behavior Package",
            vec![
                "class-behavior-package.persistence.test.ts",
                "e2e/class-behavior-package.spec.ts",
                "lookingglass-class-ui-evidence=export-import-instance-save-reopen",
            ],
        ),
        (
            "assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
            "Verify Lookingglass Teacher Share Package",
            vec!["test/project-export.test.ts"],
        ),
    ];

    for (source, step_name, required_patterns) in expectations {
        let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
        let stdout = generated_step_stdout(&generated, step_name);
        let command = generated_step_command(&generated, step_name);
        assert!(
            !command.contains("cargo-test-ok="),
            "{source} {step_name} npm command must not emit cargo-test-ok markers; command was {command}"
        );
        for required in required_patterns {
            assert!(
                stdout.contains(required),
                "{source} {step_name} must assert {required:?}; stdout assertions were {stdout:?}"
            );
        }
        for prose_only in [
            "automatic Alice 2 conversion",
            "converted Alice 3 project",
            "different AliceProject",
            "project persistence",
            "alice-web.teacher-share/v1",
            "teacher-share-metadata",
            "sha256",
        ] {
            assert!(
                !stdout.contains(prose_only),
                "{source} {step_name} must not assert prose-only evidence term {prose_only:?}; stdout assertions were {stdout:?}"
            );
        }
    }
}

#[test]
fn generated_record_steps_assert_only_stdout_markers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let expectations = [
        (
            "assets/scenarios/eatme/alice-web-a3p-save-load-parity.yaml",
            "Record A3p Parity Gap Matrix",
            "a3p-save-load-parity-gaps.md",
        ),
        (
            "assets/scenarios/eatme/alice-web-gallery-media-parity.yaml",
            "Record Gallery Media Gap Matrix",
            "gallery-media-parity-gaps.md",
        ),
        (
            "assets/scenarios/eatme/alice-web-story-api-runtime-parity.yaml",
            "Record Runtime Parity Gap Matrix",
            "story-api-runtime-parity-gaps.md",
        ),
        (
            "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
            "Record Starter World Change",
            "starter-world-change-note.txt",
        ),
        (
            "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
            "Record Run Observe Readiness Gaps",
            "starter-project-readiness-report.txt",
        ),
        (
            "assets/scenarios/eatme/vr-camera-locomotion-journey.yaml",
            "Record Vr Preflight",
            "vr-preflight.txt",
        ),
        (
            "assets/scenarios/eatme/vr-camera-locomotion-journey.yaml",
            "Record Agentic Review Guidance",
            "agentic-review-guidance.txt",
        ),
        (
            "assets/scenarios/eatme/vr-player-comfort-playtest.yaml",
            "Record Vr Player Preflight",
            "vr-player-preflight.txt",
        ),
        (
            "assets/scenarios/eatme/vr-player-comfort-playtest.yaml",
            "Record Comfort Playtest Guidance Template",
            "comfort-playtest-guidance-template.md",
        ),
    ];

    for (source, step_name, artifact) in expectations {
        let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
        let stdout = generated_step_stdout(&generated, step_name);
        assert!(
            stdout.contains("wrote="),
            "{source} {step_name} must assert the emitted wrote= marker; stdout assertions were {stdout:?}"
        );
        assert!(
            stdout.contains(artifact),
            "{source} {step_name} must assert emitted artifact {artifact:?}; stdout assertions were {stdout:?}"
        );
        let command = generated_step_command(&generated, step_name);
        assert!(
            !command.contains("cargo-test-ok="),
            "{source} {step_name} must not emit fake cargo-test-ok markers from heredoc prose; command was {command}"
        );
        for unprinted in [
            "cargo-test-ok=",
            "cargo test",
            "test result: ok",
            "starter_world_change=",
            "run_or_observe_attempt=",
            "save_reopen_export_readiness_gaps=",
            "real_vr_available=",
            "required_evidence=",
        ] {
            assert!(
                !stdout.contains(unprinted),
                "{source} {step_name} must not assert unprinted marker {unprinted:?}; stdout assertions were {stdout:?}"
            );
        }
    }
}

#[test]
fn generated_chained_cargo_test_steps_emit_and_assert_every_target_marker() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let expectations: [(&str, &str, &[&str]); 3] = [
        (
            "assets/scenarios/eatme/alice-web-a3p-save-load-parity.yaml",
            "Run A3p Closure Probes",
            &[
                "a3p_content_coverage",
                "a3p_roundtrip_coverage",
                "real_a3p_pipeline_integration",
                "malformed_input_resilience",
            ],
        ),
        (
            "assets/scenarios/eatme/alice-web-gallery-media-parity.yaml",
            "Run Gallery Media Closure Probes",
            &[
                "a3p_content_coverage",
                "camera_and_viewpoint_e2e",
                "text_and_speech_e2e",
                "import_export_support",
                "project_io_resource_management",
            ],
        ),
        (
            "assets/scenarios/eatme/alice-web-story-api-runtime-parity.yaml",
            "Run Runtime Closure Probes",
            &[
                "parameters_e2e",
                "functions_e2e",
                "loops_and_conditionals_e2e",
                "nested_control_flow_e2e",
                "events_collision_support",
                "events_and_collision_e2e",
                "text_and_speech_e2e",
            ],
        ),
    ];

    for (source, step_name, required_targets) in expectations {
        let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
        let stdout = generated_step_stdout(&generated, step_name);
        let command = generated_step_command(&generated, step_name);
        assert!(
            command.contains("set -e"),
            "{source} {step_name} must enable fail-fast before emitting success markers; command was {command}"
        );
        for required in required_targets {
            let marker = format!("cargo-test-ok={required}");
            assert!(
                stdout.contains(&marker),
                "{source} {step_name} must assert cargo test marker {marker:?}; stdout assertions were {stdout:?}"
            );
            assert!(
                command.contains(&marker),
                "{source} {step_name} command must emit cargo test marker {marker:?}; command was {command}"
            );
        }
    }
}

#[test]
fn generated_env_prefixed_cargo_test_steps_emit_success_markers_after_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generated = generate_gadugi_adapter_yaml(
        &root,
        &root.join("assets/scenarios/eatme/setup-preflight-ready-to-create.yaml"),
    )
    .unwrap();
    let stdout = generated_step_stdout(&generated, "Lookingglass Setup Readiness");
    let command = generated_step_command(&generated, "Lookingglass Setup Readiness");

    assert!(command.contains("set -e"), "{command}");
    assert!(
        command.contains("cargo-test-ok=web_platform_setup_readiness_e2e"),
        "{command}"
    );
    assert!(
        stdout.contains("cargo-test-ok=web_platform_setup_readiness_e2e"),
        "{stdout}"
    );
}

#[test]
fn generated_command_required_environment_is_declared() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for source in [
        "assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
        "assets/scenarios/eatme/modified-class-portability.yaml",
    ] {
        let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
        let adapter = generated_adapter_value(&generated);
        let requires = adapter["environment"]["requires"]
            .as_sequence()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>();
        let optional = adapter["environment"]["optional"]
            .as_sequence()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>();
        let declared = requires
            .iter()
            .chain(optional.iter())
            .copied()
            .collect::<Vec<_>>();
        let commands = adapter["steps"]
            .as_sequence()
            .unwrap()
            .iter()
            .filter_map(|step| step["params"]["command"].as_str())
            .collect::<Vec<_>>()
            .join("\n");

        for variable in ["LOOKINGGLASS_HOME", "ALICE_HOME", "EATME_REAL_ALICE"] {
            if commands.contains(&format!("${{{variable}:?}}")) {
                assert!(
                    declared.contains(&variable),
                    "{source} uses ${{{variable}:?}} but does not declare it; requires={requires:?} optional={optional:?}"
                );
            }
        }
    }
}

#[test]
fn generator_rejects_evidence_steps_without_derivable_stdout_assertions() {
    let root = scratch_root("generator-rejects-empty-evidence-stdout");
    let source_path = root.join("assets/scenarios/eatme/opaque-evidence.yaml");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(
        &source_path,
        r#"
schema_version: eatme.scenario/v1
id: opaque-evidence
title: Opaque Evidence
kind: alice_lesson_smoke
owner: eatme
purpose: Proves the Gadugi generator refuses empty stdout assertions for opaque evidence commands.
steps:
  - id: opaque-command
    command: custom-tool --do-work
    evidence:
      - custom tool produced the required durable proof
"#,
    )
    .unwrap();

    let error = generate_gadugi_adapter_yaml(&root, &source_path).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("stdout assertions would be empty"),
        "{error}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generator_rejects_opaque_steps_without_evidence_or_derivable_stdout_assertions() {
    let root = scratch_root("generator-rejects-empty-stdout-no-evidence");
    let source_path = root.join("assets/scenarios/eatme/opaque-no-evidence.yaml");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(
        &source_path,
        r#"
schema_version: eatme.scenario/v1
id: opaque-no-evidence
title: Opaque No Evidence
kind: alice_lesson_smoke
owner: eatme
purpose: Proves the Gadugi generator refuses empty stdout assertions for opaque commands.
steps:
  - id: opaque-command
    command: custom-tool --do-work
"#,
    )
    .unwrap();

    let error = generate_gadugi_adapter_yaml(&root, &source_path).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("stdout assertions would be empty"),
        "{error}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generated_setup_readiness_adapters_declare_web_optional_environment() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = "assets/scenarios/eatme/setup-support-lab-readiness.yaml";
    let generated = generate_gadugi_adapter_yaml(&root, &root.join(source)).unwrap();
    let unrelated_source = "assets/scenarios/eatme/accessibility-rescue-camera-captions.yaml";
    let unrelated_generated =
        generate_gadugi_adapter_yaml(&root, &root.join(unrelated_source)).unwrap();

    assert!(generated.contains("- ALICE_WEB_URL"), "{generated}");
    assert!(generated.contains("- ALICE_LOCAL_API_TOKEN"), "{generated}");
    assert!(
        generated.contains("- EATME_SETUP_READINESS_SCENARIO"),
        "{generated}"
    );
    assert!(
        generated.contains("timeout: 300000"),
        "LookingGlass setup readiness step needs a realistic timeout:\n{generated}"
    );
    assert!(
        !unrelated_generated.contains("- ALICE_LOCAL_API_TOKEN"),
        "unrelated instructor adapters must not declare setup token env:\n{unrelated_generated}"
    );
    assert!(
        !unrelated_generated.contains("- EATME_SETUP_READINESS_SCENARIO"),
        "unrelated instructor adapters must not declare setup selector env:\n{unrelated_generated}"
    );
}

fn assert_portable_gadugi_yaml(generated: &str, root: &Path) {
    let absolute_root = root.display().to_string();

    assert!(
        !generated.contains(&absolute_root),
        "generated gadugi YAML leaked absolute repo root {absolute_root}"
    );
    assert!(
        !generated.contains("/home/"),
        "generated gadugi YAML leaked an absolute home path"
    );
    assert!(generated.contains("cwd: ."));
    assert!(generated.contains("cd \"${EATME_REPO:-.}\""));
}

fn generated_adapter_value(generated: &str) -> Value {
    let yaml_without_header = generated
        .lines()
        .filter(|line| !line.starts_with("# "))
        .collect::<Vec<_>>()
        .join("\n");
    serde_yaml::from_str(&yaml_without_header).unwrap()
}

fn generated_step_stdout(generated: &str, step_name: &str) -> String {
    let adapter = generated_adapter_value(generated);
    adapter["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"] == step_name)
        .unwrap_or_else(|| panic!("{step_name} step is generated"))["expect"]["stdout_contains"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn generated_step_command(generated: &str, step_name: &str) -> String {
    let adapter = generated_adapter_value(generated);
    adapter["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"] == step_name)
        .unwrap_or_else(|| panic!("{step_name} step is generated"))["params"]["command"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn scratch_root(name: &str) -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-assets-tests")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_minimal_eatme_scenario(path: &Path, id: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!(
            r#"
schema_version: eatme.scenario/v1
id: {id}
title: Count Contract
kind: alice_lesson_smoke
owner: eatme
purpose: Proves generated adapters use discovered scenario inventory count.
steps:
  - id: validate-assets
    command: cargo run -q -p eatme-cli -- assets validate --json
    evidence:
      - stdout JSON has passed=true
"#
        ),
    )
    .unwrap();
}

#[cfg(test)]
#[path = "gadugi_step_block_tests.rs"]
mod gadugi_step_block_tests;
