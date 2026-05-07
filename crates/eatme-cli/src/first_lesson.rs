use anyhow::Result;
use clap::Args;
use eatme_alice::compare::{FirstLessonReadinessSequenceReport, LessonReadinessEvidenceProgress};
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
        "First-lesson readiness: {}",
        report.readiness_status
    )?;
    writeln!(
        writer,
        "Evidence progress: {}",
        report.evidence_progress.summary
    )?;
    if let Some(blocker) = next_actionable_blocker_line(&report.evidence_progress) {
        writeln!(writer, "{blocker}")?;
    }
    if let Some(proof) = &report.evidence_progress.next_missing_real_desktop_proof {
        writeln!(writer, "{proof}")?;
    }
    writeln!(
        writer,
        "Required evidence file status (present/missing/invalid/blocked; present is not proof of full UI automation):"
    )?;
    for item in &report.evidence_progress.items {
        writeln!(
            writer,
            "- {}: {} ({})",
            item.state, item.evidence, item.detail
        )?;
    }
    if !report.limitations.is_empty() {
        writeln!(writer, "Limits:")?;
        for limitation in &report.limitations {
            writeln!(writer, "- {limitation}")?;
        }
    }
    if !report.issues.is_empty() {
        writeln!(writer, "Still missing or blocked:")?;
        for issue in &report.issues {
            writeln!(writer, "- {issue}")?;
        }
    }
    Ok(())
}

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
mod tests {
    use super::*;
    use eatme_alice::compare::{
        LessonReadinessEvidenceProgressItem, LessonSessionContractCheck,
        LessonSessionReadinessEnvelope, LessonSessionReadinessReport,
    };
    use std::collections::BTreeMap;

    #[test]
    fn plain_output_includes_next_actionable_blocker_line() {
        let report = sequence_report(progress_with_blocker(Some(
            "desktop Run pixel observation is blocked: fix next: run Alice with a non-headless graphics environment",
        )));

        let mut output = Vec::new();
        write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(
            "Next blocker: desktop Run pixel observation is blocked: fix next: run Alice with a non-headless graphics environment"
        ));
    }

    #[test]
    fn plain_output_includes_next_missing_real_desktop_proof_line() {
        let mut progress = progress_with_blocker(None);
        progress.next_missing_real_desktop_proof = Some(
            "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions.".into(),
        );
        let report = sequence_report(progress);

        let mut output = Vec::new();
        write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(
            "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions."
        ));
    }

    #[test]
    fn next_actionable_blocker_line_is_absent_without_blocker_detail() {
        let progress = progress_with_blocker(None);

        assert!(next_actionable_blocker_line(&progress).is_none());
    }

    #[test]
    fn plain_output_falls_back_to_first_blocked_evidence_item() {
        let progress = LessonReadinessEvidenceProgress {
            total_required: 1,
            present: 0,
            missing: 0,
            invalid: 0,
            not_observed: 0,
            blocked: 1,
            summary: "0 of 1 required evidence items are present; 0 missing, 0 invalid, 0 not observed, 1 blocked.".into(),
            next_actionable_blocker: None,
            next_missing_real_desktop_proof: None,
            items: vec![LessonReadinessEvidenceProgressItem {
                evidence: "modernized desktop run execution observation".into(),
                state: "blocked".into(),
                detail: "blocked: no supported Alice desktop automation can run the world yet".into(),
            }],
        };

        assert_eq!(
            next_actionable_blocker_line(&progress).as_deref(),
            Some(
                "Next blocker: modernized desktop run execution observation: blocked: no supported Alice desktop automation can run the world yet"
            )
        );
    }

    fn progress_with_blocker(blocker: Option<&str>) -> LessonReadinessEvidenceProgress {
        let missing = usize::from(blocker.is_none());
        let blocked = usize::from(blocker.is_some());
        LessonReadinessEvidenceProgress {
            total_required: 1,
            present: 0,
            missing,
            invalid: 0,
            not_observed: 0,
            blocked,
            summary: format!(
                "0 of 1 required evidence items are present; {missing} missing, 0 invalid, 0 not observed, {blocked} blocked."
            ),
            next_actionable_blocker: blocker.map(str::to_string),
            next_missing_real_desktop_proof: None,
            items: vec![LessonReadinessEvidenceProgressItem {
                evidence: "modernized desktop-run-pixel-observation.json status".into(),
                state: if blocker.is_some() {
                    "blocked"
                } else {
                    "missing"
                }
                .into(),
                detail: "pixel observation detail".into(),
            }],
        }
    }

    fn sequence_report(
        progress: LessonReadinessEvidenceProgress,
    ) -> FirstLessonReadinessSequenceReport {
        let envelope = LessonSessionReadinessEnvelope {
            scenario_id: Some("first-lessons-real-ui-actions".into()),
            role: "student".into(),
            status: "blocked".into(),
            blocked_reason: Some("blocked_until_ui_automation".into()),
            human_summary: "blocked".into(),
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
        };
        let readiness_report = LessonSessionReadinessReport {
            schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
            manifest_path: "comparison-manifest.json".into(),
            scenario_id: Some("first-lessons-real-ui-actions".into()),
            passed: false,
            status: "blocked".into(),
            readiness_status: "blocked_until_ui_automation".into(),
            blocked_reason: Some("blocked_until_ui_automation".into()),
            human_summary: "blocked".into(),
            evidence_progress: progress.clone(),
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
            lesson_session_readiness: envelope.clone(),
            role_readiness: vec![envelope],
            contract_check: LessonSessionContractCheck {
                schema_version: "eatme.alice-lesson-session-check/v1".into(),
                manifest_path: "comparison-manifest.json".into(),
                scenario_id: Some("first-lessons-real-ui-actions".into()),
                session_kind: Some("first_lesson_action_contract".into()),
                automation_status: Some("blocked".into()),
                passed: false,
                issues: Vec::new(),
            },
            execute_requested: Some(true),
            target_evidence: Vec::new(),
            issues: Vec::new(),
            limitations: Vec::new(),
        };
        FirstLessonReadinessSequenceReport {
            schema_version: "eatme.first-lesson-readiness-sequence/v1".into(),
            scenario_id: "first-lessons-real-ui-actions".into(),
            run_id: "test".into(),
            execute_requested: true,
            comparison_manifest_path: "comparison-manifest.json".into(),
            passed: false,
            status: "blocked".into(),
            readiness_status: "blocked_until_ui_automation".into(),
            blocked_reason: Some("blocked_until_ui_automation".into()),
            human_summary: "blocked".into(),
            evidence_progress: progress,
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
            role_readiness: Vec::new(),
            target_statuses: BTreeMap::new(),
            issues: Vec::new(),
            limitations: Vec::new(),
            readiness_report,
        }
    }
}
