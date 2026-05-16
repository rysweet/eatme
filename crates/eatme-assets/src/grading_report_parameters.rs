//! Parameters grading — grade parameterized procedure creation and invocation
//! with arguments.
//!
//! 7-step pipeline: 3 preconditions + create-parameterized-procedure,
//! call-with-argument, run-world, save-project.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{build_preconditions, cascade_blocked, no_program_chain};

/// Input struct for parameters grading.
pub struct ParametersGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

/// Grade a student's parameters lesson attempt.
pub fn grade_parameters(input: ParametersGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("create-parameterized-procedure", &["launch-smoke"]),
            cascade_blocked("call-with-argument", &["create-parameterized-procedure"]),
            cascade_blocked("run-world", &["call-with-argument"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_parameters_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "parameters-procedure-generalization".into(),
        passed,
        steps,
    }
}

fn evaluate_parameters_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => {
            return no_program_chain(&[
                ("create-parameterized-procedure", "launch-smoke"),
                ("call-with-argument", "create-parameterized-procedure"),
                ("run-world", "call-with-argument"),
                ("save-project", "run-world"),
            ]);
        }
    };

    let parameterized_proc_names: Vec<&str> = program
        .procedures
        .iter()
        .filter(|p| !p.parameters.is_empty())
        .map(|p| p.name.as_str())
        .collect();
    let has_param_proc = !parameterized_proc_names.is_empty();

    // Check if any procedure calls a parameterized procedure with arguments
    let has_call_with_arg = program.procedures.iter().any(|p| {
        p.body.iter().any(|s| match s {
            Statement::MethodCall {
                method, arguments, ..
            } => parameterized_proc_names.contains(&method.as_str()) && !arguments.is_empty(),
            _ => false,
        })
    });

    let create_param = {
        let (status, reason) = if has_param_proc {
            (
                StepStatus::Ready,
                "parameterized procedure found in student program".into(),
            )
        } else {
            (
                StepStatus::Blocked,
                "No parameterized procedure found in student program".into(),
            )
        };
        StepGrade {
            name: "create-parameterized-procedure".into(),
            status,
            reason,
            depends_on: vec!["launch-smoke".into()],
        }
    };
    let param_blocked = create_param.status == StepStatus::Blocked;

    let call_arg = if param_blocked {
        cascade_blocked("call-with-argument", &["create-parameterized-procedure"])
    } else {
        let (status, reason) = if has_call_with_arg {
            (
                StepStatus::Ready,
                "call with argument found in student program".into(),
            )
        } else {
            (
                StepStatus::Blocked,
                "No call with argument found in student program".into(),
            )
        };
        StepGrade {
            name: "call-with-argument".into(),
            status,
            reason,
            depends_on: vec!["create-parameterized-procedure".into()],
        }
    };
    let call_blocked = call_arg.status == StepStatus::Blocked;

    let run_world = if call_blocked {
        cascade_blocked("run-world", &["call-with-argument"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["call-with-argument".into()],
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

    vec![create_param, call_arg, run_world, save_project]
}

#[cfg(test)]
#[path = "grading_report_parameters_tests.rs"]
mod parameters_tests;
