use eatme_core::ast::{Function, Program, Statement};

use crate::grading_report::StepStatus;
use crate::grading_report_textbook_integration::{
    TextbookIntegrationGradingInput, grade_textbook_integration,
};

fn ready_input(program: Program) -> TextbookIntegrationGradingInput {
    TextbookIntegrationGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        student_program: Some(program),
    }
}

fn textbook_ready_program() -> Program {
    Program {
        procedures: vec![eatme_core::ast::Procedure {
            name: "run".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "score".into(),
                    var_type: "Number".into(),
                    initial_value: "0".into(),
                },
                Statement::VariableAssignment {
                    name: "score".into(),
                    value: "score + 1".into(),
                },
                Statement::CountLoop {
                    count: 3,
                    body: vec![Statement::IfElse {
                        condition: "score < 10".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "actor".into(),
                            method: "move".into(),
                            arguments: vec!["score".into()],
                        }],
                        else_body: vec![],
                    }],
                },
                Statement::FunctionCall {
                    object: "this".into(),
                    function: "scoreBonus".into(),
                    arguments: vec![],
                },
            ],
        }],
        functions: vec![Function {
            name: "scoreBonus".into(),
            return_type: "Number".into(),
            body: vec![Statement::ReturnStatement {
                expression: "42".into(),
            }],
        }],
    }
}

fn textbook_not_ready_program() -> Program {
    Program::new(vec![eatme_core::ast::Procedure {
        name: "run".into(),
        parameters: vec![],
        body: vec![
            Statement::VariableDeclaration {
                name: "score".into(),
                var_type: "Number".into(),
                initial_value: "0".into(),
            },
            Statement::MethodCall {
                object: "actor".into(),
                method: "say".into(),
                arguments: vec!["score".into()],
            },
        ],
    }])
}

fn textbook_all_concepts_program() -> Program {
    Program {
        procedures: vec![eatme_core::ast::Procedure {
            name: "run".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "score".into(),
                    var_type: "Number".into(),
                    initial_value: "0".into(),
                },
                Statement::VariableAssignment {
                    name: "score".into(),
                    value: "score + 1".into(),
                },
                Statement::MethodCall {
                    object: "actor".into(),
                    method: "say".into(),
                    arguments: vec!["hello".into()],
                },
                Statement::FunctionCall {
                    object: "this".into(),
                    function: "computeScore".into(),
                    arguments: vec![],
                },
                Statement::CountLoop {
                    count: 3,
                    body: vec![Statement::Comment {
                        text: "repeat".into(),
                    }],
                },
                Statement::ForEachArray {
                    item_name: "item".into(),
                    array: "items".into(),
                    body: vec![Statement::Comment {
                        text: "visit".into(),
                    }],
                },
                Statement::IfElse {
                    condition: "score > 0".into(),
                    if_body: vec![Statement::Comment {
                        text: "positive".into(),
                    }],
                    else_body: vec![Statement::Comment {
                        text: "fallback".into(),
                    }],
                },
                Statement::EventListener {
                    event: "mouseClicked".into(),
                    body: vec![Statement::Comment {
                        text: "event".into(),
                    }],
                },
                Statement::CollisionListener {
                    object_a: "rabbit".into(),
                    object_b: "carrot".into(),
                    body: vec![Statement::Comment {
                        text: "collision".into(),
                    }],
                },
                Statement::DoInOrder {
                    body: vec![Statement::MethodCall {
                        object: "actor".into(),
                        method: "turn".into(),
                        arguments: vec!["LEFT".into()],
                    }],
                },
                Statement::ArrayDeclaration {
                    name: "items".into(),
                    element_type: "TextString".into(),
                    elements: vec!["apple".into(), "pear".into()],
                },
                Statement::ArrayAccess {
                    array: "items".into(),
                    index: "0".into(),
                    target: "firstItem".into(),
                },
                Statement::ArithmeticExpression {
                    operator: eatme_core::ast::ArithmeticOperator::Add,
                    left: "score".into(),
                    right: "bonus".into(),
                    result: "total".into(),
                },
                Statement::UserTypeDeclaration {
                    name: "ScoreKeeper".into(),
                    extends: None,
                    methods: vec![eatme_core::ast::Procedure {
                        name: "increment".into(),
                        parameters: vec![],
                        body: vec![Statement::MethodCall {
                            object: "actor".into(),
                            method: "jump".into(),
                            arguments: vec![],
                        }],
                    }],
                },
            ],
        }],
        functions: vec![Function {
            name: "computeScore".into(),
            return_type: "Number".into(),
            body: vec![Statement::ReturnStatement {
                expression: "42".into(),
            }],
        }],
    }
}

