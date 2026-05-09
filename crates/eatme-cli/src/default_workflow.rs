use std::fs;
use std::path::Path;

pub(crate) mod github;
mod types;
use types::{Blocker, Decision, ReadinessEvidence, ReadinessReport};

const EVIDENCE_SCHEMA_VERSION: &str = "eatme.default-workflow-pr-readiness-evidence/v1";

pub(crate) fn evaluate_pr_readiness_evidence(path: &Path) -> ReadinessOutcome {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return ReadinessOutcome::input_error(format!(
                "failed to read evidence file {}: {error}",
                path.display()
            ));
        }
    };

    let evidence = match serde_json::from_str::<ReadinessEvidence>(&content) {
        Ok(evidence) => evidence,
        Err(error) => {
            return ReadinessOutcome::input_error(format!(
                "failed to parse readiness evidence {}: {error}",
                path.display()
            ));
        }
    };

    evaluate_readiness(&evidence)
}

pub(crate) struct ReadinessOutcome {
    pub(crate) report: ReadinessReport,
    pub(crate) exit_code: i32,
}

impl ReadinessOutcome {
    fn input_error(message: String) -> Self {
        Self {
            report: ReadinessReport {
                decision: Decision::NotMergeReady,
                pr_number: None,
                head_ref_name: None,
                head_ref_oid: None,
                local_head: None,
                files_modified: Vec::new(),
                no_op_justification: None,
                blockers: vec![Blocker::new("input", "malformed_evidence", message)],
            },
            exit_code: 2,
        }
    }
}

fn evaluate_readiness(evidence: &ReadinessEvidence) -> ReadinessOutcome {
    let mut blockers = Vec::new();

    audit_schema(evidence, &mut blockers);
    audit_head_alignment(evidence, &mut blockers);
    audit_pr_metadata(evidence, &mut blockers);
    audit_github_actions(evidence, &mut blockers);
    audit_runnable_evidence(evidence, &mut blockers);
    audit_quality_cycles(evidence, &mut blockers);
    audit_diff_scope(evidence, &mut blockers);
    audit_docs_impact(evidence, &mut blockers);
    audit_pr_description(evidence, &mut blockers);

    let decision = if blockers.is_empty() {
        Decision::MergeReady
    } else {
        Decision::NotMergeReady
    };
    let no_op_justification = no_op_justification(evidence, decision);
    let exit_code = match decision {
        Decision::MergeReady => 0,
        Decision::NotMergeReady => 1,
    };

    ReadinessOutcome {
        report: ReadinessReport {
            decision,
            pr_number: Some(evidence.pr.number),
            head_ref_name: Some(evidence.pr.head_ref_name.clone()),
            head_ref_oid: Some(evidence.pr.head_ref_oid.clone()),
            local_head: Some(evidence.local.head.clone()),
            files_modified: evidence.local.repository_changes.clone(),
            no_op_justification,
            blockers,
        },
        exit_code,
    }
}

fn audit_schema(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if evidence.schema_version != EVIDENCE_SCHEMA_VERSION {
        blockers.push(Blocker::new(
            "input",
            "unsupported_schema_version",
            format!(
                "expected schema_version {EVIDENCE_SCHEMA_VERSION}, got {}",
                evidence.schema_version
            ),
        ));
    }
}

fn audit_head_alignment(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if evidence.local.head != evidence.pr.head_ref_oid {
        blockers.push(Blocker::new(
            "head_alignment",
            "head_mismatch",
            format!(
                "local HEAD {} does not match PR headRefOid {}",
                evidence.local.head, evidence.pr.head_ref_oid
            ),
        ));
    }

    if !evidence
        .local
        .checkout_mode
        .eq_ignore_ascii_case("detached")
    {
        blockers.push(Blocker::new(
            "head_alignment",
            "checkout_not_detached",
            format!(
                "checkout mode must be detached at the exact PR head, got {}",
                evidence.local.checkout_mode
            ),
        ));
    }

    if evidence.local.manual_merge_performed {
        blockers.push(Blocker::new(
            "head_alignment",
            "manual_merge_detected",
            "manual merge evidence is forbidden in the recovery lane",
        ));
    }
}

