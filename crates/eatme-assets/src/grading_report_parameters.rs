//! Parameters grading — covers the "Parameters" curriculum lesson.

use eatme_core::ast::{Parameter, Procedure, Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{build_preconditions, cascade_blocked, no_program_chain};

pub struct ParametersGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

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
        lesson: "parameters-mini-challenge".into(),
        passed,
        steps,
    }
}

#[derive(Clone, Default)]
struct ParameterizedProcedureCandidate {
    called_with_arguments: bool,
    valid_entity_targets: bool,
    sensible_execution_order: bool,
}

impl ParameterizedProcedureCandidate {
    fn score(&self) -> u8 {
        self.called_with_arguments as u8
            + self.valid_entity_targets as u8
            + self.sensible_execution_order as u8
    }

    fn is_complete(&self) -> bool {
        self.called_with_arguments && self.valid_entity_targets && self.sensible_execution_order
    }
}

#[derive(Default)]
struct ParametersEvidence {
    has_parameterized_procedure: bool,
    typed_candidates: Vec<ParameterizedProcedureCandidate>,
}

fn evaluate_parameters_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("create-parameterized-procedure", "launch-smoke"),
            ("call-with-argument", "create-parameterized-procedure"),
            ("run-world", "call-with-argument"),
            ("save-project", "run-world"),
        ]);
    };

    let evidence = analyze_parameters(program);

    let create_parameterized = if evidence.has_typed_procedure() {
        ready_step(
            "create-parameterized-procedure",
            &["launch-smoke"],
            "Parameterized procedure uses lesson-appropriate parameter types",
        )
    } else {
        let reason = if evidence.has_parameterized_procedure {
            "Parameterized procedure parameters do not match their motion-method usage"
        } else {
            "No parameterized procedure found in student program"
        };
        blocked_step("create-parameterized-procedure", &["launch-smoke"], reason)
    };

    let call_with_argument = if create_parameterized.status == StepStatus::Blocked {
        cascade_blocked("call-with-argument", &["create-parameterized-procedure"])
    } else if evidence.has_complete_candidate() {
        ready_step(
            "call-with-argument",
            &["create-parameterized-procedure"],
            "Parameterized procedure is called with arguments, targets the right entity type, and moves before turning",
        )
    } else {
        let candidate = evidence
            .best_typed_candidate()
            .expect("typed candidate exists");
        let reason = if !candidate.called_with_arguments {
            "No call passes arguments to the parameterized procedure"
        } else if !candidate.valid_entity_targets {
            "Motion methods should target SBiped-style objects, not scene-like objects"
        } else {
            "Procedure should move before turning so execution order is sensible"
        };
        blocked_step(
            "call-with-argument",
            &["create-parameterized-procedure"],
            reason,
        )
    };

    let run_world = if call_with_argument.status == StepStatus::Blocked {
        cascade_blocked("run-world", &["call-with-argument"])
    } else {
        ready_step(
            "run-world",
            &["call-with-argument"],
            "Parameterized procedure grading found a complete, runnable solution",
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
        create_parameterized,
        call_with_argument,
        run_world,
        save_project,
    ]
}

fn analyze_parameters(program: &Program) -> ParametersEvidence {
    let mut evidence = ParametersEvidence::default();

    for procedure in &program.procedures {
        if procedure.parameters.is_empty() {
            continue;
        }

        evidence.has_parameterized_procedure = true;

        let method_calls = collect_method_calls(&procedure.body);
        if !parameter_types_match_usage(&procedure.parameters, &method_calls) {
            continue;
        }

        evidence
            .typed_candidates
            .push(ParameterizedProcedureCandidate {
                called_with_arguments: program
                    .procedures
                    .iter()
                    .any(|caller| procedure_called_with_arguments(caller, procedure)),
                valid_entity_targets: motion_calls_target_biped_like_objects(&method_calls),
                sensible_execution_order: motion_calls_have_sensible_order(&method_calls),
            });
    }

    evidence
}

impl ParametersEvidence {
    fn has_typed_procedure(&self) -> bool {
        !self.typed_candidates.is_empty()
    }

