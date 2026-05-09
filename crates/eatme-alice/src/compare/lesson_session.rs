use crate::scenario::LaunchSmokeScenario;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::lesson_readiness::{
    ContractDiagnostic, ContractEvidenceItem, error_diagnostic, evidence_item,
};

const FIRST_LESSON_REQUIRED_STEPS: &[&str] = &[
    "instructor selects an Alice lesson objective and starter project",
    "student opens the configured starter project in Alice",
    "student places or modifies an object in the scene",
    "student edits a procedure or code block",
    "student runs the world and observes the visible result",
    "student saves the project and records one next revision",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LessonSessionComparisonContract {
    pub schema_version: String,
    pub scenario_id: String,
    pub session_kind: String,
    pub automation_status: String,
    pub actor_roles: Vec<String>,
    pub required_session_steps: Vec<String>,
    pub executable_evidence: Vec<String>,
    pub boundaries: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionContractCheck {
    pub schema_version: String,
    pub manifest_path: String,
    pub scenario_id: Option<String>,
    pub session_kind: Option<String>,
    pub automation_status: Option<String>,
    pub passed: bool,
    pub diagnostics: Vec<ContractDiagnostic>,
    pub contract_evidence: Vec<ContractEvidenceItem>,
    pub issues: Vec<String>,
}

pub fn check_lesson_session_contract(manifest_path: &Path) -> Result<LessonSessionContractCheck> {
    let text = fs::read_to_string(manifest_path)
        .with_context(|| format!("reading comparison manifest {}", manifest_path.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing comparison manifest {}", manifest_path.display()))?;
    check_lesson_session_contract_in_manifest(manifest_path, &manifest)
}

pub(super) fn check_lesson_session_contract_in_manifest(
    manifest_path: &Path,
    manifest: &serde_json::Value,
) -> Result<LessonSessionContractCheck> {
    let manifest_path_display = manifest_path.display().to_string();
    let manifest_scenario_id = manifest
        .get("scenario_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let mut issues = Vec::new();
    let Some(contract_value) = manifest.get("lesson_session_contract") else {
        let issues = vec!["comparison manifest is missing lesson_session_contract".into()];
        return Ok(LessonSessionContractCheck {
            schema_version: "eatme.alice-lesson-session-check/v1".into(),
            manifest_path: manifest_path_display,
            scenario_id: manifest_scenario_id,
            session_kind: None,
            automation_status: None,
            passed: false,
            diagnostics: lesson_session_diagnostics(&issues),
            contract_evidence: lesson_session_contract_evidence(false, &issues),
            issues,
        });
    };
    if !contract_value.is_object() {
        issues.push("lesson_session_contract must be an object".into());
    }
    let schema_version = string_contract_field(contract_value, "schema_version", &mut issues);
    let contract_scenario_id = string_contract_field(contract_value, "scenario_id", &mut issues);
    let session_kind = string_contract_field(contract_value, "session_kind", &mut issues);
    let automation_status = string_contract_field(contract_value, "automation_status", &mut issues);
    let actor_roles = string_vec_contract_field(contract_value, "actor_roles", &mut issues);
    let required_session_steps =
        string_vec_contract_field(contract_value, "required_session_steps", &mut issues);
    let executable_evidence =
        string_vec_contract_field(contract_value, "executable_evidence", &mut issues);
    let boundaries = string_vec_contract_field(contract_value, "boundaries", &mut issues);

    if schema_version.as_deref() != Some("eatme.alice-lesson-session-contract/v1") {
        issues.push(format!(
            "unsupported lesson_session_contract schema_version {:?}",
            schema_version.as_deref().unwrap_or("")
        ));
    }
    if contract_scenario_id.as_ref() != manifest_scenario_id.as_ref() {
        issues
            .push("lesson_session_contract scenario_id does not match manifest scenario_id".into());
    }
    require_non_empty(&mut issues, "actor_roles", &actor_roles);
    require_non_empty(
        &mut issues,
        "required_session_steps",
        &required_session_steps,
    );
    require_non_empty(&mut issues, "executable_evidence", &executable_evidence);
    require_non_empty(&mut issues, "boundaries", &boundaries);
    if session_kind.as_deref() == Some("first_lesson_action_contract") {
        require_entries(
            &mut issues,
            "required_session_steps",
            &required_session_steps,
            FIRST_LESSON_REQUIRED_STEPS,
        );
        require_fragments(
            &mut issues,
            "executable_evidence",
            &executable_evidence,
            &["automation scenarios"],
        );
    }
    require_fragments(
        &mut issues,
        "boundaries",
        &boundaries,
        &[
            "does not automate complete instructor assignment creation",
            "does not automate complete student lesson consumption",
            "does not perform creative assessment",
            "does not grade student worlds",
        ],
    );

    Ok(LessonSessionContractCheck {
        schema_version: "eatme.alice-lesson-session-check/v1".into(),
        manifest_path: manifest_path_display,
        scenario_id: contract_scenario_id,
        session_kind,
        automation_status,
        passed: issues.is_empty(),
        diagnostics: lesson_session_diagnostics(&issues),
        contract_evidence: lesson_session_contract_evidence(true, &issues),
        issues,
    })
}

pub(super) fn lesson_session_contract(
    scenario: &LaunchSmokeScenario,
) -> LessonSessionComparisonContract {
    if scenario.requires_real_ui_actions() {
        return LessonSessionComparisonContract {
            schema_version: "eatme.alice-lesson-session-contract/v1".into(),
            scenario_id: scenario.id.clone(),
            session_kind: "first_lesson_action_contract".into(),
            automation_status: "action_contract_blocked_until_ui_automation".into(),
            actor_roles: vec![
                "instructor prepares the Alice classroom task".into(),
                "student opens, changes, runs, saves, and reflects on an Alice project".into(),
            ],
            required_session_steps: FIRST_LESSON_REQUIRED_STEPS
                .iter()
                .map(|step| (*step).into())
                .collect(),
            executable_evidence: vec![
                "comparison manifest records both target runs under the same scenario id".into(),
                "target launch manifests record dependency, package, display, window, screenshot, log, and assertion evidence".into(),
                "automation scenarios name the required actions that are not automated yet".into(),
            ],
            boundaries: shared_boundaries(),
        };
    }

    let (session_kind, automation_status) = if scenario.id == "real-alice-launch-smoke" {
        ("launch_readiness", "launch_smoke_only")
    } else {
        (
            "lesson_labeled_launch_smoke",
            "lesson_labeled_launch_without_action_automation",
        )
    };

    LessonSessionComparisonContract {
        schema_version: "eatme.alice-lesson-session-contract/v1".into(),
        scenario_id: scenario.id.clone(),
        session_kind: session_kind.into(),
        automation_status: automation_status.into(),
        actor_roles: vec![
            "instructor uses target readiness evidence before class".into(),
            "student work remains outside automated assessment for this scenario".into(),
        ],
        required_session_steps: vec![
            "resolve and prepare both Alice targets".into(),
            "package each target when execution is requested".into(),
            "launch each target under an isolated virtual display".into(),
            "capture manifest, window, screenshot, log, assertion, and timing evidence".into(),
        ],
        executable_evidence: vec![
            "comparison manifest records target metadata, status, scorecard, timing, and differences".into(),
            "target launch manifests are attached when execution is requested and reaches launch smoke".into(),
        ],
        boundaries: shared_boundaries(),
    }
}

fn shared_boundaries() -> Vec<String> {
    vec![
        "does not automate complete instructor assignment creation".into(),
        "does not automate complete student lesson consumption".into(),
        "does not perform creative assessment".into(),
        "does not grade student worlds".into(),
        "does not prove broad Alice compatibility beyond the selected scenario".into(),
    ]
}

fn require_non_empty(issues: &mut Vec<String>, field: &str, entries: &[String]) {
    if entries.is_empty() {
        issues.push(format!("lesson_session_contract {field} must not be empty"));
    }
}

fn require_fragments(
    issues: &mut Vec<String>,
    field: &str,
    entries: &[String],
    fragments: &[&str],
) {
    for fragment in fragments {
        if !entries.iter().any(|entry| entry.contains(fragment)) {
            issues.push(format!(
                "lesson_session_contract {field} must include {fragment:?}"
            ));
        }
    }
}

fn require_entries(issues: &mut Vec<String>, field: &str, entries: &[String], required: &[&str]) {
    for expected in required {
        if !entries.iter().any(|entry| entry == expected) {
            issues.push(format!(
                "lesson_session_contract {field} must include exact entry {expected:?}"
            ));
        }
    }
}

fn string_contract_field(
    contract: &serde_json::Value,
    field: &str,
    issues: &mut Vec<String>,
) -> Option<String> {
    match contract.get(field).and_then(serde_json::Value::as_str) {
        Some(value) => Some(value.to_string()),
        None => {
            issues.push(format!("lesson_session_contract {field} must be a string"));
            None
        }
    }
}

fn string_vec_contract_field(
    contract: &serde_json::Value,
    field: &str,
    issues: &mut Vec<String>,
) -> Vec<String> {
    let Some(entries) = contract.get(field).and_then(serde_json::Value::as_array) else {
        issues.push(format!(
            "lesson_session_contract {field} must be a list of strings"
        ));
        return Vec::new();
    };
    let mut values = Vec::new();
    for entry in entries {
        match entry.as_str() {
            Some(value) => values.push(value.to_string()),
            None => issues.push(format!(
                "lesson_session_contract {field} entries must be strings"
            )),
        }
    }
    values
}

fn lesson_session_diagnostics(issues: &[String]) -> Vec<ContractDiagnostic> {
    issues
        .iter()
        .map(|issue| {
            if issue.contains("missing lesson_session_contract") {
                return error_diagnostic(
                    "missing_lesson_session_contract",
                    "lesson_session_contract",
                    issue.clone(),
                );
            }
            let field = lesson_session_issue_field(issue);
            error_diagnostic("invalid_lesson_session_contract", field, issue.clone())
        })
        .collect()
}

fn lesson_session_contract_evidence(
    contract_present: bool,
    issues: &[String],
) -> Vec<ContractEvidenceItem> {
    if !contract_present {
        return vec![evidence_item(
            "lesson_session_contract",
            "missing",
            "comparison manifest includes lesson_session_contract",
        )];
    }

    [
        ("lesson_session_contract.schema_version", "schema_version"),
        ("lesson_session_contract.scenario_id", "scenario_id"),
        (
            "lesson_session_contract.required_session_steps",
            "required_session_steps",
        ),
        (
            "lesson_session_contract.executable_evidence",
            "executable_evidence",
        ),
        ("lesson_session_contract.boundaries", "boundaries"),
    ]
    .into_iter()
    .map(|(id, field)| {
        evidence_item(
            id,
            if issues.iter().any(|issue| issue.contains(field)) {
                "invalid"
            } else {
                "present"
            },
            format!("{field} satisfies the lesson session contract"),
        )
    })
    .collect()
}

fn lesson_session_issue_field(issue: &str) -> String {
    for field in [
        "schema_version",
        "scenario_id",
        "session_kind",
        "automation_status",
        "actor_roles",
        "required_session_steps",
        "executable_evidence",
        "boundaries",
    ] {
        if issue.contains(field) {
            return format!("lesson_session_contract.{field}");
        }
    }
    "lesson_session_contract".into()
}
