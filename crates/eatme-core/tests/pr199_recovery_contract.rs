use eatme_core::pr199_recovery::{
    AliceEvidenceBlockerPreserver, CheckConclusion, CheckRollup, CheckRun,
    DefaultWorkflowInvocation, DefaultWorkflowRecovery, EvidenceDelta, EvidenceSnapshot,
    ExistingEvidenceFile, GitHubPrStateClient, PrStateCollector, PrStateInput,
    PushOrNoopDecisionGate, QaCommand, QaOutcome, RecoveryDecision, ScopedQaRunner,
    StructuredBlocker, WorkflowSource,
};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

#[test]
fn default_workflow_recovery_rejects_timeout_or_manual_fallback_invocations() {
    let timeout_fallback = DefaultWorkflowInvocation {
        source: WorkflowSource::TimeoutFallback,
        outcome: "completed through manual fallback".into(),
        log_reference: Some("default-workflow-attempt.log".into()),
        run_id: Some("manual-fallback-001".into()),
    };
    let manual_substitute = DefaultWorkflowInvocation {
        source: WorkflowSource::ManualSubstitute,
        outcome: "operator reconstructed recovery by hand".into(),
        log_reference: Some("manual-notes".into()),
        run_id: Some("manual-substitute-001".into()),
    };

    assert_eq!(
        DefaultWorkflowRecovery::validate_invocation(&timeout_fallback)
            .unwrap_err()
            .code(),
        "default_workflow_timeout_fallback_forbidden"
    );
    assert_eq!(
        DefaultWorkflowRecovery::validate_invocation(&manual_substitute)
            .unwrap_err()
            .code(),
        "default_workflow_manual_substitute_forbidden"
    );
}

#[test]
fn default_workflow_recovery_requires_auditable_real_no_timeout_proof() {
    let valid = DefaultWorkflowInvocation {
        source: WorkflowSource::RealDefaultWorkflowNoTimeout,
        outcome: "completed".into(),
        log_reference: Some("recipe-run/default-workflow/pr-199/2026-05-09T18:15:30Z".into()),
        run_id: Some("default-workflow-pr199-20260509".into()),
    };
    let missing_proof = DefaultWorkflowInvocation {
        log_reference: None,
        run_id: None,
        ..valid.clone()
    };

    let proof = DefaultWorkflowRecovery::validate_invocation(&valid).unwrap();
    assert_eq!(proof.source, WorkflowSource::RealDefaultWorkflowNoTimeout);
    assert_eq!(proof.outcome, "completed");
    assert!(proof.log_reference.contains("default-workflow"));

    assert_eq!(
        DefaultWorkflowRecovery::validate_invocation(&missing_proof)
            .unwrap_err()
            .code(),
        "default_workflow_proof_missing"
    );
}

#[test]
fn pr_state_collector_keeps_current_head_branch_files_and_check_categories_separate() {
    let input = PrStateInput {
        pr_number: 199,
        branch: "feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo".into(),
        head_sha: "6f815a5b6cc797da1b36bd8c86b2b2dfb1471990".into(),
        changed_files: vec![
            PathBuf::from("docs/pr-199-recovery-workflow.md"),
            PathBuf::from("docs/index.md"),
        ],
        check_runs: vec![
            CheckRun::completed("workspace tests", CheckConclusion::Success),
            CheckRun::completed("docs strict build", CheckConclusion::Success),
            CheckRun::completed("optional preview", CheckConclusion::Skipped),
            CheckRun::in_progress("quality gates"),
            CheckRun::completed("linux", CheckConclusion::Failure),
            CheckRun::completed("cancelled stale run", CheckConclusion::Cancelled),
        ],
    };

    let state = PrStateCollector::collect(input).unwrap();

    assert_eq!(state.pr_number, 199);
    assert_eq!(state.head_sha, "6f815a5b6cc797da1b36bd8c86b2b2dfb1471990");
    assert_eq!(state.changed_files.len(), 2);
    assert_eq!(
        state.check_rollup,
        CheckRollup {
            success: vec!["workspace tests".into(), "docs strict build".into()],
            failure: vec!["linux".into()],
            pending: vec!["quality gates".into()],
            cancelled: vec!["cancelled stale run".into()],
            skipped: vec!["optional preview".into()],
        }
    );
}

