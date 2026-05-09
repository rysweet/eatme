use std::fs;
use std::path::Path;
use std::sync::OnceLock;

const MATRIX_HEADING: &str = "## Scenario-to-gap matrix";
const REQUIRED_HEADER: &str =
    "| Scenario | What the user is trying to do | Remaining gap | Evidence still needed |";
const REQUIRED_DIVIDER: &str = "| --- | --- | --- | --- |";

const REQUIRED_SCENARIOS: [&str; 6] = [
    "First lesson real UI actions",
    "Starter project open, save, and export preflight",
    "Instructor lesson materials remix",
    "Instructor-to-student launch handoff",
    "Student outcomes discussion rubric",
    "Classroom gallery walk and rubric",
];

const PROHIBITED_MATRIX_LANGUAGE: [&str; 10] = [
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

#[test]
fn lesson_session_readiness_matrix_sits_after_the_scenario_map() {
    let doc = readiness_doc();
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
}

#[test]
fn lesson_session_readiness_matrix_uses_the_requested_public_claim_boundary() {
    let doc = readiness_doc();
    let matrix = matrix_section(doc);
    let intro = matrix
        .split(REQUIRED_HEADER)
        .next()
        .expect("matrix intro should precede the required table header");

    assert_all_present(
        intro,
        &[
            "what each scenario is meant to show",
            "what remains unproven",
            "evidence would be needed before claiming that capability",
            "missing proof a student or instructor would need to see",
        ],
        "scenario-to-gap matrix intro",
    );
    assert_all_absent(
        intro,
        &["adapter", "implementation", "manifest", "schema", "yaml"],
        "scenario-to-gap matrix intro",
    );
}

#[test]
fn lesson_session_readiness_matrix_has_simple_rows_with_missing_proof_and_evidence() {
    let doc = readiness_doc();
    let matrix = matrix_section(doc);
    let rows = table_rows(matrix);

    assert!(
        matrix.contains(REQUIRED_HEADER),
        "matrix must use the required header"
    );
    assert!(
        matrix.contains(REQUIRED_DIVIDER),
        "matrix must use a simple markdown divider"
    );
    assert_eq!(
        rows.len(),
        REQUIRED_SCENARIOS.len(),
        "matrix should list exactly the required scenarios"
    );

    for (row, expected_scenario) in rows.iter().zip(REQUIRED_SCENARIOS) {
        assert_eq!(
            row.len(),
            4,
            "matrix rows must have exactly four cells: {row:?}"
        );
        assert_eq!(
            row[0], expected_scenario,
            "matrix should list the required scenarios in learner-facing order"
        );
        assert!(
            row.iter().all(|cell| !cell.trim().is_empty()),
            "matrix rows must not contain empty cells: {row:?}"
        );
        assert!(
            contains_any(row[2], &["does not yet show", "not yet shown"]),
            "remaining gap must be framed as missing user-visible proof: {row:?}"
        );
        assert!(
            contains_any(row[3], &["showing", "shows", "show ", "notes", "samples"]),
            "evidence still needed must describe visible evidence: {row:?}"
        );
    }
}

#[test]
fn lesson_session_readiness_matrix_avoids_overclaiming_completion_or_assessment() {
    let doc = readiness_doc();
    let matrix = matrix_section(doc);

    assert_all_absent(
        matrix,
        &PROHIBITED_MATRIX_LANGUAGE,
        "scenario-to-gap matrix",
    );
}

fn readiness_doc() -> &'static str {
    static DOC: OnceLock<String> = OnceLock::new();

    DOC.get_or_init(|| {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lesson-session-readiness.md"),
        )
        .expect("read lesson-session-readiness.md")
    })
}

fn matrix_section(doc: &str) -> &str {
    let start = doc
        .find(MATRIX_HEADING)
        .expect("lesson readiness doc must contain the matrix heading");
    let rest = &doc[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());

    &rest[..end]
}

fn table_rows(matrix: &str) -> Vec<Vec<&str>> {
    matrix
        .lines()
        .filter(|line| line.starts_with('|'))
        .filter(|line| *line != REQUIRED_HEADER && *line != REQUIRED_DIVIDER)
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_all_present(text: &str, phrases: &[&str], label: &str) {
    let normalized_text = normalize(text);
    for phrase in phrases {
        assert!(
            normalized_text.contains(&normalize(phrase)),
            "{label} is missing required wording: {phrase}"
        );
    }
}

fn assert_all_absent(text: &str, phrases: &[&str], label: &str) {
    let normalized_text = normalize(text);
    for phrase in phrases {
        assert!(
            !normalized_text.contains(&normalize(phrase)),
            "{label} contains prohibited wording: {phrase}"
        );
    }
}

fn contains_any(text: &str, phrases: &[&str]) -> bool {
    let normalized_text = normalize(text);
    phrases
        .iter()
        .any(|phrase| normalized_text.contains(&normalize(phrase)))
}

fn normalize(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.extend(word.chars().flat_map(char::to_lowercase));
    }
    normalized
}
