use std::fmt::Write;

use super::recovery_safety::sanitize_report_text;
use super::{
    ChangeOutcome, CheckConclusion, CheckSummary, QualityAuditPhase, RecoveryReadinessReport,
    RecoveryReadinessStatus, ReviewNoteInput,
};

pub fn render_review_note(input: ReviewNoteInput) -> String {
    let snapshot = input.snapshot;
    let evidence_sha = snapshot.pr_head_sha.as_str();
    let check_summary = render_check_summary(&snapshot.checks);

    format!(
        "Default-workflow readiness recorded for PR #{}.\n\
         \n\
         Exact SHA: {evidence_sha}\n\
         Branch: {}\n\
         \n\
         Verified for this exact SHA:\n\
         - local HEAD equals PR head {evidence_sha}\n\
         - asset validation {}\n\
         - generated Gadugi freshness {}\n\
         - repository quality gates {}\n\
         - documentation build {}\n\
         - GitHub checks for {evidence_sha}: {check_summary}\n\
         - mergeStateStatus={} and mergeable={} for {evidence_sha}\n\
         - older tested-head evidence is {} and is not presented as current validation\n\
         \n\
         Nonclaims: this does not validate full Alice UI automation, grading, \
         creative assessment, visible rendering correctness, Save completion, \
         or first-lesson completion.",
        snapshot.pr_number,
        report_value(&snapshot.branch),
        evidence_word(input.local_evidence.asset_validation, evidence_sha),
        evidence_word(
            input.local_evidence.generated_gadugi_freshness,
            evidence_sha
        ),
        evidence_word(input.local_evidence.quality_gates, evidence_sha),
        evidence_word(input.local_evidence.documentation_build, evidence_sha),
        report_value(&snapshot.merge_state_status),
        report_value(&snapshot.mergeable),
        if input.stale_evidence_handled {
            "stale/non-current"
        } else {
            "not yet scrubbed"
        }
    )
}

pub fn render_final_report(report: &RecoveryReadinessReport) -> String {
    let status_label = match report.status {
        RecoveryReadinessStatus::MergeReady => "MERGE_READY",
        RecoveryReadinessStatus::NotMergeReady => "NOT_MERGE_READY",
    };
    let mut body =
        String::with_capacity(768 + report.blockers.iter().map(String::len).sum::<usize>());
    let _ = writeln!(body, "{status_label}");
    let _ = writeln!(body, "Branch: {}", report_value(&report.branch));
    let _ = writeln!(
        body,
        "Expected remote HEAD: {}",
        report
            .expected_remote_head_sha
            .as_deref()
            .unwrap_or("missing")
    );
    let _ = writeln!(body, "Final HEAD: {}", report.final_head_sha);
    let _ = writeln!(body, "Validation status: {}", report.validation_status);

    match &report.change_outcome {
        ChangeOutcome::NoOp { justification } => {
            let _ = writeln!(body, "No-op justification: {}", report_value(justification));
        }
        ChangeOutcome::FilesModified(files) => {
            body.push_str("Files modified: ");
            push_comma_separated(&mut body, files);
            body.push('\n');
        }
    }

    body.push_str("Required GitHub checks: ");
    push_comma_separated(&mut body, &report.required_github_checks);
    body.push('\n');
    push_github_checks(&mut body, report);
    push_qa_evidence(&mut body, report);
    push_quality_audit_evidence(&mut body, report);
    let _ = writeln!(
        body,
        "Diff scope: focused={} changed_files={}",
        report.diff_scope.focused,
        comma_separated(&report.diff_scope.changed_files)
    );
    let _ = writeln!(
        body,
        "Docs impact: docs_changed={} strict_build_required={}",
        report.docs_impact.docs_changed, report.docs_impact.strict_build_required
    );
    let _ = writeln!(
        body,
        "PR description evidence: head={} readiness_evidence={} bounded_nonclaims={}",
        report.pr_description_evidence.head_sha,
        report.pr_description_evidence.contains_readiness_evidence,
        report.pr_description_evidence.contains_bounded_nonclaims
    );
    body.push_str("Historical wrapper failures (context only, not readiness evidence): ");
    wrapper_failures_summary(&mut body, &report.wrapper_failures);
    body.push('\n');
    if !report.blockers.is_empty() {
        body.push_str("Blockers:\n");
        for blocker in &report.blockers {
            let _ = writeln!(body, "- {}", report_value(blocker));
        }
    }
    body
}

