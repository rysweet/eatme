use eatme_assets::default_workflow_readiness::{
    CheckConclusion, CheckRunEvidence, CheckStatus, CommandEvidence, CommandStatus,
    DiffScopeReview, DocsImpactReview, EvidenceCommands, GitHubActionsReview, HeadVerification,
    MergeReadyGate, PREvidenceReview, QualityAuditCycle, QualityAuditCycles, ReadinessInput,
};

const PR_NUMBER: u64 = 193;
const BRANCH: &str = "feat/issue-176-eatme-wave7-gap-matrix-lane-follow-default-workflo";
const HEAD: &str = "8255dcb33d4c22214c971fa22e7e6d7b9237c0b3";

const REQUIRED_COMMANDS: [&str; 4] = [
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

#[test]
fn head_verification_requires_local_branch_and_sha_to_match_pr_head() {
    let review = passing_review();
    assert!(
        HeadVerification::validate(&review).is_ok(),
        "matching local branch/SHA and PR head must pass head verification"
    );

    let wrong_branch = ReadinessInput {
        local_branch: "master".into(),
        ..passing_review()
    };
    let result = HeadVerification::validate(&wrong_branch).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("local branch"));

    let wrong_sha = ReadinessInput {
        local_head_sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        ..passing_review()
    };
    let result = HeadVerification::validate(&wrong_sha).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("headRefOid"));
}

#[test]
fn evidence_commands_require_all_allowlisted_commands_without_timeout_wrappers() {
    let review = passing_review();
    assert!(
        EvidenceCommands::validate(&review).is_ok(),
        "all required repository commands passing for the exact head must pass"
    );

    let missing_quality_gate = ReadinessInput {
        command_evidence: REQUIRED_COMMANDS[..3]
            .iter()
            .map(|command| command_passed(command))
            .collect(),
        ..passing_review()
    };
    let result = EvidenceCommands::validate(&missing_quality_gate).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(
        result
            .blocker()
            .contains("TMPDIR=/tmp ./scripts/quality-gates.sh")
    );

    let wrapped_command = ReadinessInput {
        command_evidence: vec![CommandEvidence {
            command: "timeout 30 mkdocs build --strict".into(),
            status: CommandStatus::Passed,
            head_sha: HEAD.into(),
            used_timeout_wrapper: true,
        }],
        ..passing_review()
    };
    let result = EvidenceCommands::validate(&wrapped_command).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("timeout"));

    let stale_command = ReadinessInput {
        command_evidence: REQUIRED_COMMANDS
            .iter()
            .map(|command| CommandEvidence {
                head_sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                ..command_passed(command)
            })
            .collect(),
        ..passing_review()
    };
    let result = EvidenceCommands::validate(&stale_command).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("wrong head"));
}

#[test]
fn quality_audit_requires_three_seek_validate_fix_cycles_with_clean_final_cycle() {
    let review = passing_review();
    assert!(
        QualityAuditCycles::validate(&review).is_ok(),
        "three complete cycles with a clean final cycle must pass"
    );

    let two_cycles = ReadinessInput {
        quality_audit_cycles: vec![audit_cycle(1, false), audit_cycle(2, true)],
        ..passing_review()
    };
    let result = QualityAuditCycles::validate(&two_cycles).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("three"));

    let missing_validate = ReadinessInput {
        quality_audit_cycles: vec![
            audit_cycle(1, false),
            QualityAuditCycle {
                seek: "SEEK: checked current-head evidence".into(),
                validate: String::new(),
                fix: "FIX: no-op rationale tied to the exact head".into(),
                clean: false,
            },
            audit_cycle(3, true),
        ],
        ..passing_review()
    };
    let result = QualityAuditCycles::validate(&missing_validate).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("VALIDATE"));

    let dirty_final = ReadinessInput {
        quality_audit_cycles: vec![
            audit_cycle(1, false),
            audit_cycle(2, false),
            audit_cycle(3, false),
        ],
        ..passing_review()
    };
    let result = QualityAuditCycles::validate(&dirty_final).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("final cycle"));
}

#[test]
fn diff_scope_review_rejects_unrelated_paths_and_accepts_gap_matrix_lane_scope() {
    let review = passing_review();
    assert!(
        DiffScopeReview::validate(&review).is_ok(),
        "docs plus doc-test guardrails are focused for the gap-matrix lane"
    );

    let unrelated_runtime_change = ReadinessInput {
        changed_files: vec![
            "docs/default-workflow-pr-readiness.md".into(),
            "crates/eatme-assets/tests/default_workflow_readiness_gate_tests.rs".into(),
            "crates/eatme-alice/src/launch/runtime.rs".into(),
        ],
        ..passing_review()
    };
    let result = DiffScopeReview::validate(&unrelated_runtime_change).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("unrelated"));
}

