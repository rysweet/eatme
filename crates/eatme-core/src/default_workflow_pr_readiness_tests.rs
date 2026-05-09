use crate::default_workflow_pr_readiness::{
    AuditCycleEvidence, CheckConclusion, CheckRollupEvidence, CheckRunEvidence, Decision,
    FinalizationEvidence, HandoffOptions, LocalHeadEvidence, Mergeability, PrHeadMetadata,
    PreservedPatchEvidence, ScopeChange, ScopeSurface, SupplementalValidation,
    evaluate_finalization, render_handoff, required_supplemental_validations,
};

const HEAD_SHA: &str = "5ab1cca881959b3aac063af7c5973e7f75c35c46";
const OLD_SHA: &str = "1111111111111111111111111111111111111111";

#[test]
fn pr_head_resolver_uses_live_head_ref_oid_as_the_authoritative_sha() {
    let metadata = PrHeadMetadata::from_gh_view_json(
        r#"{
            "headRefName": "wave6-deployed-sharing-gap-1778302300",
            "headRefOid": "5ab1cca881959b3aac063af7c5973e7f75c35c46",
            "state": "OPEN",
            "mergeStateStatus": "CLEAN",
            "mergeable": "MERGEABLE",
            "isDraft": false,
            "reviewDecision": "",
            "url": "https://github.com/rysweet/eatme/pull/173",
            "statusCheckRollup": []
        }"#,
    )
    .expect("live gh metadata should parse");

    assert_eq!(metadata.pr_number(), None);
    assert_eq!(
        metadata.head_branch(),
        "wave6-deployed-sharing-gap-1778302300"
    );
    assert_eq!(metadata.head_sha(), HEAD_SHA);
    assert!(metadata.is_open());
    assert!(!metadata.is_draft());
}

#[test]
fn local_head_verifier_rejects_readiness_when_local_head_differs_from_pr_head() {
    let mut evidence = clean_finalization_evidence();
    evidence.local = LocalHeadEvidence {
        branch: "wave6-deployed-sharing-gap-1778302300".into(),
        head_sha: OLD_SHA.into(),
        status_short_branch:
            "## wave6-deployed-sharing-gap-1778302300...origin/wave6-deployed-sharing-gap-1778302300\n"
                .to_string(),
        worktree_clean: true,
    };

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::NotMergeReady);
    assert!(decision.no_op_justification.is_none());
    assert_contains(&decision.blockers, "local HEAD");
    assert_contains(&decision.blockers, HEAD_SHA);
}

#[test]
fn check_evidence_reader_blocks_missing_red_pending_or_wrong_head_checks() {
    let checks = CheckRollupEvidence::for_head(
        HEAD_SHA,
        vec![
            check("quality-gates", HEAD_SHA, CheckConclusion::Success),
            check("mkdocs", HEAD_SHA, CheckConclusion::Failure),
            check("assets", HEAD_SHA, CheckConclusion::Pending),
            check("stale-head", OLD_SHA, CheckConclusion::Success),
        ],
    );

    let result = checks.require_green_current_checks();

    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(message.contains("mkdocs"));
    assert!(message.contains("assets"));
    assert!(message.contains("stale-head"));
    assert!(message.contains("wrong head"));
}

#[test]
fn repo_validation_runner_only_requires_supplemental_gates_for_scope_or_evidence_gaps() {
    let docs_only_scope = vec![ScopeChange::new(
        "docs/default-workflow-pr-readiness.md",
        ScopeSurface::Documentation,
    )];
    let green_checks = CheckRollupEvidence::for_head(
        HEAD_SHA,
        vec![check("quality-gates", HEAD_SHA, CheckConclusion::Success)],
    );

    let required = required_supplemental_validations(&docs_only_scope, &green_checks);

    assert!(required.contains(&SupplementalValidation::MkdocsStrict));
    assert!(!required.contains(&SupplementalValidation::AssetValidation));
    assert!(!required.contains(&SupplementalValidation::GadugiFreshness));
    assert!(!required.contains(&SupplementalValidation::FullQualityGate));
}

#[test]
fn scope_gate_rejects_unrelated_changes_before_no_op_or_merge_ready() {
    let mut evidence = clean_finalization_evidence();
    evidence.scope_changes.push(ScopeChange::new(
        "crates/eatme-alice/src/launch/display.rs",
        ScopeSurface::Unrelated,
    ));

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::NotMergeReady);
    assert!(decision.no_op_justification.is_none());
    assert_contains(&decision.blockers, "unrelated");
    assert_contains(&decision.blockers, "display.rs");
}

