#![allow(unexpected_cfgs)]
//! Real-Alice e2e grading integration tests.
//!
//! Loads real `.a3p` starter projects from `ALICE_HOME`, constructs `Program`
//! structures from actual Alice scene data via regex-based XML extraction,
//! and verifies grading pipelines produce correct results against real programs.
//!
//! **Gated behind `EATME_REAL_ALICE=1`** — requires an actual Alice installation
//! with starter projects on disk. CI sets the env var; local devs skip by default.
//!
//! # Covered pipelines
//!
//! | Lesson | Pipeline              | Tests                                   |
//! |--------|-----------------------|-----------------------------------------|
//! | 3      | Loops & Conditionals  | ✅ Grading + AST structure              |
//! | 4      | Events & Collision    | ✅ Grading                              |
//! | 5      | Functions             | 🔴 TDD contract (behind feature gate)  |
//! | 6      | Variables             | 🔴 TDD contract (behind feature gate)  |
//! | 7      | Parameters            | 🔴 TDD contract (behind feature gate)  |
//! | 8      | Creative Project      | 🔴 TDD contract (behind feature gate)  |

use eatme_assets::grading_report::{LoopsGradingInput, StepStatus, grade_loops_and_conditionals};
use eatme_assets::{EventsGradingInput, grade_events_and_collision};
use eatme_core::ast::{Program, Statement};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{real_alice_enabled, starter_project_path};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

// ---------------------------------------------------------------------------
// Helpers: build grading inputs from a parsed program
// ---------------------------------------------------------------------------

fn loops_input(program: Program) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

fn events_input(program: Program) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

// ===========================================================================
// Lesson 3: Loops & Conditionals — real .a3p grading
// ===========================================================================

#[test]
fn real_alice_loops_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice loops grading test (set EATME_REAL_ALICE=1 to enable)");
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

    // Parser must extract at least one procedure from real data
    assert!(
        !program.procedures.is_empty(),
        "parsed program should have at least one procedure"
    );

    let report = grade_loops_and_conditionals(loops_input(program));

    // --- Schema contract ---
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps.len(), 7);

    // --- Precondition steps: all Ready ---
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // --- AST checks against real starter data ---
    // amazonMinimum.a3p has NO CountLoop → build-counting-loop = Blocked
    assert_eq!(report.steps[3].name, "build-counting-loop");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "starter project has no CountLoop — should be Blocked"
    );
    assert!(
        report.steps[3].reason.contains("No CountLoop found"),
        "reason should explain missing construct: {}",
        report.steps[3].reason
    );

    // Cascade: build-counting-loop blocked → add-conditional-branch blocked
    assert_eq!(report.steps[4].name, "add-conditional-branch");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "blocked by upstream build-counting-loop"
    );

    // Cascade: add-conditional-branch blocked → run-world blocked
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);

    // Cascade: run-world blocked → save-project blocked
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    // Overall: not passed (blocked steps)
    assert!(!report.passed);
}

#[test]
fn real_alice_ast_structure_loops_and_conditionals() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice AST structure test (set EATME_REAL_ALICE=1 to enable)");
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

    assert!(
        !program.procedures.is_empty(),
        "parsed program should have at least one procedure"
    );

    // --- Independent AST-level assertions ---
    let all_stmts: Vec<&Statement> = program
        .procedures
        .iter()
        .flat_map(|p| p.body.iter())
        .collect();

    let if_else_count = all_stmts
        .iter()
        .filter(|s| matches!(s, Statement::IfElse { .. }))
        .count();
    let count_loop_count = all_stmts
        .iter()
        .filter(|s| matches!(s, Statement::CountLoop { .. }))
        .count();

    eprintln!(
        "AST structure: {} procedures, {} top-level statements, {} IfElse, {} CountLoop",
        program.procedures.len(),
        all_stmts.len(),
        if_else_count,
        count_loop_count
    );

    assert!(
        if_else_count > 0,
        "amazonMinimum.a3p AST should contain at least one IfElse construct \
         (found {} top-level statements across {} procedures)",
        all_stmts.len(),
        program.procedures.len()
    );
    // Starter project has no loops — the student must add them
    assert_eq!(
        count_loop_count, 0,
        "amazonMinimum.a3p starter AST should NOT contain any CountLoop construct"
    );

    // --- Grading pipeline verification ---
    let report = grade_loops_and_conditionals(loops_input(program));

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps.len(), 7);

    // Precondition steps: Ready
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // No CountLoop → build-counting-loop Blocked
    assert_eq!(report.steps[3].name, "build-counting-loop");
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(
        report.steps[3].reason.contains("No CountLoop found"),
        "reason should explain missing construct: {}",
        report.steps[3].reason
    );

    // Cascade: all downstream steps Blocked
    assert_eq!(report.steps[4].name, "add-conditional-branch");
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    assert!(!report.passed);
}

