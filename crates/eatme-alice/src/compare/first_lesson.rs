use super::{
    AliceComparisonOptions, ComparisonTargetRun, LessonSessionReadinessReport,
    check_lesson_session_readiness, run_launch_smoke_comparison,
};
use crate::scenario::LaunchSmokeScenario;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const FIRST_LESSON_SCENARIO_ID: &str = "first-lessons-real-ui-actions";

#[derive(Clone, Debug)]
pub struct FirstLessonReadinessOptions {
    pub registry_path: PathBuf,
    pub baseline_target: String,
    pub modernized_target: String,
    pub baseline_home_override: Option<PathBuf>,
    pub modernized_home_override: Option<PathBuf>,
    pub run_id: String,
    pub runs_dir: PathBuf,
    pub timeout_seconds: u64,
    pub json: bool,
    pub no_memory: bool,
    pub offline_package: bool,
    pub execute: bool,
    pub starter_project: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FirstLessonReadinessSequenceReport {
    pub schema_version: String,
    pub scenario_id: String,
    pub run_id: String,
    pub execute_requested: bool,
    pub comparison_manifest_path: String,
    pub passed: bool,
    pub status: String,
    pub readiness_status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub desktop_proof_contract: super::DesktopProofContract,
    pub shown_evidence: Vec<super::ReadinessEvidenceItem>,
    pub not_yet_shown: Vec<super::ReadinessEvidenceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_next_action: Option<super::DesktopNextActionSummary>,
    pub original_alice_action_evidence: super::OriginalAliceActionEvidenceReport,
    pub unproven_claims: Vec<String>,
    pub evidence_progress: super::LessonReadinessEvidenceProgress,
    pub evidence_boundaries: Vec<super::FirstLessonEvidenceBoundary>,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<super::LessonSessionNoGoContract>,
    pub role_readiness: Vec<super::LessonSessionReadinessEnvelope>,
    pub target_statuses: BTreeMap<String, FirstLessonTargetStatus>,
    pub issues: Vec<String>,
    pub limitations: Vec<String>,
    pub readiness_report: LessonSessionReadinessReport,
}

#[derive(Clone, Debug, Serialize)]
pub struct FirstLessonTargetStatus {
    pub target_id: String,
    pub status: String,
    pub failure_category: Option<String>,
    pub launch_manifest_present: bool,
    pub ui_action_contract_path: Option<String>,
}

pub fn run_first_lesson_readiness_sequence(
    options: &FirstLessonReadinessOptions,
) -> Result<FirstLessonReadinessSequenceReport> {
    let mut scenario = LaunchSmokeScenario::new(FIRST_LESSON_SCENARIO_ID);
    if let Some(starter_project) = &options.starter_project {
        scenario = scenario.with_starter_project(starter_project.clone());
    }

    let comparison = run_launch_smoke_comparison(&AliceComparisonOptions {
        registry_path: options.registry_path.clone(),
        baseline_target: options.baseline_target.clone(),
        modernized_target: options.modernized_target.clone(),
        baseline_home_override: options.baseline_home_override.clone(),
        modernized_home_override: options.modernized_home_override.clone(),
        scenario,
        run_id: options.run_id.clone(),
        runs_dir: options.runs_dir.clone(),
        timeout_seconds: options.timeout_seconds,
        json: options.json,
        no_memory: options.no_memory,
        offline_package: options.offline_package,
        execute: options.execute,
    })?;

    let readiness =
        check_lesson_session_readiness(&PathBuf::from(&comparison.comparison_manifest_path))?;
    let target_statuses = comparison
        .targets
        .iter()
        .map(|(role, target)| (role.clone(), target_status(target)))
        .collect();

    Ok(FirstLessonReadinessSequenceReport {
        schema_version: "eatme.first-lesson-readiness-sequence/v1".into(),
        scenario_id: FIRST_LESSON_SCENARIO_ID.into(),
        run_id: options.run_id.clone(),
        execute_requested: options.execute,
        comparison_manifest_path: comparison.comparison_manifest_path,
        passed: readiness.passed,
        status: readiness.status.clone(),
        readiness_status: readiness.readiness_status.clone(),
        blocked_reason: readiness.blocked_reason.clone(),
        human_summary: readiness.human_summary.clone(),
        desktop_proof_contract: readiness.desktop_proof_contract.clone(),
        shown_evidence: readiness.shown_evidence.clone(),
        not_yet_shown: readiness.not_yet_shown.clone(),
        desktop_next_action: readiness.desktop_next_action.clone(),
        original_alice_action_evidence: readiness.original_alice_action_evidence.clone(),
        unproven_claims: readiness.unproven_claims.clone(),
        evidence_progress: readiness.evidence_progress.clone(),
        evidence_boundaries: readiness.evidence_boundaries.clone(),
        required_evidence: readiness.required_evidence.clone(),
        no_go_contracts: readiness.no_go_contracts.clone(),
        role_readiness: readiness.role_readiness.clone(),
        target_statuses,
        issues: readiness.issues.clone(),
        limitations: readiness.limitations.clone(),
        readiness_report: readiness,
    })
}

fn target_status(target: &ComparisonTargetRun) -> FirstLessonTargetStatus {
    FirstLessonTargetStatus {
        target_id: target.target_id.clone(),
        status: target.status.clone(),
        failure_category: target.failure_category.clone(),
        launch_manifest_present: target.launch_manifest.is_some(),
        ui_action_contract_path: target
            .launch_manifest
            .as_ref()
            .and_then(|manifest| manifest.ui_action_contract.as_ref())
            .map(|artifact| artifact.path.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::{
        DesktopProofContract, LessonReadinessEvidenceProgress, LessonSessionContractCheck,
        LessonSessionReadinessEnvelope, OriginalAliceActionEvidenceReport,
        OriginalAliceActionEvidenceStatus,
    };

    #[test]
    fn first_lesson_sequence_serializes_missing_original_alice_action_evidence_at_top_level() {
        let readiness_report = readiness_report_with_missing_original_alice_action_evidence();
        let report = FirstLessonReadinessSequenceReport {
            schema_version: "eatme.first-lesson-readiness-sequence/v1".into(),
            scenario_id: FIRST_LESSON_SCENARIO_ID.into(),
            run_id: "test".into(),
            execute_requested: true,
            comparison_manifest_path: readiness_report.manifest_path.clone(),
            passed: readiness_report.passed,
            status: readiness_report.status.clone(),
            readiness_status: readiness_report.readiness_status.clone(),
            blocked_reason: readiness_report.blocked_reason.clone(),
            human_summary: readiness_report.human_summary.clone(),
            desktop_proof_contract: readiness_report.desktop_proof_contract.clone(),
            shown_evidence: readiness_report.shown_evidence.clone(),
            not_yet_shown: readiness_report.not_yet_shown.clone(),
            desktop_next_action: readiness_report.desktop_next_action.clone(),
            original_alice_action_evidence: readiness_report.original_alice_action_evidence.clone(),
            unproven_claims: readiness_report.unproven_claims.clone(),
            evidence_progress: readiness_report.evidence_progress.clone(),
            evidence_boundaries: readiness_report.evidence_boundaries.clone(),
            required_evidence: readiness_report.required_evidence.clone(),
            no_go_contracts: readiness_report.no_go_contracts.clone(),
            role_readiness: readiness_report.role_readiness.clone(),
            target_statuses: BTreeMap::new(),
            issues: readiness_report.issues.clone(),
            limitations: readiness_report.limitations.clone(),
            readiness_report,
        };

        let report_json = serde_json::to_value(&report).unwrap();

        assert_eq!(
            report_json["original_alice_action_evidence"]["status"],
            "missing"
        );
        assert_eq!(
            report_json["original_alice_action_evidence"],
            report_json["readiness_report"]["original_alice_action_evidence"],
            "sequence JSON must preserve the underlying readiness original Alice action evidence state"
        );
    }

    fn readiness_report_with_missing_original_alice_action_evidence() -> LessonSessionReadinessReport
    {
        let envelope = LessonSessionReadinessEnvelope {
            scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
            role: "student".into(),
            status: "blocked".into(),
            blocked_reason: Some("blocked_until_ui_automation".into()),
            human_summary: "blocked".into(),
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
        };
        LessonSessionReadinessReport {
            schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
            manifest_path: "comparison-manifest.json".into(),
            scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
            passed: true,
            status: "blocked".into(),
            readiness_status: "blocked_until_ui_automation".into(),
            blocked_reason: Some("blocked_until_ui_automation".into()),
            human_summary: "blocked".into(),
            desktop_proof_contract: DesktopProofContract {
                status: "launched_but_unverified".into(),
                reason_code: "desktop_pixel_observation_blocked".into(),
                detail: "desktop proof is not verified".into(),
                target_role: "modernized".into(),
                artifact: None,
            },
            shown_evidence: Vec::new(),
            not_yet_shown: Vec::new(),
            desktop_next_action: None,
            original_alice_action_evidence: OriginalAliceActionEvidenceReport {
                status: OriginalAliceActionEvidenceStatus::Missing,
                summary: "Original Alice action evidence is missing.".into(),
                detail: "Original Alice action evidence was not found in the comparison target evidence."
                    .into(),
            },
            unproven_claims: Vec::new(),
            evidence_progress: LessonReadinessEvidenceProgress {
                total_required: 0,
                present: 0,
                missing: 0,
                invalid: 0,
                not_observed: 0,
                blocked: 0,
                summary: "0 of 0 required evidence items are present; 0 missing, 0 invalid, 0 not observed, 0 blocked.".into(),
                next_actionable_blocker: None,
                next_missing_real_desktop_proof: None,
                items: Vec::new(),
            },
            evidence_boundaries: Vec::new(),
            required_evidence: Vec::new(),
            no_go_contracts: Vec::new(),
            lesson_session_readiness: envelope.clone(),
            role_readiness: vec![envelope],
            contract_check: LessonSessionContractCheck {
                schema_version: "eatme.alice-lesson-session-check/v1".into(),
                manifest_path: "comparison-manifest.json".into(),
                scenario_id: Some(FIRST_LESSON_SCENARIO_ID.into()),
                session_kind: Some("first_lesson_action_contract".into()),
                automation_status: Some("blocked".into()),
                passed: true,
                issues: Vec::new(),
            },
            execute_requested: Some(true),
            target_evidence: Vec::new(),
            issues: Vec::new(),
            limitations: Vec::new(),
        }
    }
}
