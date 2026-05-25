use crate::grading_report::{
    GradingInput, GradingReport, StepStatus, grade_first_lesson_readiness,
};
use crate::{
    ArraysArithmeticGradingInput, CommentsGradingInput, CreativeProjectGradingInput,
    EventsGradingInput, FunctionsGradingInput, GamesNarrativeGradingInput,
    InheritanceOopGradingInput, LoopsGradingInput, NestedControlFlowGradingInput,
    ParametersGradingInput, SceneBuildingGradingInput, SequencingGradingInput,
    VariablesGradingInput, grade_arrays_and_arithmetic, grade_comments, grade_creative_project,
    grade_events_and_collision, grade_functions, grade_games_and_narrative, grade_inheritance_oop,
    grade_loops_and_conditionals, grade_nested_control_flow, grade_parameters,
    grade_scene_building, grade_sequencing, grade_variables,
};
use eatme_core::ast::{
    ArithmeticOperator, CameraPose, Function, Parameter, Procedure, Program, SceneLayout,
    SceneObject, SequenceBlock, SequenceKind, Statement, Vec3,
};
use serde_json::Value;
use std::path::Path;

fn repository_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn grade_committed_assets_produces_valid_report() {
    let root = repository_root();
    let asset_report = crate::validate_assets(&root).unwrap();

    let input = GradingInput {
        assets_valid: asset_report.passed,
        asset_reason: if asset_report.passed {
            format!(
                "All {} scenario assets passed validation",
                asset_report.scenario_asset_count
            )
        } else {
            format!("{} errors found", asset_report.errors.len())
        },
        deps_available: false,
        deps_reason: "Dependencies not checked in this test".into(),
    };

    let report = grade_first_lesson_readiness(input);

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "building-a-scene-first-world");
    assert_eq!(report.steps.len(), 6);
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[1].status, StepStatus::Blocked);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[2].status, StepStatus::Blocked);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert_eq!(report.steps[3].name, "place-object");
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
    assert_eq!(report.steps[4].name, "edit-code");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);
    assert_eq!(report.steps[5].name, "run-world");
    assert!(!report.passed);
}

#[test]
fn grade_committed_assets_all_ready_path() {
    let root = repository_root();
    let asset_report = crate::validate_assets(&root).unwrap();
    assert!(
        asset_report.passed,
        "committed assets must pass: {:?}",
        asset_report.errors
    );

    let input = GradingInput {
        assets_valid: true,
        asset_reason: format!(
            "All {} scenario assets passed validation",
            asset_report.scenario_asset_count
        ),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    };

    let report = grade_first_lesson_readiness(input);

    for step in &report.steps[..3] {
        assert_eq!(
            step.status,
            StepStatus::Ready,
            "precondition step {} should be ready",
            step.name
        );
    }
    for step in &report.steps[3..] {
        assert_eq!(
            step.status,
            StepStatus::NotYetTested,
            "interaction step {} should be not-yet-tested",
            step.name
        );
    }
    assert!(!report.passed);
}

#[test]
fn grading_report_json_round_trips_cleanly() {
    let input = GradingInput {
        assets_valid: true,
        asset_reason: "All 101 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
    };
    let report = grade_first_lesson_readiness(input);
    assert_report_json_schema(
        "grade_first_lesson_readiness",
        "building-a-scene-first-world",
        false,
        &report,
    );
}

#[test]
fn grading_report_schema_version_follows_eatme_pattern() {
    let report = grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
    });
    assert!(
        report.schema_version.starts_with("eatme.assets/"),
        "schema_version should start with eatme.assets/"
    );
    assert!(
        report.schema_version.ends_with("/v1"),
        "schema_version should end with /v1"
    );
}

