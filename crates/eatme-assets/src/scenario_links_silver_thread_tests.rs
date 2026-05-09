use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

const FIRST_LESSON_READER_SCENARIOS: &[&str] = &[
    "first-lessons-real-ui-actions",
    "instructor-student-launch-evidence-handoff",
    "instructor-student-outcomes-rubric",
];

#[test]
fn mkdocs_nav_exposes_the_first_lesson_reader_path_in_order() {
    let mkdocs = read_repo_file("mkdocs.yml");

    assert_in_order(
        &mkdocs,
        &[
            "Lesson Session Readiness: lesson-session-readiness.md",
            "Scenario Authoring: scenario-authoring.md",
            "Alice Lesson Smoke: alice-lesson-smoke.md",
            "First-Lesson Evidence Guide: first-lesson-evidence-readiness.md",
            "Instructor Missions: instructor-missions.md",
            "Student Missions: student-missions.md",
        ],
    );
}

#[test]
fn first_lesson_reader_docs_link_to_their_canonical_scenario_assets() {
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
        "first-lesson reader docs must make canonical scenario assets clickable:\n{}",
        missing.join("\n")
    );
}

#[test]
fn first_lesson_reader_path_links_each_page_to_the_next_step() {
    let expected_path = [
        (
            "docs/index.md",
            "Lesson Session Readiness",
            "lesson-session-readiness.md",
        ),
        (
            "docs/lesson-session-readiness.md",
            "Scenario Authoring",
            "scenario-authoring.md",
        ),
        (
            "docs/scenario-authoring.md",
            "Alice Lesson Smoke",
            "alice-lesson-smoke.md",
        ),
        (
            "docs/alice-lesson-smoke.md",
            "First-Lesson Evidence Readiness",
            "first-lesson-evidence-readiness.md",
        ),
    ];

    let mut missing = Vec::new();
    for (doc_path, label, target) in expected_path {
        let contents = read_repo_file(doc_path);
        if !has_markdown_link(&contents, label, target) {
            missing.push(format!("{doc_path} must link `{label}` to {target}"));
        }
    }

    assert!(
        missing.is_empty(),
        "first-lesson reader path must link forward from docs entry point to validation evidence:\n{}",
        missing.join("\n")
    );
}