// ===========================================================================
// Lesson 4: Events & Collision — real .a3p grading
// ===========================================================================

#[test]
fn real_alice_events_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice events grading test (set EATME_REAL_ALICE=1 to enable)");
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

    let report = grade_events_and_collision(events_input(program));

    // --- Schema contract ---
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "events-collision-proximity-game");
    assert_eq!(report.steps.len(), 7);

    // --- Precondition steps: all Ready ---
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // --- AST checks against real starter data ---
    // amazonMinimum.a3p has NO EventListener → add-event-listener = Blocked
    assert_eq!(report.steps[3].name, "add-event-listener");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "starter project has no EventListener — should be Blocked"
    );
    assert!(
        report.steps[3].reason.contains("No EventListener found"),
        "reason should explain missing construct: {}",
        report.steps[3].reason
    );

    // Cascade: add-event-listener blocked → add-collision-listener blocked
    assert_eq!(report.steps[4].name, "add-collision-listener");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "blocked by upstream add-event-listener"
    );

    // Cascade: add-collision-listener blocked → run-world blocked
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);

    // Cascade: run-world blocked → save-project blocked
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    // Overall: not passed
    assert!(!report.passed);
}

// Parser unit tests are in a3p_parser_support.rs (shared test module).

#[test]
fn starter_project_path_uses_alice_home() {
    let path = starter_project_path("amazonMinimum");
    assert!(
        path.to_string_lossy().contains("amazonMinimum.a3p"),
        "path should include project name: {}",
        path.display()
    );
    assert!(
        path.to_string_lossy().contains("starter-projects"),
        "path should include starter-projects directory: {}",
        path.display()
    );
}

// ===========================================================================
// TDD CONTRACTS — Lessons 5-8 grading pipelines
// ===========================================================================
//
// These tests define the expected API and behavior for grading pipelines
// that have NOT been implemented yet. They are gated behind the Cargo feature
// `grading-l5-l8`, which does not currently exist. To activate:
//
//   1. Implement the grading functions + AST extensions in eatme-assets/eatme-core
//   2. Add `grading-l5-l8 = []` to [features] in crates/eatme-alice/Cargo.toml
//   3. Export the new types from eatme-assets lib.rs
//   4. Run: cargo test -p eatme-alice --test real_ast_grading --features grading-l5-l8
//
// Expected new types to implement:
//
//   eatme-core/src/ast.rs:
//     - Statement::VariableDeclaration { name: String, value: String }
//     - Statement::VariableAssignment { name: String, value: String }
//     - Statement::FunctionCall { name: String, arguments: Vec<String> }
//     - Statement::ReturnStatement { value: String }
//     - Procedure { name, parameters: Vec<Parameter>, body }  (add parameters field)
//     - Function { name: String, parameters: Vec<Parameter>, return_type: Option<String>, body: Vec<Statement> }
//     - Parameter { name: String, parameter_type: String }
//     - Program { procedures, functions: Vec<Function> }  (add functions field)
//
//   eatme-assets grading modules:
//     - FunctionsGradingInput + grade_functions() → GradingReport
//     - VariablesGradingInput + grade_variables() → GradingReport
//     - ParametersGradingInput + grade_parameters() → GradingReport
//     - CreativeGradingInput + grade_creative_project() → GradingReport

#[cfg(feature = "grading-l5-l8")]
mod future_pipeline_contracts {
    use super::*;

    // These imports will fail until the types are implemented — that's the TDD contract.
    use eatme_assets::{
        CreativeGradingInput, FunctionsGradingInput, ParametersGradingInput, VariablesGradingInput,
        grade_creative_project, grade_functions, grade_parameters, grade_variables,
    };

