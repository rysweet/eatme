use std::collections::BTreeSet;

use eatme_core::ast::{Program, Statement};

use crate::grading_report::QualityScore;

pub fn score_parameter_quality(student_program: Option<&Program>) -> Vec<QualityScore> {
    vec![parameter_types_score(student_program)]
}

pub fn score_event_quality(student_program: Option<&Program>) -> Vec<QualityScore> {
    vec![entity_types_score(student_program)]
}

pub fn score_variable_quality(student_program: Option<&Program>) -> Vec<QualityScore> {
    vec![variable_usage_score(student_program)]
}

fn parameter_types_score(student_program: Option<&Program>) -> QualityScore {
    let Some(program) = student_program else {
        return quality_score(
            "parameter_types",
            0,
            "No student program provided for parameter quality scoring",
        );
    };

    let parameter_types: Vec<&str> = program
        .procedures
        .iter()
        .flat_map(|procedure| {
            procedure
                .parameters
                .iter()
                .map(|parameter| parameter.param_type.as_str())
        })
        .collect();

    if parameter_types.is_empty() {
        return quality_score("parameter_types", 0, "No parameters found to assess");
    }

    let specific_types = parameter_types
        .iter()
        .filter(|param_type| is_specific_type(param_type))
        .count();
    let score = ratio_score(specific_types, parameter_types.len());
    let feedback = if specific_types == parameter_types.len() {
        format!(
            "All {} parameters use specific types",
            parameter_types.len()
        )
    } else {
        format!(
            "{specific_types} of {} parameters use specific types; prefer concrete parameter types over Object or empty types",
            parameter_types.len()
        )
    };

    quality_score("parameter_types", score, feedback)
}

fn entity_types_score(student_program: Option<&Program>) -> QualityScore {
    let Some(program) = student_program else {
        return quality_score(
            "entity_types",
            0,
            "No student program provided for event quality scoring",
        );
    };

    let mut tally = EntityTypeTally::default();
    for procedure in &program.procedures {
        collect_listener_entity_types(&procedure.body, false, &mut tally);
    }

    if tally.total == 0 {
        return quality_score(
            "entity_types",
            0,
            "No listener entity references found to assess",
        );
    }

    let score = ratio_score(tally.explicit_scene_entities, tally.total);
    let feedback = if tally.explicit_scene_entities == tally.total {
        format!(
            "All {} listener entity references use explicit scene entities",
            tally.total
        )
    } else {
        format!(
            "{} of {} listener entity references use explicit scene entities like this.cat",
            tally.explicit_scene_entities, tally.total
        )
    };

    quality_score("entity_types", score, feedback)
}

fn variable_usage_score(student_program: Option<&Program>) -> QualityScore {
    let Some(program) = student_program else {
        return quality_score(
            "variable_usage",
            0,
            "No student program provided for variable quality scoring",
        );
    };

    let mut declared = BTreeSet::new();
    for procedure in &program.procedures {
        collect_declared_variables(&procedure.body, &mut declared);
    }

    if declared.is_empty() {
        return quality_score("variable_usage", 0, "No declared variables found to assess");
    }

    let mut used = BTreeSet::new();
    for procedure in &program.procedures {
        collect_variable_references(&procedure.body, &declared, &mut used);
    }

    let score = ratio_score(used.len(), declared.len());
    let feedback = if used.len() == declared.len() {
        format!(
            "All {} declared variables are referenced after declaration",
            declared.len()
        )
    } else {
        format!(
            "{} of {} declared variables are referenced after declaration",
            used.len(),
            declared.len()
        )
    };

    quality_score("variable_usage", score, feedback)
}

#[derive(Default)]
struct EntityTypeTally {
    total: usize,
    explicit_scene_entities: usize,
}

fn collect_listener_entity_types(
    statements: &[Statement],
    inside_listener: bool,
    tally: &mut EntityTypeTally,
) {
    for statement in statements {
        match statement {
            Statement::EventListener { body, .. } => {
                collect_listener_entity_types(body, true, tally);
            }
            Statement::CollisionListener {
                object_a,
                object_b,
                body,
            } => {
                record_entity_type(object_a, tally);
                record_entity_type(object_b, tally);
                collect_listener_entity_types(body, true, tally);
            }
            Statement::MethodCall { object, .. } if inside_listener => {
                record_entity_type(object, tally);
            }
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => {
                collect_listener_entity_types(body, inside_listener, tally);
            }
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_listener_entity_types(if_body, inside_listener, tally);
                collect_listener_entity_types(else_body, inside_listener, tally);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_listener_entity_types(&method.body, inside_listener, tally);
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

fn collect_declared_variables(statements: &[Statement], declared: &mut BTreeSet<String>) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration { name, .. } => {
                declared.insert(name.clone());
            }
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. } => collect_declared_variables(body, declared),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_declared_variables(if_body, declared);
                collect_declared_variables(else_body, declared);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_declared_variables(&method.body, declared);
                }
            }
            Statement::MethodCall { .. }
            | Statement::EventListener { .. }
            | Statement::CollisionListener { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => {}
        }
    }
}

