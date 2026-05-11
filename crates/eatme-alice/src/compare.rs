use crate::launch_artifacts::artifact_info;
use crate::scenario::LaunchSmokeScenario;
use crate::{LaunchSmokeOptions, run_launch_smoke};
use anyhow::{Context, Result, bail};
use eatme_core::{ArtifactInfo, AssertionResult, LaunchSmokeManifest};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
mod contract;
mod desktop_evidence;
mod first_lesson;
pub mod grading_report;
pub use grading_report::*;
mod lesson_readiness;
mod lesson_session;
mod scorecard;
mod ui_action_contract;
pub use first_lesson::*;
pub use lesson_readiness::*;
pub use lesson_session::*;
pub use scorecard::ComparisonScorecard;
use scorecard::build_scorecard;
pub use {contract::ComparisonContract, desktop_evidence::FirstLessonEvidenceBoundary};
use {contract::comparison_contract, lesson_session::lesson_session_contract};
#[derive(Clone, Debug)]
pub struct AliceComparisonOptions {
    pub registry_path: PathBuf,
    pub baseline_target: String,
    pub modernized_target: String,
    pub baseline_home_override: Option<PathBuf>,
    pub modernized_home_override: Option<PathBuf>,
    pub scenario: LaunchSmokeScenario,
    pub run_id: String,
    pub runs_dir: PathBuf,
    pub timeout_seconds: u64,
    pub json: bool,
    pub no_memory: bool,
    pub offline_package: bool,
    pub execute: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AliceTargetRegistry {
    pub schema_version: String,
    pub targets: BTreeMap<String, AliceTargetDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AliceTargetDefinition {
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub alice_home: Option<PathBuf>,
    #[serde(default)]
    pub alice_home_env: Option<String>,
    #[serde(default)]
    pub required_paths: Vec<PathBuf>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AliceComparisonManifest {
    pub schema_version: String,
    pub comparison_contract: ComparisonContract,
    pub lesson_session_contract: LessonSessionComparisonContract,
    pub registry_path: String,
    pub scenario_id: String,
    pub run_id: String,
    pub execute_requested: bool,
    pub created_at_unix_ms: u128,
    pub started_at_unix_ms: u128,
    pub finished_at_unix_ms: u128,
    pub duration_ms: u128,
    pub comparison_manifest_path: String,
    pub targets: BTreeMap<String, ComparisonTargetRun>,
    pub scorecard: ComparisonScorecard,
    pub diff: ComparisonDiff,
}

#[derive(Clone, Debug, Serialize)]
pub struct ComparisonTargetRun {
    pub role: String,
    pub target_id: String,
    pub label: String,
    pub description: String,
    pub metadata: BTreeMap<String, String>,
    pub notes: Vec<String>,
    pub alice_home_env: Option<String>,
    pub required_paths: Vec<String>,
    pub resolved_alice_home: Option<String>,
    pub alice_home_source: Option<String>,
    pub run_id: String,
    pub started_at_unix_ms: u128,
    pub finished_at_unix_ms: u128,
    pub duration_ms: u128,
    pub status: String,
    pub detail: String,
    pub failure_category: Option<String>,
    pub launch_manifest: Option<LaunchSmokeManifest>,
    pub launch_manifest_artifact: Option<ArtifactInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ComparisonDiff {
    pub baseline_status: String,
    pub modernized_status: String,
    pub status_changed: bool,
    pub baseline_failure_category: Option<String>,
    pub modernized_failure_category: Option<String>,
    pub failure_category_changed: bool,
    pub assertion_diffs: Vec<AssertionDiff>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssertionDiff {
    pub assertion: String,
    pub baseline: Option<AssertionSnapshot>,
    pub modernized: Option<AssertionSnapshot>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssertionSnapshot {
    pub passed: bool,
    pub detail: String,
}

pub fn read_target_registry(path: &Path) -> Result<AliceTargetRegistry> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "reading Alice comparison target registry {}",
            path.display()
        )
    })?;
    let registry: AliceTargetRegistry = serde_yaml::from_str(&text).with_context(|| {
        format!(
            "parsing Alice comparison target registry {}",
            path.display()
        )
    })?;
    if registry.schema_version != "eatme.alice-comparison-targets/v1" {
        bail!(
            "unsupported Alice comparison target registry schema_version {:?}",
            registry.schema_version
        );
    }
    for (target_id, target) in &registry.targets {
        validate_id("target id", target_id)?;
        for required_path in &target.required_paths {
            validate_required_path(target_id, required_path)?;
        }
    }
    Ok(registry)
}

pub fn run_launch_smoke_comparison(
    options: &AliceComparisonOptions,
) -> Result<AliceComparisonManifest> {
    validate_id("scenario id", &options.scenario.id)?;
    validate_id("run id", &options.run_id)?;
    let registry = read_target_registry(&options.registry_path)?;
    let comparison_dir = options
        .runs_dir
        .join("comparisons")
        .join(&options.scenario.id)
        .join(&options.run_id);
    fs::create_dir_all(&comparison_dir)?;
    let comparison_path = comparison_dir.join("comparison-manifest.json");
    let started_at = now_ms();

    let baseline = run_target(
        "baseline",
        &options.baseline_target,
        options.baseline_home_override.as_ref(),
        &registry,
        options,
    )?;
    let modernized = run_target(
        "modernized",
        &options.modernized_target,
        options.modernized_home_override.as_ref(),
        &registry,
        options,
    )?;

    let mut targets = BTreeMap::new();
    targets.insert("baseline".into(), baseline);
    targets.insert("modernized".into(), modernized);
    let finished_at = now_ms();
    let diff = compare_status_and_assertions(&targets);
    let scorecard = build_scorecard(options.execute, &targets, &diff);
    let manifest = AliceComparisonManifest {
        schema_version: "eatme.alice-comparison/v1".into(),
        comparison_contract: comparison_contract(),
        lesson_session_contract: lesson_session_contract(&options.scenario),
        registry_path: options.registry_path.display().to_string(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        execute_requested: options.execute,
        created_at_unix_ms: started_at,
        started_at_unix_ms: started_at,
        finished_at_unix_ms: finished_at,
        duration_ms: finished_at.saturating_sub(started_at),
        comparison_manifest_path: comparison_path.display().to_string(),
        targets,
        scorecard,
        diff,
    };
    fs::write(&comparison_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(manifest)
}

fn run_target(
    role: &str,
    target_id: &str,
    home_override: Option<&PathBuf>,
    registry: &AliceTargetRegistry,
    options: &AliceComparisonOptions,
) -> Result<ComparisonTargetRun> {
    validate_id("target id", target_id)?;
    let target = registry
        .targets
        .get(target_id)
        .with_context(|| format!("Alice comparison target {target_id:?} is not in the registry"))?;
    let target_run_id = format!("{}-{role}-{target_id}", options.run_id);
    let started_at = now_ms();
    let home = resolve_alice_home(target, home_override);
    let mut run = ComparisonTargetRun {
        role: role.into(),
        target_id: target_id.into(),
        label: target.label.clone(),
        description: target.description.clone(),
        metadata: target.metadata.clone(),
        notes: target.notes.clone(),
        alice_home_env: target.alice_home_env.clone(),
        required_paths: target
            .required_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        resolved_alice_home: home.as_ref().map(|(path, _)| path.display().to_string()),
        alice_home_source: home.as_ref().map(|(_, source)| source.clone()),
        run_id: target_run_id.clone(),
        started_at_unix_ms: started_at,
        finished_at_unix_ms: started_at,
        duration_ms: 0,
        status: String::new(),
        detail: String::new(),
        failure_category: None,
        launch_manifest: None,
        launch_manifest_artifact: None,
    };

    if !options.execute {
        finish_run(
            &mut run,
            "not_run",
            "execution was not requested; rerun with --execute to invoke Alice launch smoke",
            None,
        );
        return Ok(run);
    }

    let Some((alice_home, _source)) = home else {
        finish_run(
            &mut run,
            "blocked",
            "target has no Alice home; set an override, registry alice_home, or configured environment variable",
            Some("alice_home_unresolved".into()),
        );
        return Ok(run);
    };

    let missing_paths = missing_required_paths(&alice_home, &target.required_paths);
    if !missing_paths.is_empty() {
        finish_run(
            &mut run,
            "blocked",
            format!(
                "target is missing required paths under Alice home: {}",
                missing_paths.join(", ")
            ),
            Some("target_required_path_missing".into()),
        );
        return Ok(run);
    }

    let target_options = LaunchSmokeOptions {
        alice_home,
        run_id: target_run_id,
        runs_dir: options.runs_dir.clone(),
        timeout_seconds: options.timeout_seconds,
        json: options.json,
        no_memory: options.no_memory,
        offline_package: options.offline_package,
        scenario: options.scenario.clone(),
    };
    match run_launch_smoke(&target_options) {
        Ok(launch_manifest) => {
            let manifest_path = options
                .runs_dir
                .join(&options.scenario.id)
                .join(&target_options.run_id)
                .join("manifest.json");
            let artifact = artifact_info(&manifest_path).ok();
            let failure_category = launch_manifest.failure_category.clone();
            let status = if failure_category.is_some() {
                "failed"
            } else {
                "passed"
            };
            run.launch_manifest = Some(launch_manifest);
            run.launch_manifest_artifact = artifact;
            finish_run(
                &mut run,
                status,
                "Alice launch smoke executed and recorded a target manifest",
                failure_category,
            );
        }
        Err(error) => finish_run(
            &mut run,
            "error",
            format!("Alice launch smoke invocation failed: {error:#}"),
            Some("launch_smoke_error".into()),
        ),
    }
    Ok(run)
}

fn resolve_alice_home(
    target: &AliceTargetDefinition,
    override_home: Option<&PathBuf>,
) -> Option<(PathBuf, String)> {
    if let Some(path) = override_home {
        return Some((path.clone(), "cli_override".into()));
    }
    if let Some(path) = &target.alice_home {
        return Some((path.clone(), "registry".into()));
    }
    target.alice_home_env.as_ref().and_then(|key| {
        env::var_os(key).and_then(|value| {
            if value.is_empty() {
                None
            } else {
                Some((PathBuf::from(value), format!("env:{key}")))
            }
        })
    })
}

fn missing_required_paths(alice_home: &Path, required_paths: &[PathBuf]) -> Vec<String> {
    required_paths
        .iter()
        .filter(|path| !alice_home.join(path).exists())
        .map(|path| path.display().to_string())
        .collect()
}

fn finish_run(
    run: &mut ComparisonTargetRun,
    status: impl Into<String>,
    detail: impl Into<String>,
    failure_category: Option<String>,
) {
    let finished_at = now_ms();
    run.finished_at_unix_ms = finished_at;
    run.duration_ms = finished_at.saturating_sub(run.started_at_unix_ms);
    run.status = status.into();
    run.detail = detail.into();
    run.failure_category = failure_category;
}

fn compare_status_and_assertions(
    targets: &BTreeMap<String, ComparisonTargetRun>,
) -> ComparisonDiff {
    let baseline = targets.get("baseline").expect("baseline target run exists");
    let modernized = targets
        .get("modernized")
        .expect("modernized target run exists");
    let baseline_assertions = baseline
        .launch_manifest
        .as_ref()
        .map(|manifest| &manifest.assertions);
    let modernized_assertions = modernized
        .launch_manifest
        .as_ref()
        .map(|manifest| &manifest.assertions);
    let mut assertion_names = BTreeSet::new();
    if let Some(assertions) = baseline_assertions {
        assertion_names.extend(assertions.keys().cloned());
    }
    if let Some(assertions) = modernized_assertions {
        assertion_names.extend(assertions.keys().cloned());
    }

    let assertion_diffs = assertion_names
        .into_iter()
        .filter_map(|assertion| {
            let baseline = baseline_assertions.and_then(|assertions| assertions.get(&assertion));
            let modernized =
                modernized_assertions.and_then(|assertions| assertions.get(&assertion));
            if assertion_changed(&assertion, baseline, modernized) {
                Some(AssertionDiff {
                    assertion,
                    baseline: baseline.map(assertion_snapshot),
                    modernized: modernized.map(assertion_snapshot),
                })
            } else {
                None
            }
        })
        .collect();

    ComparisonDiff {
        baseline_status: baseline.status.clone(),
        modernized_status: modernized.status.clone(),
        status_changed: baseline.status != modernized.status,
        baseline_failure_category: baseline.failure_category.clone(),
        modernized_failure_category: modernized.failure_category.clone(),
        failure_category_changed: baseline.failure_category != modernized.failure_category,
        assertion_diffs,
    }
}

fn assertion_changed(
    assertion_name: &str,
    baseline: Option<&AssertionResult>,
    modernized: Option<&AssertionResult>,
) -> bool {
    match (baseline, modernized) {
        (Some(left), Some(right)) => {
            left.passed != right.passed
                || normalized_assertion_detail(assertion_name, left)
                    != normalized_assertion_detail(assertion_name, right)
        }
        (None, None) => false,
        _ => true,
    }
}

fn normalized_assertion_detail<'a>(
    assertion_name: &str,
    assertion: &'a AssertionResult,
) -> &'a str {
    if assertion_name == "display_responsive" && assertion.passed {
        return "display responds to xdpyinfo";
    }
    &assertion.detail
}

fn assertion_snapshot(assertion: &AssertionResult) -> AssertionSnapshot {
    AssertionSnapshot {
        passed: assertion.passed,
        detail: assertion.detail.clone(),
    }
}

fn validate_id(label: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("{label} {value:?} must be kebab-case");
    }
    Ok(())
}

fn validate_required_path(target_id: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!(
            "target {target_id:?} required_paths entry {:?} must be a relative path inside Alice home",
            path
        );
    }
    Ok(())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
#[cfg(test)]
mod lesson_session_tests;
#[cfg(test)]
mod tests;