#[test]
fn first_lesson_reader_sections_use_plain_outcome_language() {
    let index = read_repo_file("docs/index.md");
    let alice_lesson_smoke = read_repo_file("docs/alice-lesson-smoke.md");
    let lesson_session_readiness = read_repo_file("docs/lesson-session-readiness.md");

    let sections = [
        (
            "docs/index.md",
            section(
                &index,
                "## First-lesson readiness path",
                "## What eatme verifies",
            ),
        ),
        (
            "docs/index.md",
            section(
                &index,
                "## Evidence for Alice lesson scenarios",
                "## Main workflows",
            ),
        ),
        (
            "docs/alice-lesson-smoke.md",
            section(
                &alice_lesson_smoke,
                "## Evidence guide for Alice lesson scenarios",
                "### Evidence reporting vocabulary",
            ),
        ),
        (
            "docs/lesson-session-readiness.md",
            section(
                &lesson_session_readiness,
                "## Scenario map",
                "## First-lesson next action readiness",
            ),
        ),
    ];

    let mut violations = Vec::new();
    for (doc_path, section_text) in sections {
        collect_plain_language_violations(doc_path, section_text, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "first-lesson reader sections must use outcome language instead of implementation terms:\n{}",
        violations.join("\n")
    );
}

#[test]
fn canonical_first_lesson_scenario_prose_uses_plain_reader_language() {
    let mut violations = Vec::new();

    for scenario_id in FIRST_LESSON_READER_SCENARIOS {
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

#[test]
fn default_workflow_readiness_docs_require_current_head_finalization_outputs() {
    let readiness = read_repo_file("docs/default-workflow-pr-readiness.md");

    for required in [
        "current head",
        "current checks",
        "Files modified",
        "No-op justification:",
    ] {
        assert!(
            readiness.contains(required),
            "readiness docs must require `{required}` in the finalization evidence"
        );
    }

    for blocked in [
        "PR #197",
        "80582c492a0025877d83d44363a8a77d16ca6e01",
        "gh pr comment",
    ] {
        assert!(
            !readiness.contains(blocked),
            "durable readiness docs must not include point-in-time recovery instruction `{blocked}`"
        );
    }
}

#[test]
fn scenario_link_docs_use_checked_evidence_language_instead_of_positive_proof_verbs() {
    let mut violations = Vec::new();

    for (path, contents) in positive_proof_language_targets() {
        for (line_index, line) in contents.lines().enumerate() {
            if uses_positive_proof_language(line) {
                violations.push(format!("{path}:{}: {line}", line_index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "scenario-link docs must describe checked evidence, not unverified proof claims:\n{}",
        violations.join("\n")
    );
}

fn positive_proof_language_targets() -> Vec<(String, String)> {
    let mut targets = vec![
        ("docs/index.md".to_string(), read_repo_file("docs/index.md")),
        (
            "docs/alice-lesson-smoke.md".to_string(),
            read_repo_file("docs/alice-lesson-smoke.md"),
        ),
        (
            "docs/lesson-session-readiness.md".to_string(),
            read_repo_file("docs/lesson-session-readiness.md"),
        ),
        (
            "docs/scenario-authoring.md".to_string(),
            read_repo_file("docs/scenario-authoring.md"),
        ),
        (
            "docs/scenario-link-generated-runners.md".to_string(),
            read_repo_file("docs/scenario-link-generated-runners.md"),
        ),
        (
            "docs/default-workflow-pr-readiness.md".to_string(),
            read_repo_file("docs/default-workflow-pr-readiness.md"),
        ),
    ];

    for scenario_id in FIRST_LESSON_READER_SCENARIOS {
        let path = format!("assets/scenarios/eatme/{scenario_id}.yaml");
        targets.push((path.clone(), read_repo_file(&path)));
    }

    targets
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
    let expected_target = format!("assets/scenarios/eatme/{scenario_id}.yaml");
    markdown_links(contents).any(|(label, target)| {
        label.trim_matches('`') == scenario_id && target.contains(&expected_target)
    })
}

fn has_markdown_link(contents: &str, expected_label: &str, expected_target: &str) -> bool {
    markdown_links(contents)
        .any(|(label, target)| label.trim() == expected_label && target.contains(expected_target))
}

fn markdown_links(contents: &str) -> impl Iterator<Item = (&str, &str)> + '_ {
    let mut rest = contents;
    std::iter::from_fn(move || {
        while let Some(open) = rest.find('[') {
            rest = &rest[open + 1..];
            let Some(close) = rest.find("](") else {
                continue;
            };
            let label = &rest[..close];
            rest = &rest[close + 2..];
            let Some(target_close) = rest.find(')') else {
                continue;
            };
            let target = &rest[..target_close];
            rest = &rest[target_close + 1..];
            return Some((label, target));
        }
        None
    })
}

fn section<'a>(contents: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = contents
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing section start `{start_marker}`"));
    let after_start = start + start_marker.len();
    let end = contents[after_start..]
        .find(end_marker)
        .map(|offset| after_start + offset)
        .unwrap_or(contents.len());
    &contents[start..end]
}

fn collect_plain_language_violations(path: &str, text: &str, violations: &mut Vec<String>) {
    let searchable = plain_language_search_text(text);
    for term in blocked_reader_terms() {
        if searchable.contains(term) {
            violations.push(format!("{path} reader section contains `{term}`"));
        }
    }
}

fn collect_reader_field_violations<'a>(
    path: &str,
    value: &'a Value,
    field_path: &mut Vec<&'a str>,
    violations: &mut Vec<String>,
) {
    match value {
        Value::Mapping(mapping) => {
            for (key, child) in mapping {
                let Some(key) = key.as_str() else {
                    continue;
                };
                field_path.push(key);
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

fn collect_value_text_violations<'a>(
    path: &str,
    value: &'a Value,
    field_path: &[&'a str],
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

fn uses_positive_proof_language(line: &str) -> bool {
    let lower = line.to_lowercase();
    if lower.contains("does not prove")
        || lower.contains("not proven")
        || lower.contains("not yet proven")
        || lower.contains("unproven")
    {
        return false;
    }

    // Strip backtick-wrapped code spans before scanning — prohibited-phrase
    // table entries list what NOT to say, so the proof verb inside the backtick
    // span is the phrase being banned, not a positive claim.
    let stripped = strip_code_spans(&lower);
    stripped
        .split(|character: char| !character.is_ascii_alphabetic())
        .any(|word| matches!(word, "prove" | "proves" | "proved" | "proven"))
}

fn strip_code_spans(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut inside_code = false;
    for ch in text.chars() {
        if ch == '`' {
            inside_code = !inside_code;
        } else if !inside_code {
            result.push(ch);
        }
    }
    result
}
