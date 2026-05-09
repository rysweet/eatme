use std::collections::BTreeSet;

use super::recovery_safety::{
    collect_input_safety, sanitize_change_outcome, sanitize_checks, sanitize_diff_scope,
    sanitize_qa_evidence, sanitize_quality_audit_cycles, sanitize_report_text,
    sanitize_report_texts,
};
use super::recovery_scope::{collect_diff_scope, collect_docs_impact};
use super::{
    ChangeOutcome, CheckConclusion, CheckStatus, PR_204_BRANCH, PrReadinessSnapshot,
    QualityAuditCycle, QualityAuditOutcome, QualityAuditPhase, ReadinessError,
    RecoveryReadinessInput, RecoveryReadinessReport, RecoveryReadinessStatus,
    RecoveryValidationEvidence, validate_sha, validate_target_branch,
};

const RECOVERY_SCHEMA_VERSION: &str = "pr-readiness-recovery.v1";
const ASSET_VALIDATE_COMMAND: &str = "cargo run -q -p eatme-cli -- assets validate --json";
const GADUGI_CHECK_COMMAND: &str =
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json";
const QUALITY_GATE_COMMAND: &str =
    "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh";
const DOCS_BUILD_COMMAND: &str = "mkdocs build --strict";

pub fn evaluate_recovery_readiness(input: &RecoveryReadinessInput) -> RecoveryReadinessReport {
    let blockers = collect_recovery_blockers(input);
    build_recovery_report(input, blockers)
}

fn collect_recovery_blockers(input: &RecoveryReadinessInput) -> Vec<String> {
    let mut blockers = Vec::new();

    push_blocker(&mut blockers, validate_schema(input));
    push_blocker(&mut blockers, validate_recovery_baseline(input));
    push_blocker(&mut blockers, validate_validation_sha(input));
    collect_input_safety(&mut blockers, input);
    collect_required_checks(
        &mut blockers,
        &input.snapshot,
        &input.validation_sha,
        &input.required_github_checks,
    );
    push_blocker(&mut blockers, validate_mergeability(&input.snapshot));
    collect_all_required_evidence(&mut blockers, input);
    collect_quality_audit(&mut blockers, input);
    collect_diff_scope(&mut blockers, input);
    collect_docs_impact(&mut blockers, input);
    push_blocker(&mut blockers, validate_pr_description_evidence(input));
    push_blocker(&mut blockers, validate_stale_evidence_handled(input));
    push_blocker(
        &mut blockers,
        validate_change_outcome(&input.change_outcome, &input.validation_sha),
    );

    sanitize_report_texts(blockers)
}

fn collect_all_required_evidence(blockers: &mut Vec<String>, input: &RecoveryReadinessInput) {
    for (evidence, name, command) in [
        (
            &input.asset_validation,
            "asset validation",
            ASSET_VALIDATE_COMMAND,
        ),
        (
            &input.generated_gadugi_check,
            "generated Gadugi freshness",
            GADUGI_CHECK_COMMAND,
        ),
        (
            &input.quality_gate,
            "repository quality gates",
            QUALITY_GATE_COMMAND,
        ),
        (
            &input.documentation_build,
            "documentation build",
            DOCS_BUILD_COMMAND,
        ),
    ] {
        collect_required_evidence(blockers, evidence, name, command, &input.validation_sha);
    }
}

