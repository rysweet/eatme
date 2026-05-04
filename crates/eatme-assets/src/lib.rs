use anyhow::Result;
use std::path::Path;

mod discovery;
mod report;
mod schema;
mod validation;

pub use report::{AssetValidationReport, ScenarioAssetValidationReport};
pub use validation::{validate_persona_crew, validate_scenario_asset};

pub fn validate_assets(root: &Path) -> Result<AssetValidationReport> {
    let persona_path = root.join("assets/personas/alice-user-crew.yaml");
    let mut report = validate_persona_crew(&persona_path)?;
    let persona_index = validation::persona_reference_index(&persona_path)?;
    report.schema_version = "eatme.assets/validation/v1".into();
    report.asset_path = root.display().to_string();

    for scenario_path in discovery::scenario_asset_paths(&root.join("assets/scenarios"))? {
        let scenario_report =
            validation::validate_scenario_asset_with_personas(&scenario_path, &persona_index)?;
        report.scenario_asset_count += 1;
        report.errors.extend(
            scenario_report
                .errors
                .into_iter()
                .map(|error| format!("{}: {error}", scenario_path.display())),
        );
        report.warnings.extend(
            scenario_report
                .warnings
                .into_iter()
                .map(|warning| format!("{}: {warning}", scenario_path.display())),
        );
    }

    report.passed = report.errors.is_empty();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn validates_committed_persona_crew_asset() {
        let asset = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/personas/alice-user-crew.yaml");
        let report = validate_persona_crew(&asset).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert_eq!(report.instructor_count, 11);
        assert_eq!(report.student_count, 10);
        assert_eq!(report.core_scenario_count, 22);
        assert_eq!(report.creative_scenario_count, 11);
    }

    #[test]
    fn validates_committed_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = validate_assets(&root).unwrap();
        assert!(report.passed, "{:?}", report.errors);
        assert_eq!(report.scenario_asset_count, 34);
    }

    #[test]
    fn validates_committed_lesson_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for asset in [
            "assets/scenarios/eatme/building-a-scene-first-world.yaml",
            "assets/scenarios/gadugi/building-a-scene-first-world.yaml",
            "assets/scenarios/eatme/code-editor-first-run.yaml",
            "assets/scenarios/gadugi/code-editor-first-run.yaml",
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
        ] {
            let report = validate_scenario_asset(&root.join(asset)).unwrap();
            assert!(report.passed, "{asset}: {:?}", report.errors);
        }
    }
}