#[test]
fn feature_complete_grading_inputs_produce_schema_valid_json() {
    let first_lesson_ready = grade_first_lesson_readiness(GradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
    });
    assert_step_status(
        &first_lesson_ready,
        "place-object",
        StepStatus::NotYetTested,
    );

    let loops_complete = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        student_program: Some(complete_loops_program()),
    });
    assert_step_status(&loops_complete, "run-world", StepStatus::NotYetTested);

    let events_complete = grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: "assets ok".into(),
        deps_available: true,
        deps_reason: "deps ok".into(),
        student_program: Some(complete_events_program()),
    });
    assert_step_status(&events_complete, "run-world", StepStatus::NotYetTested);

    let reports = vec![
        (
            "first_lesson_ready",
            "building-a-scene-first-world",
            false,
            first_lesson_ready,
        ),
        (
            "loops_complete",
            "loops-and-conditionals-mini-challenge",
            false,
            loops_complete,
        ),
        (
            "events_complete",
            "events-collision-proximity-game",
            false,
            events_complete,
        ),
        (
            "scene_building_pass",
            "building-a-scene-first-world",
            true,
            grade_scene_building(SceneBuildingGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_scene: Some(complete_scene()),
            }),
        ),
        (
            "sequencing_pass",
            "procedure-sequencing-do-in-order-do-together",
            true,
            grade_sequencing(SequencingGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                sequence_blocks: Some(vec![
                    SequenceBlock {
                        kind: SequenceKind::DoInOrder,
                        steps: vec!["walk".into(), "turn".into()],
                    },
                    SequenceBlock {
                        kind: SequenceKind::DoTogether,
                        steps: vec!["wave".into(), "smile".into()],
                    },
                ]),
            }),
        ),
        (
            "variables_pass",
            "using-variables-mini-challenge",
            true,
            grade_variables(VariablesGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_variables_program()),
            }),
        ),
        (
            "parameters_pass",
            "parameters-mini-challenge",
            true,
            grade_parameters(ParametersGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_parameters_program()),
            }),
        ),
        (
            "arrays_pass",
            "arrays-collection-choreography",
            true,
            grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_arrays_program()),
            }),
        ),
        (
            "comments_pass",
            "comments-mini-challenge",
            true,
            grade_comments(CommentsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_comments_program()),
            }),
        ),
        (
            "inheritance_pass",
            "inheritance-oop-mini-challenge",
            true,
            grade_inheritance_oop(InheritanceOopGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_inheritance_program()),
            }),
        ),
        (
            "functions_pass",
            "using-functions-mini-challenge",
            true,
            grade_functions(FunctionsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_functions_program()),
            }),
        ),
        (
            "creative_pass",
            "creative-design-project",
            true,
            grade_creative_project(CreativeProjectGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_creative_program()),
            }),
        ),
        (
            "nested_control_pass",
            "nested-control-flow-relational-expressions",
            true,
            grade_nested_control_flow(NestedControlFlowGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_nested_control_program()),
            }),
        ),
        (
            "games_narrative_pass",
            "games-and-interactive-narrative",
            true,
            grade_games_and_narrative(GamesNarrativeGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(complete_games_narrative_program()),
            }),
        ),
    ];

    for (case_name, lesson, expected_passed, report) in reports {
        assert_report_json_schema(case_name, lesson, expected_passed, &report);
        if case_name == "games_narrative_pass" {
            assert!(
                report.steps.iter().any(|step| {
                    matches!(
                        (step.name.as_str(), &step.status),
                        ("grade-game-project", StepStatus::Ready)
                            | ("grade-narrative-project", StepStatus::Ready)
                    )
                }),
                "games_narrative_pass should produce at least one ready final grade"
            );
        } else {
            assert!(
                report
                    .steps
                    .iter()
                    .all(|step| step.status != StepStatus::Blocked),
                "{case_name} should not have blocked steps in the feature-complete path"
            );
        }
    }
}

