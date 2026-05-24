//! Variables grading — covers the "Using Variables" curriculum lesson.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{build_preconditions, cascade_blocked, no_program_chain};

pub struct VariablesGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

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
        lesson: "using-variables-mini-challenge".into(),
        passed,
        steps,
    }
}

#[derive(Clone)]
struct VariableCandidate {
    name: String,
    var_type: String,
    used_after_declaration: bool,
    type_appropriate_for_usage: bool,
    assignment_changes_value: bool,
}

impl VariableCandidate {
    fn score(&self) -> u8 {
        self.used_after_declaration as u8
            + self.type_appropriate_for_usage as u8
            + self.assignment_changes_value as u8
    }
}

fn evaluate_variables_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("declare-variable", "launch-smoke"),
            ("use-variable-in-method", "declare-variable"),
            ("modify-variable", "use-variable-in-method"),
            ("run-world", "modify-variable"),
            ("save-project", "run-world"),
        ]);
    };

    let candidates = analyze_variables(program);

    let declare_variable = if candidates.is_empty() {
        blocked_step(
            "declare-variable",
            &["launch-smoke"],
            "No variable declaration found in student program",
        )
    } else {
        ready_step(
            "declare-variable",
            &["launch-smoke"],
            "Variable declaration found in student program",
        )
    };

    let use_variable = if declare_variable.status == StepStatus::Blocked {
        cascade_blocked("use-variable-in-method", &["declare-variable"])
    } else if has_live_typed_variable(&candidates) {
        ready_step(
            "use-variable-in-method",
            &["declare-variable"],
            "Variable is used after declaration with a type that matches its method-call usage",
        )
    } else {
        let candidate = best_candidate(&candidates).expect("variable candidate exists");
        let reason = if !candidate.used_after_declaration {
            format!(
                "Variable `{}` is declared but never used after declaration",
                candidate.name
            )
        } else {
            format!(
                "Variable `{}` has type `{}` that does not match its method-call usage",
                candidate.name, candidate.var_type
            )
        };
        blocked_step("use-variable-in-method", &["declare-variable"], &reason)
    };

    let modify_variable = if use_variable.status == StepStatus::Blocked {
        cascade_blocked("modify-variable", &["use-variable-in-method"])
    } else if live_typed_variable_changes_value(&candidates) {
        ready_step(
            "modify-variable",
            &["use-variable-in-method"],
            "Variable assignment changes the variable's value after it is used",
        )
    } else {
        let candidate =
            best_live_typed_candidate(&candidates).expect("live typed candidate exists");
        let reason = format!(
            "Variable `{}` is assigned, but its value never changes from the initial declaration",
            candidate.name
        );
        blocked_step("modify-variable", &["use-variable-in-method"], &reason)
    };

    let run_world = if modify_variable.status == StepStatus::Blocked {
        cascade_blocked("run-world", &["modify-variable"])
    } else {
        ready_step(
            "run-world",
            &["modify-variable"],
            "Variable grading found a complete, state-changing solution",
        )
    };

    let save_project = if run_world.status == StepStatus::Blocked {
        cascade_blocked("save-project", &["run-world"])
    } else {
        let round_trip_ok = serde_json::to_vec(program)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Program>(&bytes).ok())
            .is_some_and(|restored| restored == *program);
        if round_trip_ok {
            ready_step(
                "save-project",
                &["run-world"],
                "Program round-trip (serialize → deserialize → compare) verified",
            )
        } else {
            blocked_step(
                "save-project",
                &["run-world"],
                "Program failed round-trip verification",
            )
        }
    };

    vec![
        declare_variable,
        use_variable,
        modify_variable,
        run_world,
        save_project,
    ]
}

fn analyze_variables(program: &Program) -> Vec<VariableCandidate> {
    let mut candidates = Vec::new();

    for procedure in &program.procedures {
        let linear = collect_linear_statements(&procedure.body);
        for (index, statement) in linear.iter().enumerate() {
            if let Statement::VariableDeclaration {
                name,
                var_type,
                initial_value,
            } = statement
            {
                candidates.push(analyze_variable_candidate(
                    &linear,
                    index,
                    name,
                    var_type,
                    initial_value,
                ));
            }
        }
    }

    candidates
}

