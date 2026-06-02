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
    let yaml_without_header = generated
        .lines()
        .filter(|line| !line.starts_with("# "))
        .collect::<Vec<_>>()
        .join("\n");
    let adapter: Value = serde_yaml::from_str(&yaml_without_header).unwrap();
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
        "eatme launch-smoke evidence without claiming save/reopen/export coverage",
        "not full UI automation",
        "not creative assessment",
        "not learner-world grading",
        "not complete Alice coverage",
        "starter-world-change-note.txt",
        "run-observe-readiness-gaps.txt",
        "not visible rendering correctness proof",
        "not first-lesson completion",
        "not full Save completion",
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

// ── Step-block composition TDD tests ──────────────────────────────────

#[test]
fn step_blocks_directory_excluded_from_scenario_asset_discovery() {
    let root = scratch_root("sb-dir-excluded-from-discovery");
    let eatme_dir = root.join("assets/scenarios/eatme");
    let gadugi_dir = root.join("assets/scenarios/gadugi");
    let step_blocks_dir = gadugi_dir.join("step-blocks");
    fs::create_dir_all(&eatme_dir).unwrap();
    fs::create_dir_all(&step_blocks_dir).unwrap();

    write_minimal_eatme_scenario(&eatme_dir.join("smoke.yaml"), "smoke");
    fs::write(
        gadugi_dir.join("smoke.yaml"),
        "name: Gadugi Smoke Adapter\n",
    )
    .unwrap();
    fs::write(
        step_blocks_dir.join("alice-preflight.yaml"),
        "steps:\n  - id: validate-assets\n",
    )
    .unwrap();

    let scenario_root = root.join("assets/scenarios");
    let paths = crate::discovery::scenario_asset_paths(&scenario_root).unwrap();

    // step-blocks/ YAML must NOT appear in discovered paths
    assert!(
        !paths
            .iter()
            .any(|path| path.to_string_lossy().contains("step-blocks")),
        "step-blocks/ directory must be excluded from scenario discovery; got: {paths:?}"
    );
    // The two regular YAML files must still be discovered
    assert_eq!(
        paths.len(),
        2,
        "expected 2 scenario assets (eatme + gadugi), got {}: {paths:?}",
        paths.len()
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn step_blocks_exclusion_preserves_committed_asset_count() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenario_root = root.join("assets/scenarios");
    let paths = crate::discovery::scenario_asset_paths(&scenario_root).unwrap();

    // After adding step-blocks/ directory, the count must remain unchanged
    // because discovery skips directories named "step-blocks".
    let step_block_paths: Vec<_> = paths
        .iter()
        .filter(|path| path.to_string_lossy().contains("step-blocks"))
        .collect();
    assert!(
        step_block_paths.is_empty(),
        "step-blocks/ files must not appear in scenario asset discovery: {step_block_paths:?}"
    );
}

#[test]
fn alice_preflight_step_block_template_file_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml");
    assert!(
        template_path.is_file(),
        "alice-preflight.yaml step-block template must exist at {}",
        template_path.display()
    );
}

#[test]
fn alice_launch_smoke_step_block_template_file_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml");
    assert!(
        template_path.is_file(),
        "alice-launch-smoke.yaml step-block template must exist at {}",
        template_path.display()
    );
}

#[test]
fn alice_preflight_template_contains_validate_assets_pattern() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml");
    let content = fs::read_to_string(&template_path).unwrap();

    assert!(
        content.contains("\"passed\": true"),
        "preflight template must contain validate-assets '\"passed\": true' pattern"
    );
    assert!(
        content.contains("{{scenario-asset-count}}"),
        "preflight template must use {{{{scenario-asset-count}}}} placeholder"
    );
}

#[test]
fn alice_preflight_template_contains_check_dependencies_pattern() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml");
    let content = fs::read_to_string(&template_path).unwrap();

    assert!(
        content.contains("\"all_required_available\": true"),
        "preflight template must contain check-dependencies '\"all_required_available\": true' pattern"
    );
}

