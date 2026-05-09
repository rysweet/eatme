const PR188_READINESS_DOC: &str = include_str!("../../../docs/pr-188-recovery-readiness.md");
const DEFAULT_WORKFLOW_DOC: &str = include_str!("../../../docs/default-workflow-pr-readiness.md");
const MKDOCS_YML: &str = include_str!("../../../mkdocs.yml");

#[test]
fn recovery_workflow_envelope_is_branch_bound_current_head_and_non_merge() {
    assert_contains(
        PR188_READINESS_DOC,
        "Default Workflow PR Readiness",
        "PR #188 readiness must remain a specialization of the default workflow.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "wave6-real-alice-smoke-report-1778302300",
        "PR #188 readiness must name the recovery branch.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "current branch `HEAD`",
        "Evidence must be tied to the current branch HEAD.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "Do not accept a no-op when untracked files, staged files, unstaged files",
        "No-op acceptance must reject dirty root state.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "No manual PR merge was performed.",
        "Review/finalization evidence must keep merge completion out of the recovery handoff.",
    );
    assert_eq!(
        MKDOCS_YML.matches("PR #188 Recovery Readiness").count(),
        1,
        "mkdocs nav must expose exactly one PR #188 recovery page."
    );
}

#[test]
fn launch_smoke_scope_defines_silver_thread_and_lists_canonical_non_claims() {
    assert_contains(
        PR188_READINESS_DOC,
        "silver-thread/e2e launch-smoke means the narrow end-to-end path",
        "The launch-smoke boundary must be defined before evidence is accepted.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "It does not mean complete UI-driven lesson execution.",
        "The scope must not be described as full lesson execution.",
    );

    for non_claim in canonical_default_workflow_non_claims() {
        assert_contains(
            PR188_READINESS_DOC,
            non_claim,
            "PR #188 recovery must preserve the default-workflow non-claim boundary.",
        );
    }

    for forbidden_claim in [
        "full UI automation is proven",
        "visible rendering correctness is proven",
        "grading is proven",
        "Save completion is proven",
        "lesson completion is proven",
        "complete end-to-end lesson execution is proven",
    ] {
        assert!(
            !PR188_READINESS_DOC.contains(forbidden_claim),
            "PR #188 readiness must not overclaim: {forbidden_claim}"
        );
    }
}

#[test]
fn readiness_checks_are_current_head_executable_commands_without_timeout_wrappers() {
    let usage = section(PR188_READINESS_DOC, "## Usage", "## No-op acceptance");
    let no_op = section(
        PR188_READINESS_DOC,
        "## No-op acceptance",
        "## Review and finalization evidence",
    );
    let recovery_sequence = section(PR188_READINESS_DOC, "## Recovery command sequence", "");

    for section in [usage, no_op, recovery_sequence] {
        assert_contains(
            section,
            "NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh",
            "The quality gate evidence must use the saved Node heap preference and /tmp in one executable command.",
        );
        assert_contains(
            section,
            "cargo run -q -p eatme-cli -- assets validate --json",
            "Canonical asset validation must be executable current-HEAD evidence.",
        );
        assert_contains(
            section,
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "Generated Gadugi freshness must be executable current-HEAD evidence.",
        );
    }

    for forbidden in ["timeout ", "gtimeout ", "coreutils timeout"] {
        assert!(
            !PR188_READINESS_DOC.contains(forbidden),
            "Readiness commands must not use timeout wrappers: {forbidden}"
        );
    }
}

#[test]
fn fallback_repair_and_docs_finalization_are_bounded_to_failing_surfaces() {
    assert_contains(
        PR188_READINESS_DOC,
        "repair only the directly failing PR #188 readiness surface",
        "Failure handling must stay surgical.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "mkdocs build --strict (include only when documentation changed)",
        "Finalization evidence must include docs strict build only when docs changed.",
    );
    assert_contains(
        PR188_READINESS_DOC,
        "replace the no-op sentence with a bounded change summary",
        "Repair finalization must not reuse the no-op justification.",
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

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context}\nmissing required text: {needle}"
    );
}
