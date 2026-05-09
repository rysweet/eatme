use super::{
    PR173_RECOVERY_VALIDATION_COMMANDS, assert_contains_all, assert_no_success_claims,
    full_sha_mentions, section, sharing_readiness_boundary_doc,
};

const RECOVERY_ARTIFACT_HEADING: &str = "## Recovery evidence artifacts";

#[test]
fn sharing_recovery_docs_describe_artifact_contract_not_committed_point_in_time_state() {
    let evidence = section(sharing_readiness_boundary_doc(), RECOVERY_ARTIFACT_HEADING);

    assert_contains_all(
        "sharing recovery evidence artifact contract",
        evidence,
        &[
            "current-head evidence",
            "review artifact such as a PR comment, session artifact, or final recovery note",
            "Do not commit time-sensitive SHAs",
            "dirty/clean worktree claims",
            "validation outcomes",
        ],
    );
    let sha_mentions = full_sha_mentions(evidence);
    assert!(
        sha_mentions.is_empty(),
        "permanent sharing readiness docs must not pin point-in-time SHAs; found {} 40-hex mention(s)",
        sha_mentions.len()
    );
}

#[test]
fn sharing_recovery_docs_keep_pr173_as_reusable_profile_example() {
    let evidence = section(sharing_readiness_boundary_doc(), RECOVERY_ARTIFACT_HEADING);

    assert_contains_all(
        "PR 173 sharing recovery profile",
        evidence,
        &[
            "sharing-readiness PR such as `#173`",
            "wave6-deployed-sharing-gap-1778302300",
            "gh pr view 173",
            "headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url",
        ],
    );
}

#[test]
fn sharing_recovery_docs_require_current_head_validation_before_readiness_claims() {
    let evidence = section(sharing_readiness_boundary_doc(), RECOVERY_ARTIFACT_HEADING);
    let missing = PR173_RECOVERY_VALIDATION_COMMANDS
        .iter()
        .filter(|command| !evidence.contains(**command))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "sharing recovery evidence must list required validation commands: {missing:?}"
    );
    assert_contains_all(
        "current-head validation rules",
        evidence,
        &[
            "Rerun for the evaluated head before claiming asset readiness.",
            "Rerun whenever canonical scenario assets or generated adapters are in scope.",
            "Rerun when this guide or linked readiness docs change.",
            "Rerun before full repository readiness claims.",
        ],
    );
    assert!(
        !evidence.contains("not run in this recovery step"),
        "permanent docs must not preserve obsolete recovery-step validation status"
    );
}

#[test]
fn sharing_recovery_docs_separate_local_and_pr_head_claims_without_stale_fixture_rows() {
    let evidence = section(sharing_readiness_boundary_doc(), RECOVERY_ARTIFACT_HEADING);

    assert_contains_all(
        "local and PR head separation",
        evidence,
        &[
            "If local `HEAD` differs from the PR head",
            "must not describe local validation as proof for the published PR head",
            "If the heads match and the checks pass",
            "current head satisfies the classroom sharing-readiness boundary",
        ],
    );

    let stale_rows = [
        "published PR head SHA",
        "evaluated local HEAD SHA",
        "historical validation SHA",
        "evaluated worktree state",
        "master sync status",
    ];
    let present = stale_rows
        .iter()
        .filter(|row| evidence.contains(**row))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "permanent docs must not keep stale point-in-time recovery fixture rows: {present:?}"
    );
}

#[test]
fn sharing_recovery_evidence_keeps_forbidden_claims_explicitly_unproven() {
    let evidence = section(sharing_readiness_boundary_doc(), RECOVERY_ARTIFACT_HEADING);

    assert_contains_all(
        "sharing recovery bounded wording evidence",
        evidence,
        &[
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
    assert_no_success_claims("sharing recovery evidence", evidence);
}
