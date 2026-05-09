use super::{
    PR173_RECOVERY_VALIDATION_COMMANDS, SHARING_SUCCESS_CLAIM_PATTERNS, assert_contains_all,
    default_workflow_pr_readiness_doc, sharing_readiness_boundary_doc,
};

const EVIDENCE_TEMPLATE_HEADING: &str = "## Evidence record template";
const NO_OP_HEADING: &str = "## No-op justification";
const READINESS_COMMENT_HEADING: &str = "## Readiness comment";
const SHARING_PROFILE_HEADING: &str = "## Sharing-readiness recovery profile";

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

    assert_contains_all(
        "PR #173 sharing workflow integration contract",
        &format!("{profile}\n{boundary}"),
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

fn section<'a>(docs: &'a str, heading: &str) -> &'a str {
    let start = docs
        .find(heading)
        .unwrap_or_else(|| panic!("document must include `{heading}`"));
    let after_heading = start + heading.len();
    let rest = &docs[after_heading..];
    let end = match rest.find("\n## ") {
        Some(next_heading) => next_heading,
        None => rest.len(),
    };
    &docs[start..after_heading + end]
}

fn assert_no_success_claims(label: &str, text: &str) {
    let normalized = text.to_lowercase();
    let present = SHARING_SUCCESS_CLAIM_PATTERNS
        .iter()
        .filter(|phrase| normalized.contains(**phrase))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        present.is_empty(),
        "{label} must stay bounded to executed recovery evidence, found success claims: {present:?}"
    );
}
