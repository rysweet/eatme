use std::fs;
use std::path::{Path, PathBuf};

const MATRIX_HEADING: &str = "## Scenario-to-gap matrix";
const REQUIRED_HEADER: &str =
    "| Scenario | What the user is trying to do | Remaining gap | Evidence still needed |";

const REQUIRED_PUBLIC_FRAMING: &[&str] = &[
    "what each scenario is meant to show",
    "what remains unproven",
    "evidence would be needed before claiming that capability",
];

const REQUIRED_SCENARIOS: &[&str] = &[
    "First lesson real UI actions",
    "Starter project open, save, and export preflight",
    "Instructor lesson materials remix",
    "Instructor-to-student launch handoff",
    "Student outcomes discussion rubric",
];

const PROHIBITED_MATRIX_LANGUAGE: &[&str] = &[
    "full UI automation",
    "full user interface automation",
    "Save completion",
    "first-lesson completion",
    "first lesson completion",
    "lesson completion",
    "grading",
    "creative assessment",
    "automated creative assessment",
    "assess creativity",
];

const PROHIBITED_INTERNAL_FRAMING: &[&str] = &[
    "adapter",
    "implementation",
    "manifest",
    "schema",
    "yaml",
    "RabbitHole",
];

#[test]
fn lesson_session_readiness_matrix_has_required_public_columns_near_scenario_map() {
    let doc = read_readiness_doc();
    let scenario_map_index = doc
        .find("## Scenario map")
        .expect("lesson readiness doc must keep a Scenario map section");
    let matrix_index = doc
        .find(MATRIX_HEADING)
        .expect("lesson readiness doc must add a Scenario-to-gap matrix section");

    assert!(
        scenario_map_index < matrix_index,
        "scenario-to-gap matrix should appear after the existing scenario map"
    );

    let matrix = matrix_section(&doc);
    let mut lines = matrix.lines();
    assert_eq!(lines.next(), Some(MATRIX_HEADING));
    assert!(
        matrix.contains(REQUIRED_HEADER),
        "scenario-to-gap matrix must use the required learner-facing table header"
    );
    assert!(
        matrix.contains("| --- | --- | --- | --- |"),
        "scenario-to-gap matrix must use a simple four-column markdown table"
    );
}

#[test]
fn lesson_session_readiness_matrix_intro_states_the_claim_boundary_for_users() {
    let matrix = matrix_section(&read_readiness_doc());
    let intro = matrix
        .split(REQUIRED_HEADER)
        .next()
        .expect("matrix intro should precede the required table header");

    assert_contains_all(
        "scenario-to-gap matrix intro",
        intro,
        REQUIRED_PUBLIC_FRAMING,
    );
    assert_contains_none(
        "scenario-to-gap matrix intro",
        intro,
        PROHIBITED_INTERNAL_FRAMING,
    );
}

#[test]
fn lesson_session_readiness_matrix_covers_required_scenarios_with_missing_proof() {
    let matrix = matrix_section(&read_readiness_doc());
    let rows = table_rows(&matrix);
    let scenario_names = rows
        .iter()
        .map(|row| row[0].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "scenario-to-gap matrix scenario list",
        &scenario_names,
        REQUIRED_SCENARIOS,
    );

    for row in rows {
        assert_eq!(
            row.len(),
            4,
            "matrix rows must have exactly four cells: {row:?}"
        );
        assert!(
            row.iter().all(|cell| !cell.trim().is_empty()),
            "matrix rows must not contain empty cells: {row:?}"
        );
        assert!(
            contains_missing_proof_language(&row[2]),
            "remaining gap must be framed as missing user-visible proof: {row:?}"
        );
        assert!(
            contains_evidence_need_language(&row[3]),
            "evidence still needed must describe visible evidence: {row:?}"
        );
    }
}

#[test]
fn lesson_session_readiness_matrix_avoids_overclaiming_completion_or_assessment() {
    let matrix = matrix_section(&read_readiness_doc());

    assert_contains_none(
        "scenario-to-gap matrix",
        &matrix,
        PROHIBITED_MATRIX_LANGUAGE,
    );
}

fn read_readiness_doc() -> String {
    fs::read_to_string(repository_root().join("docs/lesson-session-readiness.md")).unwrap()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn matrix_section(doc: &str) -> String {
    let start = doc
        .find(MATRIX_HEADING)
        .expect("lesson readiness doc must contain the matrix heading");
    let rest = &doc[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());

    rest[..end].to_string()
}

fn table_rows(matrix: &str) -> Vec<Vec<String>> {
    matrix
        .lines()
        .filter(|line| line.starts_with('|'))
        .filter(|line| *line != REQUIRED_HEADER && *line != "| --- | --- | --- | --- |")
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn contains_missing_proof_language(cell: &str) -> bool {
    let normalized = normalize(cell).to_lowercase();
    normalized.contains("does not yet show") || normalized.contains("not yet shown")
}

fn contains_evidence_need_language(cell: &str) -> bool {
    let normalized = normalize(cell).to_lowercase();
    normalized.contains("showing")
        || normalized.contains("shows")
        || normalized.contains("show ")
        || normalized.contains("notes")
        || normalized.contains("samples")
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text).to_lowercase();
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required public wording: {missing:?}"
    );
}

fn assert_contains_none(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "{label} contains prohibited wording: {present:?}"
    );
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