fn build_recovery_report(
    input: &RecoveryReadinessInput,
    blockers: Vec<String>,
) -> RecoveryReadinessReport {
    let is_merge_ready = blockers.is_empty();
    let status = if is_merge_ready {
        RecoveryReadinessStatus::MergeReady
    } else {
        RecoveryReadinessStatus::NotMergeReady
    };
    let validation_status = if is_merge_ready {
        format!("passed for exact current HEAD {}", input.validation_sha)
    } else {
        format!("blocked for exact current HEAD {}", input.validation_sha)
    };
    let qa_evidence = [
        &input.asset_validation,
        &input.generated_gadugi_check,
        &input.quality_gate,
        &input.documentation_build,
    ];

    RecoveryReadinessReport {
        status,
        branch: sanitize_report_text(&input.snapshot.branch),
        expected_remote_head_sha: input.expected_remote_head_sha.clone(),
        final_head_sha: input.snapshot.local_head_sha.clone(),
        validation_status,
        change_outcome: sanitize_change_outcome(&input.change_outcome),
        required_github_checks: sanitize_report_texts(input.required_github_checks.clone()),
        github_checks: sanitize_checks(&input.snapshot.checks),
        qa_evidence: sanitize_qa_evidence(&qa_evidence),
        quality_audit_cycles: sanitize_quality_audit_cycles(&input.quality_audit_cycles),
        diff_scope: sanitize_diff_scope(&input.diff_scope),
        docs_impact: input.docs_impact.clone(),
        pr_description_evidence: input.pr_description_evidence.clone(),
        wrapper_failures: sanitize_report_texts(input.wrapper_failures.clone()),
        blockers,
    }
}

fn validate_schema(input: &RecoveryReadinessInput) -> Result<(), ReadinessError> {
    if input.schema_version == RECOVERY_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ReadinessError::new(format!(
            "schema_version must be {RECOVERY_SCHEMA_VERSION}, got {}",
            input.schema_version
        )))
    }
}

fn validate_recovery_baseline(input: &RecoveryReadinessInput) -> Result<(), ReadinessError> {
    if input.snapshot.pr_number != 204 {
        return Err(ReadinessError::new(format!(
            "recovery baseline is PR #204, got PR #{}",
            input.snapshot.pr_number
        )));
    }
    validate_target_branch(&input.snapshot, PR_204_BRANCH)?;
    let expected = input
        .expected_remote_head_sha
        .as_deref()
        .ok_or_else(|| ReadinessError::new("expected_remote_head_sha is required for PR #204"))?;
    validate_sha(expected)?;
    if input.snapshot.pr_head_sha != expected
        || input.snapshot.local_head_sha != expected
        || input.validation_sha != expected
    {
        return Err(ReadinessError::new(format!(
            "PR #204 baseline requires GitHub PR head, local HEAD, and validation SHA to equal expected remote head {expected}; got pr_head={} local_head={} validation_sha={}",
            input.snapshot.pr_head_sha, input.snapshot.local_head_sha, input.validation_sha
        )));
    }
    Ok(())
}

fn validate_validation_sha(input: &RecoveryReadinessInput) -> Result<(), ReadinessError> {
    validate_sha(&input.validation_sha)?;

    if input.validation_sha != input.snapshot.local_head_sha {
        return Err(ReadinessError::new(format!(
            "validation evidence for {} is stale for current HEAD {}; rerun validation at the final PR head",
            input.validation_sha, input.snapshot.local_head_sha
        )));
    }

    Ok(())
}

fn collect_required_checks(
    blockers: &mut Vec<String>,
    snapshot: &PrReadinessSnapshot,
    validation_sha: &str,
    required_github_checks: &[String],
) {
    collect_required_check_names(blockers, validation_sha, required_github_checks);
    collect_check_head_evidence(blockers, snapshot, validation_sha);
    collect_required_check_results(blockers, snapshot, validation_sha, required_github_checks);
}

fn collect_required_check_names(
    blockers: &mut Vec<String>,
    validation_sha: &str,
    required_github_checks: &[String],
) {
    if required_github_checks.is_empty() {
        blockers.push(format!(
            "required_github_checks must name trusted required GitHub checks for exact head {validation_sha}"
        ));
    }
    let mut required_names = BTreeSet::new();
    for required_name in required_github_checks {
        if required_name.trim().is_empty() {
            blockers.push("required_github_checks must not include empty names".to_string());
        } else if !required_names.insert(required_name.as_str()) {
            blockers.push(format!(
                "required_github_checks contains duplicate required check {required_name}"
            ));
        }
    }
}