#[test]
fn failing_grading_inputs_produce_schema_valid_json() {
    let reports = vec![
        (
            "first_lesson_blocked",
            "building-a-scene-first-world",
            false,
            grade_first_lesson_readiness(GradingInput {
                assets_valid: false,
                asset_reason: "assets missing".into(),
                deps_available: false,
                deps_reason: "deps missing".into(),
            }),
        ),
        (
            "loops_fail",
            "loops-and-conditionals-mini-challenge",
            false,
            grade_loops_and_conditionals(LoopsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_loops_program()),
            }),
        ),
        (
            "events_fail",
            "events-collision-proximity-game",
            false,
            grade_events_and_collision(EventsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_events_program()),
            }),
        ),
        (
            "scene_building_fail",
            "building-a-scene-first-world",
            false,
            grade_scene_building(SceneBuildingGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_scene: Some(incomplete_scene()),
            }),
        ),
        (
            "sequencing_fail",
            "procedure-sequencing-do-in-order-do-together",
            false,
            grade_sequencing(SequencingGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                sequence_blocks: Some(vec![SequenceBlock {
                    kind: SequenceKind::DoInOrder,
                    steps: vec!["walk".into()],
                }]),
            }),
        ),
        (
            "variables_fail",
            "using-variables-mini-challenge",
            false,
            grade_variables(VariablesGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_variables_program()),
            }),
        ),
        (
            "parameters_fail",
            "parameters-mini-challenge",
            false,
            grade_parameters(ParametersGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_parameters_program()),
            }),
        ),
        (
            "arrays_fail",
            "arrays-collection-choreography",
            false,
            grade_arrays_and_arithmetic(ArraysArithmeticGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_arrays_program()),
            }),
        ),
        (
            "comments_fail",
            "comments-mini-challenge",
            false,
            grade_comments(CommentsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_comments_program()),
            }),
        ),
        (
            "inheritance_fail",
            "inheritance-oop-mini-challenge",
            false,
            grade_inheritance_oop(InheritanceOopGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_inheritance_program()),
            }),
        ),
        (
            "functions_fail",
            "using-functions-mini-challenge",
            false,
            grade_functions(FunctionsGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_functions_program()),
            }),
        ),
        (
            "creative_fail",
            "creative-design-project",
            false,
            grade_creative_project(CreativeProjectGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_creative_program()),
            }),
        ),
        (
            "nested_control_fail",
            "nested-control-flow-relational-expressions",
            false,
            grade_nested_control_flow(NestedControlFlowGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_nested_control_program()),
            }),
        ),
        (
            "games_narrative_fail",
            "games-and-interactive-narrative",
            false,
            grade_games_and_narrative(GamesNarrativeGradingInput {
                assets_valid: true,
                asset_reason: "assets ok".into(),
                deps_available: true,
                deps_reason: "deps ok".into(),
                student_program: Some(failing_games_narrative_program()),
            }),
        ),
    ];

    for (case_name, lesson, expected_passed, report) in reports {
        assert_report_json_schema(case_name, lesson, expected_passed, &report);
        assert!(
            report
                .steps
                .iter()
                .any(|step| step.status == StepStatus::Blocked),
            "{case_name} should contain at least one blocked step in the failing path"
        );
    }
}

fn assert_report_json_schema(
    case_name: &str,
    expected_lesson: &str,
    expected_passed: bool,
    report: &GradingReport,
) {
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(
        json["schema_version"], "eatme.assets/grading/v1",
        "{case_name}"
    );
    assert_eq!(json["lesson"], expected_lesson, "{case_name}");
    assert_eq!(json["passed"], expected_passed, "{case_name}");

    let steps = json["steps"].as_array().unwrap();
    assert!(!steps.is_empty(), "{case_name} must include grading steps");
    for step in steps {
        assert_step_json_schema(case_name, step);
    }

    if let Some(quality_scores) = json.get("quality_scores") {
        let scores = quality_scores.as_array().unwrap();
        for score in scores {
            assert!(
                score["score"].is_number(),
                "{case_name}: score must be numeric"
            );
            assert!(
                score["dimension"].is_string(),
                "{case_name}: dimension must be a string"
            );
            assert!(
                score["feedback"].is_string(),
                "{case_name}: feedback must be a string"
            );
        }
    }
}

fn assert_step_json_schema(case_name: &str, step: &Value) {
    assert!(
        step["name"].is_string(),
        "{case_name}: step name must be a string"
    );
    assert!(
        step["status"].is_string(),
        "{case_name}: step status must be a string"
    );
    assert!(
        step["reason"].is_string(),
        "{case_name}: step reason must be a string"
    );
    assert!(
        step["depends_on"].is_array(),
        "{case_name}: depends_on must be an array"
    );
    let status = step["status"].as_str().unwrap();
    assert!(
        ["ready", "blocked", "not-yet-tested"].contains(&status),
        "{case_name}: unexpected status {status}"
    );
    for dep in step["depends_on"].as_array().unwrap() {
        assert!(
            dep.is_string(),
            "{case_name}: dependency names must be strings"
        );
    }
}

fn assert_step_status(report: &GradingReport, step_name: &str, expected: StepStatus) {
    let actual = report
        .steps
        .iter()
        .find(|step| step.name == step_name)
        .map(|step| step.status.clone())
        .unwrap();
    assert_eq!(actual, expected, "unexpected status for {step_name}");
}

