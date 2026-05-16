use super::*;
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement};

// --- Test fixtures ---

/// A complete program with a parameterized procedure and a call that passes an argument.
fn complete_parameters_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "greet".into(),
                    arguments: vec!["\"Hello\"".into()],
                }],
            },
            Procedure {
                name: "greet".into(),
                parameters: vec![Parameter {
                    name: "message".into(),
                    param_type: "String".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["message".into()],
                }],
            },
        ],
        functions: vec![],
        variable_declarations: vec![],
    }
}

/// A program with a parameterized procedure but no call that passes arguments.
fn program_with_param_procedure_no_call() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into()],
                }],
            },
            Procedure {
                name: "greet".into(),
                parameters: vec![Parameter {
                    name: "message".into(),
                    param_type: "String".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["message".into()],
                }],
            },
        ],
        functions: vec![],
        variable_declarations: vec![],
    }
}

/// A program with no parameterized procedures.
fn program_with_no_parameters() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into(), "1.0".into()],
            }],
        }],
        functions: vec![],
        variable_declarations: vec![],
    }
}

fn parameters_input_all_ready(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn parameters_input_blocked_assets(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn parameters_input_blocked_deps(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn parameters_input_both_blocked(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
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
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_parameters_procedure_generalization() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.lesson, "parameters-procedure-generalization");
}

#[test]
fn always_produces_seven_steps() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps.len(), 7);
}

#[test]
fn step_names_in_order() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "create-parameterized-procedure",
            "call-with-argument",
            "run-world",
            "save-project",
        ]
    );
}

// --- depends_on field tests ---

#[test]
fn root_steps_have_empty_dependencies() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert!(report.steps[0].depends_on.is_empty(), "validate-assets");
    assert!(report.steps[1].depends_on.is_empty(), "check-dependencies");
}

#[test]
fn launch_smoke_depends_on_first_two() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn create_parameterized_procedure_depends_on_launch_smoke() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn call_with_argument_depends_on_create_parameterized_procedure() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(
        report.steps[4].depends_on,
        vec!["create-parameterized-procedure"]
    );
}

#[test]
fn run_world_depends_on_call_with_argument() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[5].depends_on, vec!["call-with-argument"]);
}

#[test]
fn save_project_depends_on_run_world() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[6].depends_on, vec!["run-world"]);
}

// --- All ready with complete program ---

#[test]
fn all_ready_complete_program_report_does_not_pass() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert!(
        !report.passed,
        "report should not pass because run-world is not-yet-tested"
    );
}

#[test]
fn all_ready_complete_program_preconditions_ready() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[test]
fn all_ready_complete_program_create_parameterized_procedure_ready() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert!(
        report.steps[3]
            .reason
            .contains("parameterized procedure found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn all_ready_complete_program_call_with_argument_ready() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert!(
        report.steps[4]
            .reason
            .contains("call with argument found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn all_ready_complete_program_run_world_not_yet_tested() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
}

#[test]
fn all_ready_complete_program_save_project_ready() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    assert_eq!(report.steps[6].status, StepStatus::Ready);
    assert!(
        report.steps[6].reason.contains("round-trip"),
        "save-project reason should mention round-trip: {}",
        report.steps[6].reason
    );
}

// --- No student program ---

#[test]
fn no_program_all_interaction_steps_blocked() {
    let report = grade_parameters(parameters_input_all_ready(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
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
    let report = grade_parameters(parameters_input_all_ready(None));
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

// --- Missing parameterized procedure ---

#[test]
fn missing_parameterized_procedure_blocked() {
    let report =
        grade_parameters(parameters_input_all_ready(Some(program_with_no_parameters())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3]
            .reason
            .contains("No parameterized procedure found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn missing_parameterized_procedure_cascades_downstream() {
    let report =
        grade_parameters(parameters_input_all_ready(Some(program_with_no_parameters())));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "call-with-argument"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Parameterized procedure but no call with argument ---

#[test]
fn param_procedure_no_call_call_with_argument_blocked() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        program_with_param_procedure_no_call(),
    )));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "create-parameterized-procedure should still be ready"
    );
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No call with argument found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn param_procedure_no_call_cascades_downstream() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        program_with_param_procedure_no_call(),
    )));
    assert_eq!(report.steps[5].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "save-project");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report = grade_parameters(parameters_input_blocked_assets(Some(
        complete_parameters_program(),
    )));
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
    let report = grade_parameters(parameters_input_blocked_deps(Some(
        complete_parameters_program(),
    )));
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
    let report = grade_parameters(parameters_input_both_blocked(Some(
        complete_parameters_program(),
    )));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_all_steps_blocked() {
    let report = grade_parameters(parameters_input_both_blocked(Some(
        complete_parameters_program(),
    )));
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

// --- JSON serialization ---

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = grade_parameters(parameters_input_all_ready(Some(
        complete_parameters_program(),
    )));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "parameters-procedure-generalization");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 7);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "create-parameterized-procedure");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][4]["name"], "call-with-argument");
    assert_eq!(json["steps"][4]["status"], "ready");
    assert_eq!(json["steps"][5]["name"], "run-world");
    assert_eq!(json["steps"][5]["status"], "not-yet-tested");
    assert_eq!(json["steps"][6]["name"], "save-project");
    assert_eq!(json["steps"][6]["status"], "ready");
}