fn push_github_checks(body: &mut String, report: &RecoveryReadinessReport) {
    body.push_str("GitHub checks:\n");
    if report.github_checks.is_empty() {
        body.push_str("- none reported\n");
        return;
    }
    for check in &report.github_checks {
        let _ = writeln!(
            body,
            "- {}: status={} conclusion={} head={} required_flag={}",
            report_value(&check.name),
            check.status,
            check.conclusion,
            check.head_sha,
            check.required
        );
    }
}

fn push_qa_evidence(body: &mut String, report: &RecoveryReadinessReport) {
    body.push_str("QA evidence:\n");
    for evidence in &report.qa_evidence {
        let _ = writeln!(
            body,
            "- {}: passed={} exit={} head={} command=`{}` summary={}",
            report_value(&evidence.name),
            evidence.passed,
            evidence.exit_status,
            evidence.evidence_sha,
            report_value(&evidence.command),
            report_value(&evidence.summary)
        );
    }
}

fn push_quality_audit_evidence(body: &mut String, report: &RecoveryReadinessReport) {
    body.push_str("Quality-audit evidence:\n");
    if report.quality_audit_cycles.is_empty() {
        body.push_str("- none reported\n");
        return;
    }
    for cycle in &report.quality_audit_cycles {
        let _ = writeln!(
            body,
            "- cycle {}: outcome={:?} head={} phases={} summary={}",
            cycle.cycle_number,
            cycle.outcome,
            cycle.head_sha,
            phase_labels(&cycle.phases),
            report_value(&cycle.summary)
        );
    }
}

fn evidence_word(passed: bool, evidence_sha: &str) -> String {
    if passed {
        format!("passed for {evidence_sha}")
    } else {
        format!("not verified for {evidence_sha}")
    }
}

fn render_check_summary(checks: &[CheckSummary]) -> String {
    if checks.is_empty() {
        return "no GitHub checks reported".to_string();
    }

    let mut required_successes = Vec::new();
    let mut required_blockers = Vec::new();
    let mut optional_skipped = Vec::new();

    for check in checks {
        match (check.required, &check.conclusion) {
            (true, CheckConclusion::Success) => required_successes.push(check.name.as_str()),
            (true, conclusion) => {
                required_blockers.push(format!("{}={conclusion}", report_value(&check.name)));
            }
            (false, CheckConclusion::Skipped) => optional_skipped.push(check.name.as_str()),
            (false, _) => {}
        }
    }

    let mut parts = Vec::new();
    if required_blockers.is_empty() {
        parts.push(format!(
            "required GitHub checks completed successfully ({})",
            join_or_none(&required_successes)
        ));
    } else {
        parts.push(format!(
            "required GitHub checks are not ready ({})",
            required_blockers.join(", ")
        ));
    }

    if !optional_skipped.is_empty() {
        parts.push(format!(
            "optional skipped checks reported as skipped ({})",
            join_or_none(&optional_skipped)
        ));
    }

    parts.join("; ")
}

fn join_or_none(values: &[&str]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        let sanitized: Vec<String> = values.iter().map(|value| report_value(value)).collect();
        sanitized.join(", ")
    }
}

fn phase_labels(phases: &[QualityAuditPhase]) -> String {
    let labels: Vec<String> = phases
        .iter()
        .map(|phase| match phase {
            QualityAuditPhase::Seek => "SEEK".to_string(),
            QualityAuditPhase::Validate => "VALIDATE".to_string(),
            QualityAuditPhase::Fix => "FIX".to_string(),
        })
        .collect();
    labels.join(",")
}

fn wrapper_failures_summary(body: &mut String, wrapper_failures: &[String]) {
    if wrapper_failures.is_empty() {
        body.push_str("none recorded");
    } else {
        push_comma_separated(body, wrapper_failures);
    }
}

fn comma_separated(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    let mut body = String::new();
    push_comma_separated(&mut body, values);
    body
}

fn push_comma_separated(body: &mut String, values: &[String]) {
    if values.is_empty() {
        body.push_str("none");
        return;
    }
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            body.push_str(", ");
        }
        body.push_str(&report_value(value));
    }
}

fn report_value(value: &str) -> String {
    sanitize_report_text(value)
}
