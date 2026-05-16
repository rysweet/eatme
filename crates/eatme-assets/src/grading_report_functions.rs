//! Functions grading — grade student function definitions, return statements,
//! and function calls from procedures.
//!
//! 8-step pipeline: 3 preconditions + create-function, add-return-statement,
//! call-function-from-procedure, run-world, save-project.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

/// Input struct for functions grading.
pub struct FunctionsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

/// Grade a student's functions lesson attempt.
pub fn grade_functions(input: FunctionsGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-function", &["launch-smoke"]),
            cascade_blocked("add-return-statement", &["create-function"]),
            cascade_blocked("call-function-from-procedure", &["add-return-statement"]),
            cascade_blocked("run-world", &["call-function-from-procedure"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_functions_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "functions-mini-challenge".into(),
        passed,
        steps,
    }
}

fn evaluate_functions_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => {
            return no_program_chain(&[
                ("create-function", "launch-smoke"),
                ("add-return-statement", "create-function"),
                ("call-function-from-procedure", "add-return-statement"),
                ("run-world", "call-function-from-procedure"),
                ("save-project", "run-world"),
            ]);
        }
    };

    let has_function = !program.functions.is_empty();
    let has_return = program.functions.iter().any(|f| {
        f.body
            .iter()
            .any(|s| matches!(s, Statement::ReturnStatement { .. }))
    });
    let has_call = program.procedures.iter().any(|p| {
        p.body
            .iter()
            .any(|s| matches!(s, Statement::FunctionCall { .. }))
    });

    let create_function =
        ast_check_step("create-function", "launch-smoke", has_function, "Function");
    let function_blocked = create_function.status == StepStatus::Blocked;

    let add_return = if function_blocked {
        cascade_blocked("add-return-statement", &["create-function"])
    } else {
        ast_check_step(
            "add-return-statement",
            "create-function",
            has_return,
            "ReturnStatement",
        )
    };
    let return_blocked = add_return.status == StepStatus::Blocked;

    let call_function = if return_blocked {
        cascade_blocked("call-function-from-procedure", &["add-return-statement"])
    } else {
        ast_check_step(
            "call-function-from-procedure",
            "add-return-statement",
            has_call,
            "FunctionCall",
        )
    };
    let call_blocked = call_function.status == StepStatus::Blocked;

    let run_world = if call_blocked {
        cascade_blocked("run-world", &["call-function-from-procedure"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["call-function-from-procedure".into()],
        }
    };

    let run_world_blocked = run_world.status == StepStatus::Blocked;

    let save_project = if run_world_blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        let round_trip_ok = serde_json::to_vec(program)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Program>(&bytes).ok())
            .is_some_and(|restored| restored == *program);
        let status = if round_trip_ok {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        };
        let reason = if round_trip_ok {
            "Program round-trip (serialize → deserialize → compare) verified"
        } else {
            "Program failed round-trip verification"
        };
        StepGrade {
            name: "save-project".into(),
            status,
            reason: reason.into(),
            depends_on: vec!["run-world".into()],
        }
    };

    vec![
        create_function,
        add_return,
        call_function,
        run_world,
        save_project,
    ]
}

#[cfg(test)]
#[path = "grading_report_functions_tests.rs"]
mod functions_tests;
