//! Games and interactive narrative grading.
//!
//! Detects event-driven game mechanics (events, collisions, state tracking)
//! alongside narrative sequencing evidence (DoInOrder + dialogue-like beats).

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{ast_check_step, build_preconditions, cascade_blocked};

pub struct GamesNarrativeGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_games_and_narrative(input: GamesNarrativeGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("detect-event-listener", &["launch-smoke"]),
            cascade_blocked("detect-collision-handler", &["detect-event-listener"]),
            cascade_blocked("detect-state-tracking", &["detect-collision-handler"]),
            cascade_blocked("detect-game-loop-pattern", &["detect-state-tracking"]),
            cascade_blocked("grade-game-project", &["detect-game-loop-pattern"]),
            cascade_blocked("detect-do-in-order", &["launch-smoke"]),
            cascade_blocked("detect-dialogue-sequence", &["detect-do-in-order"]),
            cascade_blocked("grade-narrative-project", &["detect-dialogue-sequence"]),
        ]
    } else {
        evaluate_games_narrative_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps.iter().any(|step| {
            matches!(
                (step.name.as_str(), &step.status),
                ("grade-game-project", StepStatus::Ready)
                    | ("grade-narrative-project", StepStatus::Ready)
            )
        });

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "games-and-interactive-narrative",
        passed,
        steps,
    )
}

fn evaluate_games_narrative_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return vec![
            missing_program_step("detect-event-listener", &["launch-smoke"]),
            missing_program_step("detect-collision-handler", &["detect-event-listener"]),
            missing_program_step("detect-state-tracking", &["detect-collision-handler"]),
            missing_program_step("detect-game-loop-pattern", &["detect-state-tracking"]),
            missing_program_step("grade-game-project", &["detect-game-loop-pattern"]),
            missing_program_step("detect-do-in-order", &["launch-smoke"]),
            missing_program_step("detect-dialogue-sequence", &["detect-do-in-order"]),
            missing_program_step("grade-narrative-project", &["detect-dialogue-sequence"]),
        ];
    };

    let evidence = collect_games_narrative_evidence(program);
    let passes_safety_checks = !evidence.has_circular_event_reference;
    let is_game_project = evidence.has_event
        && evidence.has_collision
        && evidence.has_state_tracking
        && passes_safety_checks;
    let is_narrative_project =
        evidence.has_do_in_order && evidence.has_dialogue_sequence && passes_safety_checks;

    vec![
        ast_check_step(
            "detect-event-listener",
            "launch-smoke",
            evidence.has_event,
            "event listener",
        ),
        ast_check_step(
            "detect-collision-handler",
            "detect-event-listener",
            evidence.has_collision,
            "collision handler",
        ),
        ast_check_step(
            "detect-state-tracking",
            "detect-collision-handler",
            evidence.has_state_tracking,
            "state tracking (variable declaration or assignment)",
        ),
        ast_check_step(
            "detect-game-loop-pattern",
            "detect-state-tracking",
            evidence.has_game_loop_pattern,
            "event → condition → action pattern",
        ),
        if evidence.has_circular_event_reference {
            StepGrade {
                name: "grade-game-project".into(),
                status: StepStatus::Blocked,
                reason: "Circular event references found in student program".into(),
                depends_on: vec!["detect-game-loop-pattern".into()],
            }
        } else {
            ast_check_step(
                "grade-game-project",
                "detect-game-loop-pattern",
                is_game_project,
                "game project evidence (events + collision + state tracking)",
            )
        },
        ast_check_step(
            "detect-do-in-order",
            "launch-smoke",
            evidence.has_do_in_order,
            "DoInOrder sequence",
        ),
        ast_check_step(
            "detect-dialogue-sequence",
            "detect-do-in-order",
            evidence.has_dialogue_sequence,
            "dialogue-like sequence",
        ),
        if evidence.has_circular_event_reference {
            StepGrade {
                name: "grade-narrative-project".into(),
                status: StepStatus::Blocked,
                reason: "Circular event references found in student program".into(),
                depends_on: vec!["detect-dialogue-sequence".into()],
            }
        } else {
            ast_check_step(
                "grade-narrative-project",
                "detect-dialogue-sequence",
                is_narrative_project,
                "narrative project evidence (DoInOrder + dialogue)",
            )
        },
    ]
}

fn missing_program_step(name: &str, deps: &[&str]) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: "No student program provided".into(),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

#[derive(Default)]
struct GamesNarrativeEvidence {
    has_event: bool,
    has_collision: bool,
    has_state_tracking: bool,
    has_game_loop_pattern: bool,
    has_do_in_order: bool,
    has_dialogue_sequence: bool,
    has_circular_event_reference: bool,
}