fn collect_variable_references(
    statements: &[Statement],
    declared: &BTreeSet<String>,
    used: &mut BTreeSet<String>,
) {
    for statement in statements {
        match statement {
            Statement::MethodCall { arguments, .. } | Statement::FunctionCall { arguments, .. } => {
                for argument in arguments {
                    mark_variable_references(argument, declared, used);
                }
            }
            Statement::VariableDeclaration { initial_value, .. } => {
                mark_variable_references(initial_value, declared, used);
            }
            Statement::VariableAssignment { value, .. } => {
                mark_variable_references(value, declared, used);
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                mark_variable_references(condition, declared, used);
                collect_variable_references(if_body, declared, used);
                collect_variable_references(else_body, declared, used);
            }
            Statement::ReturnStatement { expression } => {
                mark_variable_references(expression, declared, used);
            }
            Statement::ArrayDeclaration { elements, .. } => {
                for element in elements {
                    mark_variable_references(element, declared, used);
                }
            }
            Statement::ArrayAccess {
                array,
                index,
                target,
            } => {
                mark_variable_references(array, declared, used);
                mark_variable_references(index, declared, used);
                mark_variable_references(target, declared, used);
            }
            Statement::ArithmeticExpression {
                left,
                right,
                result,
                ..
            } => {
                mark_variable_references(left, declared, used);
                mark_variable_references(right, declared, used);
                mark_variable_references(result, declared, used);
            }
            Statement::CountLoop { body, .. }
            | Statement::DoInOrder { body }
            | Statement::ForEachArray { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. } => {
                collect_variable_references(body, declared, used);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for method in methods {
                    collect_variable_references(&method.body, declared, used);
                }
            }
            Statement::Comment { .. } => {}
        }
    }
}

fn mark_variable_references(
    expression: &str,
    declared: &BTreeSet<String>,
    used: &mut BTreeSet<String>,
) {
    for token in expression
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
    {
        if declared.contains(token) {
            used.insert(token.to_string());
        }
    }
}

fn record_entity_type(entity: &str, tally: &mut EntityTypeTally) {
    tally.total += 1;
    if entity.starts_with("this.") && entity.len() > "this.".len() {
        tally.explicit_scene_entities += 1;
    }
}

fn is_specific_type(param_type: &str) -> bool {
    let normalized = param_type.trim();
    !normalized.is_empty() && !matches!(normalized, "Object" | "Any" | "Unknown")
}

fn ratio_score(matched: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }

    ((matched * 100) / total) as u8
}

fn quality_score(dimension: &str, score: u8, feedback: impl Into<String>) -> QualityScore {
    QualityScore {
        score,
        dimension: dimension.into(),
        feedback: feedback.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eatme_core::ast::{Parameter, Procedure};

    #[test]
    fn parameter_quality_scores_only_specific_parameter_types() {
        let program = Program::new(vec![Procedure {
            name: "move".into(),
            parameters: vec![
                Parameter {
                    name: "distance".into(),
                    param_type: "WholeNumber".into(),
                },
                Parameter {
                    name: "thing".into(),
                    param_type: "Object".into(),
                },
            ],
            body: vec![],
        }]);

        let score = &score_parameter_quality(Some(&program))[0];

        assert_eq!(score.dimension, "parameter_types");
        assert_eq!(score.score, 50);
        assert!(
            score
                .feedback
                .contains("1 of 2 parameters use specific types")
        );
    }

    #[test]
    fn event_quality_counts_only_listener_entity_references() {
        let program = Program::new(vec![Procedure {
            name: "setup".into(),
            parameters: vec![],
            body: vec![
                Statement::MethodCall {
                    object: "this.ignoredOutsideListener".into(),
                    method: "turn".into(),
                    arguments: vec![],
                },
                Statement::EventListener {
                    event: "mouseClickOnObject".into(),
                    body: vec![
                        Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "say".into(),
                            arguments: vec!["hello".into()],
                        },
                        Statement::MethodCall {
                            object: "enemy".into(),
                            method: "move".into(),
                            arguments: vec!["1".into()],
                        },
                    ],
                },
            ],
        }]);

        let score = &score_event_quality(Some(&program))[0];

        assert_eq!(score.dimension, "entity_types");
        assert_eq!(score.score, 50);
        assert!(
            score
                .feedback
                .contains("1 of 2 listener entity references use explicit scene entities")
        );
    }

    #[test]
    fn variable_quality_tracks_references_across_nested_control_flow() {
        let program = Program::new(vec![Procedure {
            name: "play".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "counter".into(),
                    var_type: "Number".into(),
                    initial_value: "0".into(),
                },
                Statement::VariableDeclaration {
                    name: "unused".into(),
                    var_type: "String".into(),
                    initial_value: "hello".into(),
                },
                Statement::IfElse {
                    condition: "counter > 0".into(),
                    if_body: vec![Statement::MethodCall {
                        object: "this".into(),
                        method: "say".into(),
                        arguments: vec!["counter".into()],
                    }],
                    else_body: vec![],
                },
            ],
        }]);

        let score = &score_variable_quality(Some(&program))[0];

        assert_eq!(score.dimension, "variable_usage");
        assert_eq!(score.score, 50);
        assert!(
            score
                .feedback
                .contains("1 of 2 declared variables are referenced after declaration")
        );
    }
}
