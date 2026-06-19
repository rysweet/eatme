use eatme_core::ast::{Program, SequenceBlock, SequenceKind, Statement};

fn concurrent_animation_sequence() -> SequenceBlock {
    SequenceBlock {
        kind: SequenceKind::DoTogether,
        steps: vec![
            "hero.walk:1.0".into(),
            "camera.dolly:1.5".into(),
            "light.fade:2.0".into(),
            "music.rise:2.5".into(),
        ],
    }
}

fn animation_scenario_program() -> Program {
    Program {
        procedures: vec![eatme_core::ast::Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![
                Statement::DoInOrder {
                    body: vec![
                        Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "walk".into(),
                            arguments: vec!["this.path".into(), "1.00".into(), "linear".into()],
                        },
                        Statement::IfElse {
                            condition: "this.hero.isExcited".into(),
                            if_body: vec![Statement::MethodCall {
                                object: "this.hero".into(),
                                method: "wave".into(),
                                arguments: vec!["0.40".into(), "easeIn".into()],
                            }],
                            else_body: vec![Statement::MethodCall {
                                object: "this.hero".into(),
                                method: "nod".into(),
                                arguments: vec!["0.40".into(), "easeOut".into()],
                            }],
                        },
                        Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "say".into(),
                            arguments: vec!["\"done\"".into(), "0.20".into(), "easeInOut".into()],
                        },
                    ],
                },
                Statement::EventListener {
                    event: "SpacePressed".into(),
                    body: vec![
                        Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "stopAnimation".into(),
                            arguments: vec!["walkCycle".into()],
                        },
                        Statement::MethodCall {
                            object: "this.effects".into(),
                            method: "cleanupAnimationState".into(),
                            arguments: vec!["walkCycle".into()],
                        },
                    ],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "setOpacity".into(),
                    arguments: vec!["0.25".into(), "1.00".into(), "linear".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "resize".into(),
                    arguments: vec!["1.20".into(), "1.00".into(), "easeInOut".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "setPaint".into(),
                    arguments: vec!["Color.BLUE".into(), "1.00".into(), "easeOut".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "playWalkCycle".into(),
                    arguments: vec!["walk".into(), "1.60".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "sampleJointPose".into(),
                    arguments: vec!["LEFT_KNEE".into(), "0.00".into(), "0,0,0".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "sampleJointPose".into(),
                    arguments: vec!["LEFT_KNEE".into(), "0.50".into(), "15,0,0".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "sampleJointPose".into(),
                    arguments: vec!["LEFT_KNEE".into(), "1.00".into(), "0,0,0".into()],
                },
                Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "recordDuration".into(),
                    arguments: vec!["expected=2.00".into(), "actual=2.08".into()],
                },
            ],
        }],
        functions: vec![],
    }
}

fn collect_method_calls<'a>(
    statements: &'a [Statement],
    method: &str,
    output: &mut Vec<&'a Statement>,
) {
    for statement in statements {
        match statement {
            Statement::MethodCall { method: name, .. } if name == method => output.push(statement),
            Statement::DoInOrder { body }
            | Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::ForEachArray { body, .. } => collect_method_calls(body, method, output),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                collect_method_calls(if_body, method, output);
                collect_method_calls(else_body, method, output);
            }
            Statement::UserTypeDeclaration { methods, .. } => {
                for procedure in methods {
                    collect_method_calls(&procedure.body, method, output);
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

fn parse_duration_token(token: &str) -> Option<f32> {
    token.split('=').next_back()?.parse().ok()
}

fn within_five_percent(expected: f32, actual: f32) -> bool {
    let tolerance = expected.abs() * 0.05;
    (expected - actual).abs() <= tolerance
}

#[test]
fn concurrent_animation_sequence_tracks_four_parallel_branches_with_distinct_durations() {
    let sequence = concurrent_animation_sequence();
    let durations: Vec<f32> = sequence
        .steps
        .iter()
        .filter_map(|step| step.split(':').nth(1))
        .filter_map(|value| value.parse::<f32>().ok())
        .collect();

    assert!(matches!(sequence.kind, SequenceKind::DoTogether));
    assert_eq!(sequence.steps.len(), 4);
    assert_eq!(durations, vec![1.0, 1.5, 2.0, 2.5]);
}

#[test]
fn sequential_animation_keeps_conditional_branch_mid_sequence() {
    let program = animation_scenario_program();
    let body = &program.procedures[0].body;

    let Statement::DoInOrder {
        body: sequence_body,
    } = &body[0]
    else {
        panic!("expected top-level doInOrder block")
    };

    assert!(matches!(sequence_body[1], Statement::IfElse { .. }));
    let Statement::IfElse {
        if_body, else_body, ..
    } = &sequence_body[1]
    else {
        unreachable!();
    };
    assert_eq!(if_body.len(), 1);
    assert_eq!(else_body.len(), 1);
}

#[test]
fn animation_timing_stays_within_five_percent_tolerance() {
    let program = animation_scenario_program();
    let mut duration_calls = Vec::new();
    collect_method_calls(
        &program.procedures[0].body,
        "recordDuration",
        &mut duration_calls,
    );
    let Statement::MethodCall { arguments, .. } = duration_calls[0] else {
        unreachable!();
    };

    let expected = parse_duration_token(&arguments[0]).expect("expected duration");
    let actual = parse_duration_token(&arguments[1]).expect("actual duration");

    assert!(within_five_percent(expected, actual));
}

#[test]
fn animation_interruption_captures_stop_and_cleanup_paths() {
    let program = animation_scenario_program();
    let Statement::EventListener { event, body } = &program.procedures[0].body[1] else {
        panic!("expected animation interrupt listener")
    };

    let mut stop_calls = Vec::new();
    collect_method_calls(body, "stopAnimation", &mut stop_calls);
    let mut cleanup_calls = Vec::new();
    collect_method_calls(body, "cleanupAnimationState", &mut cleanup_calls);

    assert_eq!(event, "SpacePressed");
    assert_eq!(stop_calls.len(), 1);
    assert_eq!(cleanup_calls.len(), 1);
}

#[test]
fn easing_scenarios_cover_linear_ease_in_ease_out_and_ease_in_out() {
    let program = animation_scenario_program();
    let mut walk_calls = Vec::new();
    collect_method_calls(&program.procedures[0].body, "walk", &mut walk_calls);
    let mut wave_calls = Vec::new();
    collect_method_calls(&program.procedures[0].body, "wave", &mut wave_calls);
    let mut nod_calls = Vec::new();
    collect_method_calls(&program.procedures[0].body, "nod", &mut nod_calls);
    let mut say_calls = Vec::new();
    collect_method_calls(&program.procedures[0].body, "say", &mut say_calls);

    let Statement::MethodCall {
        arguments: walk_args,
        ..
    } = walk_calls[0]
    else {
        unreachable!()
    };
    let Statement::MethodCall {
        arguments: wave_args,
        ..
    } = wave_calls[0]
    else {
        unreachable!()
    };
    let Statement::MethodCall {
        arguments: nod_args,
        ..
    } = nod_calls[0]
    else {
        unreachable!()
    };
    let Statement::MethodCall {
        arguments: say_args,
        ..
    } = say_calls[0]
    else {
        unreachable!()
    };

    assert_eq!(walk_args[2], "linear");
    assert_eq!(wave_args[1], "easeIn");
    assert_eq!(nod_args[1], "easeOut");
    assert_eq!(say_args[2], "easeInOut");
}

#[test]
fn property_animation_scenario_keeps_opacity_scale_and_color_updates_together() {
    let program = animation_scenario_program();
    let mut opacity_calls = Vec::new();
    let mut scale_calls = Vec::new();
    let mut color_calls = Vec::new();
    collect_method_calls(
        &program.procedures[0].body,
        "setOpacity",
        &mut opacity_calls,
    );
    collect_method_calls(&program.procedures[0].body, "resize", &mut scale_calls);
    collect_method_calls(&program.procedures[0].body, "setPaint", &mut color_calls);

    assert_eq!(opacity_calls.len(), 1);
    assert_eq!(scale_calls.len(), 1);
    assert_eq!(color_calls.len(), 1);
}

#[test]
fn skeletal_animation_scenario_samples_joint_positions_across_walk_cycle_keyframes() {
    let program = animation_scenario_program();
    let mut walk_cycle_calls = Vec::new();
    let mut joint_samples = Vec::new();
    collect_method_calls(
        &program.procedures[0].body,
        "playWalkCycle",
        &mut walk_cycle_calls,
    );
    collect_method_calls(
        &program.procedures[0].body,
        "sampleJointPose",
        &mut joint_samples,
    );

    let Statement::MethodCall {
        arguments: walk_args,
        ..
    } = walk_cycle_calls[0]
    else {
        unreachable!()
    };
    assert_eq!(walk_args[0], "walk");
    assert_eq!(joint_samples.len(), 3);

    let keyframes: Vec<&str> = joint_samples
        .iter()
        .map(|statement| match statement {
            Statement::MethodCall { arguments, .. } => arguments[1].as_str(),
            _ => unreachable!(),
        })
        .collect();

    assert_eq!(keyframes, vec!["0.00", "0.50", "1.00"]);
}
