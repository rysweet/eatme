use super::{
    PR173_RECOVERY_VALIDATION_COMMANDS, assert_contains_all, assert_contains_all_across,
    assert_no_success_claims, default_workflow_pr_readiness_doc, section,
    sharing_readiness_boundary_doc,
};

const EVIDENCE_TEMPLATE_HEADING: &str = "## Evidence record template";
const NO_OP_HEADING: &str = "## No-op justification";
const READINESS_COMMENT_HEADING: &str = "## Readiness comment";
const SHARING_PROFILE_HEADING: &str = "## Sharing-readiness recovery profile";

#[test]
fn owner_free_exit_no_op_guard_requires_direct_current_head_revalidation() {
    let doc = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "owner-free NO_OP_GUARD recovery contract",
        doc,
        &[
            "owner-free exit",
            "`NO_OP_GUARD`",
            "direct current-head verification",
            "must not be treated as `MERGE_READY`",
            "workflow-accepted no-op justification",
            "`NOT_MERGE_READY`",
        ],
    );
}

#[test]
fn merge_ready_gate_requires_actions_workflow_completion_and_non_check_evidence() {
    let contract = section(default_workflow_pr_readiness_doc(), "## Readiness contract");
    let decision = section(
        default_workflow_pr_readiness_doc(),
        "## Merge-ready decision",
    );

    assert_contains_all_across(
        "strict merge-ready evidence gate",
        &[contract, decision],
        &[
            "green GitHub Actions",
            "workflow completion",
            "necessary but not sufficient",
            "runnable QA/scenario evidence",
            "documentation impact review",
            "three quality-audit SEEK / VALIDATE / FIX cycles",
            "focused diff scope",
            "PR description evidence",
            "clean final cycle",
        ],
    );
}

#[test]
fn no_op_recovery_output_must_tie_current_head_checks_to_evidence_or_blockers() {
    let no_op = section(default_workflow_pr_readiness_doc(), NO_OP_HEADING);

    assert_contains_all(
        "workflow-accepted no-op output contract",
        no_op,
        &[
            "current head/checks",
            "PR head checks",
            "merge-ready blockers or evidence",
            "explicit workflow-accepted No-op justification",
            "current-head evidence",
            "current PR head",
        ],
    );
}

#[test]
fn bounded_readiness_claims_must_not_assert_tweedle_or_player_decode() {
    let doc = default_workflow_pr_readiness_doc();
    let comment = section(doc, READINESS_COMMENT_HEADING);
    let no_op = section(doc, NO_OP_HEADING);

    assert_contains_all_across(
        "bounded readiness non-claims",
        &[doc, comment, no_op],
        &[
            "full Tweedle/player decode",
            "unless directly proven",
            "does not claim",
        ],
    );
    assert_no_success_claims("readiness comment template", comment);
    assert_no_success_claims("no-op justification template", no_op);
}

#[test]
fn current_head_evidence_template_requires_readiness_review_and_finalization_fields() {
    let template = section(
        default_workflow_pr_readiness_doc(),
        EVIDENCE_TEMPLATE_HEADING,
    );

    assert_contains_all(
        "default-workflow current-head evidence template",
        template,
        &[
            "`repository`",
            "`branch`",
            "`head_sha`",
            "`worktree_status`",
            "`pr_number`",
            "`pr_head_branch`",
            "`pr_head_sha`",
            "`state`",
            "`draft`",
            "`checks`",
            "`merge_state`",
            "`asset_validation`",
            "`gadugi_freshness`",
            "`docs_build`",
            "`quality_gate`",
            "`workflow_readiness_evidence`",
            "`review_evidence`",
            "`finalization_evidence`",
            "`bounded_claim`",
        ],
    );
}

#[test]
fn workflow_no_op_justification_requires_no_repository_delta_and_explicit_non_merge() {
    let no_op = section(default_workflow_pr_readiness_doc(), NO_OP_HEADING);

    assert_contains_all(
        "default-workflow no-op justification contract",
        no_op,
        &[
            "workflow-accepted no-op justification",
            "no repository changes were required",
            "current-head evidence",
            "review evidence",
            "finalization evidence",
            "no manual merge was performed",
            "does not claim hosted sharing",
            "deployment success",
            "merge completion",
        ],
    );
}

#[test]
fn readiness_comment_template_keeps_finalization_bounded_to_executed_evidence() {
    let comment = section(
        default_workflow_pr_readiness_doc(),
        READINESS_COMMENT_HEADING,
    );

    assert_contains_all(
        "default-workflow readiness comment contract",
        comment,
        &[
            "Default-workflow recovery recorded for PR #173",
            "current branch head ${HEAD_SHA}",
            "asset validation",
            "generated Gadugi freshness",
            "strict documentation build",
            "quality gates",
            "PR metadata review",
            "bounded sharing-readiness claim review",
            "does not claim hosted sharing",
            "manual merge",
        ],
    );
    assert_no_success_claims("readiness comment template", comment);
}

#[test]
fn sharing_recovery_profile_rejects_timeout_wrappers_and_manual_merge_as_errors() {
    let doc = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "default-workflow error handling contract",
        doc,
        &[
            "Do not wrap these commands in shell `timeout` helpers.",
            "It does not merge the PR.",
            "manual merge",
            "Do not post readiness when any gate is failing, pending, stale, or tied to a different head",
            "Local/PR head mismatch",
        ],
    );
}

#[test]
fn pr173_sharing_workflow_contract_links_default_readiness_to_boundary_evidence() {
    let default_doc = default_workflow_pr_readiness_doc();
    let profile = section(default_doc, SHARING_PROFILE_HEADING);
    let boundary = sharing_readiness_boundary_doc();

    for command in PR173_RECOVERY_VALIDATION_COMMANDS {
        assert!(
            profile.contains(command)
                || boundary.contains(command)
                || default_doc.contains(command),
            "PR #173 workflow contract must preserve validation command `{command}`"
        );
    }

    assert_contains_all_across(
        "PR #173 sharing workflow integration contract",
        &[profile, boundary],
        &[
            "PR `#173`",
            "wave6-deployed-sharing-gap-1778302300",
            "student-artifact-package-share-evidence",
            "teacher-community-sharing-loop",
            "current-head evidence",
            "classroom sharing-readiness boundary",
            "must not claim hosted sharing",
            "deployed sharing",
            "platform success",
            "full UI automation",
            "rendering correctness",
            "grading correctness",
            "creative assessment",
            "Save completion",
            "lesson completion",
        ],
    );
}