fn procedure(name: &str, body: Vec<Statement>) -> Procedure {
    Procedure {
        name: name.into(),
        parameters: vec![],
        body,
    }
}

fn parameterized_procedure(name: &str, body: Vec<Statement>) -> Procedure {
    Procedure {
        name: name.into(),
        parameters: vec![Parameter {
            name: "amount".into(),
            param_type: "Number".into(),
        }],
        body,
    }
}

fn method_call(method: &str, arguments: Vec<&str>) -> Statement {
    Statement::MethodCall {
        object: "actor".into(),
        method: method.into(),
        arguments: arguments.into_iter().map(str::to_string).collect(),
    }
}

fn comment(text: &str) -> Statement {
    Statement::Comment { text: text.into() }
}

fn complete_loops_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::CountLoop {
            count: 4,
            body: vec![Statement::IfElse {
                condition: "score < 10".into(),
                if_body: vec![method_call("move", vec![])],
                else_body: vec![method_call("turn", vec![])],
            }],
        }],
    )])
}

fn failing_loops_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::CountLoop {
            count: 2,
            body: vec![method_call("move", vec![])],
        }],
    )])
}

fn complete_events_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![
            Statement::EventListener {
                event: "whenKeyPressed".into(),
                body: vec![method_call("jump", vec![])],
            },
            Statement::CollisionListener {
                object_a: "actor".into(),
                object_b: "tree".into(),
                body: vec![method_call("say", vec!["\"ouch\""])],
            },
        ],
    )])
}

fn failing_events_program() -> Program {
    Program::new(vec![procedure("run", vec![method_call("move", vec![])])])
}

fn complete_scene() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: true,
        objects: vec![scene_object("rabbit", 1.0), scene_object("tree", 2.0)],
        camera: Some(CameraPose {
            position: Vec3 {
                x: 0.0,
                y: 5.0,
                z: 10.0,
            },
        }),
    }
}

fn incomplete_scene() -> SceneLayout {
    SceneLayout {
        ground_present: true,
        sky_present: false,
        objects: vec![SceneObject {
            name: "rabbit".into(),
            kind: "Prop".into(),
            position: None,
            size: Some(1.0),
            color: Some("white".into()),
            opacity: None,
        }],
        camera: None,
    }
}

fn scene_object(name: &str, x: f32) -> SceneObject {
    SceneObject {
        name: name.into(),
        kind: "Prop".into(),
        position: Some(Vec3 { x, y: 0.0, z: 0.0 }),
        size: Some(1.0),
        color: Some("red".into()),
        opacity: Some(1.0),
    }
}

fn complete_variables_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![
            Statement::VariableDeclaration {
                name: "score".into(),
                var_type: "Number".into(),
                initial_value: "0".into(),
            },
            method_call("say", vec!["score"]),
            Statement::VariableAssignment {
                name: "score".into(),
                value: "score + 1".into(),
            },
        ],
    )])
}

fn failing_variables_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::VariableDeclaration {
            name: "score".into(),
            var_type: "Number".into(),
            initial_value: "0".into(),
        }],
    )])
}

fn complete_parameters_program() -> Program {
    Program::new(vec![parameterized_procedure(
        "setDistance",
        vec![method_call("move", vec!["amount"])],
    )])
}

fn failing_parameters_program() -> Program {
    Program::new(vec![procedure("run", vec![method_call("move", vec![])])])
}

fn complete_arrays_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![
            Statement::ArrayDeclaration {
                name: "pets".into(),
                element_type: "Object".into(),
                elements: vec!["rabbit".into(), "cat".into()],
            },
            Statement::ArrayAccess {
                array: "pets".into(),
                index: "0".into(),
                target: "leader".into(),
            },
            Statement::ForEachArray {
                item_name: "pet".into(),
                array: "pets".into(),
                body: vec![method_call("move", vec!["pet"])],
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Add,
                left: "1".into(),
                right: "2".into(),
                result: "sum".into(),
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Subtract,
                left: "5".into(),
                right: "3".into(),
                result: "difference".into(),
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Multiply,
                left: "2".into(),
                right: "4".into(),
                result: "product".into(),
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Divide,
                left: "8".into(),
                right: "2".into(),
                result: "quotient".into(),
            },
        ],
    )])
}

