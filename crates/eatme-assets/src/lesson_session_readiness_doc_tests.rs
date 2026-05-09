const SCENARIO_MAP_HEADING: &str = "## Scenario map";
const SCENARIO_MAP_HEADER: &str = "| Scenario | Role | Evidence contract |";
const SCENARIO_MAP_DIVIDER: &str = "| --- | --- | --- |";
const MATRIX_HEADING: &str = "## Scenario-to-gap matrix";
const REQUIRED_HEADER: &str =
    "| Scenario | What the user is trying to do | Remaining gap | Evidence still needed |";
const REQUIRED_DIVIDER: &str = "| --- | --- | --- | --- |";
const PR193_FINALIZATION_BOUNDARY_HEADING: &str = "## PR #193 finalization evidence boundary";
const PR193_FINALIZATION_TEMPLATE_HEADING: &str = "## PR #193 finalization template";

const REQUIRED_SCENARIO_IDS: [&str; 6] = [
    "first-lessons-real-ui-actions",
    "starter-project-open-save-export-preflight",
    "instructor-lesson-materials-remix",
    "instructor-student-launch-evidence-handoff",
    "instructor-student-outcomes-rubric",
    "classroom-gallery-walk-and-rubric",
];

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

const PR193_REQUIRED_COMMANDS: [&str; 4] = [
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

#[test]
fn lesson_session_readiness_matrix_sits_after_the_scenario_map() {
    let doc = readiness_doc();
    let scenario_map_index = doc
        .find(SCENARIO_MAP_HEADING)
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
fn lesson_session_readiness_scenario_map_tracks_the_matrix_scenarios() {
    let doc = readiness_doc();
    let scenario_map = scenario_map_section(doc);
    let rows = markdown_table_rows(scenario_map, SCENARIO_MAP_HEADER, SCENARIO_MAP_DIVIDER);

    assert_eq!(
        rows.len(),
        REQUIRED_SCENARIO_IDS.len(),
        "scenario map should list the same scenario set as the matrix"
    );

    for (row, expected_id) in rows.iter().zip(REQUIRED_SCENARIO_IDS) {
        assert_eq!(
            row.len(),
            3,
            "scenario map rows must have exactly three cells: {row:?}"
        );
        assert_eq!(
            row[0].trim_matches('`'),
            expected_id,
            "scenario map should track the matrix scenario set in the same order"
        );
        assert!(
            row[1..].iter().all(|cell| !cell.trim().is_empty()),
            "scenario map rows must not contain empty role or evidence cells: {row:?}"
        );
    }
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
            contains_any_normalized(row[2], &["does not yet show", "not yet shown"]),
            "remaining gap must be framed as missing user-visible proof: {row:?}"
        );
        assert!(
            contains_any_normalized(row[3], &["showing", "shows", "show ", "notes", "samples"]),
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

#[test]
fn default_workflow_pr193_uses_a_finalization_boundary_not_a_template() {
    let doc = default_workflow_readiness_doc();

    assert!(
        doc.contains(PR193_FINALIZATION_BOUNDARY_HEADING),
        "PR #193 readiness docs must define a finalization evidence boundary"
    );
    assert!(
        !doc.contains(PR193_FINALIZATION_TEMPLATE_HEADING),
        "PR #193 readiness evidence must not remain a placeholder template"
    );
}

#[test]
fn default_workflow_pr193_boundary_requires_external_exact_head_evidence() {
    let doc = default_workflow_readiness_doc();
    let section = pr193_finalization_section(doc);

    assert_all_present(
        section,
        &[
            "readiness contract",
            "Do not record a specific PR #193 `headRefOid`",
            "committed exact-head evidence here would become self-stale",
            "After the final commit is pushed",
            "headRefOid",
            "outside the repository commit",
            "same-repository PR body, a trusted PR comment, or status summary",
            "Do not treat uncommitted local documentation edits as evidence for a PR head",
            "externally recorded exact head",
        ],
        "PR #193 finalization evidence boundary",
    );
    assert_all_absent(
        section,
        &[
            "Evaluated `headRefOid`",
            "| Evidence time |",
            "Observed result",
            "Passed with exit 0 for the evaluated head",
            "Succeeded with exit 0 for the evaluated head",
            "this exact-head evidence snapshot",
            "<headRefOid",
            "<YYYY-MM-DDTHH:MM:SSZ>",
            "<observed result",
            "placeholder",
            "template",
        ],
        "PR #193 finalization evidence boundary",
    );
    assert!(
        !contains_40_char_hex_sha(section),
        "PR #193 committed readiness section must not hardcode a transient head SHA"
    );
    assert!(
        contains_any_normalized(
            doc,
            &["do not treat uncommitted local documentation edits as evidence for a pr head"],
        ),
        "default-workflow readiness docs must preserve the uncommitted-local-evidence warning"
    );
}

#[test]
fn default_workflow_pr193_boundary_lists_executable_command_gates() {
    let section = pr193_finalization_section(default_workflow_readiness_doc());

    for command in PR193_REQUIRED_COMMANDS {
        assert_command_row_has_required_result(section, command);
    }
}

#[test]
fn default_workflow_pr193_boundary_names_change_scope_and_no_manual_merge() {
    let section = pr193_finalization_section(default_workflow_readiness_doc());

    assert!(
        contains_any_normalized(
            section,
            &[
                "bounded change scope",
                "the allowed readiness conclusion for pr #193 is narrow",
            ],
        ),
        "PR #193 finalization boundary must require a bounded change scope"
    );
    assert!(
        contains_any_normalized(
            section,
            &[
                "without manually merging",
                "no manual merge was performed",
                "no-manual-merge",
                "do not manually merge",
            ],
        ),
        "PR #193 finalization boundary must explicitly preserve the no-manual-merge boundary"
    );
}

fn readiness_doc() -> &'static str {
    include_str!("../../../docs/lesson-session-readiness.md")
}

fn default_workflow_readiness_doc() -> &'static str {
    include_str!("../../../docs/default-workflow-pr-readiness.md")
}

fn matrix_section(doc: &str) -> &str {
    section(doc, MATRIX_HEADING)
}

fn scenario_map_section(doc: &str) -> &str {
    section(doc, SCENARIO_MAP_HEADING)
}

fn pr193_finalization_section(doc: &str) -> &str {
    section(doc, PR193_FINALIZATION_BOUNDARY_HEADING)
}

fn section<'a>(doc: &'a str, heading: &str) -> &'a str {
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("lesson readiness doc must contain the {heading} heading"));
    let rest = &doc[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());

    &rest[..end]
}

