use eatme_assets::NestedControlFlowGradingInput;
use eatme_assets::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput, GradingInput,
    InheritanceOopGradingInput, LoopsGradingInput, ParametersGradingInput,
    SceneBuildingGradingInput, SequencingGradingInput, StepStatus, TextbookIntegrationGradingInput,
    VariablesGradingInput, grade_arrays_and_arithmetic, grade_comments, grade_creative_project,
    grade_events_and_collision, grade_first_lesson_readiness, grade_functions,
    grade_games_and_narrative, grade_inheritance_oop, grade_loops_and_conditionals,
    grade_nested_control_flow, grade_parameters, grade_scene_building, grade_sequencing,
    grade_textbook_integration, grade_variables,
};

fn assert_actionable_non_ready_reasons(report_name: &str, steps: &[eatme_assets::StepGrade]) {
    let non_ready_steps: Vec<_> = steps
        .iter()
        .filter(|step| step.status != StepStatus::Ready)
        .collect();
    assert!(
        !non_ready_steps.is_empty(),
        "{report_name} should expose at least one non-ready step for guidance validation"
    );

    for step in non_ready_steps {
        assert!(
            step.reason.contains("save the project"),
            "{report_name}/{} should tell the student to save the project: {}",
            step.name,
            step.reason
        );
        assert!(
            step.reason.contains("rerun grading"),
            "{report_name}/{} should tell the student to rerun grading: {}",
            step.name,
            step.reason
        );
    }
}

#[test]
fn every_grading_function_produces_actionable_feedback_for_missing_student_evidence() {
    let reports = vec![
        (
            "grade_first_lesson_readiness",
            grade_first_lesson_readiness(GradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
            }),
        ),
        (
            "grade_scene_building",
            grade_scene_building(SceneBuildingGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_scene: None,
            }),
        ),
        (
            "grade_sequencing",
            grade_sequencing(SequencingGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                sequence_blocks: None,
            }),
        ),
        (
            "grade_loops_and_conditionals",
            grade_loops_and_conditionals(LoopsGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_variables",
            grade_variables(VariablesGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_nested_control_flow",
            grade_nested_control_flow(NestedControlFlowGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_arrays_and_arithmetic",
            grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_comments",
            grade_comments(CommentsGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_parameters",
            grade_parameters(ParametersGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_functions",
            grade_functions(FunctionsGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_events_and_collision",
            grade_events_and_collision(EventsGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_inheritance_oop",
            grade_inheritance_oop(InheritanceOopGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_games_and_narrative",
            grade_games_and_narrative(GamesNarrativeGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_textbook_integration",
            grade_textbook_integration(TextbookIntegrationGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
        (
            "grade_creative_project",
            grade_creative_project(CreativeProjectGradingInput {
                assets_valid: true,
                asset_reason: "Assets are ready. Save the project after each change.".into(),
                deps_available: true,
                deps_reason: "Dependencies are ready. Save the project after each change.".into(),
                student_program: None,
            }),
        ),
    ];

    for (name, report) in reports {
        assert_actionable_non_ready_reasons(name, &report.steps);
    }
}
