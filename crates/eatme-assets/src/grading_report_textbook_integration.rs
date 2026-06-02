//! Textbook-integration grading — covers the Alice.org "Textbook Integration" lesson area.
//!
//! The goal is to show how Alice constructs map onto Java textbook concepts and to
//! assess whether a student has practiced enough core concepts to transition into
//! text-based Java work.

use std::collections::BTreeSet;

use eatme_core::ast::{Program, Statement};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{build_preconditions, cascade_blocked, no_program_chain};

pub struct TextbookIntegrationGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub fn grade_textbook_integration(input: TextbookIntegrationGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("map-alice-constructs-to-java", &["launch-smoke"]),
            cascade_blocked(
                "identify-practiced-java-concepts",
                &["map-alice-constructs-to-java"],
            ),
            cascade_blocked(
                "assess-transition-readiness",
                &["identify-practiced-java-concepts"],
            ),
        ]
    } else {
        evaluate_textbook_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "textbook-integration-java-transition",
        passed,
        steps,
    )
}

#[derive(Default)]
struct TextbookAnalysis {
    java_mappings: BTreeSet<String>,
    practiced_concepts: BTreeSet<String>,
}

fn evaluate_textbook_steps(program: &Option<Program>) -> Vec<StepGrade> {
    let Some(program) = program else {
        return no_program_chain(&[
            ("map-alice-constructs-to-java", "launch-smoke"),
            (
                "identify-practiced-java-concepts",
                "map-alice-constructs-to-java",
            ),
            (
                "assess-transition-readiness",
                "identify-practiced-java-concepts",
            ),
        ]);
    };

    let analysis = analyze_textbook_alignment(program);
    let practiced = analysis
        .practiced_concepts
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mappings = analysis.java_mappings.iter().cloned().collect::<Vec<_>>();
    let missing = missing_transition_concepts(&analysis);

    let mapping_step = StepGrade {
        name: "map-alice-constructs-to-java".into(),
        status: if mappings.is_empty() {
            StepStatus::Blocked
        } else {
            StepStatus::Ready
        },
        reason: if mappings.is_empty() {
            "No Alice constructs were found that map cleanly to Java textbook concepts".into()
        } else {
            format!("Alice-to-Java mappings: {}", mappings.join(", "))
        },
        depends_on: vec!["launch-smoke".into()],
    };

    let practiced_step = StepGrade {
        name: "identify-practiced-java-concepts".into(),
        status: if practiced.is_empty() {
            StepStatus::Blocked
        } else {
            StepStatus::Ready
        },
        reason: if practiced.is_empty() {
            "No practiced Java concepts were found in the student program".into()
        } else {
            format!("Practiced Java concepts: {}", practiced.join(", "))
        },
        depends_on: vec!["map-alice-constructs-to-java".into()],
    };

    let readiness_step = StepGrade {
        name: "assess-transition-readiness".into(),
        status: if missing.is_empty() {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: if missing.is_empty() {
            format!(
                "Ready to move to Java textbooks; core concepts practiced: {}",
                practiced.join(", ")
            )
        } else {
            format!(
                "Not yet ready to move to Java textbooks; still missing: {}",
                missing.join(", ")
            )
        },
        depends_on: vec!["identify-practiced-java-concepts".into()],
    };

    vec![mapping_step, practiced_step, readiness_step]
}

fn analyze_textbook_alignment(program: &Program) -> TextbookAnalysis {
    let mut analysis = TextbookAnalysis::default();

    if !program.functions.is_empty() {
        analysis
            .java_mappings
            .insert("Alice function → Java method".into());
        analysis.practiced_concepts.insert("methods".into());
    }

    for function in &program.functions {
        analyze_statements(&function.body, &mut analysis);
    }
    for procedure in &program.procedures {
        analyze_statements(&procedure.body, &mut analysis);
    }

    analysis
}

fn analyze_statements(statements: &[Statement], analysis: &mut TextbookAnalysis) {
    for statement in statements {
        match statement {
            Statement::MethodCall { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice method call → Java method invocation".into());
                analysis.practiced_concepts.insert("method calls".into());
            }
            Statement::FunctionCall { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice function call → Java method call with a return value".into());
                analysis.practiced_concepts.insert("methods".into());
            }
            Statement::VariableDeclaration { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice variable declaration → Java local variable".into());
                analysis.practiced_concepts.insert("variables".into());
            }
            Statement::VariableAssignment { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice assignment → Java assignment statement".into());
                analysis.practiced_concepts.insert("assignments".into());
            }
            Statement::CountLoop { body, .. } => {
                analysis
                    .java_mappings
                    .insert("Alice count loop → Java for loop".into());
                analysis.practiced_concepts.insert("loops".into());
                analyze_statements(body, analysis);
            }
            Statement::ForEachArray { body, .. } => {
                analysis
                    .java_mappings
                    .insert("Alice for-each array → Java enhanced for loop".into());
                analysis.practiced_concepts.insert("loops".into());
                analyze_statements(body, analysis);
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                analysis
                    .java_mappings
                    .insert("Alice if/else → Java conditional".into());
                analysis.practiced_concepts.insert("conditionals".into());
                analyze_statements(if_body, analysis);
                analyze_statements(else_body, analysis);
            }
            Statement::ReturnStatement { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice return statement → Java return statement".into());
                analysis.practiced_concepts.insert("methods".into());
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                analysis
                    .java_mappings
                    .insert("Alice user type → Java class".into());
                analysis.practiced_concepts.insert("classes".into());
                for method in methods {
                    analyze_statements(&method.body, analysis);
                }
            }
            Statement::EventListener { body, .. } => {
                analysis
                    .practiced_concepts
                    .insert("event-driven thinking".into());
                analyze_statements(body, analysis);
            }
            Statement::CollisionListener { body, .. } => {
                analysis
                    .practiced_concepts
                    .insert("event-driven thinking".into());
                analyze_statements(body, analysis);
            }
            Statement::DoInOrder { body } => analyze_statements(body, analysis),
            Statement::Comment { .. } => {}
            Statement::ArrayDeclaration { .. } | Statement::ArrayAccess { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice arrays → Java arrays".into());
                analysis.practiced_concepts.insert("arrays".into());
            }
            Statement::ArithmeticExpression { .. } => {
                analysis
                    .java_mappings
                    .insert("Alice arithmetic block → Java arithmetic expression".into());
                analysis.practiced_concepts.insert("expressions".into());
            }
        }
    }
}

fn missing_transition_concepts(analysis: &TextbookAnalysis) -> Vec<&'static str> {
    ["variables", "method calls", "conditionals", "loops"]
        .into_iter()
        .filter(|concept| !analysis.practiced_concepts.contains(*concept))
        .collect()
}

#[cfg(test)]
#[path = "grading_report_textbook_integration_tests.rs"]
mod textbook_integration_tests;