#[test]
fn github_pr_state_client_fetches_pr199_state_with_retry_and_check_mapping() {
    let runner = RecordingRunner::with_outputs(vec![CommandOutput {
        command: "gh pr view".into(),
        exit_status: Some(0),
        stdout: r#"{
            "number": 199,
            "headRefName": "feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo",
            "headRefOid": "6f815a5b6cc797da1b36bd8c86b2b2dfb1471990",
            "files": [
                {"path": "docs/pr-199-recovery-workflow.md"},
                {"path": "docs/index.md"}
            ],
            "statusCheckRollup": [
                {"__typename": "CheckRun", "name": "workspace tests", "status": "completed", "conclusion": "success"},
                {"__typename": "CheckRun", "name": "optional preview", "status": "COMPLETED", "conclusion": "SKIPPED"},
                {"__typename": "CheckRun", "name": "quality gates", "status": "in_progress", "conclusion": null},
                {"__typename": "StatusContext", "context": "legacy linux", "state": "ERROR"},
                {"__typename": "CheckRun", "name": "cancelled stale run", "status": "COMPLETED", "conclusion": "CANCELLED"}
            ]
        }"#.into(),
        stderr: String::new(),
    }]);

    let state = GitHubPrStateClient::new(&runner).fetch_state().unwrap();

    assert_eq!(state.pr_number, 199);
    assert_eq!(
        state.branch,
        "feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo"
    );
    assert_eq!(state.head_sha, "6f815a5b6cc797da1b36bd8c86b2b2dfb1471990");
    assert_eq!(
        state.changed_files,
        vec![
            PathBuf::from("docs/pr-199-recovery-workflow.md"),
            PathBuf::from("docs/index.md"),
        ]
    );
    assert_eq!(
        state.check_rollup,
        CheckRollup {
            success: vec!["workspace tests".into()],
            failure: vec!["legacy linux".into()],
            pending: vec!["quality gates".into()],
            cancelled: vec!["cancelled stale run".into()],
            skipped: vec!["optional preview".into()],
        }
    );

    let specs = runner.specs();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].program, "gh");
    assert_eq!(
        specs[0].args,
        vec![
            "pr",
            "view",
            "199",
            "--json",
            "number,headRefName,headRefOid,files,statusCheckRollup",
        ]
    );
    assert_eq!(specs[0].attempts, 3);
    assert!(specs[0].timeout.is_some());
}

