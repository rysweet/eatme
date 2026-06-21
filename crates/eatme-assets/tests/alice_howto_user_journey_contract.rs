use eatme_assets::validate_scenario_asset;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct CoverageRow {
    area: String,
    scenario: String,
    user_journey: String,
    rabbit_hole: String,
    looking_glass: String,
}

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

fn coverage_rows() -> Vec<CoverageRow> {
    read_text("docs/eatme/alice-howto-coverage.md")
        .lines()
        .filter_map(|line| {
            if !line.starts_with("| ") || line.contains("---") || line.contains("HowTo area") {
                return None;
            }
            let cells = line
                .trim_matches('|')
                .split('|')
                .map(|cell| {
                    cell.trim()
                        .trim_matches('`')
                        .replace("&nbsp;", " ")
                        .to_string()
                })
                .collect::<Vec<_>>();
            if cells.len() != 5 {
                return None;
            }
            Some(CoverageRow {
                area: cells[0].clone(),
                scenario: cells[1].clone(),
                user_journey: cells[2].clone(),
                rabbit_hole: cells[3].clone(),
                looking_glass: cells[4].clone(),
            })
        })
        .collect()
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

fn joined_yaml_text(value: &Value) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn action_term_count(text: &str) -> usize {
    [
        "create",
        "open",
        "add",
        "place",
        "change",
        "edit",
        "run",
        "save",
        "reopen",
        "import",
        "export",
        "review",
        "verify",
        "check",
        "confirm",
        "prepare",
        "diagnose",
        "record",
        "build",
        "map",
        "package",
        "explain",
        "playtest",
        "revise",
        "verif",
        "use",
        "compose",
        "compare",
        "register",
        "fire",
        "move",
        "apply",
        "remix",
        "reorganize",
        "turn",
        "produce",
        "fix",
        "rerun",
        "iterate",
        "combine",
    ]
    .into_iter()
    .filter(|term| text.contains(term))
    .count()
}

fn has_expected_result_language(text: &str) -> bool {
    [
        "expected",
        "visible",
        "assertion",
        "artifact",
        "saved",
        "evidence",
        "persisted",
        "result",
        "behavior",
    ]
    .into_iter()
    .any(|term| text.contains(term))
}

#[test]
fn coverage_inventory_rows_have_matching_source_and_generated_scenarios() {
    let rows = coverage_rows();
    assert!(
        rows.len() >= 50,
        "HowTo coverage inventory should enumerate the checked-in Alice.org scenario set"
    );

    let mut seen = BTreeSet::new();
    for row in rows {
        assert!(
            seen.insert(row.scenario.clone()),
            "duplicate coverage row for {}",
            row.scenario
        );

        let eatme_relative = format!("assets/scenarios/eatme/{}.yaml", row.scenario);
        let gadugi_relative = format!("assets/scenarios/gadugi/{}.yaml", row.scenario);
        let eatme_path = repository_root().join(&eatme_relative);
        let gadugi_path = repository_root().join(&gadugi_relative);

        assert!(
            eatme_path.is_file(),
            "{} must exist for {}",
            eatme_relative,
            row.area
        );
        assert!(
            gadugi_path.is_file(),
            "{} must be regenerated from {}",
            gadugi_relative,
            eatme_relative
        );

        let report = validate_scenario_asset(&eatme_path).unwrap();
        assert!(
            report.passed,
            "{} must validate before it can count as HowTo coverage: {:?}",
            eatme_relative, report.errors
        );
    }
}

#[test]
fn covered_howto_scenarios_walk_real_user_steps_not_readiness_only_paths() {
    let mut failures = Vec::new();

    for row in coverage_rows() {
        if !row.rabbit_hole.to_ascii_lowercase().starts_with("covered") {
            failures.push(format!(
                "{}: RabbitHole status must be Covered",
                row.scenario
            ));
            continue;
        }

        let yaml = read_yaml(&format!("assets/scenarios/eatme/{}.yaml", row.scenario));
        let text = joined_yaml_text(&yaml);
        let kind = string_at(&yaml, &["kind"]);
        let launcher_command = string_at(&yaml, &["launcher", "command"]);
        let step_commands = value_at(&yaml, &["steps"])
            .and_then(Value::as_sequence)
            .map(|steps| {
                steps
                    .iter()
                    .map(|step| string_at(step, &["command"]))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if launcher_command == "alice launch-smoke" {
            failures.push(format!(
                "{}: HowTo coverage must run the lesson workflow, not only the launch readiness path",
                row.scenario
            ));
        }
        let has_user_journey_command = step_commands.iter().any(|command| {
            command.contains("alice run-howto")
                || command.contains("alice run-objects-first-world")
                || command.contains("alice objects-first-full-path")
        });
        let has_agentic_user_steps = kind == "instructor_agentic_flow"
            && text.contains("agentic")
            && action_term_count(&text) >= 5;
        if !has_user_journey_command && !has_agentic_user_steps {
            failures.push(format!(
                "{}: expected a runnable Alice user journey command such as alice run-howto",
                row.scenario
            ));
        }
        if action_term_count(&text) < 5 || !has_expected_result_language(&text) {
            failures.push(format!(
                "{}: scenario text must describe real Alice user actions and expected results",
                row.scenario
            ));
        }
        if action_term_count(&row.user_journey.to_ascii_lowercase()) < 2 {
            failures.push(format!(
                "{}: coverage inventory journey must name meaningful user actions",
                row.scenario
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "HowTo scenarios still have launch-only or incomplete user journeys:\n{}",
        failures.join("\n")
    );
}

#[test]
fn lookingglass_supported_rows_have_web_targets_and_validation_steps() {
    let mut failures = Vec::new();

    for row in coverage_rows().into_iter().filter(|row| {
        row.looking_glass
            .to_ascii_lowercase()
            .starts_with("covered")
    }) {
        let yaml = read_yaml(&format!("assets/scenarios/eatme/{}.yaml", row.scenario));
        let text = joined_yaml_text(&yaml);
        let targets = strings_at(&yaml, &["adapter", "targets"]);

        if !targets.iter().any(|target| target == "lookingglass") {
            failures.push(format!(
                "{}: LookingGlass-covered row must include adapter target lookingglass",
                row.scenario
            ));
        }
        if !text.contains("--target lookingglass")
            && !text.contains("alice_web_url")
            && !text.contains("/api/")
        {
            failures.push(format!(
                "{}: LookingGlass-covered row must define a web validation command or API assertion",
                row.scenario
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "LookingGlass coverage rows need explicit web validation:\n{}",
        failures.join("\n")
    );
}

#[test]
fn unsupported_lookingglass_rows_say_so_plainly() {
    let unclear = coverage_rows()
        .into_iter()
        .filter(|row| {
            !row.looking_glass
                .to_ascii_lowercase()
                .starts_with("covered")
        })
        .filter(|row| {
            !row.looking_glass
                .to_ascii_lowercase()
                .contains("not supported")
        })
        .map(|row| format!("{} => {}", row.scenario, row.looking_glass))
        .collect::<Vec<_>>();

    assert!(
        unclear.is_empty(),
        "LookingGlass unsupported rows must say not supported:\n{}",
        unclear.join("\n")
    );
}
