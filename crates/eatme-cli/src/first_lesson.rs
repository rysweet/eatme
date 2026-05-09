use anyhow::Result;
use clap::Args;
#[cfg(test)]
use eatme_alice::compare::LessonReadinessEvidenceProgress;
use eatme_alice::compare::{
    DesktopNextActionSummary, FirstLessonReadinessSequenceReport,
    OriginalAliceActionEvidenceReport, OriginalAliceActionEvidenceStatus, ReadinessEvidenceItem,
};
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;

#[derive(Args)]
pub struct RunFirstLessonReadinessArgs {
    #[arg(long, default_value = "assets/alice-comparison-targets.yaml")]
    pub registry: PathBuf,
    #[arg(long, default_value = "baseline")]
    pub baseline_target: String,
    #[arg(long, default_value = "modernized")]
    pub modernized_target: String,
    #[arg(long)]
    pub baseline_home: Option<PathBuf>,
    #[arg(long)]
    pub modernized_home: Option<PathBuf>,
    #[arg(long)]
    pub run_id: String,
    #[arg(long, default_value = "runs")]
    pub runs_dir: PathBuf,
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub no_memory: bool,
    #[arg(long)]
    pub offline_package: bool,
    #[arg(long)]
    pub starter_project: Option<PathBuf>,
    #[arg(long)]
    pub execute: bool,
}

pub fn print_first_lesson_readiness_result(
    json: bool,
    report: &FirstLessonReadinessSequenceReport,
) -> Result<()> {
    write_first_lesson_readiness_result(std::io::stdout().lock(), json, report)
}

fn write_first_lesson_readiness_result(
    mut writer: impl Write,
    json: bool,
    report: &FirstLessonReadinessSequenceReport,
) -> Result<()> {
    if json {
        writeln!(writer, "{}", serde_json::to_string_pretty(report)?)?;
        return Ok(());
    }

    writeln!(
        writer,
        "First-lesson automation scenario readiness: {}",
        terminal_plain(scenario_readiness_status(report))
    )?;
    writeln!(
        writer,
        "Desktop proof: {} ({}) - {}",
        terminal_plain(&report.desktop_proof_contract.status),
        terminal_plain(&report.desktop_proof_contract.reason_code),
        terminal_plain(&report.desktop_proof_contract.detail)
    )?;
    write_readiness_items(&mut writer, "Shown:", &report.shown_evidence)?;
    write_readiness_items(&mut writer, "Not yet shown:", &report.not_yet_shown)?;
    if let Some(next_missing_proof) = &report.evidence_progress.next_missing_real_desktop_proof {
        writeln!(writer, "- {}", terminal_plain(next_missing_proof))?;
    }
    if let Some(desktop_next_action) = &report.desktop_next_action {
        write_desktop_next_action(&mut writer, desktop_next_action)?;
    }
    write_original_alice_action_evidence(&mut writer, &report.original_alice_action_evidence)?;
    writeln!(writer, "Unproven:")?;
    for claim in &report.unproven_claims {
        writeln!(writer, "- {}", terminal_plain(claim))?;
    }
    Ok(())
}

fn scenario_readiness_status(report: &FirstLessonReadinessSequenceReport) -> &str {
    match report.status.as_str() {
        "ready" => "ready",
        "blocked" => "blocked",
        _ => match report.readiness_status.as_str() {
            "ready" => "ready",
            "blocked_until_ui_automation" => "blocked",
            _ => "not ready",
        },
    }
}

fn terminal_plain(value: &str) -> Cow<'_, str> {
    if value.chars().all(|ch| !ch.is_control()) {
        return Cow::Borrowed(value);
    }

    let mut text = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_control() {
            text.extend(ch.escape_default());
        } else {
            text.push(ch);
        }
    }
    Cow::Owned(text)
}

fn write_readiness_items(
    mut writer: impl Write,
    heading: &str,
    items: &[ReadinessEvidenceItem],
) -> Result<()> {
    writeln!(writer, "{heading}")?;
    if items.is_empty() {
        writeln!(writer, "- Nothing yet.")?;
        return Ok(());
    }
    for item in items {
        writeln!(writer, "- {}", terminal_plain(&item.summary))?;
    }
    Ok(())
}

fn write_desktop_next_action(
    mut writer: impl Write,
    desktop: &DesktopNextActionSummary,
) -> Result<()> {
    writeln!(writer, "Desktop next action:")?;
    writeln!(writer, "- {}", terminal_plain(&desktop.summary))?;
    for observation in &desktop.observations {
        writeln!(writer, "- {}", terminal_plain(observation))?;
    }
    for required in &desktop.requires_next_evidence {
        writeln!(
            writer,
            "- Next evidence needed: {}",
            terminal_plain(required)
        )?;
    }
    Ok(())
}

fn write_original_alice_action_evidence(
    mut writer: impl Write,
    evidence: &OriginalAliceActionEvidenceReport,
) -> Result<()> {
    if evidence.status != OriginalAliceActionEvidenceStatus::Missing {
        return Ok(());
    }

    writeln!(writer, "Original Alice action evidence:")?;
    writeln!(writer, "- {}", terminal_plain(evidence.summary))?;
    writeln!(writer, "- {}", terminal_plain(evidence.detail))?;
    Ok(())
}

#[cfg(test)]
fn next_actionable_blocker_line(progress: &LessonReadinessEvidenceProgress) -> Option<String> {
    progress
        .next_actionable_blocker
        .as_ref()
        .map(|blocker| format!("Next blocker: {blocker}"))
        .or_else(|| {
            progress
                .items
                .iter()
                .find(|item| item.state == "blocked")
                .map(|item| format!("Next blocker: {}: {}", item.evidence, item.detail))
        })
}

#[cfg(test)]
mod tests;
