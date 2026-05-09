use anyhow::{Error, Result, bail};
use eatme_alice::compare::{
    ContractDiagnostic, ContractEvidenceItem, LessonSessionContractCheck,
    LessonSessionReadinessReport, ReadinessEvidenceItem,
};
use eatme_alice::{check_lesson_session_contract, check_lesson_session_readiness};
use serde::Serialize;
use std::io::Write;
use std::path::Path;

pub fn print_lesson_session_check(manifest: &Path, json: bool) -> Result<()> {
    match check_lesson_session_contract(manifest) {
        Ok(report) => {
            write_lesson_session_check(std::io::stdout().lock(), json, &report)?;
            if !report.passed {
                bail!("lesson session contract check failed");
            }
            Ok(())
        }
        Err(error) => handle_manifest_error(
            manifest,
            json,
            error,
            "lesson session contract check failed",
        ),
    }
}

pub fn print_lesson_readiness_check(manifest: &Path, json: bool) -> Result<()> {
    match check_lesson_session_readiness(manifest) {
        Ok(report) => {
            write_lesson_readiness_check(std::io::stdout().lock(), json, &report)?;
            if !report.passed {
                bail!("lesson session readiness check failed");
            }
            Ok(())
        }
        Err(error) => handle_manifest_error(
            manifest,
            json,
            error,
            "lesson session readiness check failed",
        ),
    }
}

#[derive(Serialize)]
struct InvalidComparisonManifestReport {
    schema_version: &'static str,
    manifest_path: String,
    passed: bool,
    status: &'static str,
    readiness_status: &'static str,
    diagnostics: Vec<ContractDiagnostic>,
    contract_evidence: Vec<ContractEvidenceItem>,
    issues: Vec<String>,
}

#[derive(Clone, Copy)]
enum ComparisonManifestErrorKind {
    Unreadable,
    Malformed,
}

impl ComparisonManifestErrorKind {
    fn code(self) -> &'static str {
        match self {
            Self::Unreadable => "unreadable_comparison_manifest",
            Self::Malformed => "malformed_comparison_manifest",
        }
    }

    fn evidence_state(self) -> &'static str {
        match self {
            Self::Unreadable => "missing",
            Self::Malformed => "invalid",
        }
    }

    fn evidence_summary(self) -> &'static str {
        match self {
            Self::Unreadable => "comparison manifest must be readable",
            Self::Malformed => "comparison manifest must be valid JSON",
        }
    }

    fn diagnostic_message(self) -> &'static str {
        match self {
            Self::Unreadable => "comparison manifest is unreadable",
            Self::Malformed => "comparison manifest is malformed JSON",
        }
    }
}

fn handle_manifest_error(
    manifest: &Path,
    json: bool,
    error: Error,
    failure_message: &'static str,
) -> Result<()> {
    let Some(kind) = comparison_manifest_error_kind(&error).filter(|_| json) else {
        return Err(error);
    };
    print_json_result(&invalid_comparison_manifest_report(manifest, kind))?;
    bail!(failure_message);
}

fn invalid_comparison_manifest_report(
    manifest: &Path,
    kind: ComparisonManifestErrorKind,
) -> InvalidComparisonManifestReport {
    let issue = kind.diagnostic_message().to_string();
    InvalidComparisonManifestReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1",
        manifest_path: manifest.display().to_string(),
        passed: false,
        status: "not_ready",
        readiness_status: "incomplete",
        diagnostics: vec![ContractDiagnostic {
            code: kind.code().into(),
            severity: "error".into(),
            field: "manifest".into(),
            message: issue.clone(),
            expected: None,
        }],
        contract_evidence: vec![ContractEvidenceItem {
            id: "comparison_manifest".into(),
            state: kind.evidence_state().into(),
            required: true,
            summary: kind.evidence_summary().into(),
        }],
        issues: vec![issue],
    }
}

fn comparison_manifest_error_kind(error: &Error) -> Option<ComparisonManifestErrorKind> {
    for cause in error.chain() {
        let message = cause.to_string();
        if message.contains("reading comparison manifest") {
            return Some(ComparisonManifestErrorKind::Unreadable);
        }
        if message.contains("parsing comparison manifest") {
            return Some(ComparisonManifestErrorKind::Malformed);
        }
    }
    None
}

fn write_lesson_session_check(
    mut writer: impl Write,
    json: bool,
    report: &LessonSessionContractCheck,
) -> Result<()> {
    if json {
        return write_json_result(&mut writer, report);
    }

    writeln!(
        writer,
        "Lesson session contract check: {}",
        if report.passed { "passed" } else { "failed" }
    )?;
    writeln!(
        writer,
        "Manifest: {}",
        terminal_plain(&report.manifest_path)
    )?;
    if let Some(scenario_id) = &report.scenario_id {
        writeln!(writer, "Scenario: {}", terminal_plain(scenario_id))?;
    }
    if let Some(session_kind) = &report.session_kind {
        writeln!(writer, "Session kind: {}", terminal_plain(session_kind))?;
    }
    write_diagnostics(&mut writer, &report.diagnostics)?;
    write_issues(&mut writer, &report.issues)?;
    Ok(())
}

fn write_lesson_readiness_check(
    mut writer: impl Write,
    json: bool,
    report: &LessonSessionReadinessReport,
) -> Result<()> {
    if json {
        return write_json_result(&mut writer, report);
    }

    writeln!(
        writer,
        "Lesson readiness check: {} ({})",
        terminal_plain(&report.status),
        if report.passed { "passed" } else { "failed" }
    )?;
    writeln!(
        writer,
        "Manifest: {}",
        terminal_plain(&report.manifest_path)
    )?;
    if let Some(scenario_id) = &report.scenario_id {
        writeln!(writer, "Scenario: {}", terminal_plain(scenario_id))?;
    }
    writeln!(
        writer,
        "Readiness: {}",
        terminal_plain(&report.readiness_status)
    )?;
    writeln!(writer, "Summary: {}", terminal_plain(&report.human_summary))?;
    write_readiness_items(&mut writer, "Shown:", &report.shown_evidence)?;
    write_readiness_items(&mut writer, "Not yet shown:", &report.not_yet_shown)?;
    write_diagnostics(&mut writer, &report.diagnostics)?;
    write_issues(&mut writer, &report.issues)?;
    Ok(())
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

fn write_diagnostics(mut writer: impl Write, diagnostics: &[ContractDiagnostic]) -> Result<()> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    writeln!(writer, "Diagnostics:")?;
    for diagnostic in diagnostics {
        writeln!(
            writer,
            "- {} [{}]: {}",
            terminal_plain(&diagnostic.code),
            terminal_plain(&diagnostic.field),
            terminal_plain(&diagnostic.message)
        )?;
    }
    Ok(())
}

fn write_issues(mut writer: impl Write, issues: &[String]) -> Result<()> {
    if issues.is_empty() {
        return Ok(());
    }
    writeln!(writer, "Issues:")?;
    for issue in issues {
        writeln!(writer, "- {}", terminal_plain(issue))?;
    }
    Ok(())
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

fn print_json_result<T: Serialize>(value: &T) -> Result<()> {
    write_json_result(std::io::stdout().lock(), value)
}

fn write_json_result(mut writer: impl Write, value: &impl Serialize) -> Result<()> {
    writeln!(writer, "{}", serde_json::to_string_pretty(value)?)?;
    Ok(())
}
