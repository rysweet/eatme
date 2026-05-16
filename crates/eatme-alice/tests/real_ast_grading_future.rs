#![allow(unexpected_cfgs)]
//! TDD CONTRACTS — Lessons 5-8 grading pipelines
//!
//! These tests define the expected API and behavior for grading pipelines
//! that have NOT been implemented yet. They are gated behind the Cargo feature
//! `grading-l5-l8`, which does not currently exist. To activate:
//!
//!   1. Implement the grading functions + AST extensions in eatme-assets/eatme-core
//!   2. Add `grading-l5-l8 = []` to [features] in crates/eatme-alice/Cargo.toml
//!   3. Export the new types from eatme-assets lib.rs
//!   4. Run: cargo test -p eatme-alice --test real_ast_grading_future --features grading-l5-l8
//!
//! Expected new types to implement:
//!
//!   eatme-core/src/ast.rs:
//!     - Statement::VariableDeclaration { name: String, value: String }
//!     - Statement::VariableAssignment { name: String, value: String }
//!     - Statement::FunctionCall { name: String, arguments: Vec<String> }
//!     - Statement::ReturnStatement { value: String }
//!     - Procedure { name, parameters: Vec<Parameter>, body }  (add parameters field)
//!     - Function { name: String, parameters: Vec<Parameter>, return_type: Option<String>, body: Vec<Statement> }
//!     - Parameter { name: String, parameter_type: String }
//!     - Program { procedures, functions: Vec<Function> }  (add functions field)
//!
//!   eatme-assets grading modules:
//!     - FunctionsGradingInput + grade_functions() → GradingReport
//!     - VariablesGradingInput + grade_variables() → GradingReport
//!     - ParametersGradingInput + grade_parameters() → GradingReport
//!     - CreativeGradingInput + grade_creative_project() → GradingReport

#[cfg(feature = "grading-l5-l8")]
use eatme_assets::grading_report::StepStatus;
#[cfg(feature = "grading-l5-l8")]
use eatme_core::ast::Program;

#[cfg(feature = "grading-l5-l8")]
#[allow(dead_code)]
mod a3p_parser_support;
#[cfg(feature = "grading-l5-l8")]
#[allow(dead_code)]
mod launch_smoke_support;

#[cfg(feature = "grading-l5-l8")]
use a3p_parser_support::parse_a3p_program;
#[cfg(feature = "grading-l5-l8")]
use launch_smoke_support::{real_alice_enabled, starter_project_path};

#[cfg(feature = "grading-l5-l8")]
use eatme_assets::{
    CreativeGradingInput, FunctionsGradingInput, ParametersGradingInput, VariablesGradingInput,
    grade_creative_project, grade_functions, grade_parameters, grade_variables,
};

// ---------------------------------------------------------------------------
// Helpers: build grading inputs from a parsed program
// ---------------------------------------------------------------------------

#[cfg(feature = "grading-l5-l8")]
fn functions_input(program: Program) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

#[cfg(feature = "grading-l5-l8")]
fn variables_input(program: Program) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

#[cfg(feature = "grading-l5-l8")]
fn parameters_input(program: Program) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

#[cfg(feature = "grading-l5-l8")]
fn creative_input(program: Program) -> CreativeGradingInput {
    CreativeGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

// ---------------------------------------------------------------------------
// Lesson 5: Functions — real .a3p grading
// ---------------------------------------------------------------------------

#[cfg(feature = "grading-l5-l8")]
#[test]
fn real_alice_functions_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice functions grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    let report = grade_functions(functions_input(program));

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    assert_eq!(report.steps[3].name, "create-function");
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert_eq!(report.steps[4].name, "call-function");
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    assert!(!report.passed);
}

// ---------------------------------------------------------------------------
// Lesson 6: Variables — real .a3p grading
// ---------------------------------------------------------------------------

#[cfg(feature = "grading-l5-l8")]
#[test]
fn real_alice_variables_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice variables grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    let report = grade_variables(variables_input(program));

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    assert_eq!(report.steps[3].name, "declare-variable");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "starter has VariableDeclaration from LocalDeclarationStatement"
    );
    assert_eq!(report.steps[4].name, "modify-variable");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "starter has no VariableAssignment"
    );
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    assert!(!report.passed);
}

// ---------------------------------------------------------------------------
// Lesson 7: Parameters — real .a3p grading
// ---------------------------------------------------------------------------

#[cfg(feature = "grading-l5-l8")]
#[test]
fn real_alice_parameters_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice parameters grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    let report = grade_parameters(parameters_input(program));

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    assert_eq!(report.steps[3].name, "add-parameter");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "starter has UserParameter definitions"
    );
    assert_eq!(report.steps[4].name, "pass-argument");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "starter has MethodInvocations with arguments"
    );
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Ready);

    assert!(!report.passed);
}

// ---------------------------------------------------------------------------
// Lesson 8: Creative Project — real .a3p grading
// ---------------------------------------------------------------------------

#[cfg(feature = "grading-l5-l8")]
#[test]
fn real_alice_creative_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice creative grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    let report = grade_creative_project(creative_input(program));

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    assert_eq!(report.steps[3].name, "build-scene");
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].name, "create-custom-procedure");
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].name, "add-control-structure");
    assert_eq!(report.steps[5].status, StepStatus::Ready);
    assert_eq!(report.steps[6].name, "add-event-or-interaction");
    assert_eq!(
        report.steps[6].status,
        StepStatus::Blocked,
        "starter has no EventListener or CollisionListener"
    );

    assert!(!report.passed);
}
