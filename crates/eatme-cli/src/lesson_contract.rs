use anyhow::{Error, Result, bail};
use eatme_alice::compare::{ContractDiagnostic, ContractEvidenceItem};
use eatme_alice::{check_lesson_session_contract, check_lesson_session_readiness};
use serde::Serialize;
use std::path::Path;

pub fn print_lesson_session_check(manifest: &Path, json: bool) -> Result<()> {
    match check_lesson_session_contract(manifest) {
        Ok(report) => {
            print_result(&report)?;
            if !report.passed {
                bail!("lesson session contract check failed");
            }
            Ok(())
        }
        Err(error) if json && is_comparison_manifest_parse_error(&error) => {
            print_result(&malformed_comparison_manifest_report(manifest, &error))?;
            bail!("lesson session contract check failed");
        }
        Err(error) => Err(error),
    }
}

pub fn print_lesson_readiness_check(manifest: &Path, json: bool) -> Result<()> {
    match check_lesson_session_readiness(manifest) {
        Ok(report) => {
            print_result(&report)?;
            if !report.passed {
                bail!("lesson session readiness check failed");
            }
            Ok(())
        }
        Err(error) if json && is_comparison_manifest_parse_error(&error) => {
            print_result(&malformed_comparison_manifest_report(manifest, &error))?;
            bail!("lesson session readiness check failed");
        }
        Err(error) => Err(error),
    }
}

#[derive(Serialize)]
struct MalformedComparisonManifestReport {
    schema_version: &'static str,
    manifest_path: String,
    passed: bool,
    status: &'static str,
    readiness_status: &'static str,
    diagnostics: Vec<ContractDiagnostic>,
    contract_evidence: Vec<ContractEvidenceItem>,
    issues: Vec<String>,
}

fn malformed_comparison_manifest_report(
    manifest: &Path,
    error: &Error,
) -> MalformedComparisonManifestReport {
    let issue = error.to_string();
    MalformedComparisonManifestReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1",
        manifest_path: manifest.display().to_string(),
        passed: false,
        status: "not_ready",
        readiness_status: "incomplete",
        diagnostics: vec![ContractDiagnostic {
            code: "malformed_comparison_manifest".into(),
            severity: "error".into(),
            field: "manifest".into(),
            message: issue.clone(),
            expected: None,
        }],
        contract_evidence: vec![ContractEvidenceItem {
            id: "comparison_manifest".into(),
            state: "invalid".into(),
            required: true,
            summary: "comparison manifest must be valid JSON".into(),
        }],
        issues: vec![issue],
    }
}

fn is_comparison_manifest_parse_error(error: &Error) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains("parsing comparison manifest"))
}

fn print_result<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
