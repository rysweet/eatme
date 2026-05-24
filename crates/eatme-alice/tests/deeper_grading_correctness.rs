use eatme_assets::{
    EventsGradingInput, ParametersGradingInput, StepStatus, VariablesGradingInput,
    grade_events_and_collision, grade_parameters, grade_variables,
};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

fn parameters_input(program: Option<Program>) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

fn events_input(program: Option<Program>) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

fn variables_input(program: Option<Program>) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

fn well_written_first_lesson_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveCritter".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "distance".into()],
                    },
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "turn".into(),
                        arguments: vec!["LEFT".into(), "distance".into()],
                    },
                ],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveCritter".into(),
                    arguments: vec!["1.0".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn wrong_parameter_type_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveCritter".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "String".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "distance".into()],
                }],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveCritter".into(),
                    arguments: vec!["\"far\"".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn wrong_entity_type_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveCritter".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![Statement::MethodCall {
                    object: "this.scene".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "distance".into()],
                }],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveCritter".into(),
                    arguments: vec!["1.0".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn bad_execution_order_program() -> Program {
    Program {
        procedures: vec![
            Procedure {
                name: "moveCritter".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                }],
                body: vec![
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "turn".into(),
                        arguments: vec!["LEFT".into(), "distance".into()],
                    },
                    Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "distance".into()],
                    },
                ],
            },
            Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: vec![Statement::MethodCall {
                    object: "this".into(),
                    method: "moveCritter".into(),
                    arguments: vec!["1.0".into()],
                }],
            },
        ],
        functions: vec![],
    }
}

fn well_written_events_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::EventListener {
                    event: "KeyPress".into(),
                    body: vec![Statement::IfElse {
                        condition: "canMove".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this.player".into(),
                            method: "move".into(),
                            arguments: vec!["FORWARD".into(), "0.5".into()],
                        }],
                        else_body: vec![],
                    }],
                },
                Statement::CollisionListener {
                    object_a: "this.player".into(),
                    object_b: "this.goal".into(),
                    body: vec![Statement::IfElse {
                        condition: "notGameOver".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this.player".into(),
                            method: "say".into(),
                            arguments: vec!["\"Win!\"".into()],
                        }],
                        else_body: vec![],
                    }],
                },
            ],
        }],
        functions: vec![],
    }
}

fn wrong_event_type_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::IfElse {
                    condition: "canMove".into(),
                    if_body: vec![Statement::MethodCall {
                        object: "this.player".into(),
                        method: "move".into(),
                        arguments: vec!["FORWARD".into(), "0.5".into()],
                    }],
                    else_body: vec![],
                }],
            }],
        }],
        functions: vec![],
    }
}

fn missing_event_guard_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::EventListener {
                event: "KeyPress".into(),
                body: vec![Statement::MethodCall {
                    object: "this.player".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "0.5".into()],
                }],
            }],
        }],
        functions: vec![],
    }
}

fn invalid_collision_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::EventListener {
                    event: "KeyPress".into(),
                    body: vec![Statement::IfElse {
                        condition: "canMove".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this.player".into(),
                            method: "move".into(),
                            arguments: vec!["FORWARD".into(), "0.5".into()],
                        }],
                        else_body: vec![],
                    }],
                },
                Statement::CollisionListener {
                    object_a: "this.player".into(),
                    object_b: "this.player".into(),
                    body: vec![Statement::IfElse {
                        condition: "safe".into(),
                        if_body: vec![Statement::MethodCall {
                            object: "this.player".into(),
                            method: "say".into(),
                            arguments: vec!["\"Oops\"".into()],
                        }],
                        else_body: vec![],
                    }],
                },
            ],
        }],
        functions: vec![],
    }
}

fn well_written_variables_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
                Statement::VariableAssignment {
                    name: "speed".into(),
                    value: "1.0".into(),
                },
            ],
        }],
        functions: vec![],
    }
}

fn dead_variable_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::VariableDeclaration {
                name: "speed".into(),
                var_type: "DecimalNumber".into(),
                initial_value: "0.5".into(),
            }],
        }],
        functions: vec![],
    }
}