    fn has_complete_candidate(&self) -> bool {
        self.typed_candidates
            .iter()
            .any(ParameterizedProcedureCandidate::is_complete)
    }

    fn best_typed_candidate(&self) -> Option<&ParameterizedProcedureCandidate> {
        self.typed_candidates
            .iter()
            .max_by_key(|candidate| candidate.score())
    }
}

#[derive(Clone)]
struct MethodCallEvidence {
    object: String,
    method: String,
    arguments: Vec<String>,
}

fn collect_method_calls(statements: &[Statement]) -> Vec<MethodCallEvidence> {
    let mut calls = Vec::new();
    collect_method_calls_into(statements, &mut calls);
    calls
}

fn collect_method_calls_into(statements: &[Statement], calls: &mut Vec<MethodCallEvidence>) {
    for statement in statements {
        match statement {
            Statement::MethodCall {
                object,
                method,
                arguments,
            } => calls.push(MethodCallEvidence {
                object: object.clone(),
                method: method.clone(),
                arguments: arguments.clone(),
            }),
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => collect_method_calls_into(body, calls),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_method_calls_into(if_body, calls);
                collect_method_calls_into(else_body, calls);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_method_calls_into(&method.body, calls);
                }
            }
            Statement::ReturnStatement { .. }
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

fn parameter_types_match_usage(
    parameters: &[Parameter],
    method_calls: &[MethodCallEvidence],
) -> bool {
    let mut saw_recognized_usage = false;

    for parameter in parameters {
        for call in method_calls {
            for (index, argument) in call.arguments.iter().enumerate() {
                if argument != &parameter.name {
                    continue;
                }

                if let Some(expected_kind) = expected_argument_kind(&call.method, index) {
                    saw_recognized_usage = true;
                    if !parameter_type_matches(&parameter.param_type, expected_kind) {
                        return false;
                    }
                }
            }
        }
    }

    saw_recognized_usage
}

fn procedure_called_with_arguments(caller: &Procedure, callee: &Procedure) -> bool {
    collect_method_calls(&caller.body).into_iter().any(|call| {
        call.method == callee.name
            && !call.arguments.is_empty()
            && call.arguments.len() == callee.parameters.len()
    })
}

fn motion_calls_target_biped_like_objects(method_calls: &[MethodCallEvidence]) -> bool {
    let motion_calls: Vec<&MethodCallEvidence> = method_calls
        .iter()
        .filter(|call| is_motion_method(&call.method))
        .collect();

    !motion_calls.is_empty()
        && motion_calls
            .iter()
            .all(|call| is_biped_like_object(&call.object))
}

fn motion_calls_have_sensible_order(method_calls: &[MethodCallEvidence]) -> bool {
    let mut saw_move = false;

    for call in method_calls
        .iter()
        .filter(|call| is_motion_method(&call.method))
    {
        if is_move_method(&call.method) {
            saw_move = true;
            continue;
        }

        if is_turn_method(&call.method) && !saw_move {
            return false;
        }
    }

    saw_move
}

fn expected_argument_kind(method: &str, index: usize) -> Option<&'static str> {
    let lower = method.to_ascii_lowercase();
    match (lower.as_str(), index) {
        ("move", 1) | ("turn", 1) => Some("number"),
        ("say", 0) => Some("text"),
        _ => None,
    }
}

fn parameter_type_matches(param_type: &str, expected_kind: &str) -> bool {
    let lower = param_type.to_ascii_lowercase();
    match expected_kind {
        "number" => {
            lower.contains("number") || lower.contains("decimal") || lower.contains("float")
        }
        "text" => lower.contains("string") || lower.contains("text"),
        _ => false,
    }
}

fn is_motion_method(method: &str) -> bool {
    is_move_method(method) || is_turn_method(method)
}

fn is_move_method(method: &str) -> bool {
    method.eq_ignore_ascii_case("move")
}

fn is_turn_method(method: &str) -> bool {
    method.eq_ignore_ascii_case("turn")
}

fn is_biped_like_object(object: &str) -> bool {
    let lower = object.to_ascii_lowercase();
    !(lower == "this" || lower.contains("scene") || lower.contains("camera"))
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