    fn functions_input(program: Program) -> FunctionsGradingInput {
        FunctionsGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn variables_input(program: Program) -> VariablesGradingInput {
        VariablesGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn parameters_input(program: Program) -> ParametersGradingInput {
        ParametersGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn creative_input(program: Program) -> CreativeGradingInput {
        CreativeGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    // -----------------------------------------------------------------------
    // Lesson 5: Functions — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   create-function, call-function, run-world, save-project
    // Starter has NO user-defined Function entries → create-function = Blocked

    #[test]
    fn real_alice_functions_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice functions grading test (set EATME_REAL_ALICE=1 to enable)"
            );
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

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has no user-defined functions → create-function = Blocked
        assert_eq!(report.steps[3].name, "create-function");
        assert_eq!(report.steps[3].status, StepStatus::Blocked);

        // Cascade blocks downstream
        assert_eq!(report.steps[4].name, "call-function");
        assert_eq!(report.steps[4].status, StepStatus::Blocked);
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::Blocked);
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Blocked);

        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 6: Variables — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   declare-variable, modify-variable, run-world, save-project
    // Starter has LocalDeclarationStatement → declare-variable = Ready
    // Starter has NO VariableAssignment → modify-variable = Blocked

    #[test]
    fn real_alice_variables_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice variables grading test (set EATME_REAL_ALICE=1 to enable)"
            );
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

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has LocalDeclarationStatement → declare-variable = Ready
        assert_eq!(report.steps[3].name, "declare-variable");
        assert_eq!(
            report.steps[3].status,
            StepStatus::Ready,
            "starter has VariableDeclaration from LocalDeclarationStatement"
        );

        // No VariableAssignment in starter → modify-variable = Blocked
        assert_eq!(report.steps[4].name, "modify-variable");
        assert_eq!(
            report.steps[4].status,
            StepStatus::Blocked,
            "starter has no VariableAssignment"
        );

        // Cascade blocks downstream from modify-variable
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::Blocked);
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Blocked);

        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 7: Parameters — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   add-parameter, pass-argument, run-world, save-project
    // Starter has parameterized procedures → add-parameter = Ready
    // Starter has method calls with args → pass-argument = Ready

    #[test]
    fn real_alice_parameters_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice parameters grading test (set EATME_REAL_ALICE=1 to enable)"
            );
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

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has UserParameter definitions → add-parameter = Ready
        assert_eq!(report.steps[3].name, "add-parameter");
        assert_eq!(
            report.steps[3].status,
            StepStatus::Ready,
            "starter has UserParameter definitions"
        );

        // Starter has MethodInvocations with arguments → pass-argument = Ready
        assert_eq!(report.steps[4].name, "pass-argument");
        assert_eq!(
            report.steps[4].status,
            StepStatus::Ready,
            "starter has MethodInvocations with arguments"
        );

        // Runtime step — requires execution
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::NotYetTested);

        // Save/reopen round-trip
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Ready);

        // Not passed because run-world is not-yet-tested
        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 8: Creative Project — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   build-scene, create-custom-procedure, add-control-structure,
    //   add-event-or-interaction
    // Starter has objects → build-scene = Ready
    // Starter has UserMethod → create-custom-procedure = Ready
    // Starter has IfElse → add-control-structure = Ready
    // Starter has NO events → add-event-or-interaction = Blocked

    #[test]
    fn real_alice_creative_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice creative grading test (set EATME_REAL_ALICE=1 to enable)"
            );
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

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has scene objects → build-scene = Ready
        assert_eq!(report.steps[3].name, "build-scene");
        assert_eq!(report.steps[3].status, StepStatus::Ready);

        // Starter has UserMethod definitions → create-custom-procedure = Ready
        assert_eq!(report.steps[4].name, "create-custom-procedure");
        assert_eq!(report.steps[4].status, StepStatus::Ready);

        // Starter has IfElse → add-control-structure = Ready
        assert_eq!(report.steps[5].name, "add-control-structure");
        assert_eq!(report.steps[5].status, StepStatus::Ready);

        // No events in starter → add-event-or-interaction = Blocked
        assert_eq!(report.steps[6].name, "add-event-or-interaction");
        assert_eq!(
            report.steps[6].status,
            StepStatus::Blocked,
            "starter has no EventListener or CollisionListener"
        );

        assert!(!report.passed);
    }
}
