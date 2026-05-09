use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

#[cfg(test)]
mod accessibility_rescue_camera_captions_tests;
#[cfg(test)]
mod assessment_boundary_tests;
#[cfg(test)]
mod creature_choreography_tests;
#[cfg(test)]
mod design_process_story_or_game_tests;
mod discovery;
mod gadugi;
#[cfg(test)]
mod live_studio_workshop_tests;
#[cfg(test)]
mod neighborhood_data_story_tests;
#[cfg(test)]
mod outside_in_alice_expansion_tests;
mod report;
#[cfg(test)]
mod repository_policy_tests;
mod schema;
#[cfg(test)]
mod setup_preflight_ready_to_create_tests;
#[cfg(test)]
mod save_reopen_export_evidence_handoff_tests;
#[cfg(test)]
mod starter_project_preflight_boundary_tests;
#[cfg(test)]
mod student_reflection_artifact_tests;
mod validation;

pub use gadugi::{generate_gadugi_adapter_yaml, generate_gadugi_adapters};
pub use report::{
    AssetValidationReport, GadugiAdapterGenerationReport, ScenarioAssetValidationReport,
};
pub use validation::{validate_persona_crew, validate_scenario_asset};

pub fn validate_assets(root: &Path) -> Result<AssetValidationReport> {
    let persona_path = root.join("assets/personas/alice-user-crew.yaml");
    let scenario_root = root.join("assets/scenarios");
    let mut scenario_asset_ids = BTreeSet::new();
    let mut scenario_errors = Vec::new();
    let mut scenario_warnings = Vec::new();
    let mut scenario_asset_count = 0;

    if !scenario_root.exists() {
        scenario_errors.push(format!(
            "{} must exist and contain scenario assets",
            scenario_root.display()
        ));
    } else if !scenario_root.is_dir() {
        scenario_errors.push(format!(
            "{} must be a directory containing scenario assets",
            scenario_root.display()
        ));
    } else {
        let scenario_paths = discovery::scenario_asset_paths(&scenario_root)?;
        let persona_index = validation::persona_reference_index(&persona_path)?;
        if scenario_paths.is_empty() {
            scenario_errors.push(format!(
                "{} must contain at least one .yaml or .yml scenario asset",
                scenario_root.display()
            ));
        }
        for scenario_path in scenario_paths {
            let scenario_report =
                validation::validate_scenario_asset_with_personas(&scenario_path, &persona_index)?;
            let is_eatme_asset = scenario_report.asset_kind == "eatme";
            if is_eatme_asset {
                scenario_asset_ids.insert(scenario_report.id.clone());
            }
            scenario_asset_count += 1;
            scenario_errors.extend(
                scenario_report
                    .errors
                    .into_iter()
                    .map(|error| format!("{}: {error}", scenario_path.display())),
            );
            scenario_warnings.extend(
                scenario_report
                    .warnings
                    .into_iter()
                    .map(|warning| format!("{}: {warning}", scenario_path.display())),
            );
        }
    }

    let mut report = validation::validate_persona_crew_against_scenario_assets(
        &persona_path,
        Some(&scenario_asset_ids),
    )?;
    report.schema_version = "eatme.assets/validation/v1".into();
    report.asset_path = root.display().to_string();
    report.scenario_asset_count = scenario_asset_count;

    report.errors.extend(scenario_errors);
    report.warnings.extend(scenario_warnings);

    report.passed = report.errors.is_empty();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn validates_committed_persona_crew_asset() {
        let asset = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/personas/alice-user-crew.yaml");
        let report = validate_persona_crew(&asset).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert_eq!(report.instructor_count, 11);
        assert_eq!(report.student_count, 13);
        assert_eq!(report.core_scenario_count, 26);
        assert_eq!(report.creative_scenario_count, 12);
    }

    #[test]
    fn validates_committed_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = validate_assets(&root).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert_eq!(report.scenario_asset_count, 95);
    }

    #[test]
    fn validates_committed_lesson_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for asset in [
            "assets/scenarios/eatme/building-a-scene-first-world.yaml",
            "assets/scenarios/gadugi/building-a-scene-first-world.yaml",
            "assets/scenarios/eatme/code-editor-first-run.yaml",
            "assets/scenarios/gadugi/code-editor-first-run.yaml",
            "assets/scenarios/eatme/first-lessons-real-ui-actions.yaml",
            "assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml",
            "assets/scenarios/eatme/game-score-timer-win-lose-loop.yaml",
            "assets/scenarios/gadugi/game-score-timer-win-lose-loop.yaml",
            "assets/scenarios/eatme/modified-class-portability.yaml",
            "assets/scenarios/gadugi/modified-class-portability.yaml",
            "assets/scenarios/eatme/vr-camera-locomotion-journey.yaml",
            "assets/scenarios/gadugi/vr-camera-locomotion-journey.yaml",
            "assets/scenarios/eatme/hour-of-code-studio-kickoff.yaml",
            "assets/scenarios/gadugi/hour-of-code-studio-kickoff.yaml",
            "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/eatme/variables-scorekeeper-timekeeper.yaml",
            "assets/scenarios/gadugi/variables-scorekeeper-timekeeper.yaml",
            "assets/scenarios/eatme/arrays-collection-choreography.yaml",
            "assets/scenarios/gadugi/arrays-collection-choreography.yaml",
            "assets/scenarios/eatme/mythic-choice-event-tree.yaml",
            "assets/scenarios/gadugi/mythic-choice-event-tree.yaml",
            "assets/scenarios/eatme/vr-camera-perspective-tour.yaml",
            "assets/scenarios/gadugi/vr-camera-perspective-tour.yaml",
        ] {
            let report = validate_scenario_asset(&root.join(asset)).unwrap();
            assert!(report.passed, "{asset}: {:?}", report.errors);
        }
    }

    #[test]
    fn rejects_missing_scenario_root() {
        let root = scratch_root("missing-scenario-root");
        copy_committed_persona_asset(&root);

        let report = validate_assets(&root).unwrap();

        assert!(!report.passed);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("assets/scenarios") && error.contains("must exist")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn rejects_empty_scenario_root() {
        let root = scratch_root("empty-scenario-root");
        copy_committed_persona_asset(&root);
        fs::create_dir_all(root.join("assets/scenarios")).unwrap();

        let report = validate_assets(&root).unwrap();

        assert!(!report.passed);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error
                    .contains("must contain at least one .yaml or .yml scenario asset")),
            "{:?}",
            report.errors
        );
    }

    fn scratch_root(name: &str) -> std::path::PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/eatme-assets-tests")
            .join(format!("{name}-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn copy_committed_persona_asset(root: &Path) {
        let target = root.join("assets/personas/alice-user-crew.yaml");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/personas/alice-user-crew.yaml"),
            target,
        )
        .unwrap();
    }

    #[test]
    fn committed_gadugi_adapters_are_generated_and_fresh() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = generate_gadugi_adapters(&root, true).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert!(report.checked_count >= 2);
    }
}
