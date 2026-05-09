use super::pr_readiness::{
    ChangeOutcome, CheckConclusion, CheckStatus, CheckSummary, DiffScopeEvidence,
    DocsImpactEvidence, FinalGateInput, LocalEvidence, PrDescriptionEvidence, PrReadinessSnapshot,
    QualityAuditCycle, QualityAuditOutcome, QualityAuditPhase, ReadinessError,
    RecoveryReadinessInput, RecoveryReadinessReport, RecoveryReadinessStatus,
    RecoveryValidationEvidence, ReviewNoteInput, StaleEvidencePolicy, evaluate_recovery_readiness,
    render_final_report, render_review_note, scrub_stale_evidence, validate_exact_head_evidence,
    validate_pr_204_documentation, validate_target_branch, verify_final_gate,
};

const PR_204_BRANCH: &str = "wave7-eatme-nonclaim-audit-1778303500";
const EVIDENCE_HEAD: &str = "3c733847218f327f1d22004d6def527c0ec404e1";
const OLDER_HEAD: &str = "2222222222222222222222222222222222222222";
const NEW_HEAD: &str = "3333333333333333333333333333333333333333";
const ASSET_VALIDATE_COMMAND: &str = "cargo run -q -p eatme-cli -- assets validate --json";
const GADUGI_CHECK_COMMAND: &str =
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json";
const QUALITY_GATE_COMMAND: &str =
    "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh";
const DOCS_BUILD_COMMAND: &str = "mkdocs build --strict";

#[test]
fn target_branch_guard_accepts_pr_204_branch_when_local_and_pr_heads_match() {
    let snapshot = pr_204_snapshot(PR_204_BRANCH, EVIDENCE_HEAD, EVIDENCE_HEAD);

    assert!(validate_target_branch(&snapshot, PR_204_BRANCH).is_ok());
}

#[test]
fn target_branch_guard_rejects_master_even_when_heads_match() {
    let snapshot = pr_204_snapshot("master", EVIDENCE_HEAD, EVIDENCE_HEAD);

    let error = validate_target_branch(&snapshot, PR_204_BRANCH).unwrap_err();

    assert_error_contains(&error, PR_204_BRANCH);
    assert_error_contains(&error, "master");
}

#[test]
fn target_branch_guard_rejects_local_head_that_differs_from_pr_head() {
    let snapshot = pr_204_snapshot(PR_204_BRANCH, OLDER_HEAD, EVIDENCE_HEAD);

    let error = validate_target_branch(&snapshot, PR_204_BRANCH).unwrap_err();

    assert_error_contains(&error, OLDER_HEAD);
    assert_error_contains(&error, EVIDENCE_HEAD);
}

#[test]
fn exact_head_evidence_requires_every_item_to_name_the_full_current_sha() {
    let evidence = vec![
        "asset validation passed at abc1234".to_string(),
        "current head has successful required checks".to_string(),
    ];

    let error = validate_exact_head_evidence(EVIDENCE_HEAD, &evidence).unwrap_err();

    assert_error_contains(&error, "40-character");
    assert_error_contains(&error, EVIDENCE_HEAD);
}

#[test]
fn exact_head_evidence_accepts_bounded_items_that_all_name_the_same_sha() {
    let evidence = vec![
        format!("asset validation passed for {EVIDENCE_HEAD}"),
        format!("required GitHub checks completed successfully for {EVIDENCE_HEAD}"),
        format!("older tested-head evidence is stale/non-current for {EVIDENCE_HEAD}"),
    ];

    assert!(validate_exact_head_evidence(EVIDENCE_HEAD, &evidence).is_ok());
}

#[test]
fn stale_evidence_scrubber_labels_older_tested_head_evidence_as_non_current() {
    let evidence = vec![
        format!("readiness evidence passed at tested head {OLDER_HEAD}"),
        format!("readiness evidence passed at exact SHA {EVIDENCE_HEAD}"),
    ];

    let scrubbed: Vec<String> =
        scrub_stale_evidence(EVIDENCE_HEAD, evidence, StaleEvidencePolicy::Label).unwrap();

    assert!(
        scrubbed.iter().any(|item: &String| {
            item.contains(OLDER_HEAD)
                && item.contains("stale/non-current")
                && !item.contains("current validation")
        }),
        "{scrubbed:#?}"
    );
    assert!(
        scrubbed.iter().any(
            |item: &String| item.contains(EVIDENCE_HEAD) && !item.contains("stale/non-current")
        ),
        "{scrubbed:#?}"
    );
}

