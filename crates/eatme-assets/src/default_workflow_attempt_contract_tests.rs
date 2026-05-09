use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const ARTIFACT_PATH: &str = "default-workflow-attempt.log";
const LOCAL_HEAD_LABEL: &str = "- Local `HEAD`:";
const PR_HEAD_LABEL: &str = "- PR #175 `headRefOid`:";
const PROHIBITED_SUCCESS_CLAIMS: &[&str] = &[
    "full alice ui flow is automated",
    "full ui automation coverage is verified",
    "grading correctness is verified",
    "creative assessment correctness is verified",
    "visible rendering correctness is verified",
    "first-lesson completion is verified",
    "learner completed the first lesson",
    "rendering correctness is proven",
];

#[test]
fn evidence_artifact_binds_scope_to_observed_heads() {
    let artifact = read_artifact();
    let local_head = sha_after_label(artifact, LOCAL_HEAD_LABEL);
    let pr_head = sha_after_label(artifact, PR_HEAD_LABEL);

    assert_full_sha(local_head, "local HEAD");
    assert_full_sha(pr_head, "PR #175 headRefOid");
    assert!(
        artifact.contains(&format!(
            "All command evidence below was collected with local `HEAD` at\n`{local_head}`"
        )),
        "command evidence must name the exact SHA it was collected against"
    );
    assert!(
        artifact
            .contains("intentional uncommitted evidence artifact and evidence-contract test edits"),
        "artifact must disclose intentional uncommitted evidence edits when command evidence predates the final worktree"
    );
    assert!(
        artifact.contains(&format!("At collected local `HEAD` `{local_head}`")),
        "bounded readiness must be tied to the exact collected SHA"
    );

    let readiness = section(
        artifact,
        "## Bounded readiness statement",
        "## Limitations and unverified areas",
    );
    if local_head == pr_head {
        assert!(
            readiness.contains("Local `HEAD` matches PR #175 `headRefOid`"),
            "matching heads may be used as readiness evidence only when explicitly stated"
        );
    } else {
        assert!(
            !has_unblocked_head_mismatch(artifact),
            "a local/PR head mismatch must be explicitly documented in the blocker section"
        );
        assert!(
            readiness.contains("No PR #175 readiness claim is made"),
            "a local/PR head mismatch must not be converted into PR readiness"
        );
        assert!(
            !readiness.contains("Local `HEAD` matches PR #175"),
            "readiness must not claim local HEAD matches PR #175 when the artifact records a mismatch"
        );
    }
}

#[test]
fn repository_state_evidence_records_collection_commands() {
    let artifact = read_artifact();
    let section = section(
        artifact,
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
        artifact,
        "### Asset validation",
        "### Generated Gadugi adapter check",
        "cargo run -q -p eatme-cli -- assets validate --json",
    );
    assert_section_has_command_and_success(
        artifact,
        "### Generated Gadugi adapter check",
        "### Repository quality gates",
        "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    );
    assert_section_has_command_and_success(
        artifact,
        "### Repository quality gates",
        "## Review evidence",
        "TMPDIR=/tmp ./scripts/quality-gates.sh",
    );
    assert_section_has_command_and_success(
        artifact,
        "## GitHub PR metadata evidence",
        "## Verification gate evidence",
        "gh pr view 175 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
    );
}

#[test]
fn review_scope_and_limitations_are_explicit_without_overclaiming() {
    let artifact = read_artifact();
    let review = section(
        artifact,
        "## Review evidence",
        "## Bounded readiness statement",
    );
    let limitations = section(
        artifact,
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

    assert_no_prohibited_affirmative_claims(artifact);
}

#[test]
fn head_mismatch_fixture_requires_an_explicit_blocker() {
    let missing_blocker = "\
# PR #175 Evidence Artifact Contract

## Scope

- Local `HEAD`: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
- PR #175 `headRefOid`: `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb`

## Bounded readiness statement

At collected local `HEAD` `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`, this is ready.
No blocker was observed.
";
    let explicit_blocker = "\
# PR #175 Evidence Artifact Contract

## Scope

- Local `HEAD`: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
- PR #175 `headRefOid`: `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb`

## Evidence collection blockers

Blocker: local `HEAD` is `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`,
while PR #175 `headRefOid` is `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb`.
Impact: the local validation evidence cannot be claimed as PR #175 readiness.
";

    assert!(
        has_unblocked_head_mismatch(missing_blocker),
        "a local/PR head mismatch must be treated as a blocker instead of readiness evidence"
    );
    assert!(
        !has_unblocked_head_mismatch(explicit_blocker),
        "an explicitly documented blocker satisfies the mismatch contract"
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

fn assert_full_sha(value: &str, label: &str) {
    assert!(
        is_full_sha(value),
        "{label} must be a full 40-character SHA: {value}"
    );
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

    PROHIBITED_SUCCESS_CLAIMS
        .iter()
        .any(|claim| lower.contains(claim))
}

fn has_unblocked_head_mismatch(artifact: &str) -> bool {
    let local_head = sha_after_label(artifact, LOCAL_HEAD_LABEL);
    let pr_head = sha_after_label(artifact, PR_HEAD_LABEL);

    local_head != pr_head && !has_explicit_head_mismatch_blocker(artifact, local_head, pr_head)
}

fn has_explicit_head_mismatch_blocker(artifact: &str, local_head: &str, pr_head: &str) -> bool {
    let Some(blockers) = optional_section_to_end(artifact, "## Evidence collection blockers")
    else {
        return false;
    };
    let blockers_lower = blockers.to_ascii_lowercase();
    let normalized_blockers = blockers_lower
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    blockers.contains(&format!("Blocker: local `HEAD` is `{local_head}`"))
        && blockers.contains(pr_head)
        && normalized_blockers.contains("pr #175 `headrefoid` is")
        && normalized_blockers.contains("cannot be claimed as pr #175 readiness")
}

fn optional_section_to_end<'a>(artifact: &'a str, start_heading: &str) -> Option<&'a str> {
    let start = artifact.find(start_heading)?;
    Some(&artifact[start..])
}
