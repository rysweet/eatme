use crate::creative_assessment::{AssessmentCategory, for_building_a_scene};

#[test]
fn factory_returns_building_a_scene_lesson() {
    let report = for_building_a_scene();
    assert_eq!(report.lesson, "building-a-scene-first-world");
}

#[test]
fn machine_assessable_has_six_aspects() {
    let report = for_building_a_scene();
    assert_eq!(report.machine_assessable.len(), 6);
}

#[test]
fn human_review_needed_has_six_aspects() {
    let report = for_building_a_scene();
    assert_eq!(report.human_review_needed.len(), 6);
}

#[test]
fn total_aspects_is_twelve() {
    let report = for_building_a_scene();
    let total = report.machine_assessable.len() + report.human_review_needed.len();
    assert_eq!(total, 12);
}

#[test]
fn machine_assessable_categories_are_file_structure_and_runtime() {
    let report = for_building_a_scene();
    for aspect in &report.machine_assessable {
        assert!(
            aspect.category == AssessmentCategory::FileStructure
                || aspect.category == AssessmentCategory::RuntimeBehavior,
            "machine-assessable aspect '{}' has unexpected category {:?}",
            aspect.name,
            aspect.category
        );
    }
}

#[test]
fn human_review_categories_are_creative_and_learning() {
    let report = for_building_a_scene();
    for aspect in &report.human_review_needed {
        assert!(
            aspect.category == AssessmentCategory::CreativeExpression
                || aspect.category == AssessmentCategory::LearningEvidence,
            "human-review aspect '{}' has unexpected category {:?}",
            aspect.name,
            aspect.category
        );
    }
}

#[test]
fn all_four_categories_represented() {
    let report = for_building_a_scene();
    let all: Vec<_> = report
        .machine_assessable
        .iter()
        .chain(report.human_review_needed.iter())
        .collect();
    assert!(
        all.iter()
            .any(|a| a.category == AssessmentCategory::FileStructure)
    );
    assert!(
        all.iter()
            .any(|a| a.category == AssessmentCategory::RuntimeBehavior)
    );
    assert!(
        all.iter()
            .any(|a| a.category == AssessmentCategory::CreativeExpression)
    );
    assert!(
        all.iter()
            .any(|a| a.category == AssessmentCategory::LearningEvidence)
    );
}

#[test]
fn every_aspect_has_nonempty_name_and_rationale() {
    let report = for_building_a_scene();
    let all: Vec<_> = report
        .machine_assessable
        .iter()
        .chain(report.human_review_needed.iter())
        .collect();
    for aspect in all {
        assert!(!aspect.name.is_empty(), "aspect name must not be empty");
        assert!(
            !aspect.rationale.is_empty(),
            "aspect '{}' rationale must not be empty",
            aspect.name
        );
    }
}

#[test]
fn aspect_names_are_unique() {
    let report = for_building_a_scene();
    let names: Vec<&str> = report
        .machine_assessable
        .iter()
        .chain(report.human_review_needed.iter())
        .map(|a| a.name.as_str())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len(), "aspect names must be unique");
}

#[test]
fn category_serializes_as_kebab_case() {
    let json = serde_json::to_string(&AssessmentCategory::FileStructure).unwrap();
    assert_eq!(json, "\"file-structure\"");
    let json = serde_json::to_string(&AssessmentCategory::RuntimeBehavior).unwrap();
    assert_eq!(json, "\"runtime-behavior\"");
    let json = serde_json::to_string(&AssessmentCategory::CreativeExpression).unwrap();
    assert_eq!(json, "\"creative-expression\"");
    let json = serde_json::to_string(&AssessmentCategory::LearningEvidence).unwrap();
    assert_eq!(json, "\"learning-evidence\"");
}

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = for_building_a_scene();
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["lesson"], "building-a-scene-first-world");
    assert!(json["machine_assessable"].is_array());
    assert!(json["human_review_needed"].is_array());
    assert_eq!(json["machine_assessable"].as_array().unwrap().len(), 6);
    assert_eq!(json["human_review_needed"].as_array().unwrap().len(), 6);

    let first = &json["machine_assessable"][0];
    assert!(first["name"].is_string());
    assert!(first["category"].is_string());
    assert!(first["rationale"].is_string());
}
