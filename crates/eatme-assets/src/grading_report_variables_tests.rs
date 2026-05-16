use super::*;
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement, VariableDeclaration};

// --- Test fixtures ---

/// A complete program with a variable declaration, variable usage in a method call,
/// and a variable assignment (modification).
fn complete_variables_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["score".into()],
                },
                Statement::VariableAssignment {
                    variable: "score".into(),
                    value: "score + 1".into(),
                },
            ],
        }],
        functions: vec![],
        variable_declarations: vec![VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        }],
    }
}

/// A program with a variable declared but never used in a method call.
fn program_with_var_declared_not_used() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "walk".into(),
                arguments: vec!["FORWARD".into()],
            }],
        }],
        functions: vec![],
        variable_declarations: vec![VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        }],
    }
}

/// A program with a variable declared and used but never modified.
fn program_with_var_declared_and_used_not_modified() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["score".into()],
            }],
        }],
        functions: vec![],
        variable_declarations: vec![VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        }],
    }
}

/// A program with no variable declarations at all.
fn program_with_no_variables() -> Program {
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

fn variables_input_all_ready(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn variables_input_blocked_assets(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn variables_input_blocked_deps(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn variables_input_both_blocked(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
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
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_variables_scorekeeper() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.lesson, "variables-scorekeeper-timekeeper");
}

#[test]
fn always_produces_eight_steps() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps.len(), 8);
}

#[test]
fn step_names_in_order() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "declare-variable",
            "use-variable-in-method",
            "modify-variable",
            "run-world",
            "save-project",
        ]
    );
}

// --- depends_on field tests ---

#[test]
fn root_steps_have_empty_dependencies() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert!(report.steps[0].depends_on.is_empty(), "validate-assets");
    assert!(report.steps[1].depends_on.is_empty(), "check-dependencies");
}

#[test]
fn launch_smoke_depends_on_first_two() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn declare_variable_depends_on_launch_smoke() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn use_variable_in_method_depends_on_declare_variable() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[4].depends_on, vec!["declare-variable"]);
}

#[test]
fn modify_variable_depends_on_use_variable_in_method() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[5].depends_on, vec!["use-variable-in-method"]);
}

#[test]
fn run_world_depends_on_modify_variable() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[6].depends_on, vec!["modify-variable"]);
}

#[test]
fn save_project_depends_on_run_world() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[7].depends_on, vec!["run-world"]);
}

// --- All ready with complete program ---

#[test]
fn all_ready_complete_program_report_does_not_pass() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert!(
        !report.passed,
        "report should not pass because run-world is not-yet-tested"
    );
}

#[test]
fn all_ready_complete_program_preconditions_ready() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[test]
fn all_ready_complete_program_declare_variable_ready() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert!(
        report.steps[3].reason.contains("VariableDeclaration found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn all_ready_complete_program_use_variable_in_method_ready() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert!(
        report.steps[4]
            .reason
            .contains("variable used in method"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn all_ready_complete_program_modify_variable_ready() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[5].status, StepStatus::Ready);
    assert!(
        report.steps[5]
            .reason
            .contains("VariableAssignment found"),
        "reason: {}",
        report.steps[5].reason
    );
}

#[test]
fn all_ready_complete_program_run_world_not_yet_tested() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[6].status, StepStatus::NotYetTested);
}

#[test]
fn all_ready_complete_program_save_project_ready() {
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    assert_eq!(report.steps[7].status, StepStatus::Ready);
    assert!(
        report.steps[7].reason.contains("round-trip"),
        "save-project reason should mention round-trip: {}",
        report.steps[7].reason
    );
}

// --- No student program ---

#[test]
fn no_program_all_interaction_steps_blocked() {
    let report = grade_variables(variables_input_all_ready(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
    for i in 3..=7 {
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
    let report = grade_variables(variables_input_all_ready(None));
    for i in 3..=7 {
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

// --- Missing variable declaration ---

#[test]
fn missing_variable_declare_variable_blocked() {
    let report =
        grade_variables(variables_input_all_ready(Some(program_with_no_variables())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3]
            .reason
            .contains("No VariableDeclaration found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn missing_variable_cascades_downstream() {
    let report =
        grade_variables(variables_input_all_ready(Some(program_with_no_variables())));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "use-variable-in-method"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "modify-variable"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Variable declared but not used ---

#[test]
fn declared_not_used_use_variable_in_method_blocked() {
    let report = grade_variables(variables_input_all_ready(Some(
        program_with_var_declared_not_used(),
    )));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "declare-variable should still be ready"
    );
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("No variable used in method"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn declared_not_used_cascades_downstream() {
    let report = grade_variables(variables_input_all_ready(Some(
        program_with_var_declared_not_used(),
    )));
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "modify-variable"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Variable declared and used but not modified ---

#[test]
fn declared_used_not_modified_modify_variable_blocked() {
    let report = grade_variables(variables_input_all_ready(Some(
        program_with_var_declared_and_used_not_modified(),
    )));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "declare-variable"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "use-variable-in-method"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert!(
        report.steps[5]
            .reason
            .contains("No VariableAssignment found"),
        "reason: {}",
        report.steps[5].reason
    );
}

#[test]
fn declared_used_not_modified_cascades_downstream() {
    let report = grade_variables(variables_input_all_ready(Some(
        program_with_var_declared_and_used_not_modified(),
    )));
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report =
        grade_variables(variables_input_blocked_assets(Some(complete_variables_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(
        report.steps[0].reason,
        "3 scenario assets failed validation"
    );
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    for i in 3..=7 {
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
    let report =
        grade_variables(variables_input_blocked_deps(Some(complete_variables_program())));
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
    for i in 3..=7 {
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
    let report =
        grade_variables(variables_input_both_blocked(Some(complete_variables_program())));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_all_steps_blocked() {
    let report =
        grade_variables(variables_input_both_blocked(Some(complete_variables_program())));
    assert_eq!(report.steps[0].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    for i in 2..=7 {
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
    let report =
        grade_variables(variables_input_all_ready(Some(complete_variables_program())));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "variables-scorekeeper-timekeeper");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 8);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "declare-variable");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][4]["name"], "use-variable-in-method");
    assert_eq!(json["steps"][4]["status"], "ready");
    assert_eq!(json["steps"][5]["name"], "modify-variable");
    assert_eq!(json["steps"][5]["status"], "ready");
    assert_eq!(json["steps"][6]["name"], "run-world");
    assert_eq!(json["steps"][6]["status"], "not-yet-tested");
    assert_eq!(json["steps"][7]["name"], "save-project");
    assert_eq!(json["steps"][7]["status"], "ready");
}