#[test]
fn maps_alice_constructs_to_java_equivalents() {
    let report = grade_textbook_integration(ready_input(textbook_ready_program()));
    let step = report
        .steps
        .iter()
        .find(|step| step.name == "map-alice-constructs-to-java")
        .unwrap();

    assert_eq!(step.status, StepStatus::Ready);
    assert!(
        step.reason
            .contains("Alice variable declaration → Java local variable")
    );
    assert!(step.reason.contains("Alice count loop → Java for loop"));
    assert!(step.reason.contains("Alice if/else → Java conditional"));
}

#[test]
fn identifies_practiced_java_concepts() {
    let report = grade_textbook_integration(ready_input(textbook_ready_program()));
    let step = report
        .steps
        .iter()
        .find(|step| step.name == "identify-practiced-java-concepts")
        .unwrap();

    assert_eq!(step.status, StepStatus::Ready);
    assert!(step.reason.contains("variables"));
    assert!(step.reason.contains("method calls"));
    assert!(step.reason.contains("conditionals"));
    assert!(step.reason.contains("loops"));
}

#[test]
fn grades_transition_readiness_for_java_textbooks() {
    let ready = grade_textbook_integration(ready_input(textbook_ready_program()));
    let not_ready = grade_textbook_integration(ready_input(textbook_not_ready_program()));

    let ready_step = ready
        .steps
        .iter()
        .find(|step| step.name == "assess-transition-readiness")
        .unwrap();
    let blocked_step = not_ready
        .steps
        .iter()
        .find(|step| step.name == "assess-transition-readiness")
        .unwrap();

    assert!(ready.passed);
    assert_eq!(ready_step.status, StepStatus::Ready);
    assert!(
        ready_step
            .reason
            .contains("Ready to move to Java textbooks")
    );

    assert!(!not_ready.passed);
    assert_eq!(blocked_step.status, StepStatus::Blocked);
    assert!(blocked_step.reason.contains("conditionals"));
    assert!(blocked_step.reason.contains("loops"));
}

#[test]
fn validates_java_mappings_for_all_textbook_concept_areas() {
    let report = grade_textbook_integration(ready_input(textbook_all_concepts_program()));
    let mapping_step = report
        .steps
        .iter()
        .find(|step| step.name == "map-alice-constructs-to-java")
        .unwrap();
    let practiced_step = report
        .steps
        .iter()
        .find(|step| step.name == "identify-practiced-java-concepts")
        .unwrap();
    let readiness_step = report
        .steps
        .iter()
        .find(|step| step.name == "assess-transition-readiness")
        .unwrap();

    assert_eq!(mapping_step.status, StepStatus::Ready);
    for expected_mapping in [
        "Alice arrays → Java arrays",
        "Alice arithmetic block → Java arithmetic expression",
        "Alice assignment → Java assignment statement",
        "Alice count loop → Java for loop",
        "Alice for-each array → Java enhanced for loop",
        "Alice function → Java method",
        "Alice function call → Java method call with a return value",
        "Alice if/else → Java conditional",
        "Alice method call → Java method invocation",
        "Alice return statement → Java return statement",
        "Alice user type → Java class",
        "Alice variable declaration → Java local variable",
    ] {
        assert!(
            mapping_step.reason.contains(expected_mapping),
            "missing mapping: {expected_mapping}\nreason: {}",
            mapping_step.reason
        );
    }

    assert_eq!(practiced_step.status, StepStatus::Ready);
    for expected_concept in [
        "arrays",
        "assignments",
        "classes",
        "conditionals",
        "event-driven thinking",
        "expressions",
        "loops",
        "method calls",
        "methods",
        "variables",
    ] {
        assert!(
            practiced_step.reason.contains(expected_concept),
            "missing practiced concept: {expected_concept}\nreason: {}",
            practiced_step.reason
        );
    }

    assert!(report.passed);
    assert_eq!(readiness_step.status, StepStatus::Ready);
}
