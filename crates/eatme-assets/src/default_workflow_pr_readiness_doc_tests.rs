use std::fs;
use std::path::{Path, PathBuf};

const PR_READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";

#[test]
fn pr_171_profile_preserves_published_branch_history() {
    let document = readiness_doc();
    let profile = section_between(
        &document,
        "## PR #171 run/observe recovery profile",
        "## Implementation consistency",
    );

    assert_contains_all(
        "PR #171 recovery profile",
        profile,
        &[
            "PR: 171",
            "Branch: wave6-scenario-run-observe-gap-1778302300",
            "Evaluation target: <evaluated-head-sha>",
            "Record the exact evaluated SHA in the PR body or readiness comment",
            "git fetch origin wave6-scenario-run-observe-gap-1778302300",
            "local HEAD, origin/<branch>, and PR headRefOid are identical",
            "published PR branch head only; no manual base merge, rebase, force-push, or rewritten history is readiness evidence",
            "does not use `origin/master` ancestry or merging as readiness evidence",
            "Do not rebase the branch for this recovery profile.",
        ],
    );
}

#[test]
fn pr_171_profile_does_not_hard_code_point_in_time_sha() {
    let document = readiness_doc();
    let profile = section_between(
        &document,
        "## PR #171 run/observe recovery profile",
        "## Implementation consistency",
    );

    assert_contains_none(
        "PR #171 recovery profile",
        profile,
        &["718d5d082283a579369821b884a1a5d8101aa957"],
    );
}

#[test]
fn readiness_comment_requires_exact_head_per_command_evidence() {
    let document = readiness_doc();
    let comment = section_between(&document, "## Readiness comment", "## Blocker handling");

    assert_contains_all(
        "readiness comment template",
        comment,
        &[
            "Default-workflow readiness recorded for PR #171 at exact head <final-head-sha>.",
            "`git rev-parse HEAD`: <final-head-sha>",
            "`gh pr view 171 --json headRefOid,mergeStateStatus,mergeable,isDraft,labels,reviewDecision,latestReviews,statusCheckRollup`",
            "`cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`: <pass>",
            "`cargo run -q -p eatme-cli -- assets validate --json`: <pass>",
            "`mkdocs build --strict`: <pass>",
            "`TMPDIR=/tmp ./scripts/quality-gates.sh`: <pass>",
            "Gate summaries are not enough without the concrete command results.",
        ],
    );
    assert_contains_none(
        "readiness comment template",
        comment,
        &["Verified gates", "manual fallback"],
    );
}

#[test]
fn pr_state_review_is_documented_as_automated_gate_without_claiming_owner_approval() {
    let document = readiness_doc();
    let pr_state_review = section_between(&document, "## PR-state review", "## Local QA evidence");

    assert_contains_all(
        "PR-state review",
        pr_state_review,
        &[
            "automated readiness gates",
            "`isDraft` is true",
            "`reviewDecision` is `CHANGES_REQUESTED`",
            "decisive latest review approval or change request belongs to a commit other than the evaluated head",
            "Owner-free `REVIEW_REQUIRED` is not a blocker by itself",
            "not infer owner approval from green checks",
        ],
    );
}

#[test]
fn readiness_contract_keeps_run_observe_gaps_and_non_claims_explicit() {
    let document = readiness_doc();

    assert_contains_all(
        "readiness contract",
        &document,
        &[
            "Explicit missing Run-window and observe-state evidence",
            "full UI automation",
            "full world execution",
            "visible rendering correctness",
            "grading",
            "creative assessment",
            "Save completion",
            "deployed sharing/platform success",
            "first-lesson completion",
            "Prior manual fallback logs are not readiness evidence.",
        ],
    );
}

#[test]
fn blocker_handling_discards_invalid_manual_fallback_evidence() {
    let document = readiness_doc();
    let blockers = section_between(&document, "## Blocker handling", "\n");

    assert!(
        document.contains("Invalid manual fallback evidence")
            && document.contains("Discard it from the readiness decision"),
        "{PR_READINESS_DOC} must make invalid manual fallback evidence a blocking condition"
    );
    assert!(
        !blockers.contains("publish readiness"),
        "the first blocker-handling line must not authorize success-shaped readiness evidence"
    );
}

fn readiness_doc() -> String {
    fs::read_to_string(repository_root().join(PR_READINESS_DOC)).unwrap()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn section_between<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = document
        .find(start)
        .unwrap_or_else(|| panic!("missing section heading {start:?} in {PR_READINESS_DOC}"));
    let after_start = &document[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing section boundary {end:?} in {PR_READINESS_DOC}"));
    &after_start[..end_index]
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required readiness language: {missing:?}"
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
        "{label} contains prohibited readiness language: {present:?}"
    );
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