#[test]
fn alice_launch_smoke_template_contains_scenario_id_placeholder() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml");
    let content = fs::read_to_string(&template_path).unwrap();

    assert!(
        content.contains("{{scenario-id}}"),
        "launch-smoke template must use {{{{scenario-id}}}} placeholder"
    );
    assert!(
        content.contains("\"scenario_id\""),
        "launch-smoke template must contain '\"scenario_id\"' pattern"
    );
}

#[test]
fn alice_launch_smoke_template_contains_execution_evidence_frame() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template_path = root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml");
    let content = fs::read_to_string(&template_path).unwrap();

    assert!(
        content.contains("\"real_alice_execution_evidence\": {"),
        "launch-smoke template must contain '\"real_alice_execution_evidence\": {{' base frame"
    );
}

#[test]
fn step_block_templates_produce_byte_identical_gadugi_output() {
    // This is the primary safety net: after refactoring to use templates,
    // the generated YAML must be byte-identical to the committed files.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = super::generate_gadugi_adapters(&root, true).unwrap();

    assert!(
        report.passed,
        "generated gadugi adapters must match committed files after step-block refactor: {:?}",
        report.errors
    );
    assert_eq!(
        report.errors.len(),
        0,
        "no stale adapters expected: {:?}",
        report.errors
    );
}

#[test]
fn gadugi_generator_uses_step_block_templates_not_hardcoded_strings() {
    // Verify the generator source references step-block templates via include_str!
    let gadugi_source = include_str!("gadugi.rs");

    assert!(
        gadugi_source.contains("include_str!"),
        "gadugi.rs must use include_str!() to embed step-block templates"
    );
    assert!(
        gadugi_source.contains("alice-preflight.yaml"),
        "gadugi.rs must reference alice-preflight.yaml step-block template"
    );
    assert!(
        gadugi_source.contains("alice-launch-smoke.yaml"),
        "gadugi.rs must reference alice-launch-smoke.yaml step-block template"
    );
}

#[test]
fn step_block_driven_validate_assets_matches_hardcoded_output() {
    // Ensure the validate-assets expected_stdout from template substitution
    // matches what the current hardcoded logic produces for a known scenario.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml");
    let generated = super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    // validate-assets step must still contain both patterns
    assert!(
        generated.contains("\"passed\": true"),
        "validate-assets must contain '\"passed\": true'"
    );
    assert!(
        generated.contains("\"scenario_asset_count\":"),
        "validate-assets must contain '\"scenario_asset_count\":'"
    );
}

#[test]
fn step_block_driven_check_dependencies_matches_hardcoded_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/building-a-scene-first-world.yaml");
    let generated = super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(
        generated.contains("\"all_required_available\": true"),
        "check-dependencies must contain '\"all_required_available\": true'"
    );
}

#[test]
fn step_block_driven_launch_smoke_contains_scenario_id() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml");
    let generated = super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(
        generated.contains("\"scenario_id\": \"real-alice-launch-smoke\""),
        "launch-smoke must contain scenario_id with actual ID substituted"
    );
    assert!(
        generated.contains("\"real_alice_execution_evidence\": {"),
        "launch-smoke must contain real_alice_execution_evidence frame"
    );
}

#[test]
fn step_block_discovery_exclusion_is_idempotent_in_scratch_root() {
    // Scratch roots without step-blocks/ dir must not break discovery
    let root = scratch_root("step-blocks-exclusion-idempotent");
    let eatme_dir = root.join("assets/scenarios/eatme");
    write_minimal_eatme_scenario(&eatme_dir.join("simple.yaml"), "simple");

    let scenario_root = root.join("assets/scenarios");
    let paths = crate::discovery::scenario_asset_paths(&scenario_root).unwrap();

    assert_eq!(
        paths.len(),
        1,
        "scratch root without step-blocks/ must still discover exactly 1 asset"
    );

    let _ = fs::remove_dir_all(&root);
}
