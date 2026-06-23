use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
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
    coverage_inventory_statuses()
        .into_keys()
        .collect::<BTreeSet<_>>()
}

fn coverage_inventory_statuses() -> BTreeMap<String, (String, String)> {
    let mut rows = BTreeSet::new();
    let mut statuses = BTreeMap::new();
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
            statuses.insert(cells[1].clone(), (cells[3].clone(), cells[4].clone()));
        }
    }
    statuses
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
        let scenario_kind = string_at(&scenario_yaml, &["kind"]);
        let lookingglass_status = string_at(&row, &["looking_glass", "status"]);
        let lookingglass_command = string_at(&row, &["looking_glass", "command"]);
        let rabbit_hole_status = string_at(&row, &["rabbit_hole", "status"]);
        let rabbit_hole_command = string_at(&row, &["rabbit_hole", "command"]);
        let closure = strings_at(&row, &["closure", "required"]).join("\n");

        if !scenario_path.is_file() {
            failures.push(format!("{scenario}: missing scenario asset"));
        }
        let setup_readiness_instructor = matches!(
            scenario.as_str(),
            "setup-preflight-ready-to-create"
                | "instructor-classroom-setup-readiness"
                | "instructor-student-launch-evidence-handoff"
        );
        if setup_readiness_instructor && scenario_kind == "instructor_agentic_flow" {
            if rabbit_hole_status != "partial"
                || !rabbit_hole_command.contains("assets validate --path")
            {
                failures.push(format!(
                    "{scenario}: instructor RabbitHole closure must be partial and validate the editable scenario asset"
                ));
            }
        } else if !rabbit_hole_command.contains("EATME_REAL_ALICE=1")
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
            let is_eatme_web_command = lookingglass_command.contains("EATME_WEB_PLATFORM=1")
                && lookingglass_command.contains("ALICE_WEB_URL");
            let is_direct_lookingglass_command = lookingglass_command
                .contains("cd \"${LOOKINGGLASS_HOME:?}\"")
                && lookingglass_command.contains("npm test --");
            if !is_eatme_web_command && !is_direct_lookingglass_command {
                failures.push(format!(
                    "{scenario}: LookingGlass closure command must be runnable"
                ));
            }
            if lookingglass_command.contains("ALICE_WEB_URL")
                && !lookingglass_command
                    .contains(r#"ALICE_WEB_URL="${ALICE_WEB_URL:-http://localhost:3099}""#)
            {
                failures.push(format!(
                    "{scenario}: LookingGlass web command must default ALICE_WEB_URL when it is unset"
                ));
            }
            if closure.contains(
                "LookingGlass unsupported status remains explicit until implementation exists",
            ) {
                failures.push(format!(
                    "{scenario}: supported LookingGlass row must not use unsupported closure wording"
                ));
            }
        } else if lookingglass_status == "not_supported" {
            if string_at(&row, &["looking_glass", "reason"]).is_empty() {
                failures.push(format!(
                    "{scenario}: unsupported LookingGlass row must state a reason"
                ));
            }
            if closure.contains("LookingGlass command passes when web support is claimed") {
                failures.push(format!(
                    "{scenario}: unsupported LookingGlass row must not require a command pass"
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
fn parity_matrix_source_statuses_match_the_howto_inventory() {
    let inventory_statuses = coverage_inventory_statuses();
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    let mut failures = Vec::new();

    for row in matrix_rows(&matrix) {
        let scenario = string_at(&row, &["scenario"]);
        let Some((rabbit_hole_status, looking_glass_status)) = inventory_statuses.get(&scenario)
        else {
            failures.push(format!(
                "{scenario}: missing from docs/eatme/alice-howto-coverage.md"
            ));
            continue;
        };
        let matrix_rabbit_hole_status = string_at(&row, &["rabbit_hole", "source_status"]);
        let matrix_looking_glass_status = string_at(&row, &["looking_glass", "source_status"]);

        if &matrix_rabbit_hole_status != rabbit_hole_status {
            failures.push(format!(
                "{scenario}: RabbitHole source_status {matrix_rabbit_hole_status:?} must match docs {rabbit_hole_status:?}"
            ));
        }
        if &matrix_looking_glass_status != looking_glass_status {
            failures.push(format!(
                "{scenario}: LookingGlass source_status {matrix_looking_glass_status:?} must match docs {looking_glass_status:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "parity matrix source_status values drifted:\n{}",
        failures.join("\n")
    );
}

#[test]
fn parity_matrix_closure_families_bind_supported_rows_to_named_scenarios_and_tests() {
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
            "web-platform-curriculum-e2e".to_string(),
            "alice-web-a3p-save-load-parity".to_string(),
            "alice-web-gallery-media-parity".to_string(),
            "alice-web-story-api-runtime-parity".to_string(),
            "alice-2-migration-bridge".to_string(),
            "modified-class-portability".to_string(),
            "teacher-community-sharing-loop".to_string(),
        ])
    );

    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for family in families {
        let family_scenario = string_at(&family, &["scenario"]);
        let tests = strings_at(&family, &["closure_tests"]);
        let scenario_bindings = strings_at(&family, &["scenario_bindings"]);
        assert!(
            !scenario_bindings.is_empty(),
            "{family_scenario}: closure family must name the scenarios it closes"
        );
        assert!(
            tests.iter().all(|test| {
                test.starts_with("cargo test -p eatme-alice --test ")
                    || test.starts_with("EATME_WEB_PLATFORM=1 ")
                    || test.starts_with(r#"cd "${LOOKINGGLASS_HOME:?}" && npm test -- "#)
            }),
            "{family_scenario}: closure tests must be runnable cargo/npm commands: {tests:?}"
        );
        if string_at(&family, &["role"]) == "grouped parity closure probe" {
            assert!(
                tests.len() >= 4,
                "{family_scenario}: expected broad closure coverage"
            );
        }
        for scenario in scenario_bindings {
            bindings
                .entry(scenario)
                .or_default()
                .push(family_scenario.clone());
        }
    }

    let mut failures = Vec::new();
    let family_by_id = value_at(&matrix, &["closure_families"])
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .map(|family| (string_at(family, &["scenario"]), family))
        .collect::<BTreeMap<_, _>>();

    for row in matrix_rows(&matrix) {
        let scenario = string_at(&row, &["scenario"]);
        let status = string_at(&row, &["looking_glass", "status"]);
        if status != "covered" && status != "partial" {
            continue;
        }
        let command = string_at(&row, &["looking_glass", "command"]);
        let Some(bound_families) = bindings.get(&scenario) else {
            failures.push(format!(
                "{scenario}: supported LookingGlass row is missing closure_families.scenario_bindings"
            ));
            continue;
        };
        let command_is_bound = bound_families.iter().any(|family_id| {
            let Some(family) = family_by_id.get(family_id) else {
                return false;
            };
            strings_at(family, &["closure_tests"])
                .iter()
                .any(|test| test == &command)
        });
        if !command_is_bound {
            failures.push(format!(
                "{scenario}: closure family {bound_families:?} does not bind the row command {command:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "supported LookingGlass rows must have row-specific closure-family bindings:\n{}",
        failures.join("\n")
    );

    for direct_family in [
        "alice-2-migration-bridge",
        "modified-class-portability",
        "teacher-community-sharing-loop",
    ] {
        let family = family_by_id
            .get(direct_family)
            .unwrap_or_else(|| panic!("{direct_family}: missing closure family"));
        assert!(
            strings_at(family, &["evidence_assertions"]).len() >= 2,
            "{direct_family}: direct LookingGlass closure must name durable evidence assertions"
        );
    }
}
