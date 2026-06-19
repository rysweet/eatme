use serde_json::Value;

use crate::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput, GradingInput,
    InheritanceOopGradingInput, LoopsGradingInput, NestedControlFlowGradingInput,
    ParametersGradingInput, SceneBuildingGradingInput, SequencingGradingInput,
    TextbookIntegrationGradingInput, VariablesGradingInput, grade_arrays_and_arithmetic,
    grade_comments, grade_creative_project, grade_events_and_collision,
    grade_first_lesson_readiness, grade_functions, grade_games_and_narrative,
    grade_inheritance_oop, grade_loops_and_conditionals, grade_nested_control_flow,
    grade_parameters, grade_scene_building, grade_sequencing, grade_textbook_integration,
    grade_variables,
};

fn assert_report_shape(report: Value, lesson: &str) {
    let object = report
        .as_object()
        .expect("grading report should serialize to an object");

    assert_eq!(
        object.get("schema_version").and_then(Value::as_str),
        Some("eatme.assets/grading/v1")
    );
    assert_eq!(object.get("lesson").and_then(Value::as_str), Some(lesson));
    assert!(
        matches!(object.get("passed"), Some(Value::Bool(_))),
        "expected passed boolean in {object:?}"
    );

    let steps = object
        .get("steps")
        .and_then(Value::as_array)
        .expect("grading report should include a steps array");
    assert!(
        !steps.is_empty(),
        "grading report for {lesson} should contain at least one step"
    );
}

#[test]
fn every_grading_function_emits_a_well_formed_report() {
    let reports = [
        (
            "building-a-scene-first-world",
            serde_json::to_value(grade_first_lesson_readiness(GradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
            }))
            .unwrap(),
        ),
        (
            "loops-and-conditionals-mini-challenge",
            serde_json::to_value(grade_loops_and_conditionals(LoopsGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "arrays-collection-choreography",
            serde_json::to_value(grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "comments-mini-challenge",
            serde_json::to_value(grade_comments(CommentsGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "creative-design-project",
            serde_json::to_value(grade_creative_project(CreativeProjectGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "events-collision-proximity-game",
            serde_json::to_value(grade_events_and_collision(EventsGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "using-functions-mini-challenge",
            serde_json::to_value(grade_functions(FunctionsGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "games-and-interactive-narrative",
            serde_json::to_value(grade_games_and_narrative(GamesNarrativeGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "inheritance-oop-mini-challenge",
            serde_json::to_value(grade_inheritance_oop(InheritanceOopGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "nested-control-flow-relational-expressions",
            serde_json::to_value(grade_nested_control_flow(NestedControlFlowGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "parameters-mini-challenge",
            serde_json::to_value(grade_parameters(ParametersGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
        (
            "building-a-scene-first-world",
            serde_json::to_value(grade_scene_building(SceneBuildingGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_scene: None,
            }))
            .unwrap(),
        ),
        (
            "procedure-sequencing-do-in-order-do-together",
            serde_json::to_value(grade_sequencing(SequencingGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                sequence_blocks: None,
            }))
            .unwrap(),
        ),
        (
            "textbook-integration-java-transition",
            serde_json::to_value(grade_textbook_integration(
                TextbookIntegrationGradingInput {
                    assets_valid: true,
                    asset_reason: "assets ready".into(),
                    deps_available: true,
                    deps_reason: "dependencies ready".into(),
                    student_program: None,
                },
            ))
            .unwrap(),
        ),
        (
            "using-variables-mini-challenge",
            serde_json::to_value(grade_variables(VariablesGradingInput {
                assets_valid: true,
                asset_reason: "assets ready".into(),
                deps_available: true,
                deps_reason: "dependencies ready".into(),
                student_program: None,
            }))
            .unwrap(),
        ),
    ];

    for (lesson, report) in reports {
        assert_report_shape(report, lesson);
    }
}