fn audit_pr_metadata(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if !evidence.pr.state.eq_ignore_ascii_case("OPEN") {
        blockers.push(Blocker::new(
            "pr_metadata",
            "pr_not_open",
            format!("PR state must be OPEN, got {}", evidence.pr.state),
        ));
    }

    if evidence.pr.is_draft {
        blockers.push(Blocker::new(
            "pr_metadata",
            "pr_is_draft",
            "draft pull requests are not merge-ready",
        ));
    }

    if !evidence.pr.mergeable.eq_ignore_ascii_case("MERGEABLE") {
        blockers.push(Blocker::new(
            "pr_metadata",
            "pr_not_mergeable",
            format!("mergeable must be MERGEABLE, got {}", evidence.pr.mergeable),
        ));
    }

    if !evidence.pr.merge_state_status.eq_ignore_ascii_case("CLEAN") {
        blockers.push(Blocker::new(
            "pr_metadata",
            "merge_state_not_clean",
            format!(
                "mergeStateStatus must be CLEAN, got {}",
                evidence.pr.merge_state_status
            ),
        ));
    }
}

fn audit_github_actions(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    let mut required_checks = 0;

    for check in &evidence.checks {
        if !check.required {
            continue;
        }
        required_checks += 1;

        if check.head_sha != evidence.pr.head_ref_oid {
            blockers.push(Blocker::new(
                "github_actions",
                "stale_check_sha",
                format!(
                    "check {} is for {}, expected {}",
                    check.name, check.head_sha, evidence.pr.head_ref_oid
                ),
            ));
        }

        if !check.status.eq_ignore_ascii_case("COMPLETED") {
            blockers.push(Blocker::new(
                "github_actions",
                "check_not_complete",
                format!("check {} status is {}", check.name, check.status),
            ));
        }

        match check.conclusion.as_deref() {
            Some(conclusion) if conclusion.eq_ignore_ascii_case("SUCCESS") => {}
            Some(conclusion) if conclusion.eq_ignore_ascii_case("SKIPPED") => {
                blockers.push(Blocker::new(
                    "github_actions",
                    "required_check_skipped",
                    format!("required check {} was skipped", check.name),
                ));
            }
            Some(conclusion) => blockers.push(Blocker::new(
                "github_actions",
                "check_not_successful",
                format!("check {} conclusion is {}", check.name, conclusion),
            )),
            None => blockers.push(Blocker::new(
                "github_actions",
                "check_not_successful",
                format!("check {} has no success conclusion", check.name),
            )),
        }
    }

    if required_checks == 0 {
        blockers.push(Blocker::new(
            "github_actions",
            "missing_required_check_evidence",
            "no required exact-head GitHub Actions checks were provided",
        ));
    }
}

fn audit_runnable_evidence(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    require_command(evidence, blockers, "quality_gates", "missing_quality_gates");
    require_command(
        evidence,
        blockers,
        "assets_validate",
        "missing_assets_validate",
    );
    require_command(evidence, blockers, "gadugi_check", "missing_gadugi_check");
    require_command(evidence, blockers, "docs_build", "missing_docs_build");

    for command in &evidence.commands {
        if command.used_timeout_wrapper || command_uses_timeout_wrapper(&command.command) {
            blockers.push(Blocker::new(
                "runnable_evidence",
                "timeout_wrapper_used",
                format!("command {} used a timeout wrapper", command.id),
            ));
        }

        if command.exit_status != 0 {
            blockers.push(Blocker::new(
                "runnable_evidence",
                "evidence_command_failed",
                format!(
                    "command {} exited with status {}",
                    command.id, command.exit_status
                ),
            ));
        }
    }
}

fn require_command(
    evidence: &ReadinessEvidence,
    blockers: &mut Vec<Blocker>,
    id: &str,
    missing_code: &str,
) {
    if !evidence.commands.iter().any(|command| command.id == id) {
        blockers.push(Blocker::new(
            "runnable_evidence",
            missing_code,
            format!("mandatory command evidence {id} is missing"),
        ));
    }
}

fn command_uses_timeout_wrapper(command: &str) -> bool {
    let trimmed = command.trim_start();
    trimmed.starts_with("timeout ")
        || trimmed.starts_with("gtimeout ")
        || trimmed.contains(" timeout ")
        || trimmed.contains(" gtimeout ")
}

fn audit_quality_cycles(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if evidence.audit_cycles.len() < 3 {
        blockers.push(Blocker::new(
            "quality_audit",
            "insufficient_audit_cycles",
            format!(
                "at least three SEEK/VALIDATE/FIX cycles are required, got {}",
                evidence.audit_cycles.len()
            ),
        ));
    }

    match evidence.audit_cycles.last() {
        Some(cycle) if cycle.clean => {}
        Some(cycle) => blockers.push(Blocker::new(
            "quality_audit",
            "final_audit_cycle_not_clean",
            format!("final audit cycle {} is not clean", cycle.name),
        )),
        None => blockers.push(Blocker::new(
            "quality_audit",
            "final_audit_cycle_not_clean",
            "no final audit cycle was provided",
        )),
    }

    for cycle in &evidence.audit_cycles {
        if cycle.seek.trim().is_empty()
            || cycle.validate.trim().is_empty()
            || cycle.fix.trim().is_empty()
        {
            blockers.push(Blocker::new(
                "quality_audit",
                "audit_cycle_incomplete",
                format!(
                    "audit cycle {} must include SEEK, VALIDATE, and FIX evidence",
                    cycle.name
                ),
            ));
        }
    }
}

