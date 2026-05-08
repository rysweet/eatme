use anyhow::Result;
use clap::Args;
use eatme_alice::compare::{
    FirstLessonEvidenceBoundary, FirstLessonReadinessSequenceReport,
    LessonReadinessEvidenceProgress,
};
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
    writeln!(
        writer,
        "Evidence progress: {}",
        terminal_plain(&report.evidence_progress.summary)
    )?;
    if let Some(blocker) = next_actionable_blocker_line(&report.evidence_progress) {
        writeln!(writer, "{}", terminal_plain(&blocker))?;
    }
    if let Some(proof) = &report.evidence_progress.next_missing_real_desktop_proof {
        writeln!(writer, "{}", terminal_plain(proof))?;
    }
    writeln!(
        writer,
        "automation scenarios evidence (present/missing/invalid/blocked; present is bounded scenario evidence only):"
    )?;
    for boundary in &report.evidence_boundaries {
        writeln!(
            writer,
            "- {}: {} ({})",
            terminal_plain(&boundary.status),
            terminal_plain(&boundary.label),
            terminal_plain(&boundary.detail)
        )?;
    }
    if let Some(blockers) = scenario_blockers(&report.evidence_boundaries) {
        writeln!(writer, "Blockers:")?;
        for blocker in blockers {
            writeln!(writer, "- {}", terminal_plain(&blocker))?;
        }
    }
    if !report.limitations.is_empty() {
        writeln!(writer, "Limits:")?;
        for limitation in &report.limitations {
            writeln!(writer, "- {}", terminal_plain(limitation))?;
        }
    }
    if !report.issues.is_empty() {
        writeln!(writer, "Still missing or blocked:")?;
        for issue in &report.issues {
            writeln!(writer, "- {}", terminal_plain(issue))?;
        }
    }
    Ok(())
}

fn scenario_readiness_status(report: &FirstLessonReadinessSequenceReport) -> &str {
    if report.passed
        && !report.evidence_boundaries.is_empty()
        && report
            .evidence_boundaries
            .iter()
            .all(|boundary| boundary.status == "present")
    {
        "ready"
    } else {
        "not ready"
    }
}

fn scenario_blockers(boundaries: &[FirstLessonEvidenceBoundary]) -> Option<Vec<String>> {
    let blockers = boundaries
        .iter()
        .filter(|boundary| boundary.status != "present")
        .map(|boundary| boundary.detail.clone())
        .collect::<Vec<_>>();
    (!blockers.is_empty()).then_some(blockers)
}

fn terminal_plain(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_control() {
            text.extend(ch.escape_default());
        } else {
            text.push(ch);
        }
    }
    text
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
        DesktopProofContract, LessonReadinessEvidenceProgressItem, LessonSessionContractCheck,
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

    #[test]
    fn plain_output_escapes_control_characters_from_report_data() {
        let mut progress =
            progress_with_blocker(Some("blocked proof artifact\x1b[31m\nInjected line"));
        progress.items[0].detail = "pixel detail\x1b[0m\nInjected detail".into();
        let report = sequence_report(progress);

        let mut output = Vec::new();
        write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(
            !output.contains('\x1b'),
            "plain output must not contain raw terminal control characters: {output:?}"
        );
        assert!(
            !output.contains("\nInjected"),
            "plain output must not allow evidence text to inject extra lines: {output:?}"
        );
        assert!(output.contains("\\u{1b}"));
        assert!(output.contains("\\nInjected"));
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
            desktop_proof_contract: desktop_proof_contract(),
            evidence_progress: progress.clone(),
            evidence_boundaries: Vec::new(),
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
            desktop_proof_contract: desktop_proof_contract(),
            evidence_progress: progress,
            evidence_boundaries: Vec::new(),
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
            role_readiness: Vec::new(),
            target_statuses: BTreeMap::new(),
            issues: Vec::new(),
            limitations: Vec::new(),
            readiness_report,
        }
    }

    fn desktop_proof_contract() -> DesktopProofContract {
        DesktopProofContract {
            status: "launched_but_unverified".into(),
            reason_code: "desktop_pixel_observation_blocked".into(),
            detail: "desktop proof is not verified".into(),
            target_role: "modernized".into(),
            artifact: None,
        }
    }
}
