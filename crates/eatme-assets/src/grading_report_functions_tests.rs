use super::*;
use eatme_core::ast::{Function, Procedure, Program, Statement};

// --- Test fixtures ---

/// A complete program with a user-defined function that has a return statement,
/// and a procedure that calls the function.
fn complete_functions_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::FunctionCall {
                function_name: "getGreeting".into(),
            }],
        }],
        functions: vec![Function {
            name: "getGreeting".into(),
            return_type: "String".into(),
            body: vec![Statement::ReturnStatement {
                value: "\"Hello world!\"".into(),
            }],
        }],
        variable_declarations: vec![],
    }
}

/// A program with a function defined but no return statement inside it.
fn program_with_function_no_return() -> Program {
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
        functions: vec![Function {
            name: "doSomething".into(),
            return_type: "Void".into(),
            body: vec![Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["\"Hi\"".into()],
            }],
        }],
        variable_declarations: vec![],
    }
}

/// A program with a function and return statement, but no call from a procedure.
fn program_with_function_and_return_no_call() -> Program {
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
        functions: vec![Function {
            name: "getGreeting".into(),
            return_type: "String".into(),
            body: vec![Statement::ReturnStatement {
                value: "\"Hello!\"".into(),
            }],
        }],
        variable_declarations: vec![],
    }
}

/// A program with no functions at all (only procedures with method calls).
fn program_with_no_functions() -> Program {
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

fn functions_input_all_ready(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn functions_input_blocked_assets(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: false,
        asset_reason: "3 scenario assets failed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn functions_input_blocked_deps(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
        student_program: program,
    }
}

fn functions_input_both_blocked(program: Option<Program>) -> FunctionsGradingInput {
    FunctionsGradingInput {
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
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
}

#[test]
fn lesson_is_functions_mini_challenge() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.lesson, "functions-mini-challenge");
}

#[test]
fn always_produces_eight_steps() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps.len(), 8);
}

#[test]
fn step_names_in_order() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    let names: Vec<&str> = report.steps.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "launch-smoke",
            "create-function",
            "add-return-statement",
            "call-function-from-procedure",
            "run-world",
            "save-project",
        ]
    );
}

// --- depends_on field tests ---

#[test]
fn root_steps_have_empty_dependencies() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert!(report.steps[0].depends_on.is_empty(), "validate-assets");
    assert!(report.steps[1].depends_on.is_empty(), "check-dependencies");
}

#[test]
fn launch_smoke_depends_on_first_two() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(
        report.steps[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn create_function_depends_on_launch_smoke() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[3].depends_on, vec!["launch-smoke"]);
}

#[test]
fn add_return_statement_depends_on_create_function() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[4].depends_on, vec!["create-function"]);
}

#[test]
fn call_function_from_procedure_depends_on_add_return_statement() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[5].depends_on, vec!["add-return-statement"]);
}

#[test]
fn run_world_depends_on_call_function_from_procedure() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(
        report.steps[6].depends_on,
        vec!["call-function-from-procedure"]
    );
}

#[test]
fn save_project_depends_on_run_world() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[7].depends_on, vec!["run-world"]);
}

// --- All ready with complete program ---

#[test]
fn all_ready_complete_program_report_does_not_pass() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert!(
        !report.passed,
        "report should not pass because run-world is not-yet-tested"
    );
}

#[test]
fn all_ready_complete_program_preconditions_ready() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[0].status, StepStatus::Ready, "validate-assets");
    assert_eq!(
        report.steps[1].status,
        StepStatus::Ready,
        "check-dependencies"
    );
    assert_eq!(report.steps[2].status, StepStatus::Ready, "launch-smoke");
}

