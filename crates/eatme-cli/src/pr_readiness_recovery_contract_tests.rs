use super::pr_readiness::{
    ChangeOutcome, CheckConclusion, CheckStatus, CheckSummary, DiffScopeEvidence,
    DocsImpactEvidence, PrDescriptionEvidence, PrReadinessSnapshot, QualityAuditCycle,
    QualityAuditOutcome, QualityAuditPhase, RecoveryReadinessInput, RecoveryReadinessStatus,
    RecoveryValidationEvidence, evaluate_recovery_readiness, render_final_report,
};

const PR_204_BRANCH: &str = "wave7-eatme-nonclaim-audit-1778303500";
const EVIDENCE_HEAD: &str = "3c733847218f327f1d22004d6def527c0ec404e1";
const OLDER_HEAD: &str = "2222222222222222222222222222222222222222";
const ASSET_VALIDATE_COMMAND: &str = "cargo run -q -p eatme-cli -- assets validate --json";
const GADUGI_CHECK_COMMAND: &str =
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json";
const QUALITY_GATE_COMMAND: &str =
    "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh";
const DOCS_BUILD_COMMAND: &str = "mkdocs build --strict";

#[test]
fn baseline_schema_validation_and_github_actions_are_bound_to_exact_head() {
    let mut schema_mismatch = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    schema_mismatch.schema_version = "legacy-readiness.v0".into();
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&schema_mismatch),
        "pr-readiness-recovery.v1",
    );

    let mut wrong_head_check = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    wrong_head_check.snapshot.checks[0].head_sha = OLDER_HEAD.into();
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&wrong_head_check),
        "GitHub Actions",
    );
}

#[test]
fn expected_remote_head_is_required_and_must_match_current_pr_head_evidence() {
    let mut missing_expected_head = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    missing_expected_head.expected_remote_head_sha = None;
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&missing_expected_head),
        "expected_remote_head_sha is required",
    );

    let mut wrong_expected_head = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    wrong_expected_head.expected_remote_head_sha = Some(OLDER_HEAD.into());
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&wrong_expected_head),
        "GitHub PR head, local HEAD, and validation SHA to equal expected remote head",
    );
}

#[test]
fn trusted_required_check_list_blocks_missing_or_skipped_checks() {
    let mut missing_required = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    missing_required.snapshot.checks.clear();
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&missing_required),
        "required GitHub Actions check quality-gates is missing or omitted",
    );

    let mut skipped_required = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    skipped_required.snapshot.checks[0].required = false;
    skipped_required.snapshot.checks[0].conclusion = CheckConclusion::Skipped;
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&skipped_required),
        "conclusion=skipped",
    );
}

#[test]
fn runnable_qa_evidence_includes_exit_status_summary_and_exact_head() {
    let mut input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    input.asset_validation.exit_status = 1;
    input.asset_validation.summary = "assets validate reported invalid scenario JSON".into();
    input.asset_validation.passed = false;

    let report = evaluate_recovery_readiness(&input);

    assert_report_blocker_contains(&report, ASSET_VALIDATE_COMMAND);
    assert_report_blocker_contains(&report, "invalid scenario JSON");
}

#[test]
fn quality_audit_diff_scope_and_docs_impact_are_fail_closed() {
    let mut missing_audit = valid_recovery_input(ChangeOutcome::FilesModified(vec![
        "src/unrelated.rs".into(),
        "docs/default-workflow-pr-readiness.md".into(),
    ]));
    missing_audit.quality_audit_cycles.pop();
    missing_audit.diff_scope.focused = false;
    missing_audit.documentation_build.passed = false;

    let report = evaluate_recovery_readiness(&missing_audit);

    assert_report_blocker_contains(&report, "three SEEK/VALIDATE/FIX quality-audit cycles");
    assert_report_blocker_contains(&report, "final cycle clean");
    assert_report_blocker_contains(&report, "focused diff scope");
    assert_report_blocker_contains(&report, "docs impact");
    assert_report_blocker_contains(&report, DOCS_BUILD_COMMAND);
}

#[test]
fn focused_diff_scope_accepts_uvx_wrapper_for_remote_branch_recovery() {
    let mut input = valid_recovery_input(ChangeOutcome::FilesModified(vec![
        "src/eatme_uvx/cli.py".into(),
        "crates/eatme-cli/src/pr_readiness/recovery.rs".into(),
        "docs/default-workflow-pr-readiness.md".into(),
    ]));
    input.diff_scope.changed_files = vec![
        "src/eatme_uvx/cli.py".into(),
        "crates/eatme-cli/src/pr_readiness/recovery.rs".into(),
        "docs/default-workflow-pr-readiness.md".into(),
    ];

    let report = evaluate_recovery_readiness(&input);

    assert_eq!(report.status, RecoveryReadinessStatus::MergeReady);
}

#[test]
fn quality_audit_cycle_numbers_must_be_contiguous_and_increasing() {
    let mut duplicate = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    duplicate.quality_audit_cycles[1].cycle_number = 1;
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&duplicate),
        "expected cycle 2, got 1",
    );

    let mut missing = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    missing.quality_audit_cycles[1].cycle_number = 3;
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&missing),
        "expected cycle 2, got 3",
    );

    let mut decreasing = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    });
    decreasing.quality_audit_cycles[0].cycle_number = 2;
    decreasing.quality_audit_cycles[1].cycle_number = 1;
    assert_report_blocker_contains(
        &evaluate_recovery_readiness(&decreasing),
        "expected cycle 1, got 2",
    );
}

