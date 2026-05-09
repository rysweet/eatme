use super::{
    ChangeOutcome, CheckSummary, DiffScopeEvidence, QualityAuditCycle, RecoveryReadinessInput,
    RecoveryValidationEvidence,
};

const SUMMARY_TEXT_LIMIT: usize = 400;
const PATH_TEXT_LIMIT: usize = 240;
const JUSTIFICATION_TEXT_LIMIT: usize = 400;
const RENDER_VALUE_LIMIT: usize = 512;

pub(super) fn collect_input_safety(blockers: &mut Vec<String>, input: &RecoveryReadinessInput) {
    for evidence in [
        &input.asset_validation,
        &input.generated_gadugi_check,
        &input.quality_gate,
        &input.documentation_build,
    ] {
        collect_text_safety(
            blockers,
            &format!("{} summary", evidence.name),
            &evidence.summary,
            SUMMARY_TEXT_LIMIT,
        );
    }
    for cycle in &input.quality_audit_cycles {
        collect_text_safety(
            blockers,
            &format!("quality-audit cycle {} summary", cycle.cycle_number),
            &cycle.summary,
            SUMMARY_TEXT_LIMIT,
        );
    }
    for path in &input.diff_scope.changed_files {
        collect_text_safety(blockers, "diff scope changed path", path, PATH_TEXT_LIMIT);
    }
    match &input.change_outcome {
        ChangeOutcome::NoOp { justification } => collect_text_safety(
            blockers,
            "No-op justification",
            justification,
            JUSTIFICATION_TEXT_LIMIT,
        ),
        ChangeOutcome::FilesModified(files) => {
            for file in files {
                collect_text_safety(blockers, "Files modified path", file, PATH_TEXT_LIMIT);
            }
        }
    }
}

fn collect_text_safety(blockers: &mut Vec<String>, field: &str, value: &str, limit: usize) {
    if value.chars().any(char::is_control) {
        blockers.push(format!("{field} contains control characters or newlines"));
    }
    if value.chars().count() > limit {
        blockers.push(format!("{field} exceeds {limit} characters"));
    }
}

pub(super) fn sanitize_report_texts(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| sanitize_report_text(&value))
        .collect()
}

pub(super) fn sanitize_change_outcome(outcome: &ChangeOutcome) -> ChangeOutcome {
    match outcome {
        ChangeOutcome::NoOp { justification } => ChangeOutcome::NoOp {
            justification: sanitize_report_text(justification),
        },
        ChangeOutcome::FilesModified(files) => {
            ChangeOutcome::FilesModified(sanitize_strings(files))
        }
    }
}

pub(super) fn sanitize_checks(checks: &[CheckSummary]) -> Vec<CheckSummary> {
    checks
        .iter()
        .map(|check| CheckSummary {
            name: sanitize_report_text(&check.name),
            status: check.status.clone(),
            conclusion: check.conclusion.clone(),
            required: check.required,
            head_sha: check.head_sha.clone(),
        })
        .collect()
}

pub(super) fn sanitize_qa_evidence(
    evidence: &[&RecoveryValidationEvidence],
) -> Vec<RecoveryValidationEvidence> {
    evidence
        .iter()
        .map(|evidence| RecoveryValidationEvidence {
            name: sanitize_report_text(&evidence.name),
            command: sanitize_report_text(&evidence.command),
            evidence_sha: evidence.evidence_sha.clone(),
            exit_status: evidence.exit_status,
            summary: sanitize_report_text(&evidence.summary),
            passed: evidence.passed,
        })
        .collect()
}

pub(super) fn sanitize_quality_audit_cycles(
    cycles: &[QualityAuditCycle],
) -> Vec<QualityAuditCycle> {
    cycles
        .iter()
        .map(|cycle| QualityAuditCycle {
            cycle_number: cycle.cycle_number,
            phases: cycle.phases.clone(),
            outcome: cycle.outcome.clone(),
            head_sha: cycle.head_sha.clone(),
            summary: sanitize_report_text(&cycle.summary),
        })
        .collect()
}

pub(super) fn sanitize_diff_scope(diff_scope: &DiffScopeEvidence) -> DiffScopeEvidence {
    DiffScopeEvidence {
        changed_files: sanitize_strings(&diff_scope.changed_files),
        focused: diff_scope.focused,
    }
}

pub(super) fn sanitize_report_text(value: &str) -> String {
    let normalized = normalize_control_characters(value);
    let redacted = redact_obvious_tokens(&normalized);
    if redacted.chars().count() <= RENDER_VALUE_LIMIT {
        redacted
    } else {
        let mut truncated: String = redacted.chars().take(RENDER_VALUE_LIMIT).collect();
        truncated.push_str("...");
        truncated
    }
}

fn sanitize_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| sanitize_report_text(value))
        .collect()
}

fn normalize_control_characters(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut previous_space = false;
    for character in value.chars() {
        let normalized = if character.is_control() {
            ' '
        } else {
            character
        };
        if normalized.is_whitespace() {
            if !previous_space {
                output.push(' ');
                previous_space = true;
            }
        } else {
            output.push(normalized);
            previous_space = false;
        }
    }
    output.trim().to_string()
}

fn redact_obvious_tokens(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if contains_secret_marker(token) {
                "[REDACTED]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_secret_marker(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    lower.contains(concat!("github", "_pat", "_"))
        || lower.contains(concat!("gh", "p_"))
        || lower.contains(concat!("gh", "o_"))
        || lower.contains(concat!("gh", "u_"))
        || lower.contains(concat!("gh", "s_"))
        || lower.contains(concat!("gh", "r_"))
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || lower.contains("xoxp-")
        || lower.contains("token=")
        || lower.contains("api_key=")
        || lower.contains("apikey=")
        || lower.contains("secret=")
        || token.starts_with("AKIA")
}