fn collect_check_head_evidence(
    blockers: &mut Vec<String>,
    snapshot: &PrReadinessSnapshot,
    validation_sha: &str,
) {
    if snapshot.checks.is_empty() {
        blockers.push(format!(
            "GitHub Actions evidence is missing for exact head {validation_sha}"
        ));
    }

    for check in &snapshot.checks {
        if let Err(error) = validate_sha(&check.head_sha) {
            blockers.push(format!(
                "GitHub Actions check {} has invalid head SHA: {error}",
                check.name
            ));
            continue;
        }
        if check.head_sha != validation_sha {
            blockers.push(format!(
                "GitHub Actions check {} is for {}, not exact head {validation_sha}",
                check.name, check.head_sha
            ));
        }
    }
}

fn collect_required_check_results(
    blockers: &mut Vec<String>,
    snapshot: &PrReadinessSnapshot,
    validation_sha: &str,
    required_github_checks: &[String],
) {
    for required_name in required_github_checks {
        if required_name.trim().is_empty() {
            continue;
        }
        let Some(check) = snapshot
            .checks
            .iter()
            .find(|check| check.name == *required_name)
        else {
            blockers.push(format!(
                "required GitHub Actions check {required_name} is missing or omitted at {validation_sha}"
            ));
            continue;
        };
        if check.status != CheckStatus::Completed || check.conclusion != CheckConclusion::Success {
            blockers.push(format!(
                "required GitHub Actions check {} is not green at {validation_sha}: status={} conclusion={}",
                check.name, check.status, check.conclusion
            ));
        }
    }
}

fn validate_mergeability(snapshot: &PrReadinessSnapshot) -> Result<(), ReadinessError> {
    if snapshot.merge_state_status != "CLEAN" || snapshot.mergeable != "MERGEABLE" {
        return Err(ReadinessError::new(format!(
            "PR #{} is not merge-ready at {}: mergeStateStatus={} mergeable={}",
            snapshot.pr_number,
            snapshot.local_head_sha,
            snapshot.merge_state_status,
            snapshot.mergeable
        )));
    }

    Ok(())
}

fn collect_required_evidence(
    blockers: &mut Vec<String>,
    evidence: &RecoveryValidationEvidence,
    expected_name: &str,
    expected_command: &str,
    validation_sha: &str,
) {
    push_blocker(blockers, validate_evidence_head(evidence, validation_sha));

    if evidence.name != expected_name {
        blockers.push(format!(
            "expected evidence named {expected_name}, got {}",
            evidence.name
        ));
    }

    if evidence.command != expected_command {
        blockers.push(format!(
            "{} must use command {expected_command} at {validation_sha}, got {}",
            evidence.name, evidence.command
        ));
    }

    if evidence.summary.trim().is_empty() {
        blockers.push(format!(
            "{} must include a summarized runnable QA result for {validation_sha}",
            evidence.name
        ));
    }

    if !evidence.passed || evidence.exit_status != 0 {
        blockers.push(format!(
            "{} did not pass at {validation_sha}; command `{}` exited {}; summary: {}",
            evidence.name, evidence.command, evidence.exit_status, evidence.summary
        ));
    }
}

fn validate_evidence_head(
    evidence: &RecoveryValidationEvidence,
    validation_sha: &str,
) -> Result<(), ReadinessError> {
    validate_sha(&evidence.evidence_sha)?;

    if evidence.evidence_sha != validation_sha {
        return Err(ReadinessError::new(format!(
            "{} evidence names {}, but final validation requires {}; rerun current-head validation",
            evidence.name, evidence.evidence_sha, validation_sha
        )));
    }

    Ok(())
}

fn collect_quality_audit(blockers: &mut Vec<String>, input: &RecoveryReadinessInput) {
    if input.quality_audit_cycles.len() < 3 {
        blockers.push(format!(
            "three SEEK/VALIDATE/FIX quality-audit cycles are required for {}",
            input.validation_sha
        ));
    }

    for (expected_cycle_number, cycle) in (1..).zip(input.quality_audit_cycles.iter()) {
        collect_quality_audit_cycle(
            blockers,
            expected_cycle_number,
            cycle,
            &input.validation_sha,
        );
    }

    if input
        .quality_audit_cycles
        .last()
        .is_none_or(|cycle| cycle.outcome != QualityAuditOutcome::Clean)
    {
        blockers.push(format!(
            "final cycle clean quality-audit outcome is required for {}",
            input.validation_sha
        ));
    }
}

