use super::*;

// --- JSON round-trip tests ---

#[test]
fn program_with_all_variants_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "myFirstMethod".into(),
            body: vec![
                Statement::MethodCall {
                    object: "this.cat".into(),
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
                Statement::IfElse {
                    condition: "this.cat isCloseTo this.dog".into(),
                    if_body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Hello!\"".into()],
                    }],
                    else_body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "think".into(),
                        arguments: vec!["\"Hmm...\"".into()],
                    }],
                },
                Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Scene started!\"".into()],
                    }],
                },
                Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Ouch!\"".into()],
                    }],
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn empty_program_round_trips() {
    let program = Program {
        procedures: vec![],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn empty_procedure_body_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "emptyMethod".into(),
            body: vec![],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn multiple_procedures_round_trip() {
    let program = Program {
        procedures: vec![
            Procedure {
                name: "methodOne".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec![],
                }],
                ..Default::default()
            },
            Procedure {
                name: "methodTwo".into(),
                body: vec![Statement::MethodCall {
                    object: "this.dog".into(),
                    method: "run".into(),
                    arguments: vec!["FAST".into()],
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// --- Nested structure tests ---

#[test]
fn count_loop_containing_method_call_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "loopMethod".into(),
            body: vec![Statement::CountLoop {
                count: 5,
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn if_else_containing_count_loop_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "nestedMethod".into(),
            body: vec![Statement::IfElse {
                condition: "this.cat isCloseTo this.dog".into(),
                if_body: vec![Statement::CountLoop {
                    count: 3,
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "walk".into(),
                        arguments: vec!["FORWARD".into()],
                    }],
                }],
                else_body: vec![],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn deeply_nested_three_levels_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "deepMethod".into(),
            body: vec![Statement::CountLoop {
                count: 2,
                body: vec![Statement::IfElse {
                    condition: "true".into(),
                    if_body: vec![Statement::CountLoop {
                        count: 1,
                        body: vec![Statement::MethodCall {
                            object: "this.cat".into(),
                            method: "walk".into(),
                            arguments: vec![],
                        }],
                    }],
                    else_body: vec![],
                }],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

// --- serde tag discrimination ---

#[test]
fn method_call_json_has_kind_field() {
    let stmt = Statement::MethodCall {
        object: "this.cat".into(),
        method: "walk".into(),
        arguments: vec![],
    };
    let json: serde_json::Value = serde_json::to_value(&stmt).unwrap();
    assert_eq!(json["kind"], "MethodCall");
}

#[test]
fn count_loop_json_has_kind_field() {
    let stmt = Statement::CountLoop {
        count: 3,
        body: vec![],
    };
    let json: serde_json::Value = serde_json::to_value(&stmt).unwrap();
    assert_eq!(json["kind"], "CountLoop");
}

#[test]
fn if_else_json_has_kind_field() {
    let stmt = Statement::IfElse {
        condition: "true".into(),
        if_body: vec![],
        else_body: vec![],
    };
    let json: serde_json::Value = serde_json::to_value(&stmt).unwrap();
    assert_eq!(json["kind"], "IfElse");
}

#[test]
fn unknown_kind_rejected_at_deserialization() {
    let json = r#"{"kind":"UnknownThing","foo":"bar"}"#;
    let result: Result<Statement, _> = serde_json::from_str(json);
    assert!(result.is_err(), "unknown variant should be rejected");
}

// --- Edge cases ---

#[test]
fn method_call_with_no_arguments() {
    let stmt = Statement::MethodCall {
        object: "this.cat".into(),
        method: "walk".into(),
        arguments: vec![],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn count_loop_with_zero_count() {
    let stmt = Statement::CountLoop {
        count: 0,
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "walk".into(),
            arguments: vec![],
        }],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn if_else_with_empty_bodies() {
    let stmt = Statement::IfElse {
        condition: "true".into(),
        if_body: vec![],
        else_body: vec![],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn count_loop_with_empty_body() {
    let stmt = Statement::CountLoop {
        count: 10,
        body: vec![],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn method_call_with_many_arguments() {
    let stmt = Statement::MethodCall {
        object: "this.cat".into(),
        method: "complexAction".into(),
        arguments: vec![
            "arg1".into(),
            "arg2".into(),
            "arg3".into(),
            "arg4".into(),
            "arg5".into(),
        ],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

// --- EventListener and CollisionListener tests ---

#[test]
fn event_listener_round_trips() {
    let stmt = Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Hello world!\"".into()],
        }],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn collision_listener_round_trips() {
    let stmt = Statement::CollisionListener {
        object_a: "this.cat".into(),
        object_b: "this.dog".into(),
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "say".into(),
            arguments: vec!["\"Ouch!\"".into()],
        }],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn event_listener_json_has_kind_field() {
    let stmt = Statement::EventListener {
        event: "KeyPress".into(),
        body: vec![],
    };
    let json: serde_json::Value = serde_json::to_value(&stmt).unwrap();
    assert_eq!(json["kind"], "EventListener");
}

#[test]
fn collision_listener_json_has_kind_field() {
    let stmt = Statement::CollisionListener {
        object_a: "this.cat".into(),
        object_b: "this.dog".into(),
        body: vec![],
    };
    let json: serde_json::Value = serde_json::to_value(&stmt).unwrap();
    assert_eq!(json["kind"], "CollisionListener");
}

#[test]
fn event_listener_with_empty_body_round_trips() {
    let stmt = Statement::EventListener {
        event: "MouseClick".into(),
        body: vec![],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn collision_listener_with_empty_body_round_trips() {
    let stmt = Statement::CollisionListener {
        object_a: "this.cat".into(),
        object_b: "this.dog".into(),
        body: vec![],
    };
    let json = serde_json::to_string(&stmt).unwrap();
    let restored: Statement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt, restored);
}

#[test]
fn event_listener_nested_in_count_loop_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "nestedEvent".into(),
            body: vec![Statement::CountLoop {
                count: 2,
                body: vec![Statement::EventListener {
                    event: "SceneActivated".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "walk".into(),
                        arguments: vec![],
                    }],
                }],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}

#[test]
fn collision_listener_nested_in_if_else_round_trips() {
    let program = Program {
        procedures: vec![Procedure {
            name: "nestedCollision".into(),
            body: vec![Statement::IfElse {
                condition: "true".into(),
                if_body: vec![Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![Statement::MethodCall {
                        object: "this.cat".into(),
                        method: "say".into(),
                        arguments: vec!["\"Collided!\"".into()],
                    }],
                }],
                else_body: vec![],
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
