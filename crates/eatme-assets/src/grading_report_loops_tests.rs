use super::*;
use eatme_core::ast::{Procedure, Program, Statement};

// --- Test fixtures ---

fn complete_program() -> Program {
    Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![
            Statement::CountLoop {
                count: 3,
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
            },
            Statement::IfElse {
                condition: "this.cat isCloseTo this.dog".into(),
                if_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                }],
                else_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "think".into(),
                    arguments: vec!["\"Hmm...\"".into()],
                }],
            },
        ],
    }])
}

fn program_with_loop_only() -> Program {
    Program::new(vec![Procedure {
        name: "loopOnly".into(),
        body: vec![Statement::CountLoop {
            count: 5,
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into()],
            }],
        }],
    }])
}

fn program_with_conditional_only() -> Program {
    Program::new(vec![Procedure {
        name: "conditionalOnly".into(),
        body: vec![Statement::IfElse {
            condition: "this.cat isCloseTo this.dog".into(),
            if_body: vec![],
            else_body: vec![],
        }],
    }])
}

fn loops_input_all_ready(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn loops_input_blocked_assets(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}
fn loops_input_blocked_deps(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}
fn loops_input_both_blocked(program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

// --- Schema and structure tests ---

#[test]
fn schema_version_is_grading_v1() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_loops_and_conditionals_mini_challenge() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
}

#[test]
fn always_produces_seven_steps() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps.len(), 7);
}

#[test]
fn step_names_in_order() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "build-counting-loop",
            "add-conditional-branch",
            "run-world",
            "save-project",
        ]
    );
}

// --- depends_on field tests ---

#[test]
fn root_steps_have_empty_dependencies() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert!(
        report.steps[0].depends_on.is_empty(),
        "validate-assets should have no dependencies"
    );
    assert!(
        report.steps[1].depends_on.is_empty(),
        "check-dependencies should have no dependencies"
    );
}

#[test]
fn launch_smoke_depends_on_first_two() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn build_counting_loop_depends_on_launch_smoke() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn add_conditional_branch_depends_on_build_counting_loop() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[4].depends_on, vec!["build-counting-loop"]);
}

#[test]
fn run_world_depends_on_add_conditional_branch() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[5].depends_on, vec!["add-conditional-branch"]);
}

#[test]
fn save_project_depends_on_run_world() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[6].depends_on, vec!["run-world"]);
}

// --- All ready with complete program ---

#[test]
fn all_ready_complete_program_report_does_not_pass() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert!(
        !report.passed,
        "report should not pass because run-world is not-yet-tested"
    );
}

#[test]
fn all_ready_complete_program_preconditions_ready() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[test]
fn all_ready_complete_program_build_counting_loop_ready() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
}

#[test]
fn all_ready_complete_program_add_conditional_branch_ready() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
}

#[test]
fn all_ready_complete_program_run_world_not_yet_tested() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
}

#[test]
fn all_ready_complete_program_save_project_ready() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(report.steps[6].status, StepStatus::Ready);
}

#[test]
fn all_ready_reasons_propagate() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    assert_eq!(
        report.steps[0].reason,
        "All 101 scenario assets passed validation"
    );
    assert_eq!(report.steps[1].reason, "All required tools available");
}

// --- No student program ---

#[test]
fn no_program_all_interaction_steps_blocked() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(None));
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

#[test]
fn no_program_all_interaction_steps_mention_no_student_program() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(None));
    for i in 3..=6 {
        assert!(
            report.steps[i]
                .reason
                .contains("No student program provided"),
            "step {} ({}) reason should mention 'No student program provided': {}",
            i,
            report.steps[i].name,
            report.steps[i].reason
        );
    }
}

#[test]
fn no_program_preconditions_still_ready() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
}

// --- Missing loop construct ---

#[test]
fn missing_loop_build_counting_loop_blocked() {
    let report =
        grade_loops_and_conditionals(loops_input_all_ready(Some(program_with_conditional_only())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3].reason.contains("No CountLoop found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn missing_loop_cascades_downstream() {
    let report =
        grade_loops_and_conditionals(loops_input_all_ready(Some(program_with_conditional_only())));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-conditional-branch"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Missing conditional construct ---

#[test]
fn missing_conditional_add_conditional_branch_blocked() {
    let report =
        grade_loops_and_conditionals(loops_input_all_ready(Some(program_with_loop_only())));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "build-counting-loop should still be ready"
    );
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4].reason.contains("No IfElse found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn missing_conditional_cascades_downstream() {
    let report =
        grade_loops_and_conditionals(loops_input_all_ready(Some(program_with_loop_only())));
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report = grade_loops_and_conditionals(loops_input_blocked_assets(Some(complete_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[0].reason,
        "3 scenario assets failed validation"
    );
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Blocked dependencies scenario ---

#[test]
fn blocked_deps_cascades_all_downstream() {
    let report = grade_loops_and_conditionals(loops_input_blocked_deps(Some(complete_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[1].reason,
        "Missing required tools: Xvfb, wmctrl"
    );
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert!(
        report.steps[2].reason.contains("check-dependencies"),
        "launch-smoke reason: {}",
        report.steps[2].reason
    );
    for i in 3..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Both blocked scenario ---

#[test]
fn both_blocked_launch_smoke_mentions_both_blockers() {
    let report = grade_loops_and_conditionals(loops_input_both_blocked(Some(complete_program())));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_all_steps_blocked() {
    let report = grade_loops_and_conditionals(loops_input_both_blocked(Some(complete_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    for i in 2..=6 {
        assert_eq!(
            report.steps[i].status,
            StepStatus::Blocked,
            "step {} ({}) should be blocked",
            i,
            report.steps[i].name
        );
    }
}

// --- Nested AST detection ---

#[test]
fn nested_count_loop_inside_if_else_is_detected() {
    let program = Program::new(vec![Procedure {
        name: "nestedLoop".into(),
        body: vec![Statement::IfElse {
            condition: "true".into(),
            if_body: vec![Statement::CountLoop {
                count: 2,
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec![],
                }],
            }],
            else_body: vec![],
        }],
    }]);
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "CountLoop nested inside IfElse should be detected"
    );
}

#[test]
fn nested_if_else_inside_count_loop_is_detected() {
    let program = Program::new(vec![Procedure {
        name: "nestedConditional".into(),
        body: vec![Statement::CountLoop {
            count: 3,
            body: vec![Statement::IfElse {
                condition: "true".into(),
                if_body: vec![],
                else_body: vec![],
            }],
        }],
    }]);
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(program)));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "CountLoop at top level should be detected"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "IfElse nested inside CountLoop should be detected"
    );
}

// --- JSON serialization ---

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = grade_loops_and_conditionals(loops_input_all_ready(Some(complete_program())));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "loops-and-conditionals-mini-challenge");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 7);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "build-counting-loop");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][5]["name"], "run-world");
    assert_eq!(json["steps"][5]["status"], "not-yet-tested");
    assert_eq!(json["steps"][6]["name"], "save-project");
    assert_eq!(json["steps"][6]["status"], "ready");
}
