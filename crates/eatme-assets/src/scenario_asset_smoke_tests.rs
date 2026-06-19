use crate::{discovery::scenario_asset_paths, validate_scenario_asset};
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn smoke_validates_every_committed_scenario_yaml() {
    let scenario_paths = all_scenario_paths();
    assert_eq!(
        107,
        scenario_paths.len(),
        "expected committed scenario count to stay stable"
    );

    for scenario_path in &scenario_paths {
        let report = validate_scenario_asset(scenario_path).unwrap();
        assert!(
            report.passed,
            "{}: {:?}",
            relative_path(scenario_path),
            report.errors
        );
    }
}

#[test]
fn every_scenario_defines_name_steps_and_expected_outcomes() {
    for scenario_path in all_scenario_paths() {
        let yaml = fs::read_to_string(&scenario_path).unwrap();
        let value: Value = serde_yaml::from_str(&yaml).unwrap();
        let mapping = value.as_mapping().unwrap_or_else(|| {
            panic!(
                "{} should deserialize to a mapping",
                relative_path(&scenario_path)
            )
        });

        if is_gadugi_scenario(&scenario_path) {
            assert!(
                has_nonempty_string(mapping, "name"),
                "{} must define a non-empty name",
                relative_path(&scenario_path)
            );
            assert!(
                has_nonempty_sequence(mapping, "steps"),
                "{} must define one or more steps",
                relative_path(&scenario_path)
            );
            assert!(
                has_nonempty_sequence(mapping, "assertions"),
                "{} must define one or more expected outcomes in assertions",
                relative_path(&scenario_path)
            );
        } else {
            assert!(
                has_nonempty_string(mapping, "title"),
                "{} must define a non-empty title",
                relative_path(&scenario_path)
            );
            assert!(
                has_nonempty_sequence(mapping, "steps"),
                "{} must define one or more steps",
                relative_path(&scenario_path)
            );
            assert!(
                eatme_has_expected_outcomes(mapping),
                "{} must define expected outcomes via acceptance criteria, probes, rubric, smoke evidence, or agentic expected outputs",
                relative_path(&scenario_path)
            );
        }
    }
}

#[test]
fn discovery_based_smoke_suite_leaves_no_orphaned_scenarios() {
    let discovered: BTreeSet<String> = all_scenario_paths()
        .into_iter()
        .map(|path| relative_path(&path))
        .collect();

    let mut validated = BTreeSet::new();
    for scenario_path in all_scenario_paths() {
        let report = validate_scenario_asset(&scenario_path).unwrap();
        assert!(
            report.passed,
            "{}: {:?}",
            relative_path(&scenario_path),
            report.errors
        );
        validated.insert(relative_path(&scenario_path));
    }

    assert_eq!(
        discovered, validated,
        "every discovered scenario YAML should be covered by the smoke validation suite"
    );
}

fn all_scenario_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/scenarios");
    scenario_asset_paths(&root).unwrap()
}

fn relative_path(path: &Path) -> String {
    path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .unwrap()
        .display()
        .to_string()
        .replace('\\', "/")
}

fn is_gadugi_scenario(path: &Path) -> bool {
    relative_path(path).contains("/gadugi/")
}

fn has_nonempty_string(mapping: &serde_yaml::Mapping, key: &str) -> bool {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn has_nonempty_sequence(mapping: &serde_yaml::Mapping, key: &str) -> bool {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_sequence)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}

fn eatme_has_expected_outcomes(mapping: &serde_yaml::Mapping) -> bool {
    has_nonempty_sequence(mapping, "acceptance_criteria")
        || has_nonempty_sequence(mapping, "acceptance_probes")
        || has_nonempty_sequence(mapping, "rubric")
        || has_nonempty_mapping(mapping, "artifacts")
        || nested_nonempty_sequence(mapping, "smoke_ready", "evidence")
        || nested_nonempty_sequence(mapping, "agentic_flow", "expected_outputs")
        || sequence_items_have_nonempty_sequence(mapping, "steps", "evidence")
}

fn has_nonempty_mapping(mapping: &serde_yaml::Mapping, key: &str) -> bool {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_mapping)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}

fn sequence_items_have_nonempty_sequence(
    mapping: &serde_yaml::Mapping,
    sequence_key: &str,
    nested_key: &str,
) -> bool {
    mapping
        .get(Value::String(sequence_key.to_string()))
        .and_then(Value::as_sequence)
        .map(|items| {
            items.iter().any(|item| {
                item.as_mapping()
                    .and_then(|nested| nested.get(Value::String(nested_key.to_string())))
                    .and_then(Value::as_sequence)
                    .map(|evidence| !evidence.is_empty())
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn nested_nonempty_sequence(mapping: &serde_yaml::Mapping, parent: &str, key: &str) -> bool {
    mapping
        .get(Value::String(parent.to_string()))
        .and_then(Value::as_mapping)
        .and_then(|nested| nested.get(Value::String(key.to_string())))
        .and_then(Value::as_sequence)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}
