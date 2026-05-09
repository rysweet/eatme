use anyhow::Result;
use eatme_core::pr199_recovery::service::fetch_pr199_metadata;
use eatme_core::pr199_recovery::{
    AliceActionEvidence, Pr199RecoveryEvidence, Pr199RecoveryReport, QaCommandProof,
    evaluate_pr199_recovery_readiness,
};
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use serde_json::json;
use std::cell::RefCell;
use std::path::Path;

const REQUIRED_QA_COMMANDS: [&str; 5] = [
    "cargo test --workspace --all-features",
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

#[test]
fn accepts_only_real_default_workflow_no_timeout_proof() -> Result<()> {
    let accepted = evaluate_pr199_recovery_readiness(ready_evidence_with_workflow(Some(
        "RealDefaultWorkflowNoTimeout",
    )))?;

    assert_eq!(accepted.pr, 199);
    assert_eq!(
        accepted.workflow.proof.as_deref(),
        Some("RealDefaultWorkflowNoTimeout")
    );
    assert_no_blocker(&accepted, "workflow_proof");

    for rejected in [
        "TimeoutShortcut",
        "ManualFallbackAttempt",
        "default-workflow-attempt.log",
        "SubstituteWorkflowProof",
    ] {
        let report =
            evaluate_pr199_recovery_readiness(ready_evidence_with_workflow(Some(rejected)))?;

        assert_eq!(
            report.status, "not_ready",
            "{rejected} must not be accepted"
        );
        assert_blocker(&report, "invalid_workflow_proof", "workflow_proof");
    }

    let missing = evaluate_pr199_recovery_readiness(ready_evidence_with_workflow(None))?;
    assert_eq!(missing.status, "not_ready");
    assert_blocker(&missing, "missing_workflow_proof", "workflow_proof");

    Ok(())
}

#[test]
fn preserves_missing_original_alice_actions_as_structured_blockers() -> Result<()> {
    let report = evaluate_pr199_recovery_readiness(
        evidence_with_workflow_qa_and_metadata()
            .with_alice_action(AliceActionEvidence::missing_original("save-project"))
            .with_alice_action(AliceActionEvidence::missing_original("place-object")),
    )?;

    assert_eq!(report.status, "not_ready");
    assert_blocker_with_action(
        &report,
        "missing_real_action_evidence",
        "alice.original.actions.save-project",
        "save-project",
        "original",
    );
    assert_blocker_with_action(
        &report,
        "missing_real_action_evidence",
        "alice.original.actions.place-object",
        "place-object",
        "original",
    );

    Ok(())
}

#[test]
fn rejects_synthetic_or_reconstructed_alice_action_evidence() -> Result<()> {
    let report = evaluate_pr199_recovery_readiness(
        evidence_with_workflow_qa_and_metadata()
            .with_alice_action(AliceActionEvidence::synthetic_original(
                "save-project",
                "reconstructed from PR comment",
            ))
            .with_alice_action(AliceActionEvidence::real_original(
                "place-object",
                "evidence/alice/original/place-object.json",
            )),
    )?;

    assert_eq!(report.status, "not_ready");
    assert_blocker(
        &report,
        "invalid_alice_action_evidence",
        "alice.original.actions.save-project",
    );
    assert_blocker_with_action(
        &report,
        "missing_real_action_evidence",
        "alice.original.actions.save-project",
        "save-project",
        "original",
    );

    Ok(())
}

#[test]
fn requires_exact_scoped_qa_command_set_from_current_worktree() -> Result<()> {
    let accepted = evaluate_pr199_recovery_readiness(ready_evidence())?;

    assert_eq!(accepted.status, "ready");
    assert_eq!(accepted.qa.required_commands, REQUIRED_QA_COMMANDS);
    assert_no_blocker(&accepted, "qa");

    let missing_command = evaluate_pr199_recovery_readiness(
        ready_evidence().without_qa_command("mkdocs build --strict"),
    )?;
    assert_eq!(missing_command.status, "not_ready");
    assert_blocker(
        &missing_command,
        "missing_qa_proof",
        "qa.mkdocs build --strict",
    );
    assert_blocker(&missing_command, "incomplete_qa_proof", "qa");

    let renamed_quality_gate = evaluate_pr199_recovery_readiness(
        ready_evidence()
            .without_qa_command("TMPDIR=/tmp ./scripts/quality-gates.sh")
            .with_qa_command(QaCommandProof::passed_in_worktree(
                "./scripts/quality-gates.sh",
                current_worktree(),
            )),
    )?;
    assert_eq!(renamed_quality_gate.status, "not_ready");
    assert_blocker(
        &renamed_quality_gate,
        "invalid_qa_command",
        "qa.TMPDIR=/tmp ./scripts/quality-gates.sh",
    );

    let stale_worktree = evaluate_pr199_recovery_readiness(
        ready_evidence()
            .without_qa_command("cargo test --workspace --all-features")
            .with_qa_command(QaCommandProof::passed_in_worktree(
                "cargo test --workspace --all-features",
                "/tmp/other-eatme-worktree",
            )),
    )?;
    assert_eq!(stale_worktree.status, "not_ready");
    assert_blocker(
        &stale_worktree,
        "stale_qa_proof",
        "qa.cargo test --workspace --all-features",
    );

    let failed_command = evaluate_pr199_recovery_readiness(
        ready_evidence()
            .without_qa_command("cargo run -q -p eatme-cli -- assets validate --json")
            .with_qa_command(QaCommandProof::failed_in_worktree(
                "cargo run -q -p eatme-cli -- assets validate --json",
                current_worktree(),
                101,
            )),
    )?;
    assert_eq!(failed_command.status, "not_ready");
    assert_blocker(
        &failed_command,
        "failed_qa_command",
        "qa.cargo run -q -p eatme-cli -- assets validate --json",
    );

    Ok(())
}

#[test]
fn fetches_only_fixed_pr199_metadata_through_command_runner() -> Result<()> {
    let runner = RecordingRunner::with_output(CommandOutput {
        command: "gh pr view 199 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup"
            .to_string(),
        exit_status: Some(0),
        stdout: json!({
            "headRefOid": "abc123",
            "mergeStateStatus": "CLEAN",
            "mergeable": "MERGEABLE",
            "statusCheckRollup": []
        })
        .to_string(),
        stderr: String::new(),
    });

    let metadata = fetch_pr199_metadata(&runner)?;

    assert_eq!(
        runner.commands(),
        vec![
            "gh pr view 199 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup"
                .to_string()
        ]
    );
    assert_eq!(metadata.number, 199);
    assert!(metadata.supporting_context_only);

    Ok(())
}

#[test]
fn pr_metadata_never_overrides_local_blockers() -> Result<()> {
    let report = evaluate_pr199_recovery_readiness(
        evidence_with_workflow_qa_and_metadata()
            .with_workflow_proof("default-workflow-attempt.log")
            .with_alice_action(AliceActionEvidence::missing_original("save-project"))
            .with_pr_metadata(json!({
                "number": 199,
                "mergeStateStatus": "CLEAN",
                "mergeable": "MERGEABLE",
                "statusCheckRollup": []
            })),
    )?;

    assert_eq!(report.status, "not_ready");
    assert!(report.pr_metadata.supporting_context_only);
    assert_blocker(&report, "invalid_workflow_proof", "workflow_proof");
    assert_blocker_with_action(
        &report,
        "missing_real_action_evidence",
        "alice.original.actions.save-project",
        "save-project",
        "original",
    );

    Ok(())
}

#[test]
fn rejects_non_pr199_recovery_scope() -> Result<()> {
    let report = evaluate_pr199_recovery_readiness(ready_evidence().with_pr(200))?;

    assert_eq!(report.status, "not_ready");
    assert_blocker(&report, "wrong_pr_scope", "pr");

    Ok(())
}

fn ready_evidence() -> Pr199RecoveryEvidence {
    evidence_with_workflow_qa_and_metadata()
        .with_alice_action(AliceActionEvidence::real_original(
            "save-project",
            "evidence/alice/original/save-project.json",
        ))
        .with_alice_action(AliceActionEvidence::real_original(
            "place-object",
            "evidence/alice/original/place-object.json",
        ))
}

fn evidence_with_workflow_qa_and_metadata() -> Pr199RecoveryEvidence {
    REQUIRED_QA_COMMANDS.iter().fold(
        Pr199RecoveryEvidence::for_pr199()
            .with_workflow_proof("RealDefaultWorkflowNoTimeout")
            .with_pr_metadata(json!({
                "number": 199,
                "supporting_context_only": true,
                "headRefOid": "abc123",
                "mergeStateStatus": "CLEAN",
                "mergeable": "MERGEABLE",
                "statusCheckRollup": []
            })),
        |evidence: Pr199RecoveryEvidence, command| {
            evidence.with_qa_command(QaCommandProof::passed_in_worktree(
                *command,
                current_worktree(),
            ))
        },
    )
}

fn current_worktree() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("eatme-core must live under crates/eatme-core")
        .to_string_lossy()
        .into_owned()
}

