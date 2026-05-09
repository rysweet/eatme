use std::fs;
use std::path::{Path, PathBuf};

const ARTIFACT_PATH: &str = "default-workflow-attempt.log";
const LOCAL_HEAD_LABEL: &str = "- Local `HEAD`:";
const PR_HEAD_LABEL: &str = "- PR #175 `headRefOid`:";

#[test]
fn evidence_artifact_binds_scope_to_the_exact_pr_head() {
    let artifact = read_artifact();
    let local_head = sha_after_label(&artifact, LOCAL_HEAD_LABEL);
    let pr_head = sha_after_label(&artifact, PR_HEAD_LABEL);

    assert_eq!(
        local_head, pr_head,
        "the artifact must not claim readiness when local HEAD differs from PR #175 headRefOid"
    );
    assert!(
        is_full_sha(local_head),
        "local HEAD must be a full 40-character SHA: {local_head}"
    );
    assert!(
        artifact.contains(&format!(
            "All command evidence below is bound to\n`{local_head}`."
        )),
        "command evidence must name the exact SHA it was generated against"
    );
    assert!(
        artifact.contains(&format!("At exact head `{local_head}`")),
        "bounded readiness must be tied to the exact reviewed SHA"
    );
}

#[test]
fn repository_state_evidence_records_collection_commands() {
    let artifact = read_artifact();
    let section = section(
        &artifact,
        "## Repository state evidence",
        "## GitHub PR metadata evidence",
    );
    let required_commands = [
        "git rev-parse --abbrev-ref HEAD",
        "git rev-parse HEAD",
        "git status --short --branch",
        "git diff --name-only --diff-filter=U",
        "rg -n '^(<<<<<<<|=======|>>>>>>>)' --glob '!target/**' --glob '!**/.git/**' .",
    ];

    for command in required_commands {
        assert!(
            section.contains(command),
            "repository state evidence must be traceable to the exact collection command: {command}"
        );
    }
}

#[test]
fn verification_gate_evidence_records_required_commands_and_results() {
    let artifact = read_artifact();

    assert_section_has_command_and_success(
        &artifact,
        "### Asset validation",
        "### Generated Gadugi adapter check",
        "cargo run -q -p eatme-cli -- assets validate --json",
    );
    assert_section_has_command_and_success(
        &artifact,
        "### Generated Gadugi adapter check",
        "### Repository quality gates",
        "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    );
    assert_section_has_command_and_success(
        &artifact,
        "### Repository quality gates",
        "## Review evidence",
        "TMPDIR=/tmp ./scripts/quality-gates.sh",
    );
    assert_section_has_command_and_success(
        &artifact,
        "## GitHub PR metadata evidence",
        "## Verification gate evidence",
        "gh pr view 175 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
    );
}

#[test]
fn review_scope_and_limitations_are_explicit_without_overclaiming() {
    let artifact = read_artifact();
    let review = section(
        &artifact,
        "## Review evidence",
        "## Bounded readiness statement",
    );
    let limitations = section(
        &artifact,
        "## Limitations and unverified areas",
        "## Evidence collection blockers",
    );

    for reviewed_item in [
        "Local branch and exact `HEAD` identity.",
        "PR #175 exact `headRefOid` identity and mergeability metadata.",
        "Asset validation JSON summary.",
        "Generated Gadugi adapter check JSON summary.",
        "Repository quality gate completion and final coverage summary.",
        "Evidence wording in this artifact for unsupported overclaims.",
    ] {
        assert!(
            review.contains(reviewed_item),
            "review evidence must state what was reviewed: {reviewed_item}"
        );
    }

    for limitation in [
        "Full UI automation coverage.",
        "Grading correctness.",
        "Creative assessment correctness.",
        "Visible rendering correctness.",
        "First-lesson completion by a learner or automated end-to-end flow.",
        "Manual real Alice launch behavior",
        "GitHub Pages deployment behavior",
    ] {
        assert!(
            limitations.contains(limitation),
            "limitations must explicitly preserve this unverified area: {limitation}"
        );
    }

    assert_no_prohibited_affirmative_claims(&artifact);
}

#[test]
fn head_mismatch_fixture_requires_an_explicit_blocker() {
    let fixture = "\
# PR #175 Evidence Artifact Contract

## Scope

- Local `HEAD`: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
- PR #175 `headRefOid`: `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb`

## Bounded readiness statement

At exact head `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`, this is ready.
";

    assert!(
        head_mismatch_without_blocker(fixture),
        "a local/PR head mismatch must be treated as a blocker instead of readiness evidence"
    );
}

#[test]
fn prohibited_success_claim_fixture_is_rejected() {
    let fixture = "The full Alice UI flow is automated and grading correctness is verified.";

    let violation = prohibited_affirmative_claim_lines(fixture);

    assert_eq!(
        violation,
        vec!["The full Alice UI flow is automated and grading correctness is verified."]
    );
}

fn read_artifact() -> String {
    fs::read_to_string(repository_root().join(ARTIFACT_PATH))
        .unwrap_or_else(|error| panic!("failed to read {ARTIFACT_PATH}: {error}"))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha_after_label<'a>(artifact: &'a str, label: &str) -> &'a str {
    artifact
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .and_then(first_backtick_value)
        .unwrap_or_else(|| panic!("missing SHA label {label}"))
}

fn first_backtick_value(value: &str) -> Option<&str> {
    let start = value.find('`')? + 1;
    let end = value[start..].find('`')?;
    Some(&value[start..start + end])
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
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

fn assert_section_has_command_and_success(
    artifact: &str,
    start_heading: &str,
    end_heading: &str,
    command: &str,
) {
    let content = section(artifact, start_heading, end_heading);
    assert!(
        content.contains(command),
        "{start_heading} must record command {command}"
    );
    assert!(
        content.contains("- Exit code: 0"),
        "{start_heading} must record a successful command result"
    );
}

fn assert_no_prohibited_affirmative_claims(artifact: &str) {
    let violations = prohibited_affirmative_claim_lines(artifact);
    assert!(
        violations.is_empty(),
        "artifact contains unsupported affirmative claims:\n{}",
        violations.join("\n")
    );
}

fn prohibited_affirmative_claim_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| contains_prohibited_success_claim(line))
        .collect()
}

fn contains_prohibited_success_claim(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let prohibited = [
        "full alice ui flow is automated",
        "full ui automation coverage is verified",
        "grading correctness is verified",
        "creative assessment correctness is verified",
        "visible rendering correctness is verified",
        "first-lesson completion is verified",
        "learner completed the first lesson",
        "rendering correctness is proven",
    ];

    prohibited.iter().any(|claim| lower.contains(claim))
}

fn head_mismatch_without_blocker(artifact: &str) -> bool {
    let local_head = sha_after_label(artifact, LOCAL_HEAD_LABEL);
    let pr_head = sha_after_label(artifact, PR_HEAD_LABEL);

    local_head != pr_head && !artifact.to_ascii_lowercase().contains("blocker")
}
