use eatme_core::default_workflow_pr_readiness::{
    AuditFix, ChangeReporter, CheckConclusion, CheckStatus, DiffScopeReviewer, DocsImpact,
    DocsImpactReviewer, EvidenceCollector, FocusedFile, LocalQARunner, LocalQaCommandOutput,
    PrHeadEvidence, PrHeadSynchronizer, PrMetadata, PrReviewDecision, QualityAuditCycle,
    QualityAuditCycleRunner, ReadinessErrorKind, ReadinessEvidence, ReadinessGate, ReadinessStatus,
    ScenarioClaim, ScenarioEvidence, ScenarioEvidenceReviewer, StatusCheck,
};

const PR_NUMBER: u64 = 171;
const BRANCH: &str = "wave6-scenario-run-observe-gap-1778302300";
const HEAD: &str = "1778302300abcdef1778302300abcdef17783023";
const OTHER_HEAD: &str = "0000000000000000000000000000000000000000";

#[test]
fn pr_head_synchronizer_accepts_only_the_exact_remote_pr_head() {
    let evidence = PrHeadEvidence {
        branch: BRANCH.into(),
        local_head: HEAD.into(),
        remote_head: HEAD.into(),
        pr_head_ref_oid: HEAD.into(),
        manually_merged: false,
        rebased_or_rewritten: false,
    };

    let verified = PrHeadSynchronizer::verify(evidence).unwrap();

    assert_eq!(verified.evaluated_head(), HEAD);
    assert_eq!(verified.branch(), BRANCH);
}

#[test]
fn pr_head_synchronizer_blocks_wrong_head_and_manual_history_changes() {
    let wrong_head = PrHeadEvidence {
        branch: BRANCH.into(),
        local_head: HEAD.into(),
        remote_head: OTHER_HEAD.into(),
        pr_head_ref_oid: HEAD.into(),
        manually_merged: false,
        rebased_or_rewritten: false,
    };

    let wrong_head_error = PrHeadSynchronizer::verify(wrong_head).unwrap_err();
    assert_eq!(wrong_head_error.kind(), ReadinessErrorKind::WrongHead);

    let manual_merge = PrHeadEvidence {
        branch: BRANCH.into(),
        local_head: HEAD.into(),
        remote_head: HEAD.into(),
        pr_head_ref_oid: HEAD.into(),
        manually_merged: true,
        rebased_or_rewritten: false,
    };

    let manual_merge_error = PrHeadSynchronizer::verify(manual_merge).unwrap_err();
    assert_eq!(
        manual_merge_error.kind(),
        ReadinessErrorKind::ManualMergeOrHistoryRewrite
    );
}

#[test]
fn evidence_collector_requires_green_completed_checks_for_the_evaluated_head() {
    let metadata = pr_metadata(vec![green_check("quality-gates"), green_check("docs")]);
    let evidence = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap();
    assert!(evidence.github_actions_green());
    assert_eq!(evidence.evaluated_head(), HEAD);
    assert_eq!(evidence.pr_number(), PR_NUMBER);
}

#[test]
fn evidence_collector_accepts_skipped_optional_checks_from_status_rollup() {
    let metadata = pr_metadata(vec![
        green_check("quality-gates"),
        optional_skipped_check("manual real Alice launch smoke"),
    ]);

    let evidence = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap();

    assert!(evidence.github_actions_green());
}

#[test]
fn evidence_collector_requires_mergeable_pr_metadata() {
    let unmergeable = PrMetadata {
        mergeable: false,
        ..pr_metadata(vec![green_check("quality-gates")])
    };
    let unmergeable_error = EvidenceCollector::from_pr_metadata(unmergeable, HEAD).unwrap_err();
    assert_eq!(
        unmergeable_error.kind(),
        ReadinessErrorKind::MergeabilityBlocked
    );
}

#[test]
fn evidence_collector_blocks_unacceptable_merge_state_statuses() {
    for merge_state_status in ["DIRTY", "BLOCKED", "BEHIND", "DRAFT", "UNKNOWN", "UNSTABLE"] {
        let metadata = PrMetadata {
            merge_state_status: merge_state_status.into(),
            ..pr_metadata(vec![green_check("quality-gates")])
        };
        let error = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap_err();
        assert_eq!(error.kind(), ReadinessErrorKind::MergeabilityBlocked);
    }
}