fn analyze_variable_candidate(
    linear: &[Statement],
    declaration_index: usize,
    name: &str,
    var_type: &str,
    initial_value: &str,
) -> VariableCandidate {
    let mut used_after_declaration = false;
    let mut type_appropriate_for_usage = true;
    let mut assignment_changes_value = false;
    let mut saw_recognized_usage = false;
    let mut current_value = initial_value.to_string();

    for statement in linear.iter().skip(declaration_index + 1) {
        match statement {
            Statement::MethodCall {
                method, arguments, ..
            } => {
                for (index, argument) in arguments.iter().enumerate() {
                    if argument != name {
                        continue;
                    }

                    used_after_declaration = true;
                    if let Some(expected_kind) = expected_argument_kind(method, index) {
                        saw_recognized_usage = true;
                        if !variable_type_matches(var_type, expected_kind) {
                            type_appropriate_for_usage = false;
                        }
                    }
                }
            }
            Statement::VariableAssignment {
                name: assigned_name,
                value,
            } if assigned_name == name => {
                if value != &current_value {
                    assignment_changes_value = true;
                }
                current_value = value.clone();
            }
            _ => {}
        }
    }

    if !used_after_declaration {
        type_appropriate_for_usage = false;
    } else if !saw_recognized_usage {
        type_appropriate_for_usage = true;
    }

    VariableCandidate {
        name: name.into(),
        var_type: var_type.into(),
        used_after_declaration,
        type_appropriate_for_usage,
        assignment_changes_value,
    }
}

fn collect_linear_statements(statements: &[Statement]) -> Vec<Statement> {
    let mut linear = Vec::new();
    collect_linear_statements_into(statements, &mut linear);
    linear
}

fn collect_linear_statements_into(statements: &[Statement], linear: &mut Vec<Statement>) {
    for statement in statements {
        linear.push(statement.clone());
        match statement {
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => {
                collect_linear_statements_into(body, linear)
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_linear_statements_into(if_body, linear);
                collect_linear_statements_into(else_body, linear);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_linear_statements_into(&method.body, linear);
                }
            }
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => {}
        }
    }
}

fn has_live_typed_variable(candidates: &[VariableCandidate]) -> bool {
    candidates
        .iter()
        .any(|candidate| candidate.used_after_declaration && candidate.type_appropriate_for_usage)
}

fn live_typed_variable_changes_value(candidates: &[VariableCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.used_after_declaration
            && candidate.type_appropriate_for_usage
            && candidate.assignment_changes_value
    })
}

fn best_candidate(candidates: &[VariableCandidate]) -> Option<&VariableCandidate> {
    candidates.iter().max_by_key(|candidate| candidate.score())
}

fn best_live_typed_candidate(candidates: &[VariableCandidate]) -> Option<&VariableCandidate> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.used_after_declaration && candidate.type_appropriate_for_usage
        })
        .max_by_key(|candidate| candidate.assignment_changes_value as u8)
}

fn expected_argument_kind(method: &str, index: usize) -> Option<&'static str> {
    let lower = method.to_ascii_lowercase();
    match (lower.as_str(), index) {
        ("move", 1) | ("turn", 1) => Some("number"),
        ("say", 0) => Some("text"),
        _ => None,
    }
}

fn variable_type_matches(var_type: &str, expected_kind: &str) -> bool {
    let lower = var_type.to_ascii_lowercase();
    match expected_kind {
        "number" => {
            lower.contains("number") || lower.contains("decimal") || lower.contains("float")
        }
        "text" => lower.contains("string") || lower.contains("text"),
        _ => false,
    }
}

fn ready_step(name: &str, deps: &[&str], reason: &str) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Ready,
        reason: reason.into(),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

fn blocked_step(name: &str, deps: &[&str], reason: &str) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: reason.into(),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}