#[test]
fn stale_evidence_scrubber_replaces_older_tested_head_evidence_when_requested() {
    let evidence = vec![
        format!("tested-head evidence for {OLDER_HEAD}"),
        format!("verified exact head {EVIDENCE_HEAD}"),
    ];

    let scrubbed: Vec<String> =
        scrub_stale_evidence(EVIDENCE_HEAD, evidence, StaleEvidencePolicy::Remove).unwrap();

    assert!(
        scrubbed
            .iter()
            .all(|item: &String| !item.contains(OLDER_HEAD)),
        "{scrubbed:#?}"
    );
    assert!(
        scrubbed
            .iter()
            .any(|item: &String| item.contains(EVIDENCE_HEAD)),
        "{scrubbed:#?}"
    );
}

#[test]
fn stale_evidence_scrubber_rejects_invalid_current_sha() {
    let evidence = vec![format!("tested-head evidence for {OLDER_HEAD}")];

    let error = scrub_stale_evidence("not-a-sha", evidence, StaleEvidencePolicy::Remove)
        .expect_err("invalid current SHA should fail before scrubbing evidence");

    assert_error_contains(&error, "40-character SHA");
}

#[test]
fn review_note_names_exact_sha_checks_ci_stale_handling_and_all_nonclaims() {
    let note: String = render_review_note(review_note_input());

    assert!(note.contains(EVIDENCE_HEAD), "{note}");
    assert!(note.contains(PR_204_BRANCH), "{note}");
    assert!(note.contains("asset validation"), "{note}");
    assert!(note.contains("generated Gadugi freshness"), "{note}");
    assert!(note.contains("repository quality gates"), "{note}");
    assert!(note.contains("documentation build"), "{note}");
    assert!(
        note.contains("optional skipped checks") || note.contains("optional checks are skipped"),
        "{note}"
    );
    assert!(note.contains("mergeStateStatus=CLEAN"), "{note}");
    assert!(note.contains("mergeable=MERGEABLE"), "{note}");
    assert!(note.contains("stale/non-current"), "{note}");
    assert_forbidden_behavior_is_nonclaimed(&note);
}

#[test]
fn review_note_does_not_call_optional_skipped_checks_green_or_passed() {
    let note: String = render_review_note(review_note_input());

    assert!(
        !note.contains("optional skipped checks passed")
            && !note.contains("optional skipped checks are green")
            && !note.contains("skipped optional checks passed")
            && !note.contains("skipped optional checks are green"),
        "{note}"
    );
}

#[test]
fn readiness_documentation_validator_accepts_required_pr_204_contract_wording() {
    let docs = format!(
        r#"
Exact SHA: {EVIDENCE_HEAD}
Branch: {PR_204_BRANCH}
asset validation passed for {EVIDENCE_HEAD}
generated Gadugi freshness passed for {EVIDENCE_HEAD}
repository quality gates passed for {EVIDENCE_HEAD}
documentation build passed for {EVIDENCE_HEAD}
required GitHub checks completed successfully for {EVIDENCE_HEAD}; optional checks are skipped
mergeStateStatus=CLEAN and mergeable=MERGEABLE for {EVIDENCE_HEAD}
older tested-head evidence is stale/non-current and is not current validation
Nonclaims: this does not validate full Alice UI automation, grading,
creative assessment, visible rendering correctness, Save completion, or
first-lesson completion.
"#
    );

    assert!(validate_pr_204_documentation(EVIDENCE_HEAD, &docs).is_ok());
}

#[test]
fn readiness_documentation_validator_rejects_forbidden_current_validation_claims() {
    let docs = format!(
        r#"
Exact SHA: {EVIDENCE_HEAD}
Branch: {PR_204_BRANCH}
Verified full Alice UI automation, grading, creative assessment, visible
rendering correctness, Save completion, and first-lesson completion for
{EVIDENCE_HEAD}.
"#
    );

    let error = validate_pr_204_documentation(EVIDENCE_HEAD, &docs).unwrap_err();

    assert_error_contains(&error, "nonclaim");
    assert_error_contains(&error, "full Alice UI automation");
}

#[test]
fn final_gate_accepts_when_pr_head_still_equals_evidence_sha() {
    let input = FinalGateInput {
        evidence_sha: EVIDENCE_HEAD.into(),
        latest_pr_head_sha: EVIDENCE_HEAD.into(),
        latest_review_note_body: format!("Exact SHA: {EVIDENCE_HEAD}\nstale/non-current"),
    };

    assert!(verify_final_gate(&input).is_ok());
}

