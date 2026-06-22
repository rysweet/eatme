use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_text(relative_path: &str) -> String {
    let path = repository_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn read_yaml(relative_path: &str) -> Value {
    serde_yaml::from_str(&read_text(relative_path))
        .unwrap_or_else(|error| panic!("failed to parse {relative_path}: {error}"))
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
        .to_string()
}

fn strings_at(value: &Value, path: &[&str]) -> Vec<String> {
    value_at(value, path)
        .and_then(Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn coverage_inventory_scenarios() -> BTreeSet<String> {
    let mut rows = BTreeSet::new();
    let mut in_table = false;
    for line in read_text("docs/eatme/alice-howto-coverage.md").lines() {
        if line.starts_with("| HowTo area") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if !line.starts_with("| ") {
            if !rows.is_empty() {
                break;
            }
            continue;
        }
        if line.contains("---") {
            continue;
        }
        let cells = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().trim_matches('`').replace("&nbsp;", " "))
            .collect::<Vec<_>>();
        if cells.len() == 5 {
            rows.insert(cells[1].clone());
        }
    }
    rows
}

fn matrix_rows(matrix: &Value) -> Vec<Value> {
    value_at(matrix, &["rows"])
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default()
}

#[test]
fn parity_matrix_has_exactly_one_row_for_each_howto_inventory_scenario() {
    let inventory = coverage_inventory_scenarios();
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    let rows = matrix_rows(&matrix);
    let row_scenarios = rows
        .iter()
        .map(|row| string_at(row, &["scenario"]))
        .collect::<Vec<_>>();
    let row_set = row_scenarios.iter().cloned().collect::<BTreeSet<_>>();

    assert_eq!(inventory, row_set);
    assert_eq!(
        row_scenarios.len(),
        row_set.len(),
        "matrix must not contain duplicate scenario rows"
    );
}

#[test]
fn parity_matrix_rows_reference_existing_scenarios_and_explicit_closure_commands() {
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    let mut failures = Vec::new();

    for row in matrix_rows(&matrix) {
        let scenario = string_at(&row, &["scenario"]);
        let scenario_path = repository_root()
            .join("assets/scenarios/eatme")
            .join(format!("{scenario}.yaml"));
        let scenario_yaml = read_yaml(&format!("assets/scenarios/eatme/{scenario}.yaml"));
        let lookingglass_status = string_at(&row, &["looking_glass", "status"]);
        let lookingglass_command = string_at(&row, &["looking_glass", "command"]);
        let rabbit_hole_command = string_at(&row, &["rabbit_hole", "command"]);
        let closure = strings_at(&row, &["closure", "required"]).join("\n");

        if !scenario_path.is_file() {
            failures.push(format!("{scenario}: missing scenario asset"));
        }
        if !rabbit_hole_command.contains("EATME_REAL_ALICE=1")
            || !rabbit_hole_command.contains("--alice-home")
            || !rabbit_hole_command.contains("--scenario")
        {
            failures.push(format!(
                "{scenario}: RabbitHole command must be a real-Alice closure command"
            ));
        }
        if lookingglass_status == "covered" || lookingglass_status == "partial" {
            let targets = strings_at(&scenario_yaml, &["adapter", "targets"]);
            if !targets.iter().any(|target| target == "lookingglass") {
                failures.push(format!(
                    "{scenario}: claimed LookingGlass support without lookingglass adapter target"
                ));
            }
            if !lookingglass_command.contains("EATME_WEB_PLATFORM=1")
                || !lookingglass_command.contains("ALICE_WEB_URL")
            {
                failures.push(format!(
                    "{scenario}: LookingGlass closure command must be runnable"
                ));
            }
        } else if lookingglass_status == "not_supported" {
            if string_at(&row, &["looking_glass", "reason"]).is_empty() {
                failures.push(format!(
                    "{scenario}: unsupported LookingGlass row must state a reason"
                ));
            }
        } else {
            failures.push(format!(
                "{scenario}: unsupported status {lookingglass_status:?}"
            ));
        }
        if !closure.contains("RabbitHole") || !closure.contains("behavior evidence") {
            failures.push(format!(
                "{scenario}: closure requirements must name platform evidence"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "parity matrix rows are incomplete:\n{}",
        failures.join("\n")
    );
}

#[test]
fn parity_matrix_closure_families_reference_the_three_gap_scenarios_and_tests() {
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    let families = value_at(&matrix, &["closure_families"])
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let scenarios = families
        .iter()
        .map(|family| string_at(family, &["scenario"]))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        scenarios,
        BTreeSet::from([
            "alice-web-a3p-save-load-parity".to_string(),
            "alice-web-gallery-media-parity".to_string(),
            "alice-web-story-api-runtime-parity".to_string(),
        ])
    );

    for family in families {
        let scenario = string_at(&family, &["scenario"]);
        let tests = strings_at(&family, &["closure_tests"]);
        assert!(
            tests
                .iter()
                .all(|test| test.starts_with("cargo test -p eatme-alice --test ")),
            "{scenario}: closure tests must be runnable cargo test commands: {tests:?}"
        );
        assert!(
            tests.len() >= 4,
            "{scenario}: expected broad closure coverage"
        );
    }
}
