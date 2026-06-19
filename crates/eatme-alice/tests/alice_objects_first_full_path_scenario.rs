use eatme_alice::LaunchSmokeScenario;
use serde_json::Value;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "alice-objects-first-full-path";

#[test]
fn objects_first_full_path_is_classified_as_real_ui_action_scenario() {
    let scenario = LaunchSmokeScenario::new(SCENARIO_ID);

    assert!(
        scenario.requires_real_ui_actions(),
        "{SCENARIO_ID} must require structured real Alice UI/backend action evidence"
    );
    assert!(
        !scenario.accepts_window_evidence(),
        "{SCENARIO_ID} must not pass from launch/window evidence alone"
    );
}

#[test]
fn canonical_eatme_scenario_asset_exists_and_validates() {
    let asset_path =
        workspace_root().join("assets/scenarios/eatme/alice-objects-first-full-path.yaml");

    let report = eatme_assets::validate_scenario_asset(&asset_path)
        .unwrap_or_else(|error| panic!("validate {}: {error:#}", asset_path.display()));

    assert!(
        report.passed,
        "canonical scenario asset must validate: {:?}",
        report.errors
    );
    assert_eq!(report.id, SCENARIO_ID);
    assert!(
        report.step_count >= 3,
        "scenario must include executable validation and launch steps"
    );
    assert!(
        report.assertion_count >= 8,
        "scenario must encode the full path acceptance criteria"
    );
}

#[test]
fn canonical_scenario_declares_required_full_path_phases() {
    let asset_path =
        workspace_root().join("assets/scenarios/eatme/alice-objects-first-full-path.yaml");
    let yaml = std::fs::read_to_string(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));
    let scenario: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("canonical scenario YAML parses");
    let text = serde_yaml::to_string(&scenario).expect("canonical scenario re-serializes");

    for expected in [
        "create-or-open-project",
        "place-object",
        "transform-object",
        "edit-movement-procedure",
        "run-world",
        "save-project",
        "reopen-project",
        "verify-persistence",
    ] {
        assert!(
            text.contains(expected),
            "scenario must declare ordered phase {expected:?}; scenario:\n{text}"
        );
    }
}

#[test]
fn generated_gadugi_adapter_exists_and_points_to_canonical_asset() {
    let adapter_path =
        workspace_root().join("assets/scenarios/gadugi/alice-objects-first-full-path.yaml");
    let report = eatme_assets::validate_scenario_asset(&adapter_path)
        .unwrap_or_else(|error| panic!("validate {}: {error:#}", adapter_path.display()));

    assert!(
        report.passed,
        "generated Gadugi adapter must validate: {:?}",
        report.errors
    );

    let yaml = std::fs::read_to_string(&adapter_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", adapter_path.display()));
    let adapter: Value = serde_yaml::from_str(&yaml).expect("Gadugi adapter YAML parses");
    assert_eq!(
        adapter["metadata"]["source_eatme_asset"],
        "assets/scenarios/eatme/alice-objects-first-full-path.yaml"
    );
    assert!(
        adapter["metadata"]["tags"]
            .as_array()
            .expect("tags are an array")
            .iter()
            .filter_map(Value::as_str)
            .any(|tag| tag == SCENARIO_ID),
        "generated adapter tags must include {SCENARIO_ID}"
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
