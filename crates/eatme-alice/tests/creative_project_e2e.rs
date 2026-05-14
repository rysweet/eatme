// Creative/Design project E2E tests

use eatme_assets::{CreativeProjectGradingInput, grade_creative_project};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn complete_creative_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Welcome!\"".into()],
                    },
                    Statement::MethodCall {
                        object: "this.dog".into(),
                        method: "walk".into(),
                        arguments: vec!["FORWARD".into(), "1.0".into()],
                    },
                    Statement::CountLoop {
                        count: 3,
                        body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "turn".into(),
                            arguments: vec!["LEFT".into(), "0.25".into()],
                        }],
                    },
                    Statement::EventListener {
                        event: "SceneActivated".into(),
                        body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "say".into(),
                            arguments: vec!["\"Game on!\"".into()],
                        }],
                    },
                ],
            },
            Procedure {
                name: "doSpecialMove".into(),
                parameters: vec![Parameter {
                    name: "speed".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn all_ready_input(program: Option<Program>) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn creative_grading_all_ready() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert!(report.passed, "report should pass: {:?}", report.steps);
    assert_eq!(report.lesson, "creative-design-project");
}

#[test]
fn creative_grading_blocked_without_program() {
    let report = grade_creative_project(all_ready_input(None));
    assert!(!report.passed);
}

#[test]
fn creative_grading_minimal_fails() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Hello\"".into()],
        }],
    }]);
    let report = grade_creative_project(all_ready_input(Some(program)));
    assert!(
        !report.passed,
        "minimal program should not pass creative assessment"
    );
}

#[test]
fn creative_report_has_nine_steps() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert_eq!(report.steps.len(), 9, "3 preconditions + 6 interaction");
}

#[test]
fn creative_schema_and_lesson() {
    let report = grade_creative_project(all_ready_input(Some(complete_creative_program())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "creative-design-project");
}
