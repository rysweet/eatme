//! Nested control flow and relational-expression grading.

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{ast_check_step, build_preconditions, cascade_blocked};

pub struct NestedControlFlowGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_nested_control_flow(input: NestedControlFlowGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("detect-relational-expressions", &["launch-smoke"]),
            cascade_blocked("grade-basic-nesting", &["detect-relational-expressions"]),
            cascade_blocked("grade-intermediate-nesting", &["grade-basic-nesting"]),
            cascade_blocked("grade-advanced-nesting", &["grade-intermediate-nesting"]),
        ]
    } else {
        evaluate_nested_control_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .find(|step| step.name == "detect-relational-expressions")
            .is_some_and(|step| step.status == StepStatus::Ready)
        && interaction_steps
            .iter()
            .find(|step| step.name == "grade-basic-nesting")
            .is_some_and(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "nested-control-flow-relational-expressions",
        passed,
        steps,
    )
}

fn evaluate_nested_control_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return vec![
            missing_program_step("detect-relational-expressions", &["launch-smoke"]),
            missing_program_step("grade-basic-nesting", &["detect-relational-expressions"]),
            missing_program_step("grade-intermediate-nesting", &["grade-basic-nesting"]),
            missing_program_step("grade-advanced-nesting", &["grade-intermediate-nesting"]),
        ];
    };

    let evidence = collect_nested_control_evidence(program);
    let has_all_relational = evidence.has_less_than
        && evidence.has_greater_than
        && evidence.has_equals
        && evidence.has_and
        && evidence.has_or;

    vec![
        ast_check_step(
            "detect-relational-expressions",
            "launch-smoke",
            has_all_relational,
            "less than, greater than, equals, and, or relational expressions",
        ),
        ast_check_step(
            "grade-basic-nesting",
            "detect-relational-expressions",
            evidence.max_depth >= 1,
            "basic nesting depth",
        ),
        ast_check_step(
            "grade-intermediate-nesting",
            "grade-basic-nesting",
            evidence.max_depth >= 2,
            "intermediate nesting depth",
        ),
        ast_check_step(
            "grade-advanced-nesting",
            "grade-intermediate-nesting",
            evidence.max_depth >= 3,
            "advanced nesting depth",
        ),
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
struct NestedControlEvidence {
    max_depth: usize,
    has_less_than: bool,
    has_greater_than: bool,
    has_equals: bool,
    has_and: bool,
    has_or: bool,
}

fn collect_nested_control_evidence(program: &Program) -> NestedControlEvidence {
    let mut evidence = NestedControlEvidence::default();
    for procedure in &program.procedures {
        scan_nested_control(&procedure.body, 0, &mut evidence);
    }
    for function in &program.functions {
        scan_nested_control(&function.body, 0, &mut evidence);
    }
    evidence
}

fn scan_nested_control(stmts: &[Statement], depth: usize, evidence: &mut NestedControlEvidence) {
    for stmt in stmts {
        match stmt {
            Statement::CountLoop { body, .. } | Statement::ForEachArray { body, .. } => {
                let next_depth = depth + 1;
                evidence.max_depth = evidence.max_depth.max(next_depth);
                scan_nested_control(body, next_depth, evidence);
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                let next_depth = depth + 1;
                evidence.max_depth = evidence.max_depth.max(next_depth);
                update_relational_evidence(condition, evidence);
                scan_nested_control(if_body, next_depth, evidence);
                scan_nested_control(else_body, next_depth, evidence);
            }
            Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::DoInOrder { body } => scan_nested_control(body, depth, evidence),
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    scan_nested_control(&method.body, depth, evidence);
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

fn update_relational_evidence(condition: &str, evidence: &mut NestedControlEvidence) {
    let normalized = condition.to_ascii_lowercase();
    evidence.has_less_than |= normalized.contains("<") || normalized.contains("less than");
    evidence.has_greater_than |= normalized.contains(">") || normalized.contains("greater than");
    evidence.has_equals |= normalized.contains("==")
        || normalized.contains("equals")
        || normalized.contains("equal to");
    evidence.has_and |= normalized.contains("&&") || normalized.contains(" and ");
    evidence.has_or |= normalized.contains("||") || normalized.contains(" or ");
}
