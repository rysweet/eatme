//! Variables grading — grade student variable declarations, usage in method calls,
//! and variable modification.
//!
//! 8-step pipeline: 3 preconditions + declare-variable, use-variable-in-method,
//! modify-variable, run-world, save-project.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};

/// Input struct for variables grading.
pub struct VariablesGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

/// Grade a student's variables lesson attempt.
pub fn grade_variables(input: VariablesGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("declare-variable", &["launch-smoke"]),
            cascade_blocked("use-variable-in-method", &["declare-variable"]),
            cascade_blocked("modify-variable", &["use-variable-in-method"]),
            cascade_blocked("run-world", &["modify-variable"]),
            cascade_blocked("save-project", &["run-world"]),
        ]
    } else {
        evaluate_variables_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "variables-scorekeeper-timekeeper".into(),
        passed,
        steps,
    }
}

fn evaluate_variables_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let program = match program {
        Some(p) => p,
        None => {
            return no_program_chain(&[
                ("declare-variable", "launch-smoke"),
                ("use-variable-in-method", "declare-variable"),
                ("modify-variable", "use-variable-in-method"),
                ("run-world", "modify-variable"),
                ("save-project", "run-world"),
            ]);
        }
    };

    let has_declaration = !program.variable_declarations.is_empty();
    let var_names: Vec<&str> = program
        .variable_declarations
        .iter()
        .map(|v| v.name.as_str())
        .collect();
    let has_usage = program.procedures.iter().any(|p| {
        p.body.iter().any(|s| match s {
            Statement::MethodCall { arguments, .. } => arguments
                .iter()
                .any(|arg| var_names.contains(&arg.as_str())),
            _ => false,
        })
    });
    let has_assignment = program.procedures.iter().any(|p| {
        p.body
            .iter()
            .any(|s| matches!(s, Statement::VariableAssignment { .. }))
    });

    let declare_var = ast_check_step(
        "declare-variable",
        "launch-smoke",
        has_declaration,
        "VariableDeclaration",
    );
    let declare_blocked = declare_var.status == StepStatus::Blocked;

    let use_var = if declare_blocked {
        cascade_blocked("use-variable-in-method", &["declare-variable"])
    } else {
        let (status, reason) = if has_usage {
            (
                StepStatus::Ready,
                "variable used in method found in student program".into(),
            )
        } else {
            (
                StepStatus::Blocked,
                "No variable used in method found in student program".into(),
            )
        };
        StepGrade {
            name: "use-variable-in-method".into(),
            status,
            reason,
            depends_on: vec!["declare-variable".into()],
        }
    };
    let use_blocked = use_var.status == StepStatus::Blocked;

    let modify_var = if use_blocked {
        cascade_blocked("modify-variable", &["use-variable-in-method"])
    } else {
        ast_check_step(
            "modify-variable",
            "use-variable-in-method",
            has_assignment,
            "VariableAssignment",
        )
    };
    let modify_blocked = modify_var.status == StepStatus::Blocked;

    let run_world = if modify_blocked {
        cascade_blocked("run-world", &["modify-variable"])
    } else {
        StepGrade {
            name: "run-world".into(),
            status: StepStatus::NotYetTested,
            reason: "Run the world and observe results — requires human interaction".into(),
            depends_on: vec!["modify-variable".into()],
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

    vec![declare_var, use_var, modify_var, run_world, save_project]
}

#[cfg(test)]
#[path = "grading_report_variables_tests.rs"]
mod variables_tests;
