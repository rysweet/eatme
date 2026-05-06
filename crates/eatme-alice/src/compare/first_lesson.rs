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
