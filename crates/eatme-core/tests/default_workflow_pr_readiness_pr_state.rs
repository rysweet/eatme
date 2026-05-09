use eatme_core::default_workflow_pr_readiness::{
    CheckConclusion, CheckStatus, EvidenceCollector, PrMetadata, PrReviewDecision, PrReviewState,
    ReadinessErrorKind, ReviewEvidence, StatusCheck,
};

const PR_NUMBER: u64 = 171;
const BRANCH: &str = "wave6-scenario-run-observe-gap-1778302300";
const HEAD: &str = "1778302300abcdef1778302300abcdef17783023";
const OTHER_HEAD: &str = "0000000000000000000000000000000000000000";

#[test]
fn evidence_collector_blocks_draft_prs_blocking_labels_and_requested_changes() {
    let draft = PrMetadata {
        is_draft: true,
        ..pr_metadata(vec![green_check("quality-gates")])
    };
    let draft_error = EvidenceCollector::from_pr_metadata(draft, HEAD).unwrap_err();
    assert_eq!(draft_error.kind(), ReadinessErrorKind::DraftPullRequest);

    let blocked_label = PrMetadata {
        labels: vec!["do-not-merge".into()],
        ..pr_metadata(vec![green_check("quality-gates")])
    };
    let label_error = EvidenceCollector::from_pr_metadata(blocked_label, HEAD).unwrap_err();
    assert_eq!(label_error.kind(), ReadinessErrorKind::BlockingPrLabel);

    let changes_requested = PrMetadata {
        review_decision: PrReviewDecision::ChangesRequested,
        latest_reviews: vec![review(PrReviewState::ChangesRequested, HEAD)],
        ..pr_metadata(vec![green_check("quality-gates")])
    };
    let review_error = EvidenceCollector::from_pr_metadata(changes_requested, HEAD).unwrap_err();
    assert_eq!(review_error.kind(), ReadinessErrorKind::BlockingReviewState);
}

#[test]
fn evidence_collector_accepts_owner_free_review_required_state_when_other_gates_are_clear() {
    let owner_free = PrMetadata {
        review_decision: PrReviewDecision::ReviewRequired,
        latest_reviews: Vec::new(),
        ..pr_metadata(vec![green_check("quality-gates")])
    };

    let evidence = EvidenceCollector::from_pr_metadata(owner_free, HEAD).unwrap();

    assert!(evidence.github_actions_green());
}

#[test]
fn evidence_collector_rejects_stale_latest_review_evidence_for_the_evaluated_head() {
    let stale_approval = PrMetadata {
        review_decision: PrReviewDecision::Approved,
        latest_reviews: vec![review(PrReviewState::Approved, OTHER_HEAD)],
        ..pr_metadata(vec![green_check("quality-gates")])
    };

    let error = EvidenceCollector::from_pr_metadata(stale_approval, HEAD).unwrap_err();

    assert_eq!(error.kind(), ReadinessErrorKind::StaleReviewEvidence);
}

fn pr_metadata(checks: Vec<StatusCheck>) -> PrMetadata {
    PrMetadata {
        number: PR_NUMBER,
        title: "Recover run/observe readiness evidence".into(),
        body: "Default-workflow recovery for PR #171".into(),
        head_ref_name: BRANCH.into(),
        head_ref_oid: HEAD.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: true,
        is_draft: false,
        labels: Vec::new(),
        review_decision: PrReviewDecision::ReviewRequired,
        latest_reviews: Vec::new(),
        status_checks: checks,
        files: vec!["docs/default-workflow-pr-readiness.md".into()],
    }
}

fn green_check(name: &str) -> StatusCheck {
    StatusCheck {
        name: name.into(),
        status: CheckStatus::Completed,
        conclusion: CheckConclusion::Success,
        head_sha: HEAD.into(),
        required: true,
    }
}

fn review(state: PrReviewState, commit_oid: &str) -> ReviewEvidence {
    ReviewEvidence {
        state,
        commit_oid: commit_oid.into(),
        author_login: "reviewer".into(),
    }
}