fn collect_games_narrative_evidence(program: &Program) -> GamesNarrativeEvidence {
    let mut evidence = GamesNarrativeEvidence::default();
    for procedure in &program.procedures {
        scan_games_narrative_statements(&procedure.body, &mut evidence);
    }
    for function in &program.functions {
        scan_games_narrative_statements(&function.body, &mut evidence);
    }
    evidence.has_circular_event_reference = has_circular_event_reference(program);
    evidence
}

fn scan_games_narrative_statements(stmts: &[Statement], evidence: &mut GamesNarrativeEvidence) {
    for stmt in stmts {
        match stmt {
            Statement::EventListener { body, .. } => {
                evidence.has_event = true;
                evidence.has_game_loop_pattern |= contains_condition_action(body);
                scan_games_narrative_statements(body, evidence);
            }
            Statement::CollisionListener { body, .. } => {
                evidence.has_collision = true;
                evidence.has_game_loop_pattern |= contains_condition_action(body);
                scan_games_narrative_statements(body, evidence);
            }
            Statement::DoInOrder { body } => {
                evidence.has_do_in_order = true;
                evidence.has_dialogue_sequence |= count_dialogue_actions(body) >= 2;
                scan_games_narrative_statements(body, evidence);
            }
            Statement::CountLoop { body, .. } | Statement::ForEachArray { body, .. } => {
                scan_games_narrative_statements(body, evidence)
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                scan_games_narrative_statements(if_body, evidence);
                scan_games_narrative_statements(else_body, evidence);
            }
            Statement::VariableDeclaration { .. } | Statement::VariableAssignment { .. } => {
                evidence.has_state_tracking = true;
            }
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. }
            | Statement::UserTypeDeclaration { .. } => {}
        }
    }
}

fn has_circular_event_reference(program: &Program) -> bool {
    program
        .procedures
        .iter()
        .any(|procedure| find_circular_event_reference(&procedure.body, &mut Vec::new()))
        || program
            .functions
            .iter()
            .any(|function| find_circular_event_reference(&function.body, &mut Vec::new()))
}

fn find_circular_event_reference(stmts: &[Statement], active_events: &mut Vec<String>) -> bool {
    for stmt in stmts {
        match stmt {
            Statement::EventListener { event, body } => {
                if active_events.iter().any(|active| active == event) {
                    return true;
                }
                active_events.push(event.clone());
                let has_cycle = find_circular_event_reference(body, active_events);
                active_events.pop();
                if has_cycle {
                    return true;
                }
            }
            Statement::CollisionListener { body, .. }
            | Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => {
                if find_circular_event_reference(body, active_events) {
                    return true;
                }
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                if find_circular_event_reference(if_body, active_events)
                    || find_circular_event_reference(else_body, active_events)
                {
                    return true;
                }
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                if methods
                    .iter()
                    .any(|method| find_circular_event_reference(&method.body, active_events))
                {
                    return true;
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
    false
}

fn contains_condition_action(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::IfElse {
            if_body, else_body, ..
        } => branch_contains_action(if_body) || branch_contains_action(else_body),
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => contains_condition_action(body),
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. }
        | Statement::UserTypeDeclaration { .. } => false,
    })
}

fn branch_contains_action(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::MethodCall { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableAssignment { .. }
        | Statement::CollisionListener { .. }
        | Statement::EventListener { .. }
        | Statement::DoInOrder { .. } => true,
        Statement::CountLoop { body, .. } | Statement::ForEachArray { body, .. } => {
            branch_contains_action(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => branch_contains_action(if_body) || branch_contains_action(else_body),
        Statement::ReturnStatement { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. }
        | Statement::UserTypeDeclaration { .. } => false,
    })
}

fn count_dialogue_actions(stmts: &[Statement]) -> usize {
    stmts
        .iter()
        .map(|stmt| match stmt {
            Statement::MethodCall {
                method, arguments, ..
            } if matches!(method.as_str(), "say" | "think")
                || arguments
                    .iter()
                    .any(|arg| arg.starts_with('"') && arg.ends_with('"')) =>
            {
                1
            }
            Statement::DoInOrder { body }
            | Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::ForEachArray { body, .. } => count_dialogue_actions(body),
            Statement::IfElse {
                if_body, else_body, ..
            } => count_dialogue_actions(if_body) + count_dialogue_actions(else_body),
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. }
            | Statement::UserTypeDeclaration { .. } => 0,
        })
        .sum()
}