#[test]
fn final_gate_rejects_when_a_new_commit_appears_after_evidence_collection() {
    let input = FinalGateInput {
        evidence_sha: EVIDENCE_HEAD.into(),
        latest_pr_head_sha: NEW_HEAD.into(),
        latest_review_note_body: format!("Exact SHA: {EVIDENCE_HEAD}\nstale/non-current"),
    };

    let error = verify_final_gate(&input).unwrap_err();

    assert_error_contains(&error, "rerun");
    assert_error_contains(&error, NEW_HEAD);
    assert_error_contains(&error, EVIDENCE_HEAD);
}

#[test]
fn readiness_evaluator_accepts_manual_bypass_noop_for_exact_current_head() {
    let input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!(
            "final branch {PR_204_BRANCH} at {EVIDENCE_HEAD} already satisfies readiness"
        ),
    });

    let report = evaluate_recovery_readiness(&input);

    assert_eq!(report.status, RecoveryReadinessStatus::MergeReady);
    assert_eq!(report.branch, PR_204_BRANCH);
    assert_eq!(report.final_head_sha, EVIDENCE_HEAD);
    assert!(report.validation_status.contains("passed"), "{report:#?}");
    assert!(matches!(report.change_outcome, ChangeOutcome::NoOp { .. }));
}

#[test]
fn readiness_evaluator_rejects_stale_validation_sha_after_fix_commit() {
    let mut input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("no files changed at {NEW_HEAD}"),
    });
    input.snapshot.local_head_sha = NEW_HEAD.into();
    input.snapshot.pr_head_sha = NEW_HEAD.into();

    let report = evaluate_recovery_readiness(&input);

    assert_report_blocker_contains(&report, "rerun");
    assert_report_blocker_contains(&report, EVIDENCE_HEAD);
    assert_report_blocker_contains(&report, NEW_HEAD);
}

#[test]
fn asset_validation_surface_requires_assets_validate_and_gadugi_check_at_exact_head() {
    let mut input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("no files changed at {EVIDENCE_HEAD}"),
    });
    input.generated_gadugi_check.passed = false;

    let report = evaluate_recovery_readiness(&input);

    assert_report_blocker_contains(&report, "generated Gadugi");
    assert_report_blocker_contains(&report, GADUGI_CHECK_COMMAND);
    assert_report_blocker_contains(&report, EVIDENCE_HEAD);
}

#[test]
fn quality_gate_surface_requires_tmpdir_and_saved_node_options() {
    let mut input = valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!("no files changed at {EVIDENCE_HEAD}"),
    });
    input.quality_gate.command = "./scripts/quality-gates.sh".into();

    let report = evaluate_recovery_readiness(&input);

    assert_report_blocker_contains(&report, "TMPDIR=/tmp");
    assert_report_blocker_contains(&report, "NODE_OPTIONS=--max-old-space-size=32768");
}

#[test]
fn final_report_includes_branch_head_validation_and_noop_or_files_modified() {
    let noop_report = evaluate_recovery_readiness(&valid_recovery_input(ChangeOutcome::NoOp {
        justification: format!(
            "final branch {PR_204_BRANCH} at {EVIDENCE_HEAD} already satisfies readiness"
        ),
    }));
    let files_report =
        evaluate_recovery_readiness(&valid_recovery_input(ChangeOutcome::FilesModified(vec![
            "docs/default-workflow-pr-readiness.md".into(),
        ])));
    assert_eq!(noop_report.status, RecoveryReadinessStatus::MergeReady);
    assert_eq!(files_report.status, RecoveryReadinessStatus::MergeReady);

    let noop_body = render_final_report(&noop_report);
    let files_body = render_final_report(&files_report);

    assert!(
        noop_body.contains(&format!("Branch: {PR_204_BRANCH}")),
        "{noop_body}"
    );
    assert!(
        noop_body.contains(&format!("Final HEAD: {EVIDENCE_HEAD}")),
        "{noop_body}"
    );
    assert!(
        noop_body.contains("Validation status: passed"),
        "{noop_body}"
    );
    assert!(noop_body.contains("No-op justification:"), "{noop_body}");
    assert!(
        noop_body.contains("Historical wrapper failures (context only, not readiness evidence)"),
        "{noop_body}"
    );
    assert!(
        files_body.contains("Files modified: docs/default-workflow-pr-readiness.md"),
        "{files_body}"
    );
    assert!(!files_body.contains("No-op justification:"), "{files_body}");
}