#[test]
fn github_pr_state_client_surfaces_external_failures_without_success_fallback() {
    let runner = RecordingRunner::with_outputs(vec![CommandOutput {
        command: "gh pr view".into(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "GitHub API unavailable".into(),
    }]);

    let error = GitHubPrStateClient::new(&runner).fetch_state().unwrap_err();

    assert_eq!(error.code(), "github_pr_state_fetch_failed");
    assert!(error.to_string().contains("GitHub API unavailable"));
}

#[test]
fn github_pr_state_client_rejects_malformed_service_json() {
    let runner = RecordingRunner::with_outputs(vec![CommandOutput {
        command: "gh pr view".into(),
        exit_status: Some(0),
        stdout: "not json".into(),
        stderr: String::new(),
    }]);

    let error = GitHubPrStateClient::new(&runner).fetch_state().unwrap_err();

    assert_eq!(error.code(), "github_pr_state_json_invalid");
}

#[test]
fn alice_evidence_preserver_keeps_missing_real_action_evidence_blockers_without_synthesis() {
    let blocker = StructuredBlocker {
        code: "missing_real_action_evidence".into(),
        status: "blocked".into(),
        subject: "original_alice_action_evidence".into(),
        reason: "Original Alice action evidence is unavailable.".into(),
        resolution: "Preserve as explicit blocker until real evidence is provided.".into(),
    };
    let snapshot = EvidenceSnapshot::with_blockers(vec![blocker.clone()]);

    let preserved = AliceEvidenceBlockerPreserver::preserve(snapshot).unwrap();

    assert!(preserved.has_blocker_code("missing_real_action_evidence"));
    assert_eq!(
        preserved.original_alice_action_evidence.status, "missing",
        "missing real action evidence must remain missing, not inferred available"
    );
    assert_eq!(preserved.blockers, vec![blocker]);
    assert!(
        preserved
            .original_alice_action_evidence
            .synthetic_sources
            .is_empty(),
        "recovery must not invent or reconstruct Alice action evidence"
    );
}

#[test]
fn scoped_qa_runner_requires_all_pr199_recovery_commands_and_surfaces_failures() {
    let outcomes = vec![
        QaOutcome::passed(QaCommand::CargoWorkspaceAllFeatures),
        QaOutcome::passed(QaCommand::AssetsValidateJson),
        QaOutcome::passed(QaCommand::GenerateGadugiCheckJson),
        QaOutcome::passed(QaCommand::MkdocsBuildStrict),
        QaOutcome::failed(
            QaCommand::QualityGatesWithTmpdir,
            1,
            "module line-count gate failed",
        ),
    ];

    let report = ScopedQaRunner::summarize(outcomes).unwrap();

    assert_eq!(report.commands.len(), 5);
    assert!(report.includes(QaCommand::CargoWorkspaceAllFeatures));
    assert!(report.includes(QaCommand::AssetsValidateJson));
    assert!(report.includes(QaCommand::GenerateGadugiCheckJson));
    assert!(report.includes(QaCommand::MkdocsBuildStrict));
    assert!(report.includes(QaCommand::QualityGatesWithTmpdir));
    assert!(
        !report.passed,
        "any QA failure must block merge-ready recovery"
    );
    assert_eq!(
        report.blockers[0].code, "scoped_qa_failed",
        "environmental or command failures are blockers, not merge-ready success"
    );
}

#[test]
fn merge_ready_evidence_updater_updates_only_existing_evidence_file_when_facts_change() {
    let existing = ExistingEvidenceFile {
        path: PathBuf::from("docs/pr-199-merge-readiness-evidence.md"),
        head_sha: "old-head".into(),
        branch: "feat/old".into(),
        check_rollup: CheckRollup::default(),
        qa_summary: "old QA".into(),
        blocker_codes: vec!["missing_real_action_evidence".into()],
        default_workflow_run_id: "old-run".into(),
    };
    let current = EvidenceSnapshot::for_pr199_recovery()
        .with_head_sha("6f815a5b6cc797da1b36bd8c86b2b2dfb1471990")
        .with_branch("feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo")
        .with_existing_blocker_code("missing_real_action_evidence")
        .with_default_workflow_run_id("default-workflow-pr199-20260509");

    let delta = EvidenceDelta::from_existing_and_current(&existing, &current).unwrap();
    let update = delta.required_update().unwrap();

    assert_eq!(update.path, existing.path);
    assert!(
        update
            .body
            .contains("6f815a5b6cc797da1b36bd8c86b2b2dfb1471990")
    );
    assert!(update.body.contains("default-workflow-pr199-20260509"));
    assert!(update.body.contains("missing_real_action_evidence"));
    assert!(
        !update.body.contains("feature expansion"),
        "merge-ready evidence updates must stay recovery-scoped"
    );
}

#[test]
fn merge_ready_evidence_updater_returns_no_delta_when_current_facts_match_existing_record() {
    let existing = matching_pr199_recovery_snapshot();
    let current = EvidenceSnapshot::from_existing(&existing);

    let delta = EvidenceDelta::from_existing_and_current(&existing, &current).unwrap();

    assert!(
        delta.required_update().is_none(),
        "no evidence rewrite is allowed when head, checks, QA, blockers, and workflow proof already match"
    );
}

#[test]
fn push_or_noop_decision_gate_requires_literal_current_head_noop_justification_when_no_update_exists()
 {
    let existing = matching_pr199_recovery_snapshot();
    let current = EvidenceSnapshot::from_existing(&existing);
    let delta = EvidenceDelta::from_existing_and_current(&existing, &current).unwrap();

    let decision = PushOrNoopDecisionGate::decide(delta).unwrap();

    match decision {
        RecoveryDecision::NoOp { justification } => {
            assert!(
                justification
                    .starts_with("No-op: PR #199 recovery required no repository modification.")
            );
            assert!(justification.contains("Current PR branch:"));
            assert!(justification.contains("Current changed files:"));
            assert!(justification.contains("Current PR head:"));
            assert!(justification.contains("Current checks: success="));
            assert!(justification.contains("failure="));
            assert!(justification.contains("pending="));
            assert!(justification.contains("cancelled="));
            assert!(justification.contains("skipped="));
            assert!(justification.contains("Default-workflow proof:"));
            assert!(justification.contains("Scoped QA rerun:"));
            assert!(
                justification
                    .contains("Blockers preserved: missing_real_action_evidence remains explicit")
            );
            assert!(justification.contains("Scope decision: existing PR #199 merge-ready evidence already matches current branch/files/head/checks/QA/default-workflow/blocker state"));
        }
        RecoveryDecision::Push { .. } => panic!("matching evidence must not request a push"),
    }
}

#[test]
fn push_or_noop_decision_gate_pushes_only_focused_recovery_evidence_changes() {
    let existing = matching_pr199_recovery_snapshot();
    let changed = EvidenceSnapshot::from_existing(&existing)
        .with_head_sha("6f815a5b6cc797da1b36bd8c86b2b2dfb1471990")
        .with_default_workflow_run_id("default-workflow-pr199-20260509-rerun");
    let delta = EvidenceDelta::from_existing_and_current(&existing, &changed).unwrap();

    let decision = PushOrNoopDecisionGate::decide(delta).unwrap();

    match decision {
        RecoveryDecision::Push { files, message } => {
            assert_eq!(files, vec![existing.path]);
            assert!(message.contains("PR #199 recovery"));
            assert!(message.contains("missing_real_action_evidence"));
            assert!(
                !message.contains("feature"),
                "push rationale must not broaden the scope beyond recovery evidence"
            );
        }
        RecoveryDecision::NoOp { .. } => {
            panic!("changed evidence facts must request a focused push")
        }
    }
}

struct RecordingRunner {
    outputs: RefCell<VecDeque<CommandOutput>>,
    specs: RefCell<Vec<CommandSpec>>,
}

impl RecordingRunner {
    fn with_outputs(outputs: Vec<CommandOutput>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into()),
            specs: RefCell::new(Vec::new()),
        }
    }

    fn specs(&self) -> Vec<CommandSpec> {
        self.specs.borrow().clone()
    }
}

impl CommandRunner for RecordingRunner {
    fn run(&self, spec: &CommandSpec) -> anyhow::Result<CommandOutput> {
        self.specs.borrow_mut().push(spec.clone());
        Ok(self
            .outputs
            .borrow_mut()
            .pop_front()
            .expect("test runner output queue exhausted"))
    }
}

fn matching_pr199_recovery_snapshot() -> ExistingEvidenceFile {
    ExistingEvidenceFile {
        path: PathBuf::from("docs/pr-199-merge-readiness-evidence.md"),
        branch: "feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo".into(),
        head_sha: "matching-pr199-head".into(),
        check_rollup: CheckRollup::default(),
        qa_summary: "all scoped QA passed".into(),
        blocker_codes: vec!["missing_real_action_evidence".into()],
        default_workflow_run_id: "default-workflow-pr199-20260509".into(),
    }
}
