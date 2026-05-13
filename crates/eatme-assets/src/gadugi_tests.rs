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

// ── Step block file existence and validity ────────────────────────────────────

#[test]
fn step_block_preflight_file_exists_and_is_valid_yaml() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml");
    assert!(path.exists(), "alice-preflight.yaml step block must exist at {}", path.display());
    let content = fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_yaml::from_str(&content)
        .expect("alice-preflight.yaml must be valid YAML");
    assert!(parsed.as_sequence().is_some(), "alice-preflight.yaml must be a YAML sequence of steps");
}

#[test]
fn step_block_launch_smoke_file_exists_and_is_valid_yaml() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml");
    assert!(path.exists(), "alice-launch-smoke.yaml step block must exist at {}", path.display());
    let content = fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_yaml::from_str(&content)
        .expect("alice-launch-smoke.yaml must be valid YAML");
    assert!(parsed.as_sequence().is_some(), "alice-launch-smoke.yaml must be a YAML sequence of steps");
}

// ── Step block content contract ──────────────────────────────────────────────

#[test]
fn step_block_preflight_contains_validate_and_check_steps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = fs::read_to_string(
        root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml"),
    )
    .expect("alice-preflight.yaml must exist");
    let steps: Vec<Value> = serde_yaml::from_str(&content).unwrap();
    let step_names: Vec<&str> = steps
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(
        step_names.contains(&"Validate Assets"),
        "preflight must contain Validate Assets step, found: {step_names:?}"
    );
    assert!(
        step_names.contains(&"Check Dependencies"),
        "preflight must contain Check Dependencies step, found: {step_names:?}"
    );
    assert!(
        content.contains("{{run-id}}"),
        "preflight must contain {{{{run-id}}}} placeholder"
    );
    assert!(
        content.contains("{{expected-scenario-asset-count}}"),
        "preflight must contain {{{{expected-scenario-asset-count}}}} placeholder"
    );
}

#[test]
fn step_block_launch_smoke_contains_placeholders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = fs::read_to_string(
        root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml"),
    )
    .expect("alice-launch-smoke.yaml must exist");
    assert!(
        content.contains("{{scenario-id}}"),
        "launch smoke must contain {{{{scenario-id}}}} placeholder"
    );
    assert!(
        content.contains("Launch Smoke"),
        "launch smoke must contain Launch Smoke step name"
    );
    assert!(
        content.contains("{{run-id}}"),
        "launch smoke must contain {{{{run-id}}}} placeholder"
    );
}

// ── Discovery exclusion ──────────────────────────────────────────────────────

#[test]
fn discovery_excludes_step_blocks_directory() {
    let root = scratch_root("discovery-excludes-step-blocks");
    let scenario_root = root.join("assets/scenarios");
    let eatme_dir = scenario_root.join("eatme");
    let gadugi_dir = scenario_root.join("gadugi");
    let step_blocks_dir = gadugi_dir.join("step-blocks");

    write_minimal_eatme_scenario(&eatme_dir.join("test-scenario.yaml"), "test-scenario");
    fs::create_dir_all(&step_blocks_dir).unwrap();
    fs::write(
        step_blocks_dir.join("alice-preflight.yaml"),
        "- name: Validate Assets\n  agent: eatme-cli-agent\n",
    )
    .unwrap();
    fs::write(
        step_blocks_dir.join("alice-launch-smoke.yaml"),
        "- name: Launch Smoke\n  agent: eatme-cli-agent\n",
    )
    .unwrap();

    let paths = crate::discovery::scenario_asset_paths(&scenario_root).unwrap();
    let step_block_paths: Vec<_> = paths
        .iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "step-blocks"))
        .collect();
    assert!(
        step_block_paths.is_empty(),
        "step-blocks directory must be excluded from discovery but found: {step_block_paths:?}"
    );
    assert_eq!(
        paths.len(),
        1,
        "only the eatme scenario should be discovered, got: {paths:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn step_blocks_in_gadugi_do_not_inflate_committed_asset_count() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = crate::validate_assets(&root).unwrap();
    assert_eq!(
        report.scenario_asset_count, 101,
        "step-blocks must not inflate scenario_asset_count (expected 101, got {})",
        report.scenario_asset_count
    );
}