#[test]
fn docs_impact_review_requires_strict_mkdocs_and_bounded_claims() {
    let review = passing_review();
    assert!(
        DocsImpactReview::validate(&review).is_ok(),
        "strict MkDocs plus bounded docs claims must pass docs impact review"
    );

    let missing_mkdocs = ReadinessInput {
        docs_impact: DocsImpactReview {
            mkdocs_strict_passed: false,
            ..passing_review().docs_impact
        },
        ..passing_review()
    };
    let result = DocsImpactReview::validate(&missing_mkdocs).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("mkdocs build --strict"));

    let overclaim = ReadinessInput {
        docs_impact: DocsImpactReview {
            bounded_claims: vec!["full UI automation".into()],
            ..passing_review().docs_impact
        },
        ..passing_review()
    };
    let result = DocsImpactReview::validate(&overclaim).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("full UI automation"));
}

#[test]
fn github_actions_review_requires_every_current_head_check_to_complete_successfully() {
    let review = passing_review();
    assert!(
        GitHubActionsReview::validate(&review).is_ok(),
        "successful completed check runs for the exact head must pass"
    );

    let skipped_check = ReadinessInput {
        check_runs: vec![CheckRunEvidence {
            name: "manual real Alice launch smoke".into(),
            status: CheckStatus::Completed,
            conclusion: CheckConclusion::Skipped,
            head_sha: HEAD.into(),
        }],
        ..passing_review()
    };
    let result = GitHubActionsReview::validate(&skipped_check).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("skipped"));

    let pending_check = ReadinessInput {
        check_runs: vec![CheckRunEvidence {
            name: "tests".into(),
            status: CheckStatus::InProgress,
            conclusion: CheckConclusion::Unknown,
            head_sha: HEAD.into(),
        }],
        ..passing_review()
    };
    let result = GitHubActionsReview::validate(&pending_check).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("pending"));
}

#[test]
fn pr_evidence_review_requires_current_head_evidence_and_recheck_after_updates() {
    let review = passing_review();
    assert!(
        PREvidenceReview::validate(&review).is_ok(),
        "PR evidence naming the current head and all local evidence must pass"
    );

    let stale_pr_evidence = ReadinessInput {
        pr_evidence: PREvidenceReview {
            head_sha: "cccccccccccccccccccccccccccccccccccccccc".into(),
            ..passing_review().pr_evidence
        },
        ..passing_review()
    };
    let result = PREvidenceReview::validate(&stale_pr_evidence).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("stale"));

    let updated_without_recheck = ReadinessInput {
        pr_evidence: PREvidenceReview {
            updated_during_review: true,
            reconfirmed_head_sha: None,
            ..passing_review().pr_evidence
        },
        ..passing_review()
    };
    let result = PREvidenceReview::validate(&updated_without_recheck).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("reconfirm"));

    let untrusted_pr_evidence = ReadinessInput {
        pr_evidence: PREvidenceReview {
            trusted_provenance: false,
            ..passing_review().pr_evidence
        },
        ..passing_review()
    };
    let result = PREvidenceReview::validate(&untrusted_pr_evidence).unwrap_err();
    assert_eq!(result.marker(), "NOT_MERGE_READY");
    assert!(result.blocker().contains("not trusted"));
}

#[test]
fn merge_ready_gate_emits_success_only_when_every_gate_passes() {
    let artifact = MergeReadyGate::evaluate(passing_review());

    assert_eq!(artifact.marker(), "MERGE_READY_EVIDENCE");
    assert!(artifact.text().starts_with("MERGE_READY_EVIDENCE\n"));
    assert!(artifact.text().contains("PR: #193"));
    assert!(artifact.text().contains(HEAD));
    assert!(artifact.text().contains("quality audit"));
    assert!(artifact.text().contains("no manual merge"));
    assert!(!artifact.text().contains("full UI automation"));
    assert!(!artifact.text().contains("grading"));
    assert!(!artifact.text().contains("creative assessment"));
}

#[test]
fn merge_ready_gate_emits_not_ready_with_specific_blocker_for_any_missing_gate() {
    let not_ready = MergeReadyGate::evaluate(ReadinessInput {
        pr_evidence: PREvidenceReview {
            head_sha: String::new(),
            ..passing_review().pr_evidence
        },
        ..passing_review()
    });

    assert_eq!(not_ready.marker(), "NOT_MERGE_READY");
    assert!(not_ready.text().starts_with("NOT_MERGE_READY\n"));
    assert!(not_ready.text().contains("Blocker:"));
    assert!(not_ready.text().contains("Required next action:"));
    assert!(
        !not_ready.text().contains("MERGE_READY_EVIDENCE"),
        "a blocked decision must not include a success marker anywhere"
    );
}