fn audit_diff_scope(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if !evidence.diff.focused {
        blockers.push(Blocker::new(
            "diff_scope",
            "diff_not_focused",
            "PR diff scope is not marked focused",
        ));
    }

    if !evidence.diff.unrelated_churn.is_empty() {
        blockers.push(Blocker::new(
            "diff_scope",
            "unrelated_churn",
            format!(
                "unrelated churn: {}",
                evidence.diff.unrelated_churn.join(", ")
            ),
        ));
    }

    if !evidence.diff.generated_artifacts.is_empty() {
        blockers.push(Blocker::new(
            "diff_scope",
            "generated_artifact_in_diff",
            format!(
                "generated artifacts: {}",
                evidence.diff.generated_artifacts.join(", ")
            ),
        ));
    }

    for file in &evidence.diff.files {
        if file.starts_with("target/") || file.contains("/target/") {
            blockers.push(Blocker::new(
                "diff_scope",
                "generated_artifact_in_diff",
                format!("generated build artifact appears in diff: {file}"),
            ));
        }
    }
}

fn audit_docs_impact(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if !evidence.docs.impact_reviewed {
        blockers.push(Blocker::new(
            "docs_impact",
            "docs_impact_not_reviewed",
            "documentation impact review is missing",
        ));
    }

    if !evidence.docs.updated_or_ruled_out {
        blockers.push(Blocker::new(
            "docs_impact",
            "docs_update_not_proven_or_ruled_out",
            "documentation update or explicit no-impact ruling is missing",
        ));
    }

    if !evidence.docs.strict_build_passed
        || evidence.docs.strict_build_command.trim() != "mkdocs build --strict"
    {
        blockers.push(Blocker::new(
            "docs_impact",
            "docs_strict_build_missing_or_failed",
            "mkdocs build --strict evidence is missing or failed",
        ));
    }
}

fn audit_pr_description(evidence: &ReadinessEvidence, blockers: &mut Vec<Blocker>) {
    if evidence.pr_description_evidence.head_ref_oid != evidence.pr.head_ref_oid {
        blockers.push(Blocker::new(
            "pr_description",
            "stale_pr_description_head",
            format!(
                "PR description evidence head {} does not match {}",
                evidence.pr_description_evidence.head_ref_oid, evidence.pr.head_ref_oid
            ),
        ));
    }

    if !evidence.pr_description_evidence.mentions_green_actions {
        blockers.push(Blocker::new(
            "pr_description",
            "missing_green_actions_evidence",
            "PR description does not mention exact-head green Actions evidence",
        ));
    }

    if !evidence.pr_description_evidence.mentions_runnable_qa {
        blockers.push(Blocker::new(
            "pr_description",
            "missing_runnable_qa_evidence",
            "PR description does not mention runnable QA evidence",
        ));
    }

    if !evidence.pr_description_evidence.mentions_docs_impact {
        blockers.push(Blocker::new(
            "pr_description",
            "missing_docs_impact_evidence",
            "PR description does not mention docs impact evidence",
        ));
    }

    if !evidence
        .pr_description_evidence
        .mentions_quality_audit_cycles
    {
        blockers.push(Blocker::new(
            "pr_description",
            "missing_quality_audit_evidence",
            "PR description does not mention three quality-audit cycles",
        ));
    }

    if !evidence
        .pr_description_evidence
        .unsupported_claims
        .is_empty()
    {
        blockers.push(Blocker::new(
            "pr_description",
            "unsupported_readiness_claim",
            format!(
                "unsupported claims: {}",
                evidence
                    .pr_description_evidence
                    .unsupported_claims
                    .join(", ")
            ),
        ));
    }
}

fn no_op_justification(evidence: &ReadinessEvidence, decision: Decision) -> Option<String> {
    if !evidence.local.repository_changes.is_empty() {
        return None;
    }

    let suffix = match decision {
        Decision::MergeReady => {
            "workflow-accepted no-op: exact-head checks, mandatory runnable evidence, docs impact, PR description evidence, focused diff, and three clean quality-audit cycles are present"
        }
        Decision::NotMergeReady => {
            "no repository changes are reported, but the no-op is not merge-ready until all blockers are resolved"
        }
    };

    Some(format!(
        "No repository files changed at PR head {}; {suffix}.",
        evidence.pr.head_ref_oid
    ))
}
