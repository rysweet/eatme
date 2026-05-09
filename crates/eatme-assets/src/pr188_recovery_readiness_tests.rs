const PR188_READINESS_DOC: &str = include_str!("../../../docs/pr-188-recovery-readiness.md");
const DEFAULT_WORKFLOW_DOC: &str = include_str!("../../../docs/default-workflow-pr-readiness.md");
const MKDOCS_YML: &str = include_str!("../../../mkdocs.yml");
const PR188_BRANCH: &str = "wave6-real-alice-smoke-report-1778302300";
const CURRENT_HEAD_COMMANDS: &[&str] = &[
    "NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh",
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
];

#[test]
fn recovery_workflow_envelope_is_branch_bound_current_head_and_non_merge() {
    assert_contains_all(
        "PR #188 readiness workflow envelope",
        PR188_READINESS_DOC,
        &[
            "Default Workflow PR Readiness",
            PR188_BRANCH,
            "current branch `HEAD`",
            "Do not accept a no-op when untracked files, staged files, unstaged files",
            "No manual PR merge was performed.",
        ],
    );
    assert_eq!(
        MKDOCS_YML.matches("PR #188 Recovery Readiness").count(),
        1,
        "mkdocs nav must expose exactly one PR #188 recovery page."
    );
}

#[test]
fn launch_smoke_scope_defines_silver_thread_and_lists_canonical_non_claims() {
    assert_contains_all(
        "launch-smoke scope",
        PR188_READINESS_DOC,
        &[
            "silver-thread/e2e launch-smoke means the narrow end-to-end path",
            "It does not mean complete UI-driven lesson execution.",
        ],
    );

    let non_claims = canonical_default_workflow_non_claims();
    assert_contains_all(
        "PR #188 default-workflow non-claim boundary",
        PR188_READINESS_DOC,
        &non_claims,
    );

    assert_contains_none(
        "PR #188 readiness",
        PR188_READINESS_DOC,
        &[
            "full UI automation is proven",
            "visible rendering correctness is proven",
            "grading is proven",
            "Save completion is proven",
            "lesson completion is proven",
            "complete end-to-end lesson execution is proven",
        ],
    );
}

#[test]
fn readiness_checks_are_current_head_executable_commands_without_timeout_wrappers() {
    let evidence_sections = [
        (
            "usage",
            section(PR188_READINESS_DOC, "## Usage", "## No-op acceptance"),
        ),
        (
            "no-op acceptance",
            section(
                PR188_READINESS_DOC,
                "## No-op acceptance",
                "## Review and finalization evidence",
            ),
        ),
        (
            "recovery command sequence",
            section(PR188_READINESS_DOC, "## Recovery command sequence", ""),
        ),
    ];

    for (label, text) in evidence_sections {
        assert_contains_all(label, text, CURRENT_HEAD_COMMANDS);
    }

    assert_contains_none(
        "readiness commands",
        PR188_READINESS_DOC,
        &["timeout ", "gtimeout ", "coreutils timeout"],
    );
}

#[test]
fn fallback_repair_and_docs_finalization_are_bounded_to_failing_surfaces() {
    assert_contains_all(
        "bounded repair and docs finalization",
        PR188_READINESS_DOC,
        &[
            "repair only the directly failing PR #188 readiness surface",
            "mkdocs build --strict (include only when documentation changed)",
            "replace the no-op sentence with a bounded change summary",
        ],
    );
}

fn canonical_default_workflow_non_claims() -> Vec<&'static str> {
    let start = DEFAULT_WORKFLOW_DOC
        .find("First-lesson completion is not proven.")
        .expect("default workflow canonical non-claims must start with first-lesson completion");
    let end = DEFAULT_WORKFLOW_DOC[start..]
        .find("```")
        .map(|offset| start + offset)
        .expect("default workflow canonical non-claims must end at a fenced block");

    DEFAULT_WORKFLOW_DOC[start..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn section<'a>(document: &'a str, heading: &str, next_heading: &str) -> &'a str {
    let start = document
        .find(heading)
        .unwrap_or_else(|| panic!("missing section heading: {heading}"));
    let tail = &document[start..];
    if next_heading.is_empty() {
        return tail;
    }
    let end = tail
        .find(next_heading)
        .unwrap_or_else(|| panic!("missing next section heading: {next_heading}"));
    &tail[..end]
}

fn assert_contains_all(label: &str, haystack: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .filter(|needle| !haystack.contains(*needle))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required text: {missing:?}"
    );
}

fn assert_contains_none(label: &str, haystack: &str, needles: &[&str]) {
    let present = needles
        .iter()
        .filter(|needle| haystack.contains(*needle))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "{label} contains forbidden text: {present:?}"
    );
}
