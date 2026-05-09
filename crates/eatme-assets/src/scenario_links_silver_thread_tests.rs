use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

const SILVER_THREAD_SCENARIOS: &[&str] = &[
    "first-lessons-real-ui-actions",
    "instructor-student-launch-evidence-handoff",
    "instructor-student-outcomes-rubric",
];

#[test]
fn mkdocs_nav_exposes_the_silver_thread_reader_path_in_order() {
    let mkdocs = read_repo_file("mkdocs.yml");

    assert_in_order(
        &mkdocs,
        &[
            "Alice Integration: alice-integration.md",
            "Alice Lesson Smoke: alice-lesson-smoke.md",
            "Lesson Session Readiness: lesson-session-readiness.md",
            "First-Lesson Evidence Guide: first-lesson-evidence-readiness.md",
            "Instructor Missions: instructor-missions.md",
            "Student Missions: student-missions.md",
        ],
    );
}

#[test]
fn silver_thread_docs_link_to_their_canonical_scenario_assets() {
    let required_links = [
        (
            "docs/index.md",
            &[
                "first-lessons-real-ui-actions",
                "instructor-student-launch-evidence-handoff",
                "instructor-student-outcomes-rubric",
            ][..],
        ),
        (
            "docs/alice-lesson-smoke.md",
            &[
                "real-alice-launch-smoke",
                "first-lessons-real-ui-actions",
                "instructor-student-launch-evidence-handoff",
                "instructor-student-outcomes-rubric",
            ][..],
        ),
        (
            "docs/lesson-session-readiness.md",
            &[
                "first-lessons-real-ui-actions",
                "instructor-lesson-materials-remix",
                "instructor-student-launch-evidence-handoff",
                "instructor-student-outcomes-rubric",
            ][..],
        ),
        (
            "docs/first-lesson-evidence-readiness.md",
            &["first-lessons-real-ui-actions"][..],
        ),
    ];

    let mut missing = Vec::new();
    for (doc_path, scenario_ids) in required_links {
        let contents = read_repo_file(doc_path);
        for scenario_id in scenario_ids {
            if !has_markdown_link_to_scenario_asset(&contents, scenario_id) {
                missing.push(format!(
                    "{doc_path} must link `{scenario_id}` to assets/scenarios/eatme/{scenario_id}.yaml"
                ));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "silver-thread docs must make canonical scenario assets clickable:\n{}",
        missing.join("\n")
    );
}

#[test]
fn silver_thread_reader_sections_use_plain_outcome_language() {
    let sections = [
        (
            "docs/index.md",
            section(
                &read_repo_file("docs/index.md"),
                "## Silver-thread lesson path",
                "## What eatme proves",
            ),
        ),
        (
            "docs/index.md",
            section(
                &read_repo_file("docs/index.md"),
                "## Outside-in evidence for Alice lesson scenarios",
                "## Main workflows",
            ),
        ),
        (
            "docs/alice-lesson-smoke.md",
            section(
                &read_repo_file("docs/alice-lesson-smoke.md"),
                "## Outside-in evidence guide for Alice lesson scenarios",
                "### Evidence reporting vocabulary",
            ),
        ),
        (
            "docs/lesson-session-readiness.md",
            section(
                &read_repo_file("docs/lesson-session-readiness.md"),
                "## Scenario map",
                "## First-lesson next action readiness",
            ),
        ),
    ];

    let mut violations = Vec::new();
    for (doc_path, section_text) in sections {
        collect_plain_language_violations(doc_path, &section_text, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "reader-facing silver-thread sections must use outcome language instead of implementation terms:\n{}",
        violations.join("\n")
    );
}

#[test]
fn canonical_silver_thread_scenario_prose_uses_plain_reader_language() {
    let mut violations = Vec::new();

    for scenario_id in SILVER_THREAD_SCENARIOS {
        let path = format!("assets/scenarios/eatme/{scenario_id}.yaml");
        let yaml = read_repo_file(&path);
        let value = serde_yaml::from_str::<Value>(&yaml).expect("scenario YAML parses");
        collect_reader_field_violations(&path, &value, &mut Vec::new(), &mut violations);
    }

    assert!(
        violations.is_empty(),
        "reader-facing scenario prose must explain outcomes without implementation terms:\n{}",
        violations.join("\n")
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_in_order(contents: &str, expected: &[&str]) {
    let mut previous_index = 0;
    for expected_item in expected {
        let index = contents[previous_index..]
            .find(expected_item)
            .map(|offset| previous_index + offset)
            .unwrap_or_else(|| panic!("missing nav item `{expected_item}`"));
        previous_index = index + expected_item.len();
    }
}

fn has_markdown_link_to_scenario_asset(contents: &str, scenario_id: &str) -> bool {
    markdown_links(contents).any(|(label, target)| {
        label.trim_matches('`') == scenario_id
            && target.contains(&format!("assets/scenarios/eatme/{scenario_id}.yaml"))
    })
}

fn markdown_links(contents: &str) -> impl Iterator<Item = (String, String)> + '_ {
    let mut rest = contents;
    std::iter::from_fn(move || {
        loop {
            let open = rest.find('[')?;
            rest = &rest[open + 1..];
            let close = rest.find("](")?;
            let label = rest[..close].to_string();
            rest = &rest[close + 2..];
            let target_close = rest.find(')')?;
            let target = rest[..target_close].to_string();
            rest = &rest[target_close + 1..];
            return Some((label, target));
        }
    })
}

fn section(contents: &str, start_marker: &str, end_marker: &str) -> String {
    let start = contents
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing section start `{start_marker}`"));
    let after_start = start + start_marker.len();
    let end = contents[after_start..]
        .find(end_marker)
        .map(|offset| after_start + offset)
        .unwrap_or(contents.len());
    contents[start..end].to_string()
}

fn collect_plain_language_violations(path: &str, text: &str, violations: &mut Vec<String>) {
    let searchable = plain_language_search_text(text);
    for term in blocked_reader_terms() {
        if searchable.contains(term) {
            violations.push(format!("{path} reader section contains `{term}`"));
        }
    }
}

fn collect_reader_field_violations(
    path: &str,
    value: &Value,
    field_path: &mut Vec<String>,
    violations: &mut Vec<String>,
) {
    match value {
        Value::Mapping(mapping) => {
            for (key, child) in mapping {
                let Some(key) = key.as_str() else {
                    continue;
                };
                field_path.push(key.to_string());
                if is_reader_facing_field(key) {
                    collect_value_text_violations(path, child, field_path, violations);
                } else {
                    collect_reader_field_violations(path, child, field_path, violations);
                }
                field_path.pop();
            }
        }
        Value::Sequence(items) => {
            for child in items {
                collect_reader_field_violations(path, child, field_path, violations);
            }
        }
        _ => {}
    }
}

fn collect_value_text_violations(
    path: &str,
    value: &Value,
    field_path: &[String],
    violations: &mut Vec<String>,
) {
    match value {
        Value::String(text) => {
            let searchable = plain_language_search_text(text);
            for term in blocked_reader_terms() {
                if searchable.contains(term) {
                    violations.push(format!(
                        "{} {} contains `{}`",
                        path,
                        field_path.join("."),
                        term
                    ));
                }
            }
        }
        Value::Sequence(items) => {
            for child in items {
                collect_value_text_violations(path, child, field_path, violations);
            }
        }
        Value::Mapping(mapping) => {
            for child in mapping.values() {
                collect_value_text_violations(path, child, field_path, violations);
            }
        }
        _ => {}
    }
}

fn is_reader_facing_field(key: &str) -> bool {
    matches!(
        key,
        "title"
            | "purpose"
            | "use"
            | "instructor_goal"
            | "agentic_test_prompt"
            | "acceptance_criteria"
            | "acceptance_probes"
            | "rubric"
            | "avoid"
            | "unsupported_policy"
    )
}

fn plain_language_search_text(text: &str) -> String {
    text.to_lowercase()
        .replace("ui-action-contract.json", "")
        .replace("scenario_id", "scenario id")
}

fn blocked_reader_terms() -> &'static [&'static str] {
    &[
        "adapter",
        "contract",
        "manifest",
        "schema",
        "rabbithole",
        "deterministic",
    ]
}
