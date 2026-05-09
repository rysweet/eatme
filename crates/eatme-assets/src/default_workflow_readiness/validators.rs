use super::{
    CheckConclusion, CheckRunEvidence, CheckStatus, CommandEvidence, CommandStatus,
    REQUIRED_COMMANDS, ReadinessArtifact, ReadinessInput,
};

pub(super) fn validate_required_commands(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
    let mut required_evidence: [Option<&CommandEvidence>; REQUIRED_COMMANDS.len()] =
        [None; REQUIRED_COMMANDS.len()];

    for evidence in &input.command_evidence {
        if evidence.used_timeout_wrapper || uses_timeout_wrapper(&evidence.command) {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("timeout wrapper used for '{}'", evidence.command),
                "rerun repository evidence commands directly with no timeout wrapper",
            ));
        }
        if let Some(index) = required_command_index(&evidence.command) {
            required_evidence[index].get_or_insert(evidence);
        }
    }

    for (index, command) in REQUIRED_COMMANDS.iter().enumerate() {
        let Some(evidence) = required_evidence[index] else {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("missing required command evidence for '{command}'"),
                "run the missing repository-supported QA command for the current head",
            ));
        };

        if evidence.head_sha != input.head_ref_oid {
            return Err(ReadinessArtifact::blocked(
                input,
                format!(
                    "command '{}' is tied to wrong head '{}'",
                    evidence.command, evidence.head_sha
                ),
                "rerun command evidence after verifying the current PR head",
            ));
        }
        if evidence.status != CommandStatus::Passed {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("command '{}' did not pass", evidence.command),
                "fix the command failure or wait for pending evidence before readiness",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_recorded_commands(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
    let mut recorded_commands = [false; REQUIRED_COMMANDS.len()];

    for command in &input.pr_evidence.recorded_commands {
        if let Some(index) = required_command_index(command) {
            recorded_commands[index] = true;
        }
    }

    for (index, command) in REQUIRED_COMMANDS.iter().enumerate() {
        if !recorded_commands[index] {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("PR evidence is missing command '{command}'"),
                "record every required command in the PR evidence",
            ));
        }
    }
    Ok(())
}

fn required_command_index(command: &str) -> Option<usize> {
    REQUIRED_COMMANDS
        .iter()
        .position(|required_command| *required_command == command)
}

fn uses_timeout_wrapper(command: &str) -> bool {
    command
        .split_whitespace()
        .next()
        .map(|first| first == "timeout" || first.ends_with("/timeout"))
        .unwrap_or(false)
}

pub(super) fn validate_check_run(
    input: &ReadinessInput,
    check: &CheckRunEvidence,
) -> Result<(), ReadinessArtifact> {
    if check.head_sha != input.head_ref_oid {
        return Err(ReadinessArtifact::blocked(
            input,
            format!(
                "check '{}' is tied to wrong head '{}'",
                check.name, check.head_sha
            ),
            "refresh check-run evidence for the current PR head",
        ));
    }
    if check.status != CheckStatus::Completed {
        return Err(ReadinessArtifact::blocked(
            input,
            format!("pending check '{}'", check.name),
            "wait for every current-head check run to complete",
        ));
    }
    if check.conclusion != CheckConclusion::Success {
        return Err(ReadinessArtifact::blocked(
            input,
            format!(
                "check '{}' concluded {}",
                check.name,
                check_conclusion_label(&check.conclusion)
            ),
            "fix or rerun unsuccessful current-head checks before readiness",
        ));
    }
    Ok(())
}

fn check_conclusion_label(conclusion: &CheckConclusion) -> &'static str {
    match conclusion {
        CheckConclusion::Success => "success",
        CheckConclusion::Failure => "failure",
        CheckConclusion::Cancelled => "cancelled",
        CheckConclusion::Skipped => "skipped",
        CheckConclusion::TimedOut => "timed_out",
        CheckConclusion::Neutral => "neutral",
        CheckConclusion::Unknown => "unknown",
    }
}
