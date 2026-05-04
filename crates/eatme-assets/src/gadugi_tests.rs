use super::*;
use crate::validate_scenario_asset;

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