fn ready_evidence_with_workflow(workflow_proof: Option<&str>) -> Pr199RecoveryEvidence {
    match workflow_proof {
        Some(proof) => ready_evidence().with_workflow_proof(proof),
        None => ready_evidence().without_workflow_proof(),
    }
}

fn assert_no_blocker(report: &Pr199RecoveryReport, field_prefix: &str) {
    assert!(
        report
            .blockers
            .iter()
            .all(|blocker| !blocker.field.starts_with(field_prefix)),
        "unexpected blocker for {field_prefix}: {:?}",
        report.blockers
    );
}

fn assert_blocker(report: &Pr199RecoveryReport, code: &str, field: &str) {
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.code == code && blocker.field == field),
        "expected blocker {code} for {field}; got {:?}",
        report.blockers
    );
}

fn assert_blocker_with_action(
    report: &Pr199RecoveryReport,
    code: &str,
    field: &str,
    action: &str,
    target: &str,
) {
    assert!(
        report.blockers.iter().any(|blocker| {
            blocker.code == code
                && blocker.field == field
                && blocker.action.as_deref() == Some(action)
                && blocker.target.as_deref() == Some(target)
        }),
        "expected blocker {code} for {field}/{action}/{target}; got {:?}",
        report.blockers
    );
}

struct RecordingRunner {
    output: CommandOutput,
    commands: RefCell<Vec<String>>,
}

impl RecordingRunner {
    fn with_output(output: CommandOutput) -> Self {
        Self {
            output,
            commands: RefCell::new(Vec::new()),
        }
    }

    fn commands(&self) -> Vec<String> {
        self.commands.borrow().clone()
    }
}

impl CommandRunner for RecordingRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        self.commands.borrow_mut().push(spec.shell_display());
        Ok(self.output.clone())
    }
}
