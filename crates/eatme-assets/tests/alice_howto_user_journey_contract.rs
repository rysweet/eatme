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
        let yaml = read_yaml(&format!("assets/scenarios/eatme/{}.yaml", row.scenario));
        let kind = string_at(&yaml, &["kind"]);
        let rabbit_hole = row.rabbit_hole.to_ascii_lowercase();
        let setup_readiness_instructor = matches!(
            row.scenario.as_str(),
            "setup-preflight-ready-to-create"
                | "instructor-classroom-setup-readiness"
                | "instructor-student-launch-evidence-handoff"
        );
        if setup_readiness_instructor && kind == "instructor_agentic_flow" {
            if !rabbit_hole.starts_with("partial:") || !rabbit_hole.contains("agentic") {
                failures.push(format!(
                    "{}: setup readiness instructor RabbitHole status must be explicit Partial agentic evidence",
                    row.scenario
                ));
            }
        } else if !rabbit_hole.starts_with("covered") {
            failures.push(format!(
                "{}: RabbitHole status must be Covered",
                row.scenario
            ));
            continue;
        }

        let text = joined_yaml_text(&yaml);
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
            && !text.contains("playwright")
            && !text.contains("browser ui")
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
    let mut unclear = Vec::new();

    for row in coverage_rows() {
        let looking_glass = row.looking_glass.to_ascii_lowercase();
        if looking_glass.starts_with("covered") {
            if looking_glass.contains("covered where") {
                unclear.push(format!(
                    "{} => Covered rows must not use conditional 'Covered where' language; use Partial with missing evidence or explicit covered scope: {}",
                    row.scenario, row.looking_glass
                ));
            }
            continue;
        }
        if looking_glass.starts_with("partial:") {
            if !looking_glass.contains("covered")
                || !looking_glass.contains("missing")
                || !looking_glass.contains("evidence")
            {
                unclear.push(format!(
                    "{} => Partial rows must name covered and missing evidence plainly: {}",
                    row.scenario, row.looking_glass
                ));
            }
            continue;
        }
        if !looking_glass.contains("not supported") {
            unclear.push(format!(
                "{} => Unsupported rows must say not supported: {}",
                row.scenario, row.looking_glass
            ));
        }
    }

    assert!(
        unclear.is_empty(),
        "LookingGlass non-covered rows must explain what is missing:\n{}",
        unclear.join("\n")
    );
}

#[test]
fn alice_web_parity_gap_scenarios_define_behavior_baseline_probe_and_closure_tests() {
    let expected: &[(&str, &[&str], &[&str])] = &[
        (
            "alice-web-a3p-save-load-parity",
            &[
                "project identity",
                "resources",
                "statements",
                "archive safety",
            ],
            &[
                "a3p_content_coverage",
                "a3p_roundtrip_coverage",
                "real_a3p_pipeline_integration",
                "malformed_input_resilience",
            ],
        ),
        (
            "alice-web-story-api-runtime-parity",
            &[
                "procedures",
                "loops",
                "events",
                "collision",
                "text",
                "speech",
            ],
            &[
                "parameters_e2e",
                "functions_e2e",
                "loops_and_conditionals_e2e",
                "events_collision_support",
                "text_and_speech_e2e",
            ],
        ),
        (
            "alice-web-gallery-media-parity",
            &["starter gallery", "camera", "media", "import/export"],
            &[
                "a3p_content_coverage",
                "camera_and_viewpoint_e2e",
                "text_and_speech_e2e",
                "import_export_support",
                "project_io_resource_management",
            ],
        ),
    ];

    for (scenario_id, gap_terms, closure_tests) in expected {
        let yaml = read_yaml(&format!("assets/scenarios/eatme/{scenario_id}.yaml"));
        let text = joined_yaml_text(&yaml);
        let closure_step = value_at(&yaml, &["steps"])
            .and_then(Value::as_sequence)
            .and_then(|steps| {
                steps.iter().find(|step| {
                    string_at(step, &["id"]).starts_with("run-")
                        && string_at(step, &["id"]).contains("closure-probes")
                })
            })
            .unwrap_or_else(|| panic!("{scenario_id} must define a closure probe step"));

        for required in ["java alice baseline", "web/eatme probe", "closure test"] {
            assert!(
                text.contains(required),
                "{scenario_id} must include {required:?} in its parity gap matrix"
            );
        }
        for &term in *gap_terms {
            assert!(
                text.contains(term),
                "{scenario_id} must map parity gap family {term:?}"
            );
        }
        for &test_name in *closure_tests {
            assert!(
                text.contains(test_name),
                "{scenario_id} must document closure test {test_name}"
            );
            assert!(
                string_at(closure_step, &["command"]).contains(test_name),
                "{scenario_id} closure probe step must run {test_name}"
            );
        }
        assert!(
            strings_at(&yaml, &["adapter", "targets"])
                .iter()
                .any(|target| target == "gadugi-cli"),
            "{scenario_id} must generate a Gadugi adapter for scenario execution"
        );
    }
}