#[test]
fn evidence_collector_accepts_clean_mergeable_pr_states() {
    for merge_state_status in ["CLEAN", "HAS_HOOKS"] {
        let metadata = PrMetadata {
            merge_state_status: merge_state_status.into(),
            ..pr_metadata(vec![green_check("quality-gates")])
        };
        let evidence = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap();
        assert!(evidence.github_actions_green());
    }
}

#[test]
fn evidence_collector_blocks_pending_failing_missing_or_stale_checks() {
    for (status, conclusion, expected) in [
        (
            CheckStatus::Pending,
            CheckConclusion::Unknown,
            ReadinessErrorKind::IncompleteChecks,
        ),
        (
            CheckStatus::Completed,
            CheckConclusion::Failure,
            ReadinessErrorKind::FailingChecks,
        ),
        (
            CheckStatus::Missing,
            CheckConclusion::Unknown,
            ReadinessErrorKind::MissingChecks,
        ),
    ] {
        let metadata = pr_metadata(vec![check("quality-gates", status, conclusion)]);
        let error = EvidenceCollector::from_pr_metadata(metadata, HEAD).unwrap_err();
        assert_eq!(error.kind(), expected);
    }

    let stale_metadata = PrMetadata {
        head_ref_oid: OTHER_HEAD.into(),
        ..pr_metadata(vec![green_check("quality-gates")])
    };

    let stale_error = EvidenceCollector::from_pr_metadata(stale_metadata, HEAD).unwrap_err();
    assert_eq!(stale_error.kind(), ReadinessErrorKind::WrongHead);
}

#[test]
fn local_qa_runner_contract_lists_required_commands_without_timeout_wrappers() {
    let commands = LocalQARunner::required_commands();
    let shell_lines: Vec<String> = commands.shell_lines();

    assert_eq!(
        shell_lines,
        vec![
            "cargo run -q -p eatme-cli -- assets validate --json".to_string(),
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json".to_string(),
            "mkdocs build --strict".to_string(),
            "TMPDIR=/tmp ./scripts/quality-gates.sh".to_string(),
        ],
    );
    assert!(
        shell_lines.iter().all(|command: &String| {
            !command.starts_with("timeout ") && !command.contains(" timeout ")
        }),
        "readiness commands must not be wrapped by shell timeout"
    );
}

#[test]
fn local_qa_runner_requires_every_command_to_pass_and_rejects_substitutions() {
    let outputs = [
        qa_output(
            "cargo run -q -p eatme-cli -- assets validate --json",
            Some(0),
        ),
        qa_output(
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            Some(0),
        ),
        qa_output("mkdocs build --strict", Some(0)),
        qa_output("TMPDIR=/tmp ./scripts/quality-gates.sh", Some(0)),
    ];

    let report = LocalQARunner::summarize(&outputs).unwrap();
    assert!(report.passed());

    let missing_docs = [
        qa_output(
            "cargo run -q -p eatme-cli -- assets validate --json",
            Some(0),
        ),
        qa_output(
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            Some(0),
        ),
        qa_output("TMPDIR=/tmp ./scripts/quality-gates.sh", Some(0)),
    ];
    let missing_error = LocalQARunner::summarize(&missing_docs).unwrap_err();
    assert_eq!(missing_error.kind(), ReadinessErrorKind::MissingLocalQa);

    let substituted = [
        qa_output(
            "cargo run -q -p eatme-cli -- assets validate --json",
            Some(0),
        ),
        qa_output(
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            Some(0),
        ),
        qa_output("mdbook build", Some(0)),
        qa_output("TMPDIR=/tmp ./scripts/quality-gates.sh", Some(0)),
    ];
    let substitution_error = LocalQARunner::summarize(&substituted).unwrap_err();
    assert_eq!(
        substitution_error.kind(),
        ReadinessErrorKind::UnsupportedEvidenceSubstitution
    );
}

#[test]
fn scenario_evidence_reviewer_accepts_runnable_bounded_claims() {
    let evidence = ScenarioEvidence {
        runnable_artifacts: vec![
            "cargo run -q -p eatme-cli -- assets validate --json".into(),
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json".into(),
        ],
        claims: vec![
            ScenarioClaim::AssetSchemaValid,
            ScenarioClaim::GadugiAdaptersFresh,
        ],
    };

    let review = ScenarioEvidenceReviewer::review(evidence).unwrap();

    assert!(review.runnable());
    assert!(review.bounded_claims_only());
}

