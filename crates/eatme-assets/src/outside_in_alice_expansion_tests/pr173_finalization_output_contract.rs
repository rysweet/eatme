use super::{assert_contains_all, default_workflow_pr_readiness_doc, section};

const GITHUB_METADATA_HEADING: &str = "## GitHub metadata fields";
const NO_OP_HEADING: &str = "## No-op justification";
const READINESS_COMMENT_HEADING: &str = "## Readiness comment";

#[test]
fn no_op_justification_names_current_head_checks_scope_and_blockers() {
    let no_op = section(default_workflow_pr_readiness_doc(), NO_OP_HEADING);

    assert_contains_all(
        "PR #173 no-op finalization output contract",
        no_op,
        &[
            "No-op justification: PR #173 current head ${PR_HEAD_SHA}",
            "current GitHub checks/mergeability",
            "sharing readiness is limited to classroom review handoff readiness",
            "no merge-ready blockers remain",
            "no repository edits or commits were required",
            "Changed-file scope",
            "Blockers",
        ],
    );
}

#[test]
fn readiness_comment_supersedes_stale_body_and_lists_skipped_jobs_as_non_evidence() {
    let comment = section(
        default_workflow_pr_readiness_doc(),
        READINESS_COMMENT_HEADING,
    );

    assert_contains_all(
        "PR #173 readiness evidence publisher contract",
        comment,
        &[
            "Supersedes stale PR-body evidence",
            "Skipped or manual jobs treated as non-evidence",
            "Deploy to GitHub Pages",
            "manual real Alice launch smoke",
        ],
    );
}

#[test]
fn ci_status_reader_records_check_run_urls_and_workflow_names() {
    let metadata = section(default_workflow_pr_readiness_doc(), GITHUB_METADATA_HEADING);

    assert_contains_all(
        "PR #173 CI/check status reader contract",
        metadata,
        &[
            "`statusCheckRollup`",
            "`detailsUrl`",
            "`workflowName`",
            "check-run names",
            "conclusions",
            "source URLs",
        ],
    );
}

#[test]
fn no_op_error_handling_rejects_dirty_or_mismatched_heads() {
    let no_op = section(default_workflow_pr_readiness_doc(), NO_OP_HEADING);

    assert_contains_all(
        "PR #173 no-op error handling contract",
        no_op,
        &[
            "Do not emit `No-op` when local `HEAD` differs from the PR head",
            "Do not emit `No-op` when the final worktree is dirty",
            "`NOT_MERGE_READY`",
            "specific blockers",
        ],
    );
}
