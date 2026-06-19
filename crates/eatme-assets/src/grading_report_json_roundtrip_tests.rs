use crate::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput, GradingInput,
    GradingReport, InheritanceOopGradingInput, LoopsGradingInput, NestedControlFlowGradingInput,
    ParametersGradingInput, SceneBuildingGradingInput, SequencingGradingInput,
    VariablesGradingInput, grade_arrays_and_arithmetic, grade_comments, grade_creative_project,
    grade_events_and_collision, grade_first_lesson_readiness, grade_functions,
    grade_games_and_narrative, grade_inheritance_oop, grade_loops_and_conditionals,
    grade_nested_control_flow, grade_parameters, grade_scene_building, grade_sequencing,
    grade_variables,
};

fn assert_round_trip(label: &str, report: GradingReport) {
    let json = serde_json::to_string_pretty(&report).unwrap();
    let restored: GradingReport = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, report, "{label} should round-trip through JSON");
}

fn blocked_reason(name: &str) -> String {
    format!("{name} unavailable during test")
}

#[test]
fn every_grading_function_serializes_and_deserializes_back_to_grading_report() {
    assert_round_trip(
        "grade_first_lesson_readiness",
        grade_first_lesson_readiness(GradingInput {
            assets_valid: false,
            asset_reason: blocked_reason("assets"),
            deps_available: false,
            deps_reason: blocked_reason("dependencies"),
        }),
    );

    assert_round_trip(
        "grade_loops_and_conditionals",
        grade_loops_and_conditionals(LoopsGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_scene_building",
        grade_scene_building(SceneBuildingGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_scene: None,
        }),
    );

    assert_round_trip(
        "grade_sequencing",
        grade_sequencing(SequencingGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            sequence_blocks: None,
        }),
    );

    assert_round_trip(
        "grade_variables",
        grade_variables(VariablesGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_parameters",
        grade_parameters(ParametersGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_functions",
        grade_functions(FunctionsGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_arrays_and_arithmetic",
        grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_comments",
        grade_comments(CommentsGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_events_and_collision",
        grade_events_and_collision(EventsGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_nested_control_flow",
        grade_nested_control_flow(NestedControlFlowGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_inheritance_oop",
        grade_inheritance_oop(InheritanceOopGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_games_and_narrative",
        grade_games_and_narrative(GamesNarrativeGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );

    assert_round_trip(
        "grade_creative_project",
        grade_creative_project(CreativeProjectGradingInput {
            assets_valid: true,
            asset_reason: "assets ready".into(),
            deps_available: true,
            deps_reason: "dependencies ready".into(),
            student_program: None,
        }),
    );
}