#[test]
fn scenario_evidence_reviewer_rejects_missing_evidence_and_overclaims() {
    let missing = ScenarioEvidence {
        runnable_artifacts: Vec::new(),
        claims: vec![ScenarioClaim::AssetSchemaValid],
    };
    let missing_error = ScenarioEvidenceReviewer::review(missing).unwrap_err();
    assert_eq!(
        missing_error.kind(),
        ReadinessErrorKind::MissingScenarioEvidence
    );

    for claim in [
        ScenarioClaim::FullUiAutomation,
        ScenarioClaim::VisibleRenderingCorrect,
        ScenarioClaim::GradingComplete,
        ScenarioClaim::CreativeAssessmentComplete,
        ScenarioClaim::FullLessonComplete,
        ScenarioClaim::FullWorldExecution,
        ScenarioClaim::SaveComplete,
        ScenarioClaim::DeployedSharingComplete,
        ScenarioClaim::FullTweedlePlayerDecode,
    ] {
        let overclaim = ScenarioEvidence {
            runnable_artifacts: vec!["run-observe-report.json".into()],
            claims: vec![claim],
        };
        let overclaim_error = ScenarioEvidenceReviewer::review(overclaim).unwrap_err();
        assert_eq!(
            overclaim_error.kind(),
            ReadinessErrorKind::OverclaimedScenarioEvidence
        );
    }
}

#[test]
fn docs_impact_reviewer_requires_docs_or_a_no_impact_reason() {
    let documented = DocsImpact {
        changed_files: vec!["assets/scenarios/eatme/run-observe-gap.yaml".into()],
        docs_files: vec!["docs/run-observe-readiness.md".into()],
        no_docs_impact_reason: None,
    };
    assert!(DocsImpactReviewer::review(documented).unwrap().passed());

    let no_impact = DocsImpact {
        changed_files: vec!["crates/eatme-core/tests/default_workflow_pr_readiness.rs".into()],
        docs_files: Vec::new(),
        no_docs_impact_reason: Some("tests only; no documented behavior changes".into()),
    };
    assert!(DocsImpactReviewer::review(no_impact).unwrap().passed());

    let missing = DocsImpact {
        changed_files: vec!["assets/scenarios/eatme/run-observe-gap.yaml".into()],
        docs_files: Vec::new(),
        no_docs_impact_reason: None,
    };
    let error = DocsImpactReviewer::review(missing).unwrap_err();
    assert_eq!(error.kind(), ReadinessErrorKind::MissingDocsImpact);
}

#[test]
fn diff_scope_reviewer_accepts_focused_files_and_blocks_unrelated_artifacts() {
    let focused = vec![
        FocusedFile::canonical_asset("assets/scenarios/eatme/run-observe-gap.yaml"),
        FocusedFile::generated_asset("assets/scenarios/gadugi/run-observe-gap.yaml", true),
        FocusedFile::test("crates/eatme-core/tests/default_workflow_pr_readiness.rs"),
        FocusedFile::documentation("docs/default-workflow-pr-readiness.md"),
    ];

    assert!(DiffScopeReviewer::review(&focused).unwrap().focused());

    let unrelated = vec![FocusedFile::unknown("default-workflow-attempt.log")];
    let unrelated_error = DiffScopeReviewer::review(&unrelated).unwrap_err();
    assert_eq!(unrelated_error.kind(), ReadinessErrorKind::UnfocusedDiff);

    let stale_generated = vec![FocusedFile::generated_asset(
        "assets/scenarios/gadugi/run-observe-gap.yaml",
        false,
    )];
    let stale_error = DiffScopeReviewer::review(&stale_generated).unwrap_err();
    assert_eq!(stale_error.kind(), ReadinessErrorKind::StaleGeneratedAsset);
}