#[test]
fn final_head_drift_prevents_no_op_even_when_prior_evidence_was_green() {
    let mut evidence = clean_finalization_evidence();
    evidence.final_pr_head_sha = OLD_SHA.into();

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::NotMergeReady);
    assert!(decision.no_op_justification.is_none());
    assert_contains(&decision.blockers, "final PR head");
    assert_contains(&decision.blockers, OLD_SHA);
}

#[test]
fn dirty_worktree_prevents_success_shaped_no_op() {
    let mut evidence = clean_finalization_evidence();
    evidence.local.worktree_clean = false;
    evidence.local.status_short_branch = "## wave6-deployed-sharing-gap-1778302300...origin/wave6-deployed-sharing-gap-1778302300\n M docs/default-workflow-pr-readiness.md\n".into();

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::NotMergeReady);
    assert!(decision.no_op_justification.is_none());
    assert_contains(&decision.blockers, "worktree");
    assert_contains(&decision.blockers, "dirty");
}

#[test]
fn unreadable_required_preserved_patch_is_blocked_not_no_op() {
    let mut evidence = clean_finalization_evidence();
    evidence.preserved_patch = Some(PreservedPatchEvidence::unreadable(
        "/tmp/recovery/default-workflow.patch",
        "permission denied",
    ));

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::Blocked);
    assert!(decision.no_op_justification.is_none());
    assert_contains(&decision.blockers, "preserved patch");
    assert_contains(&decision.blockers, "permission denied");
}

#[test]
fn clean_current_head_produces_owner_free_no_op_handoff_inside_sharing_boundary() {
    let evidence = clean_finalization_evidence();

    let decision = evaluate_finalization(evidence.clone());
    let handoff = render_handoff(&evidence, &decision, HandoffOptions::owner_free()).unwrap();

    assert_eq!(decision.decision, Decision::MergeReady);
    assert!(decision.no_op_justification.is_some());
    assert!(handoff.contains("No-op justification"));
    assert!(handoff.contains(HEAD_SHA));
    assert!(handoff.contains("classroom review handoff readiness"));
    assert!(handoff.contains("no repository edits or commits were required"));
    assert!(handoff.contains("does not claim"));
    assert!(handoff.contains("deployed sharing"));
    assert!(handoff.contains("production readiness"));
    assert!(handoff.contains("merge completion"));
    assert!(!handoff.contains("deployed sharing readiness"));
}

fn clean_finalization_evidence() -> FinalizationEvidence {
    FinalizationEvidence {
        repository: "rysweet/eatme".into(),
        pr_number: 173,
        pr: PrHeadMetadata::new(
            "wave6-deployed-sharing-gap-1778302300",
            HEAD_SHA,
            "OPEN",
            false,
        ),
        local: LocalHeadEvidence {
            branch: "wave6-deployed-sharing-gap-1778302300".into(),
            head_sha: HEAD_SHA.into(),
            status_short_branch: "## wave6-deployed-sharing-gap-1778302300...origin/wave6-deployed-sharing-gap-1778302300\n".into(),
            worktree_clean: true,
        },
        final_pr_head_sha: HEAD_SHA.into(),
        mergeability: Mergeability {
            merge_state_status: "CLEAN".into(),
            mergeable: "MERGEABLE".into(),
        },
        checks: CheckRollupEvidence::for_head(
            HEAD_SHA,
            vec![
                check("quality-gates", HEAD_SHA, CheckConclusion::Success),
                check("mkdocs", HEAD_SHA, CheckConclusion::Success),
            ],
        ),
        supplemental_validations: vec![SupplementalValidation::passed("mkdocs build --strict")],
        scope_changes: vec![ScopeChange::new(
            "docs/default-workflow-pr-readiness.md",
            ScopeSurface::Documentation,
        )],
        preserved_patch: None,
        audit_cycles: vec![
            AuditCycleEvidence::clean("scope and claim accuracy"),
            AuditCycleEvidence::clean("canonical and generated asset consistency"),
            AuditCycleEvidence::clean("gate completeness and final readiness"),
        ],
    }
}

fn check(name: &str, head_sha: &str, conclusion: CheckConclusion) -> CheckRunEvidence {
    CheckRunEvidence {
        name: name.into(),
        head_sha: head_sha.into(),
        conclusion,
        required: true,
        workflow_name: Some("CI".into()),
        details_url: Some(format!(
            "https://github.com/rysweet/eatme/actions/runs/{name}"
        )),
    }
}

fn assert_contains(lines: &[String], needle: &str) {
    assert!(
        lines.iter().any(|line| line.contains(needle)),
        "expected {lines:?} to contain {needle}"
    );
}
