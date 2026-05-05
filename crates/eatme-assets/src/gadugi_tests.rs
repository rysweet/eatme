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
    for source_path in scenario_asset_paths(&root.join("assets/scenarios/eatme")).unwrap() {
        let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
        let scenario = read_eatme_scenario(&source_path).unwrap();
        let target_path = root
            .join("assets/scenarios/gadugi")
            .join(format!("{}.yaml", scenario.id));
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
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::create_dir_all(existing_gadugi_path.parent().unwrap()).unwrap();
    fs::write(
        &source_path,
        r#"
schema_version: eatme.scenario/v1
id: count-contract
title: Count Contract
kind: alice_lesson_smoke
owner: eatme
purpose: Proves generated adapters use discovered scenario inventory count.
steps:
  - id: validate-assets
    command: cargo run -q -p eatme-cli -- assets validate --json
    evidence:
      - stdout JSON has passed=true
"#,
    )
    .unwrap();
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
    assert!(
        expected_stdout.contains(r#""failure_category": "ui_action_automation_unimplemented""#)
    );
    assert!(expected_stdout.contains(r#""ui_action_contract": {"#));
    assert_eq!(
        launch_assertion["type"].as_str(),
        Some("output_contains_all")
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
