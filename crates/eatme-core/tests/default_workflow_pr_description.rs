use eatme_core::default_workflow_pr_readiness::{
    PrDescriptionReview, PrDescriptionReviewerUpdater, ReadinessErrorKind,
};

const PR_NUMBER: u64 = 171;
const BRANCH: &str = "wave6-scenario-run-observe-gap-1778302300";
const HEAD: &str = "1778302300abcdef1778302300abcdef17783023";
const OTHER_HEAD: &str = "0000000000000000000000000000000000000000";

#[test]
fn pr_description_reviewer_requires_current_bounded_evidence_in_the_pr_body() {
    let body = format!(
        "Default-workflow recovery for PR #171\n\
         Evaluated head: {HEAD}\n\
         Branch: {BRANCH}\n\
         Final verdict: NOT_MERGE_READY\n\
         GitHub Actions: all required checks completed and green for {HEAD}\n\
         Local QA:\n\
         - cargo run -q -p eatme-cli -- assets validate --json: pass\n\
         - cargo run -q -p eatme-cli -- assets generate-gadugi --check --json: pass\n\
         - mkdocs build --strict: pass\n\
         - TMPDIR=/tmp ./scripts/quality-gates.sh: pass\n\
         Scenario evidence: bounded runnable asset and Gadugi evidence reviewed\n\
         Docs impact: reviewed\n\
         Focused diff: focused\n\
         Quality audit cycles: three cycles documented; final clean\n\
         Files modified: crates/eatme-core/tests/default_workflow_pr_readiness.rs\n\
         Evidence boundary: no claim of full UI automation, visible rendering correctness, \
         grading, creative assessment, full lesson completion, full world execution, \
         Save completion, deployed sharing/platform success, or full Tweedle/player decode"
    );

    let review = PrDescriptionReviewerUpdater::review(PR_NUMBER, &body, HEAD).unwrap();

    assert_eq!(review, PrDescriptionReview::Current);

    let stale = body.replace(HEAD, OTHER_HEAD);
    let stale_error = PrDescriptionReviewerUpdater::review(PR_NUMBER, &stale, HEAD).unwrap_err();
    assert_eq!(stale_error.kind(), ReadinessErrorKind::StalePrDescription);
}

#[test]
fn pr_description_reviewer_accepts_current_noop_recovery_body_format() {
    let body = format!(
        "Default-workflow recovery for PR #171\n\
         Evaluated head: `{HEAD}` on branch `{BRANCH}`.\n\
         GitHub Actions for the evaluated head are complete and green for required checks.\n\
         Local QA passed without timeout wrappers: \
         `cargo run -q -p eatme-cli -- assets validate --json`, \
         `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`, \
         `mkdocs build --strict`, and \
         `TMPDIR=/tmp ./scripts/quality-gates.sh`.\n\
         Scenario evidence: bounded runnable evidence reviewed; no full Alice UI automation, \
         visible rendering correctness, grading, creative assessment, full lesson completion, \
         Save completion, deployed sharing/platform success, or full Tweedle/player decode \
         claim is made.\n\
         Docs impact: reviewed.\n\
         Focused diff: reviewed.\n\
         Final verdict: `MERGE_READY`\n\
         Quality audit cycles: three cycles documented and final cycle clean.\n\
         Files modified by this recovery step: none.\n\
         Workflow-accepted no-op justification: current head already satisfies all gates."
    );

    let review = PrDescriptionReviewerUpdater::review(PR_NUMBER, &body, HEAD).unwrap();

    assert_eq!(review, PrDescriptionReview::Current);
}

#[test]
fn pr_description_reviewer_accepts_pr_171_recovery_body_wording() {
    let body = format!(
        "## Summary\n\
         - Does not claim full UI automation, visible rendering correctness, grading, \
         creative assessment, Save completion, first-lesson completion, full world execution, \
         deployed sharing/platform success, or full Tweedle/player decode.\n\
         \n\
         ## Validation\n\
         - Evaluated head: `{HEAD}` on branch `{BRANCH}`.\n\
         - GitHub Actions for the evaluated head are complete and green for required checks: \
         Documentation Site, Quality Gates, and GitGuardian Security Checks. Non-required skipped \
         jobs are not used as readiness evidence.\n\
         - Local QA passed without timeout wrappers: \
         `cargo run -q -p eatme-cli -- assets validate --json`, \
         `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`, \
         `mkdocs build --strict`, and `TMPDIR=/tmp ./scripts/quality-gates.sh`.\n\
         - Docs impact reviewed: readiness wording is bounded to run/observe gaps.\n\
         - Focused diff reviewed: changed files are scoped to readiness docs and tests.\n\
         \n\
         ## Default-workflow recovery evidence\n\
         \n\
         Evaluated head: `{HEAD}`  \n\
         Branch: `{BRANCH}`  \n\
         Final verdict: `MERGE_READY`\n\
         \n\
         Scenario evidence: bounded runnable evidence below; no full Alice UI automation, \
         visible rendering correctness, grading, creative assessment, full lesson completion, \
         Save completion, deployed sharing/platform success, or full Tweedle/player decode \
         claim is made.\n\
         \n\
         Docs impact: reviewed and directly related to run/observe readiness.\n\
         \n\
         Focused diff: reviewed as focused on readiness evidence.\n\
         \n\
         Quality audit cycles:\n\
         \n\
         | Cycle | SEEK | VALIDATE | FIX |\n\
         | --- | --- | --- | --- |\n\
         | 1. Exact head and checks | Risk: evaluating a stale branch head. | Validated. | No repository change needed. |\n\
         | 2. Runnable QA and docs | Risk: missing runnable evidence. | Validated. | No repository change needed. |\n\
         | 3. Scope and bounded claims | Risk: overclaiming UI behavior. | Validated. | Final cycle clean. |\n\
         \n\
         Files modified by this recovery step: none.\n\
         \n\
         Workflow-accepted no-op justification:\n\
         - Evaluated exact remote PR head: `{HEAD}`.\n\
         - Required GitHub Actions are complete and green for `{HEAD}`.\n\
         - Local QA commands passed without timeout wrappers.\n\
         - Scenario evidence, docs impact, focused diff, and PR description evidence were reviewed.\n\
         - Three quality-audit SEEK / VALIDATE / FIX cycles are documented above and the final cycle is clean.\n\
         \n\
         ## Step 16b: Outside-In Testing Results\n\
         PR #{PR_NUMBER}"
    );

    let review = PrDescriptionReviewerUpdater::review(PR_NUMBER, &body, HEAD).unwrap();

    assert_eq!(review, PrDescriptionReview::Current);
}