fn collect_quality_audit_cycle(
    blockers: &mut Vec<String>,
    expected_cycle_number: u64,
    cycle: &QualityAuditCycle,
    validation_sha: &str,
) {
    if cycle.cycle_number != expected_cycle_number {
        blockers.push(format!(
            "quality-audit cycle numbers must be contiguous and strictly increasing from 1; expected cycle {expected_cycle_number}, got {}",
            cycle.cycle_number
        ));
    }
    if let Err(error) = validate_sha(&cycle.head_sha) {
        blockers.push(format!(
            "quality-audit cycle {} has invalid head SHA: {error}",
            cycle.cycle_number
        ));
    } else if cycle.head_sha != validation_sha {
        blockers.push(format!(
            "quality-audit cycle {} names {}, not exact head {validation_sha}",
            cycle.cycle_number, cycle.head_sha
        ));
    }
    if !has_all_quality_audit_phases(&cycle.phases) {
        blockers.push(format!(
            "quality-audit cycle {} must include SEEK, VALIDATE, and FIX phases",
            cycle.cycle_number
        ));
    }
    if cycle.summary.trim().is_empty() {
        blockers.push(format!(
            "quality-audit cycle {} must include a summary",
            cycle.cycle_number
        ));
    }
}

fn has_all_quality_audit_phases(phases: &[QualityAuditPhase]) -> bool {
    let mut has_seek = false;
    let mut has_validate = false;
    let mut has_fix = false;

    for phase in phases {
        match phase {
            QualityAuditPhase::Seek => has_seek = true,
            QualityAuditPhase::Validate => has_validate = true,
            QualityAuditPhase::Fix => has_fix = true,
        }
        if has_seek && has_validate && has_fix {
            return true;
        }
    }

    false
}

fn validate_pr_description_evidence(input: &RecoveryReadinessInput) -> Result<(), ReadinessError> {
    validate_sha(&input.pr_description_evidence.head_sha)?;
    if input.pr_description_evidence.head_sha != input.validation_sha {
        return Err(ReadinessError::new(format!(
            "PR description evidence names {}, but current head is {}",
            input.pr_description_evidence.head_sha, input.validation_sha
        )));
    }
    if !input.pr_description_evidence.contains_readiness_evidence {
        return Err(ReadinessError::new(format!(
            "missing PR description evidence for current head {}",
            input.validation_sha
        )));
    }
    if !input.pr_description_evidence.contains_bounded_nonclaims {
        return Err(ReadinessError::new(format!(
            "PR description evidence for {} must include bounded nonclaims",
            input.validation_sha
        )));
    }
    Ok(())
}

fn validate_stale_evidence_handled(input: &RecoveryReadinessInput) -> Result<(), ReadinessError> {
    if input.stale_evidence_handled {
        Ok(())
    } else {
        Err(ReadinessError::new(format!(
            "older tested-head evidence must be labeled stale/non-current before reporting {} as ready",
            input.validation_sha
        )))
    }
}

fn validate_change_outcome(
    outcome: &ChangeOutcome,
    validation_sha: &str,
) -> Result<(), ReadinessError> {
    match outcome {
        ChangeOutcome::NoOp { justification } => {
            if justification.trim().is_empty() {
                Err(ReadinessError::new(format!(
                    "No-op justification is required when no files changed at {validation_sha}"
                )))
            } else {
                Ok(())
            }
        }
        ChangeOutcome::FilesModified(files) => {
            if files.is_empty() {
                return Err(ReadinessError::new(format!(
                    "Files modified must list at least one path, or provide a No-op justification for {validation_sha}"
                )));
            }

            if let Some(invalid) = files.iter().find(|file| file.trim().is_empty()) {
                Err(ReadinessError::new(format!(
                    "Files modified contains an empty path near {invalid:?}"
                )))
            } else {
                Ok(())
            }
        }
    }
}

fn push_blocker(blockers: &mut Vec<String>, result: Result<(), ReadinessError>) {
    if let Err(error) = result {
        blockers.push(error.to_string());
    }
}