#[test]
fn all_ready_complete_program_create_function_ready() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert!(
        report.steps[3].reason.contains("Function found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn all_ready_complete_program_add_return_statement_ready() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert!(
        report.steps[4].reason.contains("ReturnStatement found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn all_ready_complete_program_call_function_from_procedure_ready() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[5].status, StepStatus::Ready);
    assert!(
        report.steps[5].reason.contains("FunctionCall found"),
        "reason: {}",
        report.steps[5].reason
    );
}

#[test]
fn all_ready_complete_program_run_world_not_yet_tested() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    assert_eq!(report.steps[6].status, StepStatus::NotYetTested);
}

#[test]
fn all_ready_complete_program_save_project_ready() {
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
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
    let report = grade_functions(functions_input_all_ready(None));
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
    let report = grade_functions(functions_input_all_ready(None));
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

#[test]
fn no_program_preconditions_still_ready() {
    let report = grade_functions(functions_input_all_ready(None));
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);
}

// --- Missing function construct ---

#[test]
fn missing_function_create_function_blocked() {
    let report = grade_functions(functions_input_all_ready(Some(program_with_no_functions())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3].reason.contains("No Function found"),
        "reason: {}",
        report.steps[3].reason
    );
}

#[test]
fn missing_function_cascades_downstream() {
    let report = grade_functions(functions_input_all_ready(Some(program_with_no_functions())));
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "add-return-statement"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "call-function-from-procedure"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Missing return statement ---

#[test]
fn missing_return_add_return_statement_blocked() {
    let report = grade_functions(functions_input_all_ready(Some(
        program_with_function_no_return(),
    )));
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "create-function should still be ready"
    );
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4].reason.contains("No ReturnStatement found"),
        "reason: {}",
        report.steps[4].reason
    );
}

#[test]
fn missing_return_cascades_downstream() {
    let report = grade_functions(functions_input_all_ready(Some(
        program_with_function_no_return(),
    )));
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "call-function-from-procedure"
    );
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Missing function call ---

#[test]
fn missing_call_call_function_from_procedure_blocked() {
    let report = grade_functions(functions_input_all_ready(Some(
        program_with_function_and_return_no_call(),
    )));
    assert_eq!(report.steps[3].status, StepStatus::Ready, "create-function");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "add-return-statement"
    );
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert!(
        report.steps[5].reason.contains("No FunctionCall found"),
        "reason: {}",
        report.steps[5].reason
    );
}

#[test]
fn missing_call_cascades_downstream() {
    let report = grade_functions(functions_input_all_ready(Some(
        program_with_function_and_return_no_call(),
    )));
    assert_eq!(report.steps[6].status, StepStatus::Blocked, "run-world");
    assert_eq!(report.steps[7].status, StepStatus::Blocked, "save-project");
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_cascades_all_downstream() {
    let report = grade_functions(functions_input_blocked_assets(Some(
        complete_functions_program(),
    )));
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
    let report = grade_functions(functions_input_blocked_deps(Some(
        complete_functions_program(),
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
    let report = grade_functions(functions_input_both_blocked(Some(
        complete_functions_program(),
    )));
    let reason = &report.steps[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "launch-smoke should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_all_steps_blocked() {
    let report = grade_functions(functions_input_both_blocked(Some(
        complete_functions_program(),
    )));
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
    let report = grade_functions(functions_input_all_ready(
        Some(complete_functions_program()),
    ));
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/grading/v1");
    assert_eq!(json["lesson"], "functions-mini-challenge");
    assert!(!json["passed"].as_bool().unwrap());
    assert!(json["steps"].is_array());
    assert_eq!(json["steps"].as_array().unwrap().len(), 8);
    assert_eq!(json["steps"][0]["name"], "validate-assets");
    assert_eq!(json["steps"][0]["status"], "ready");
    assert_eq!(json["steps"][3]["name"], "create-function");
    assert_eq!(json["steps"][3]["status"], "ready");
    assert_eq!(json["steps"][4]["name"], "add-return-statement");
    assert_eq!(json["steps"][4]["status"], "ready");
    assert_eq!(json["steps"][5]["name"], "call-function-from-procedure");
    assert_eq!(json["steps"][5]["status"], "ready");
    assert_eq!(json["steps"][6]["name"], "run-world");
    assert_eq!(json["steps"][6]["status"], "not-yet-tested");
    assert_eq!(json["steps"][7]["name"], "save-project");
    assert_eq!(json["steps"][7]["status"], "ready");
}