fn failing_arrays_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::ArrayDeclaration {
            name: "pets".into(),
            element_type: "Object".into(),
            elements: vec!["rabbit".into()],
        }],
    )])
}

fn complete_comments_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![
            comment("Explain why the rabbit waits before speaking"),
            method_call("say", vec!["\"hello\""]),
        ],
    )])
}

fn failing_comments_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![comment("todo"), method_call("say", vec!["\"hello\""])],
    )])
}

fn complete_inheritance_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::UserTypeDeclaration {
            name: "FlyingRabbit".into(),
            extends: Some("Rabbit".into()),
            methods: vec![procedure("glide", vec![method_call("move", vec![])])],
        }],
    )])
}

fn failing_inheritance_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::UserTypeDeclaration {
            name: "FlyingRabbit".into(),
            extends: None,
            methods: vec![],
        }],
    )])
}

fn complete_functions_program() -> Program {
    Program {
        procedures: vec![procedure(
            "run",
            vec![Statement::FunctionCall {
                object: "this".into(),
                function: "scoreBonus".into(),
                arguments: vec![],
            }],
        )],
        functions: vec![Function {
            name: "scoreBonus".into(),
            return_type: "Number".into(),
            body: vec![Statement::ReturnStatement {
                expression: "42".into(),
            }],
        }],
    }
}

fn failing_functions_program() -> Program {
    Program {
        procedures: vec![procedure("run", vec![])],
        functions: vec![Function {
            name: "scoreBonus".into(),
            return_type: "Number".into(),
            body: vec![],
        }],
    }
}

fn complete_creative_program() -> Program {
    Program::new(vec![
        procedure(
            "setupScene",
            vec![method_call("move", vec![]), method_call("turn", vec![])],
        ),
        procedure(
            "run",
            vec![
                Statement::CountLoop {
                    count: 2,
                    body: vec![method_call("jump", vec![])],
                },
                Statement::EventListener {
                    event: "whenMouseClicked".into(),
                    body: vec![method_call("say", vec!["\"clicked\""])],
                },
            ],
        ),
    ])
}

fn failing_creative_program() -> Program {
    Program::new(vec![procedure("run", vec![method_call("move", vec![])])])
}

fn complete_nested_control_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::CountLoop {
            count: 3,
            body: vec![Statement::IfElse {
                condition: "score < 10 && lives > 0 || score == 5".into(),
                if_body: vec![Statement::CountLoop {
                    count: 2,
                    body: vec![Statement::IfElse {
                        condition: "timer < 3 && bonus > 1 || bonus == 2".into(),
                        if_body: vec![Statement::ForEachArray {
                            item_name: "pet".into(),
                            array: "pets".into(),
                            body: vec![Statement::IfElse {
                                condition: "petCount < 9 && score > 1 || score == 2".into(),
                                if_body: vec![method_call("say", vec!["\"nested\""])],
                                else_body: vec![],
                            }],
                        }],
                        else_body: vec![],
                    }],
                }],
                else_body: vec![],
            }],
        }],
    )])
}

fn failing_nested_control_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![Statement::IfElse {
            condition: "score < 10".into(),
            if_body: vec![method_call("move", vec![])],
            else_body: vec![],
        }],
    )])
}

fn complete_games_narrative_program() -> Program {
    Program::new(vec![procedure(
        "run",
        vec![
            Statement::VariableDeclaration {
                name: "score".into(),
                var_type: "Number".into(),
                initial_value: "0".into(),
            },
            Statement::EventListener {
                event: "whenKeyPressed".into(),
                body: vec![Statement::IfElse {
                    condition: "score < 10".into(),
                    if_body: vec![
                        method_call("say", vec!["\"keep going\""]),
                        Statement::VariableAssignment {
                            name: "score".into(),
                            value: "score + 1".into(),
                        },
                    ],
                    else_body: vec![],
                }],
            },
            Statement::CollisionListener {
                object_a: "actor".into(),
                object_b: "goal".into(),
                body: vec![method_call("turn", vec![])],
            },
        ],
    )])
}

fn failing_games_narrative_program() -> Program {
    Program::new(vec![procedure("run", vec![method_call("move", vec![])])])
}
