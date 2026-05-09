use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const ARTIFACT_PATH: &str = "docs/default-workflow-pr-readiness.md";
const REQUIRED_VALIDATION_COMMANDS: &[&str] = &[
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];
const UNSUPPORTED_SUCCESS_CLAIMS: &[&str] = &[
    "full alice ui automation is verified",
    "full ui automation is verified",
    "rendering correctness is verified",
    "grading correctness is verified",
    "creative assessment is verified",
    "lesson completion is verified",
    "manual real alice launch is verified",
];

#[test]
fn readiness_artifact_has_required_structural_sections() {
    let artifact = read_artifact();

    assert_ordered_sections(
        artifact,
        &[
            "## Readiness contract",
            "## Configuration",
            "## Exact-head setup",
            "## GitHub evidence",
            "## Local QA evidence",
            "## Decision gate",
            "## Verdicts",
            "## Starter-project evidence boundary",
            "## Executable starter-project boundary check",
        ],
    );
}

#[test]
fn readiness_contract_includes_required_validation_commands() {
    let artifact = read_artifact();

    for command in REQUIRED_VALIDATION_COMMANDS {
        assert!(
            artifact.contains(command),
            "readiness artifact must include validation command: {command}"
        );
    }

    assert!(
        !contains_timeout_wrapper(artifact),
        "readiness artifact must not include timeout wrapper commands"
    );
}

#[test]
fn starter_project_evidence_boundary_section_exists_with_bounded_language() {
    let boundary = section(
        read_artifact(),
        "## Starter-project evidence boundary",
        "## Executable starter-project boundary check",
    );

    assert_contains_all_normalized(
        boundary,
        &["bounded setup evidence", "not PR readiness"],
        "starter-project evidence boundary",
    );
}

#[test]
fn executable_boundary_check_has_overclaim_table() {
    let artifact = read_artifact();
    let check_section = section(
        artifact,
        "## Executable starter-project boundary check",
        "## Implementation consistency",
    );

    assert!(
        check_section.contains("| Prohibited phrase | Bounded replacement |"),
        "executable boundary check must include the overclaim rule table"
    );
}

#[test]
fn unsupported_success_claim_fixture_is_rejected() {
    let fixture = "\
This recovery proves full Alice UI automation is verified.
Rendering correctness is verified.
Grading correctness is verified.
";

    assert_eq!(
        unsupported_success_claim_lines(fixture),
        vec![
            "This recovery proves full Alice UI automation is verified.",
            "Rendering correctness is verified.",
            "Grading correctness is verified.",
        ]
    );
}

#[test]
fn readiness_artifact_does_not_make_unsupported_success_claims() {
    let violations = unsupported_success_claim_lines(read_artifact());

    assert!(
        violations.is_empty(),
        "readiness artifact contains unsupported success claims:\n{}",
        violations.join("\n")
    );
}

fn read_artifact() -> &'static str {
    static ARTIFACT: OnceLock<String> = OnceLock::new();

    ARTIFACT
        .get_or_init(|| {
            fs::read_to_string(repository_root().join(ARTIFACT_PATH))
                .unwrap_or_else(|error| panic!("failed to read {ARTIFACT_PATH}: {error}"))
        })
        .as_str()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn assert_ordered_sections(artifact: &str, headings: &[&str]) {
    let mut search_start = 0;

    for heading in headings {
        let relative_position = artifact[search_start..].find(heading).unwrap_or_else(|| {
            panic!("missing or out-of-order readiness artifact section: {heading}")
        });
        search_start += relative_position + heading.len();
    }
}

fn section<'a>(artifact: &'a str, start_heading: &str, end_heading: &str) -> &'a str {
    let start = artifact
        .find(start_heading)
        .unwrap_or_else(|| panic!("missing section heading {start_heading}"));
    let after_start = start + start_heading.len();
    let relative_end = artifact[after_start..]
        .find(end_heading)
        .unwrap_or_else(|| panic!("missing section heading {end_heading} after {start_heading}"));

    &artifact[start..after_start + relative_end]
}

fn contains_timeout_wrapper(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .any(|line| line.starts_with("timeout ") || line.contains("`timeout "))
}

fn assert_contains_all_normalized(text: &str, expected_fragments: &[&str], context: &str) {
    let normalized_text = normalize_whitespace(text);

    for expected in expected_fragments {
        assert!(
            normalized_text.contains(&normalize_whitespace(expected)),
            "{context} must record: {expected}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());

    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }

    normalized
}

fn unsupported_success_claim_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| contains_unsupported_success_claim(line))
        .collect()
}

fn contains_unsupported_success_claim(line: &str) -> bool {
    UNSUPPORTED_SUCCESS_CLAIMS
        .iter()
        .any(|claim| contains_ascii_case_insensitive(line, claim))
}

fn contains_ascii_case_insensitive(text: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    text.as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
