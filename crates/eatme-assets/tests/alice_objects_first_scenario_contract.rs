use eatme_assets::validate_scenario_asset;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "alice-objects-first-world";
const EATME_ASSET: &str = "assets/scenarios/eatme/alice-objects-first-world.yaml";
const GADUGI_ASSET: &str = "assets/scenarios/gadugi/alice-objects-first-world.yaml";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_yaml(relative_path: &str) -> Value {
    let path = repository_root().join(relative_path);
    serde_yaml::from_str(&fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    }))
    .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn string_at(value: &Value, path: &[&str]) -> String {
    value_at(value, path)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn strings_at(value: &Value, path: &[&str]) -> Vec<String> {
    value_at(value, path)
        .and_then(Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn map_keys_at(value: &Value, path: &[&str]) -> BTreeSet<String> {
    value_at(value, path)
        .and_then(Value::as_mapping)
        .map(|mapping| {
            mapping
                .keys()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn step_ids(value: &Value) -> BTreeSet<String> {
    value_at(value, &["steps"])
        .and_then(Value::as_sequence)
        .map(|steps| {
            steps
                .iter()
                .filter_map(|step| value_at(step, &["id"]).and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn joined_text(value: &Value) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .filter(|needle| !text.contains(&needle.to_ascii_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(missing.is_empty(), "{label} missing {missing:?}");
}

#[test]
fn canonical_objects_first_assets_exist_and_validate() {
    let root = repository_root();
    for relative_path in [EATME_ASSET, GADUGI_ASSET] {
        let path = root.join(relative_path);
        assert!(
            path.is_file(),
            "canonical Alice objects-first scenario asset must exist at {relative_path}"
        );

        let report = validate_scenario_asset(&path).unwrap();
        assert!(
            report.passed,
            "{relative_path} must pass scenario validation: {:?}",
            report.errors
        );
    }
}

#[test]
fn canonical_eatme_asset_defines_the_full_learner_workflow() {
    let yaml = read_yaml(EATME_ASSET);

    assert_eq!(string_at(&yaml, &["id"]), SCENARIO_ID);
    assert_eq!(string_at(&yaml, &["owner"]), "eatme");
    assert_eq!(
        string_at(&yaml, &["launcher", "command"]),
        "alice run-objects-first-world",
        "the canonical workflow should have its own runner instead of reusing a launch-only command"
    );
    assert_eq!(string_at(&yaml, &["launcher", "scenario"]), SCENARIO_ID);

    let ids = step_ids(&yaml);
    for required in [
        "validate-assets",
        "check-dependencies",
        "create-or-open-project",
        "add-visible-object",
        "position-and-transform-object",
        "edit-movement-procedure",
        "run-world",
        "save-project",
        "reopen-project",
        "verify-persisted-state",
        "record-evidence",
    ] {
        assert!(
            ids.contains(required),
            "{SCENARIO_ID} must include workflow step {required}; got {ids:?}"
        );
    }

    let artifacts = map_keys_at(&yaml, &["artifacts"]);
    for required in [
        "manifest",
        "ui_action_contract",
        "object_placement",
        "object_transform",
        "procedure_edit",
        "world_run",
        "project_save",
        "project_reopen",
        "persisted_state",
        "evidence_summary",
    ] {
        assert!(
            artifacts.contains(required),
            "{SCENARIO_ID} must define artifact {required}; got {artifacts:?}"
        );
    }
}

#[test]
fn canonical_eatme_asset_rejects_launch_only_evidence() {
    let yaml = read_yaml(EATME_ASSET);
    let text = joined_text(&yaml);

    assert!(
        !text.contains("launch-smoke"),
        "{SCENARIO_ID} learner-facing text must not route through or describe a launch-only check"
    );
    assert!(
        !text.contains("smoke test"),
        "{SCENARIO_ID} learner-facing text must not use the forbidden label"
    );
    assert_contains_all(
        "objects-first workflow evidence",
        &text,
        &[
            "visible object",
            "transform",
            "movement",
            "procedure",
            "run the world",
            "save",
            "reopen",
            "persisted state",
            "evidence",
        ],
    );
}

#[test]
fn canonical_eatme_asset_requires_proof_for_each_major_step() {
    let yaml = read_yaml(EATME_ASSET);
    let steps = value_at(&yaml, &["steps"])
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut failures = Vec::new();

    for step in steps {
        let id = string_at(&step, &["id"]);
        if id == "validate-assets" || id == "check-dependencies" {
            continue;
        }
        let evidence = strings_at(&step, &["evidence"]);
        let evidence_text = evidence.join("\n").to_ascii_lowercase();
        if evidence.is_empty() {
            failures.push(format!("{id}: missing evidence"));
        }
        if id != "record-evidence"
            && !evidence_text.contains("artifact")
            && !evidence_text.contains("assertion")
        {
            failures.push(format!(
                "{id}: evidence must name an artifact or manifest assertion"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{SCENARIO_ID} must require proof for every major workflow step:\n{}",
        failures.join("\n")
    );
}

#[test]
fn generated_gadugi_asset_matches_the_canonical_workflow() {
    let yaml = read_yaml(GADUGI_ASSET);
    let text = joined_text(&yaml);

    assert_eq!(
        string_at(&yaml, &["metadata", "source_eatme_asset"]),
        EATME_ASSET
    );
    assert_eq!(
        string_at(&yaml, &["metadata", "test_type"]),
        "objects-first-workflow"
    );
    assert_contains_all(
        "Gadugi adapter evidence",
        &text,
        &[
            SCENARIO_ID,
            "alice run-objects-first-world",
            "visible object",
            "transform",
            "movement",
            "save",
            "reopen",
            "persisted state",
        ],
    );
}