fn table_rows(matrix: &str) -> Vec<Vec<&str>> {
    markdown_table_rows(matrix, REQUIRED_HEADER, REQUIRED_DIVIDER)
}

fn markdown_table_rows<'a>(section: &'a str, header: &str, divider: &str) -> Vec<Vec<&'a str>> {
    section
        .lines()
        .filter(|line| line.starts_with('|'))
        .filter(|line| *line != header && *line != divider)
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

fn assert_command_row_has_required_result(section: &str, command: &str) {
    let row = section
        .lines()
        .find(|line| line.contains(command))
        .unwrap_or_else(|| panic!("PR #193 finalization boundary is missing command: {command}"));

    assert!(
        !row.contains('<') && !row.contains('>'),
        "command gate must not be a placeholder: {row}"
    );
    assert!(
        contains_any_normalized(
            row,
            &["required result", "passed", "succeeds", "successfully"]
        ),
        "command gate must describe the required final-head result: {row}"
    );
}

fn contains_40_char_hex_sha(text: &str) -> bool {
    text.split(|ch: char| !ch.is_ascii_hexdigit())
        .any(|token| token.len() == 40)
}

fn contains_any_normalized(text: &str, normalized_phrases: &[&str]) -> bool {
    let normalized_text = normalize(text);
    normalized_phrases
        .iter()
        .any(|phrase| normalized_text.contains(phrase))
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