#[test]
fn quality_audit_runner_requires_three_seek_validate_fix_cycles_and_clean_final_cycle() {
    let cycles = vec![
        cycle(
            "exact head and checks",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
        cycle(
            "runnable QA and docs",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
        cycle(
            "scope and bounded claims",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
    ];

    let report = QualityAuditCycleRunner::review(&cycles).unwrap();
    assert_eq!(report.cycle_count(), 3);
    assert!(report.final_cycle_clean());

    let too_few = vec![
        cycle(
            "exact head and checks",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
        cycle(
            "runnable QA and docs",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
    ];
    let too_few_error = QualityAuditCycleRunner::review(&too_few).unwrap_err();
    assert_eq!(
        too_few_error.kind(),
        ReadinessErrorKind::MissingQualityAuditCycle
    );

    let dirty_final = vec![
        cycle(
            "exact head and checks",
            AuditFix::NoRepositoryChangeNeeded,
            true,
        ),
        cycle(
            "runnable QA and docs",
            AuditFix::RepositoryChangeRequired,
            true,
        ),
        cycle(
            "scope and bounded claims",
            AuditFix::RemainingBlocker,
            false,
        ),
    ];
    let dirty_final_error = QualityAuditCycleRunner::review(&dirty_final).unwrap_err();
    assert_eq!(
        dirty_final_error.kind(),
        ReadinessErrorKind::UncleanFinalAuditCycle
    );
}

#[test]
fn readiness_gate_requires_all_evidence_not_green_ci_alone() {
    let green_checks_only = ReadinessEvidence::new(PR_NUMBER, BRANCH, HEAD)
        .with_github_actions_green(true)
        .with_workflow_completed(true);

    let blocked = ReadinessGate::evaluate(green_checks_only);

    assert_eq!(blocked.status(), ReadinessStatus::NotMergeReady);
    assert!(blocked.has_blocker(ReadinessErrorKind::MissingLocalQa));
    assert!(blocked.has_blocker(ReadinessErrorKind::MissingScenarioEvidence));
    assert!(blocked.has_blocker(ReadinessErrorKind::MissingPrStateReview));
    assert!(blocked.has_blocker(ReadinessErrorKind::MissingQualityAuditCycle));
}

#[test]
fn readiness_gate_accepts_complete_evidence_and_change_reporter_formats_noop_output() {
    let complete = ReadinessEvidence::new(PR_NUMBER, BRANCH, HEAD)
        .with_exact_head_verified(true)
        .with_workflow_completed(true)
        .with_github_actions_green(true)
        .with_local_qa_passed(true)
        .with_scenario_evidence_reviewed(true)
        .with_docs_impact_reviewed(true)
        .with_focused_diff_reviewed(true)
        .with_pr_state_reviewed(true)
        .with_pr_description_current(true)
        .with_quality_audit_cycles(3, true)
        .with_files_modified(Vec::new())
        .with_noop_justification(
            "current head already satisfies all gates and no repository file changes were needed",
        );

    let verdict = ReadinessGate::evaluate(complete);
    assert_eq!(verdict.status(), ReadinessStatus::MergeReady);

    let output = ChangeReporter::format_final_output(&verdict);
    assert!(output.contains("MERGE_READY"));
    assert!(output.contains("Workflow-accepted no-op justification"));
    assert!(output.contains(HEAD));
    assert!(!output.contains("probably ready"));
    assert!(!output.contains("ready except"));
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

fn check(name: &str, status: CheckStatus, conclusion: CheckConclusion) -> StatusCheck {
    StatusCheck {
        name: name.into(),
        status,
        conclusion,
        head_sha: HEAD.into(),
        required: true,
    }
}

fn green_check(name: &str) -> StatusCheck {
    check(name, CheckStatus::Completed, CheckConclusion::Success)
}

fn optional_skipped_check(name: &str) -> StatusCheck {
    StatusCheck {
        name: name.into(),
        status: CheckStatus::Completed,
        conclusion: CheckConclusion::Skipped,
        head_sha: HEAD.into(),
        required: false,
    }
}

fn qa_output(command: &str, exit_status: Option<i32>) -> LocalQaCommandOutput {
    LocalQaCommandOutput {
        command: command.into(),
        exit_status,
        stdout: "{}".into(),
        stderr: String::new(),
    }
}

fn cycle(seek: &str, fix: AuditFix, clean: bool) -> QualityAuditCycle {
    QualityAuditCycle {
        seek: seek.into(),
        validate: format!("validated {seek} with exact-head evidence"),
        fix,
        clean,
    }
}
