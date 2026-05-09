use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const ARTIFACT_PATH: &str = "docs/default-workflow-pr-readiness.md";
const REQUIRED_VALIDATED_EVIDENCE_HEAD_COMMANDS: &[&str] = &[
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
fn readiness_artifact_has_readiness_review_and_finalization_sections() {
    let artifact = read_artifact();

    assert_ordered_sections(
        artifact,
        &[
            "## Readiness evidence",
            "## Review evidence",
            "## Finalization evidence",
            "## Nonclaims",
        ],
    );
}

#[test]
fn validated_evidence_head_records_required_commands_without_timeout_wrappers() {
    let artifact = read_artifact();
    let evidence_head = section(
        artifact,
        "### Validated evidence-head executable evidence",
        "### Historical same-head outside-in testing evidence",
    );

    assert!(
        evidence_head.contains("`NODE_OPTIONS=--max-old-space-size=32768`"),
        "validated evidence-head evidence must record the required Node memory setting"
    );
    assert!(
        evidence_head.contains("no timeout wrapper"),
        "validated evidence-head evidence must explicitly say timeout wrappers were not used"
    );

    for command in REQUIRED_VALIDATED_EVIDENCE_HEAD_COMMANDS {
        assert!(
            evidence_head.contains(command),
            "validated evidence-head evidence must include executable command: {command}"
        );
    }

    assert!(
        !contains_timeout_wrapper(evidence_head),
        "validated evidence-head evidence must not include timeout wrapper commands"
    );
}

#[test]
fn local_git_evidence_frames_clean_status_as_pre_refinement_capture() {
    let local_git = section(
        read_artifact(),
        "### Local Git observations",
        "### GitHub PR #175 observations",
    );

    assert_contains_all_normalized(
        local_git,
        &[
            "before this refinement changed the artifact/test files",
            "pre-refinement observation for the validated evidence head",
            "not a claim about the post-edit worktree or the eventual publication head",
            "This refinement intentionally changes only the readiness artifact and the contract tests that guard it",
            "docs/default-workflow-pr-readiness.md",
            "crates/eatme-assets/src/default_workflow_pr_readiness_contract_tests.rs",
        ],
        "local git evidence",
    );
    assert!(
        !local_git.contains("The local branch was clean"),
        "local git evidence must not frame historical clean status as current handoff state"
    );
}

#[test]
fn artifact_distinguishes_validated_evidence_head_from_publication_head() {
    let artifact = read_artifact();
    let scope = section(artifact, "## Scope", "## Readiness evidence");

    assert_contains_all_normalized(
        scope,
        &[
            "Validated evidence head",
            "Artifact publication head",
            "not embedded in this committed artifact",
            "committing a documentation refinement changes the PR head",
            "does not claim that its own eventual publication commit has checked itself",
        ],
        "scope evidence",
    );
    assert!(
        !scope.contains("| Checked-out local HEAD |"),
        "scope must not frame the evidence SHA as the committed artifact publication head"
    );
    assert!(
        !artifact.contains("### Current-head executable evidence"),
        "artifact must use validated evidence-head wording instead of self-staling current-head wording"
    );
}

#[test]
fn finalization_evidence_records_unmerged_pr_status_and_workflow_boundary() {
    let finalization = section(read_artifact(), "## Finalization evidence", "## Nonclaims");

    assert_contains_all_normalized(
        finalization,
        &[
            "PR #175 remains unmerged",
            "No manual merge was performed",
            "workflow readiness/review/finalization evidence",
            "Finalization status: `merge-ready-after-publication-head-checks`",
            "post-push publication head/check rollup recorded outside this file",
        ],
        "finalization evidence",
    );
    assert!(
        !finalization.contains("limited-ready"),
        "finalization evidence must assert the exact finalization status instead of passing on stale limited-ready wording"
    );
    assert!(
        !finalization.contains("checked-out local branch head is the same SHA"),
        "finalization evidence must not claim the committed artifact publication head is the validated evidence head"
    );
}

#[test]
fn historical_branch_ref_evidence_is_bounded_to_recorded_execution_context() {
    let historical = section(
        read_artifact(),
        "### Historical same-head outside-in testing evidence",
        "## Review evidence",
    );

    assert!(
        historical.contains("branch ref as resolved at execution time"),
        "historical uvx evidence must state that the install target was a mutable branch ref"
    );
    assert_contains_all_normalized(
        historical,
        &[
            "not an immutable SHA-pinned install reference",
            "same-head claim depends on the recorded execution context",
        ],
        "historical uvx evidence",
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
    let mut previous_position = 0;

    for heading in headings {
        let position = artifact
            .find(heading)
            .unwrap_or_else(|| panic!("missing required readiness artifact section: {heading}"));
        assert!(
            position >= previous_position,
            "section {heading} must appear after the previous required section"
        );
        previous_position = position;
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