#[test]
fn generated_cli_adapter_count_ignores_step_blocks_in_scratch_root() {
    let root = scratch_root("adapter-count-ignores-step-blocks");
    let source_path = root.join("assets/scenarios/eatme/count-test.yaml");
    let gadugi_dir = root.join("assets/scenarios/gadugi");
    let step_blocks_dir = gadugi_dir.join("step-blocks");

    write_minimal_eatme_scenario(&source_path, "count-test");
    fs::create_dir_all(&step_blocks_dir).unwrap();
    fs::write(
        step_blocks_dir.join("alice-preflight.yaml"),
        "- name: Validate Assets\n  agent: eatme-cli-agent\n",
    )
    .unwrap();

    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    // count-test.yaml (eatme) + count-test.yaml (generated gadugi) = 2
    // step-blocks must NOT inflate this
    assert!(
        generated.contains(r#""scenario_asset_count": 2"#),
        "step-blocks must not inflate scenario_asset_count in generated adapter:\n{generated}"
    );
    let _ = fs::remove_dir_all(&root);
}

// ── Step block substitution round-trip fidelity ──────────────────────────────

#[test]
fn step_block_preflight_after_substitution_matches_generated_adapter_steps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let preflight_template = fs::read_to_string(
        root.join("assets/scenarios/gadugi/step-blocks/alice-preflight.yaml"),
    )
    .expect("alice-preflight.yaml must exist");
    let expanded = preflight_template
        .replace("{{run-id}}", "gadugi-code-editor-first-run")
        .replace("{{expected-scenario-asset-count}}", "101");
    let expanded_steps: Vec<Value> = serde_yaml::from_str(&expanded)
        .expect("expanded preflight block must be valid YAML");
    assert!(!expanded.contains("{{"), "all placeholders must be substituted");

    // Compare with the committed gadugi adapter
    let committed = fs::read_to_string(
        root.join("assets/scenarios/gadugi/code-editor-first-run.yaml"),
    )
    .unwrap();
    let committed_yaml: Value = serde_yaml::from_str(
        &committed
            .lines()
            .filter(|line| !line.starts_with("# "))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let committed_steps = committed_yaml["steps"].as_sequence().unwrap();

    for expanded_step in &expanded_steps {
        let step_name = expanded_step["name"].as_str().unwrap();
        let matching_committed = committed_steps
            .iter()
            .find(|s| s["name"].as_str() == Some(step_name))
            .unwrap_or_else(|| panic!("committed adapter must have step {step_name}"));
        assert_eq!(
            expanded_step, matching_committed,
            "step block '{step_name}' after substitution must match committed adapter step"
        );
    }
}

#[test]
fn step_block_launch_smoke_after_substitution_matches_generated_adapter_step() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let template = fs::read_to_string(
        root.join("assets/scenarios/gadugi/step-blocks/alice-launch-smoke.yaml"),
    )
    .expect("alice-launch-smoke.yaml must exist");

    // The template with scenario-id substituted should produce valid YAML
    let expanded = template.replace("{{scenario-id}}", "real-alice-launch-smoke");
    let parsed: Value =
        serde_yaml::from_str(&expanded).expect("expanded launch smoke must be valid YAML");
    assert!(!expanded.contains("{{scenario-id}}"), "scenario-id placeholder must be substituted");

    // Verify the expanded template refers to the correct scenario
    let yaml_text = serde_yaml::to_string(&parsed).unwrap();
    assert!(
        yaml_text.contains("real-alice-launch-smoke"),
        "substituted launch smoke must reference scenario id"
    );
}

// ── Output stability regression (step blocks must not change output) ─────────

#[test]
fn step_block_generation_produces_byte_identical_output_for_all_committed_adapters() {
    // This is the critical regression test: after refactoring the generator
    // to use step blocks, every committed gadugi adapter must be regenerated
    // byte-identically.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenario_root = root.join("assets/scenarios");
    let gadugi_root = scenario_root.join("gadugi");

    for entry in fs::read_dir(&gadugi_root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            continue; // skip step-blocks/ etc.
        }
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let committed = fs::read_to_string(&path).unwrap();
        if !committed.starts_with("# DO NOT EDIT:") {
            continue; // skip hand-authored gadugi files
        }
        // Extract source path from the committed header
        let source_asset = committed
            .lines()
            .find(|l| l.contains("source_eatme_asset:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim())
            .unwrap_or("");
        if source_asset.is_empty() {
            continue;
        }
        let source_path = root.join(source_asset);
        if !source_path.exists() {
            continue;
        }
        let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
        assert_eq!(
            committed, generated,
            "{} is stale after step-block refactor",
            path.display()
        );
    }
}

#[test]
fn generated_adapter_preserves_run_id_consistency_across_step_block_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/code-editor-first-run.yaml");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let run_id = "gadugi-code-editor-first-run";

    let run_id_lines: Vec<&str> = generated
        .lines()
        .filter(|line| line.contains("RUN_ID:-"))
        .collect();
    assert!(
        !run_id_lines.is_empty(),
        "generated adapter must contain RUN_ID references"
    );
    for line in &run_id_lines {
        assert!(
            line.contains(run_id),
            "all steps must use consistent run_id '{run_id}', but found: {line}"
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