fn passing_review() -> ReadinessInput {
    ReadinessInput {
        pr_number: PR_NUMBER,
        head_ref_name: BRANCH.into(),
        head_ref_oid: HEAD.into(),
        local_branch: BRANCH.into(),
        local_head_sha: HEAD.into(),
        merge_state_status: "CLEAN".into(),
        mergeable: "MERGEABLE".into(),
        command_evidence: passing_command_evidence(),
        check_runs: passing_check_runs(),
        quality_audit_cycles: vec![
            audit_cycle(1, false),
            audit_cycle(2, false),
            audit_cycle(3, true),
        ],
        changed_files: focused_changed_files(),
        docs_impact: passing_docs_impact(),
        pr_evidence: passing_pr_evidence(),
        manual_merge_attempted: false,
    }
}

fn passing_command_evidence() -> Vec<CommandEvidence> {
    REQUIRED_COMMANDS
        .iter()
        .map(|command| command_passed(command))
        .collect()
}

fn passing_check_runs() -> Vec<CheckRunEvidence> {
    vec![
        check_passed("Build MkDocs site"),
        check_passed("detect changed files"),
        check_passed("fmt, clippy, module size"),
        check_passed("tests"),
        check_passed("coverage"),
        check_passed("fmt, clippy, tests, module size, coverage"),
        check_passed("GitGuardian Security Checks"),
    ]
}

fn focused_changed_files() -> Vec<String> {
    vec![
        "Cargo.lock".into(),
        "crates/eatme-assets/Cargo.toml".into(),
        "crates/eatme-assets/src/default_workflow_readiness.rs".into(),
        "crates/eatme-assets/src/default_workflow_readiness/github.rs".into(),
        "crates/eatme-assets/src/default_workflow_readiness/model.rs".into(),
        "crates/eatme-assets/src/default_workflow_readiness/validators.rs".into(),
        "crates/eatme-assets/src/lesson_session_readiness_doc_tests.rs".into(),
        "crates/eatme-assets/src/lib.rs".into(),
        "crates/eatme-assets/tests/default_workflow_readiness_external_service_tests.rs".into(),
        "crates/eatme-assets/tests/default_workflow_readiness_gate_tests.rs".into(),
        "docs/default-workflow-pr-readiness.md".into(),
        "docs/lesson-session-readiness.md".into(),
        "pyproject.toml".into(),
    ]
}

fn passing_docs_impact() -> DocsImpactReview {
    DocsImpactReview {
        mkdocs_strict_passed: true,
        bounded_claims: vec![
            "lesson-session silver-thread/e2e gap-matrix documentation lane".into(),
            "asset validation".into(),
            "generated adapter freshness".into(),
            "strict docs build".into(),
        ],
    }
}

fn passing_pr_evidence() -> PREvidenceReview {
    PREvidenceReview {
        location: "PR body".into(),
        trusted_provenance: true,
        head_sha: HEAD.into(),
        recorded_commands: REQUIRED_COMMANDS
            .iter()
            .map(|command| (*command).into())
            .collect(),
        records_github_checks: true,
        records_diff_scope: true,
        records_docs_impact: true,
        records_quality_audit: true,
        records_no_manual_merge: true,
        updated_during_review: false,
        reconfirmed_head_sha: Some(HEAD.into()),
    }
}

fn command_passed(command: &str) -> CommandEvidence {
    CommandEvidence {
        command: command.into(),
        status: CommandStatus::Passed,
        head_sha: HEAD.into(),
        used_timeout_wrapper: false,
    }
}

fn check_passed(name: &str) -> CheckRunEvidence {
    CheckRunEvidence {
        name: name.into(),
        status: CheckStatus::Completed,
        conclusion: CheckConclusion::Success,
        head_sha: HEAD.into(),
    }
}

fn audit_cycle(number: usize, clean: bool) -> QualityAuditCycle {
    QualityAuditCycle {
        seek: format!(
            "SEEK {number}: reviewed exact-head evidence, checks, diff, docs, and PR evidence"
        ),
        validate: format!(
            "VALIDATE {number}: bound findings to current-head command and GitHub evidence"
        ),
        fix: format!("FIX {number}: recorded no-op or minimal fix rationale for head {HEAD}"),
        clean,
    }
}
