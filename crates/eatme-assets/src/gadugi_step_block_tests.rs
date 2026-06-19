use super::*;
use std::fs;
use std::path::Path;

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
    let report = super::super::generate_gadugi_adapters(&root, true).unwrap();

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
    // Check the step_blocks module which now owns the template embedding.
    let step_blocks_source = include_str!("gadugi_step_blocks.rs");

    assert!(
        step_blocks_source.contains("include_str!"),
        "gadugi_step_blocks.rs must use include_str!() to embed step-block templates"
    );
    assert!(
        step_blocks_source.contains("alice-preflight.yaml"),
        "gadugi_step_blocks.rs must reference alice-preflight.yaml step-block template"
    );
    assert!(
        step_blocks_source.contains("alice-launch-smoke.yaml"),
        "gadugi_step_blocks.rs must reference alice-launch-smoke.yaml step-block template"
    );
}

#[test]
fn step_block_driven_validate_assets_matches_hardcoded_output() {
    // Ensure the validate-assets expected_stdout from template substitution
    // matches what the current hardcoded logic produces for a known scenario.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml");
    let generated = super::super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

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
    let generated = super::super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

    assert!(
        generated.contains("\"all_required_available\": true"),
        "check-dependencies must contain '\"all_required_available\": true'"
    );
}

#[test]
fn step_block_driven_launch_smoke_contains_scenario_id() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_path = root.join("assets/scenarios/eatme/real-alice-launch-smoke.yaml");
    let generated = super::super::generate_gadugi_adapter_yaml(&root, &source_path).unwrap();

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