#[test]
fn report_rendering_rejects_control_character_injection_and_redacts_tokens() {
    let token = "scanner-safe-token-sentinel";
    let mut input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("safe justification at {EVIDENCE_HEAD}\nMERGE_READY token={token}"),
    });
    input.asset_validation.summary = format!("asset validation failed with token={token}");
    input.asset_validation.passed = false;
    input.asset_validation.exit_status = 1;

    let report = evaluate_recovery_readiness(&input);
    let body = render_final_report(&report);
    let json = serde_json::to_string(&report).unwrap();

    assert_report_blocker_contains(&report, "control characters or newlines");
    assert!(!body.contains(token), "{body}");
    assert!(!json.contains(token), "{json}");
    assert!(!body.contains("token="), "{body}");
    assert!(!json.contains("token="), "{json}");
    assert!(body.contains("[REDACTED]"), "{body}");
    assert!(json.contains("[REDACTED]"), "{json}");
    assert!(!body.contains("\nMERGE_READY token="), "{body}");
    assert!(!json.contains("\\nMERGE_READY token="), "{json}");
}

#[test]
fn final_report_renders_merge_ready_or_explicit_not_merge_ready_blockers() {
    let ready = evaluate_recovery_readiness(&valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("workflow-accepted no-op at {EVIDENCE_HEAD}"),
    }));
    assert_eq!(ready.status, RecoveryReadinessStatus::MergeReady);
    let mut blocked = ready.clone();
    blocked.status = RecoveryReadinessStatus::NotMergeReady;
    blocked.blockers = vec!["missing PR description evidence for current head".into()];

    let ready_body = render_final_report(&ready);
    let blocked_body = render_final_report(&blocked);
    let blocked_json = serde_json::to_string(&blocked).unwrap();

    assert!(ready_body.contains("MERGE_READY"), "{ready_body}");
    assert!(
        ready_body.contains("PR description evidence") && ready_body.contains(EVIDENCE_HEAD),
        "{ready_body}"
    );
    assert!(blocked_body.contains("NOT_MERGE_READY"), "{blocked_body}");
    assert!(
        blocked_body.contains("missing PR description evidence"),
        "{blocked_body}"
    );
    assert!(blocked_json.contains("NOT_MERGE_READY"), "{blocked_json}");
}

fn valid_recovery_input(change_outcome: ChangeOutcome) -> RecoveryReadinessInput {
    RecoveryReadinessInput {
        schema_version: "pr-readiness-recovery.v1".into(),
        expected_remote_head_sha: Some(EVIDENCE_HEAD.into()),
        snapshot: pr_204_snapshot(),
        validation_sha: EVIDENCE_HEAD.into(),
        required_github_checks: vec!["quality-gates".into()],
        asset_validation: validation("asset validation", ASSET_VALIDATE_COMMAND),
        generated_gadugi_check: validation("generated Gadugi freshness", GADUGI_CHECK_COMMAND),
        quality_gate: validation("repository quality gates", QUALITY_GATE_COMMAND),
        documentation_build: validation("documentation build", DOCS_BUILD_COMMAND),
        quality_audit_cycles: clean_quality_audit_cycles(),
        diff_scope: DiffScopeEvidence {
            changed_files: vec!["crates/eatme-cli/src/pr_readiness_tests.rs".into()],
            focused: true,
        },
        docs_impact: DocsImpactEvidence {
            docs_changed: true,
            strict_build_required: true,
        },
        pr_description_evidence: PrDescriptionEvidence {
            head_sha: EVIDENCE_HEAD.into(),
            contains_readiness_evidence: true,
            contains_bounded_nonclaims: true,
        },
        stale_evidence_handled: true,
        wrapper_failures: vec!["rate-limit".into(), "no-op guard".into()],
        change_outcome,
    }
}

fn validation(name: &str, command: &str) -> RecoveryValidationEvidence {
    RecoveryValidationEvidence {
        name: name.into(),
        command: command.into(),
        evidence_sha: EVIDENCE_HEAD.into(),
        exit_status: 0,
        summary: format!("{name} completed without failures"),
        passed: true,
    }
}

fn pr_204_snapshot() -> PrReadinessSnapshot {
    PrReadinessSnapshot {
        pr_number: 204,
        branch: PR_204_BRANCH.into(),
        local_head_sha: EVIDENCE_HEAD.into(),
        pr_head_sha: EVIDENCE_HEAD.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        checks: vec![CheckSummary {
            name: "quality-gates".into(),
            status: CheckStatus::Completed,
            conclusion: CheckConclusion::Success,
            required: true,
            head_sha: EVIDENCE_HEAD.into(),
        }],
    }
}

fn clean_quality_audit_cycles() -> Vec<QualityAuditCycle> {
    [
        (1, QualityAuditOutcome::FixApplied),
        (2, QualityAuditOutcome::FixApplied),
        (3, QualityAuditOutcome::Clean),
    ]
    .into_iter()
    .map(|(cycle_number, outcome)| QualityAuditCycle {
        cycle_number,
        phases: vec![
            QualityAuditPhase::Seek,
            QualityAuditPhase::Validate,
            QualityAuditPhase::Fix,
        ],
        outcome,
        head_sha: EVIDENCE_HEAD.into(),
        summary: format!("quality-audit cycle {cycle_number} completed"),
    })
    .collect()
}

fn assert_report_blocker_contains(
    report: &super::pr_readiness::RecoveryReadinessReport,
    expected: &str,
) {
    assert_eq!(report.status, RecoveryReadinessStatus::NotMergeReady);
    let blockers = report.blockers.join("; ");
    assert!(
        blockers.contains(expected),
        "expected blockers to contain {expected:?}, got {blockers:?}"
    );
}