fn wrong_variable_type_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "String".into(),
                    initial_value: "\"fast\"".into(),
                },
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
            ],
        }],
        functions: vec![],
    }
}

fn unchanged_assignment_program() -> Program {
    Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::VariableDeclaration {
                    name: "speed".into(),
                    var_type: "DecimalNumber".into(),
                    initial_value: "0.5".into(),
                },
                Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "speed".into()],
                },
                Statement::VariableAssignment {
                    name: "speed".into(),
                    value: "0.5".into(),
                },
            ],
        }],
        functions: vec![],
    }
}

#[test]
fn first_lesson_well_written_program_gets_full_marks() {
    let report = grade_parameters(parameters_input(Some(well_written_first_lesson_program())));
    assert!(report.passed);
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
}

#[test]
fn first_lesson_wrong_parameter_type_gets_specific_feedback() {
    let report = grade_parameters(parameters_input(Some(wrong_parameter_type_program())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("parameters do not match"));
}

#[test]
fn first_lesson_wrong_entity_type_gets_specific_feedback() {
    let report = grade_parameters(parameters_input(Some(wrong_entity_type_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(report.steps[4].reason.contains("SBiped-style objects"));
}

#[test]
fn first_lesson_bad_execution_order_gets_specific_feedback() {
    let report = grade_parameters(parameters_input(Some(bad_execution_order_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(report.steps[4].reason.contains("move before turning"));
}

#[test]
fn first_lesson_empty_program_gets_minimum_marks() {
    let report = grade_parameters(parameters_input(None));
    assert!(!report.passed);
    assert!(
        report.steps[3..]
            .iter()
            .all(|step| step.status == StepStatus::Blocked)
    );
}

#[test]
fn events_well_written_program_gets_full_marks() {
    let report = grade_events_and_collision(events_input(Some(well_written_events_program())));
    assert!(report.passed);
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
}

#[test]
fn events_wrong_event_type_gets_specific_feedback() {
    let report = grade_events_and_collision(events_input(Some(wrong_event_type_program())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("key press or mouse click"));
}

#[test]
fn events_missing_guard_gets_specific_feedback() {
    let report = grade_events_and_collision(events_input(Some(missing_event_guard_program())));
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert!(report.steps[3].reason.contains("guard condition"));
}

#[test]
fn events_invalid_collision_gets_specific_feedback() {
    let report = grade_events_and_collision(events_input(Some(invalid_collision_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(report.steps[4].reason.contains("two different entities"));
}

#[test]
fn events_empty_program_gets_minimum_marks() {
    let report = grade_events_and_collision(events_input(None));
    assert!(!report.passed);
    assert!(
        report.steps[3..]
            .iter()
            .all(|step| step.status == StepStatus::Blocked)
    );
}

#[test]
fn variables_well_written_program_gets_full_marks() {
    let report = grade_variables(variables_input(Some(well_written_variables_program())));
    assert!(report.passed);
    assert!(
        report
            .steps
            .iter()
            .all(|step| step.status == StepStatus::Ready)
    );
}

#[test]
fn variables_dead_code_gets_specific_feedback() {
    let report = grade_variables(variables_input(Some(dead_variable_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("never used after declaration")
    );
}

#[test]
fn variables_wrong_type_gets_specific_feedback() {
    let report = grade_variables(variables_input(Some(wrong_variable_type_program())));
    assert_eq!(report.steps[3].status, StepStatus::Ready);
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert!(
        report.steps[4]
            .reason
            .contains("does not match its method-call usage")
    );
}

#[test]
fn variables_unchanged_assignment_gets_specific_feedback() {
    let report = grade_variables(variables_input(Some(unchanged_assignment_program())));
    assert_eq!(report.steps[4].status, StepStatus::Ready);
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert!(report.steps[5].reason.contains("never changes"));
}

#[test]
fn variables_empty_program_gets_minimum_marks() {
    let report = grade_variables(variables_input(None));
    assert!(!report.passed);
    assert!(
        report.steps[3..]
            .iter()
            .all(|step| step.status == StepStatus::Blocked)
    );
}