fn pr_204_snapshot(branch: &str, local_head_sha: &str, pr_head_sha: &str) -> PrReadinessSnapshot {
    PrReadinessSnapshot {
        pr_number: 204,
        branch: branch.into(),
        local_head_sha: local_head_sha.into(),
        pr_head_sha: pr_head_sha.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        checks: vec![
            CheckSummary {
                name: "quality-gates".into(),
                status: CheckStatus::Completed,
                conclusion: CheckConclusion::Success,
                required: true,
                head_sha: local_head_sha.into(),
            },
            CheckSummary {
                name: "optional-preview".into(),
                status: CheckStatus::Completed,
                conclusion: CheckConclusion::Skipped,
                required: false,
                head_sha: local_head_sha.into(),
            },
        ],
    }
}

fn valid_recovery_input(change_outcome: ChangeOutcome) -> RecoveryReadinessInput {
    RecoveryReadinessInput {
        schema_version: "pr-readiness-recovery.v1".into(),
        expected_remote_head_sha: Some(EVIDENCE_HEAD.into()),
        snapshot: pr_204_snapshot(PR_204_BRANCH, EVIDENCE_HEAD, EVIDENCE_HEAD),
        validation_sha: EVIDENCE_HEAD.into(),
        required_github_checks: vec!["quality-gates".into()],
        asset_validation: RecoveryValidationEvidence {
            name: "asset validation".into(),
            command: ASSET_VALIDATE_COMMAND.into(),
            evidence_sha: EVIDENCE_HEAD.into(),
            exit_status: 0,
            summary: "asset validation completed without failures".into(),
            passed: true,
        },
        generated_gadugi_check: RecoveryValidationEvidence {
            name: "generated Gadugi freshness".into(),
            command: GADUGI_CHECK_COMMAND.into(),
            evidence_sha: EVIDENCE_HEAD.into(),
            exit_status: 0,
            summary: "generated Gadugi freshness completed without failures".into(),
            passed: true,
        },
        quality_gate: RecoveryValidationEvidence {
            name: "repository quality gates".into(),
            command: QUALITY_GATE_COMMAND.into(),
            evidence_sha: EVIDENCE_HEAD.into(),
            exit_status: 0,
            summary: "repository quality gates completed without failures".into(),
            passed: true,
        },
        documentation_build: RecoveryValidationEvidence {
            name: "documentation build".into(),
            command: DOCS_BUILD_COMMAND.into(),
            evidence_sha: EVIDENCE_HEAD.into(),
            exit_status: 0,
            summary: "documentation build completed without failures".into(),
            passed: true,
        },
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

fn review_note_input() -> ReviewNoteInput {
    ReviewNoteInput {
        snapshot: pr_204_snapshot(PR_204_BRANCH, EVIDENCE_HEAD, EVIDENCE_HEAD),
        local_evidence: LocalEvidence {
            asset_validation: true,
            generated_gadugi_freshness: true,
            quality_gates: true,
            documentation_build: true,
        },
        stale_evidence_handled: true,
    }
}

fn assert_forbidden_behavior_is_nonclaimed(note: &str) {
    for nonclaim in [
        "full Alice UI automation",
        "grading",
        "creative assessment",
        "visible rendering correctness",
        "Save completion",
        "first-lesson completion",
    ] {
        let nonclaim_position = note
            .find(nonclaim)
            .unwrap_or_else(|| panic!("{nonclaim} missing from note:\n{note}"));
        let prefix = &note[..nonclaim_position];
        assert!(
            prefix.contains("does not validate") || prefix.contains("Nonclaims"),
            "{nonclaim} must be listed only as an explicit nonclaim:\n{note}"
        );
    }
}

fn assert_error_contains(error: &ReadinessError, expected: &str) {
    let message = error.to_string();
    assert!(
        message.contains(expected),
        "expected error to contain {expected:?}, got {message:?}"
    );
}

fn assert_report_blocker_contains(report: &RecoveryReadinessReport, expected: &str) {
    assert_eq!(report.status, RecoveryReadinessStatus::NotMergeReady);
    let blockers = report.blockers.join("; ");
    assert!(
        blockers.contains(expected),
        "expected blockers to contain {expected:?}, got {blockers:?}"
    );
}
