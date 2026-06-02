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

#[cfg(test)]
#[path = "gadugi_step_block_tests.rs"]
mod gadugi_step_block_tests;
