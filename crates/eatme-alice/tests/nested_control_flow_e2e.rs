// Nested control flow and relational expression E2E tests.
// Exercises: structured .a3p parsing → nesting inspection → grading report.

use eatme_assets::{NestedControlFlowGradingInput, StepStatus, grade_nested_control_flow};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
#[allow(dead_code)]
mod launch_smoke_support;
mod structured_a3p_support;

use a3p_parser_support::parse_a3p_program;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

fn all_ready_input(program: Option<Program>) -> NestedControlFlowGradingInput {
    NestedControlFlowGradingInput {
        assets_valid: true,
        asset_reason: "All scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn flat_control_xml() -> &'static str {
    r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="CountLoop" count="3">
            <body>
              <statement type="MethodInvocation" object="this.hero" method="move">
                <argument value="FORWARD" />
                <argument value="1.0" />
              </statement>
            </body>
          </statement>
          <statement type="ConditionalStatement" condition="score less than 3 and timer greater than 0">
            <ifBody>
              <statement type="MethodInvocation" object="this.hero" method="turn">
                <argument value="LEFT" />
              </statement>
            </ifBody>
            <elseBody>
              <statement type="MethodInvocation" object="this.hero" method="turn">
                <argument value="RIGHT" />
              </statement>
            </elseBody>
          </statement>
          <statement type="ConditionalStatement" condition="score equals 3 or bonus equals 1">
            <ifBody>
              <statement type="MethodInvocation" object="this.hero" method="say">
                <argument value='"Ready"' />
              </statement>
            </ifBody>
            <elseBody>
              <statement type="MethodInvocation" object="this.hero" method="think">
                <argument value='"Keep trying"' />
              </statement>
            </elseBody>
          </statement>
        </body>
      </procedure>
    </program>
    "#
}

fn nested_control_xml() -> &'static str {
    r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="CountLoop" count="4">
            <body>
              <statement type="ConditionalStatement" condition="score less than 10 and timer greater than 0">
                <ifBody>
                  <statement type="MethodInvocation" object="this.hero" method="move">
                    <argument value="FORWARD" />
                    <argument value="1.0" />
                  </statement>
                </ifBody>
                <elseBody>
                  <statement type="MethodInvocation" object="this.hero" method="think">
                    <argument value='"Pause"' />
                  </statement>
                </elseBody>
              </statement>
            </body>
          </statement>
          <statement type="ConditionalStatement" condition="round equals 2 or bonus equals 1">
            <ifBody>
              <statement type="CountLoop" count="2">
                <body>
                  <statement type="MethodInvocation" object="this.hero" method="jump" />
                </body>
              </statement>
            </ifBody>
            <elseBody>
              <statement type="MethodInvocation" object="this.hero" method="turn">
                <argument value="LEFT" />
              </statement>
            </elseBody>
          </statement>
        </body>
      </procedure>
    </program>
    "#
}

fn deep_nested_control_xml() -> &'static str {
    r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="CountLoop" count="3">
            <body>
              <statement type="ConditionalStatement" condition="score less than 10 and lives greater than 0">
                <ifBody>
                  <statement type="CountLoop" count="2">
                    <body>
                      <statement type="MethodInvocation" object="this.hero" method="move">
                        <argument value="FORWARD" />
                        <argument value="0.5" />
                      </statement>
                    </body>
                  </statement>
                </ifBody>
                <elseBody>
                  <statement type="MethodInvocation" object="this.hero" method="say">
                    <argument value='"Stop"' />
                  </statement>
                </elseBody>
              </statement>
              <statement type="CountLoop" count="2">
                <body>
                  <statement type="MethodInvocation" object="this.hero" method="turn">
                    <argument value="LEFT" />
                  </statement>
                </body>
              </statement>
            </body>
          </statement>
          <statement type="ConditionalStatement" condition="round equals 2 or bonus equals 1">
            <ifBody>
              <statement type="CountLoop" count="2">
                <body>
                  <statement type="ConditionalStatement" condition="coins greater than 3 and timer less than 30">
                    <ifBody>
                      <statement type="MethodInvocation" object="this.hero" method="jump" />
                    </ifBody>
                    <elseBody>
                      <statement type="MethodInvocation" object="this.hero" method="think">
                        <argument value='"Wait"' />
                      </statement>
                    </elseBody>
                  </statement>
                </body>
              </statement>
            </ifBody>
            <elseBody>
              <statement type="MethodInvocation" object="this.hero" method="say">
                <argument value='"Retry"' />
              </statement>
            </elseBody>
          </statement>
        </body>
      </procedure>
    </program>
    "#
}

fn parse_fixture(name: &str, xml: &str) -> Program {
    let path = write_structured_a3p(name, xml);
    parse_structured_a3p_program(&path)
        .unwrap_or_else(|| panic!("failed to parse structured fixture {}", path.display()))
}

fn has_if_inside_loop(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::CountLoop { body, .. } => {
            body.iter()
                .any(|nested| matches!(nested, Statement::IfElse { .. }))
                || has_if_inside_loop(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => has_if_inside_loop(if_body) || has_if_inside_loop(else_body),
        Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::DoInOrder { body } => has_if_inside_loop(body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| has_if_inside_loop(&method.body)),
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. } => false,
    })
}

fn has_loop_inside_if(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::IfElse {
            if_body, else_body, ..
        } => {
            if_body
                .iter()
                .any(|nested| matches!(nested, Statement::CountLoop { .. }))
                || else_body
                    .iter()
                    .any(|nested| matches!(nested, Statement::CountLoop { .. }))
                || has_loop_inside_if(if_body)
                || has_loop_inside_if(else_body)
        }
        Statement::CountLoop { body, .. }
        | Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::DoInOrder { body } => has_loop_inside_if(body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| has_loop_inside_if(&method.body)),
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. } => false,
    })
}

fn has_nested_loops(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::CountLoop { body, .. } => {
            body.iter()
                .any(|nested| matches!(nested, Statement::CountLoop { .. }))
                || has_nested_loops(body)
        }
        Statement::IfElse {
            if_body, else_body, ..
        } => has_nested_loops(if_body) || has_nested_loops(else_body),
        Statement::EventListener { body, .. }
        | Statement::CollisionListener { body, .. }
        | Statement::ForEachArray { body, .. }
        | Statement::DoInOrder { body } => has_nested_loops(body),
        Statement::UserTypeDeclaration { methods, .. } => {
            methods.iter().any(|method| has_nested_loops(&method.body))
        }
        Statement::MethodCall { .. }
        | Statement::ReturnStatement { .. }
        | Statement::FunctionCall { .. }
        | Statement::VariableDeclaration { .. }
        | Statement::VariableAssignment { .. }
        | Statement::ArrayDeclaration { .. }
        | Statement::ArrayAccess { .. }
        | Statement::ArithmeticExpression { .. }
        | Statement::Comment { .. } => false,
    })
}

#[test]
fn nested_control_a3p_parses_if_inside_loop_loop_inside_if_and_nested_loops() {
    let program = parse_fixture("deep-nested-control-parse", deep_nested_control_xml());
    let body = &program.procedures[0].body;

    assert!(
        has_if_inside_loop(body),
        "fixture should contain if-inside-loop"
    );
    assert!(
        has_loop_inside_if(body),
        "fixture should contain loop-inside-if"
    );
    assert!(
        has_nested_loops(body),
        "fixture should contain nested loops"
    );
}

#[test]
fn nested_control_grading_detects_relational_expressions() {
    let program = parse_fixture("nested-relational", deep_nested_control_xml());
    let report = grade_nested_control_flow(all_ready_input(Some(program)));

    assert!(
        report.passed,
        "deep nested fixture should satisfy grading report"
    );
    assert_eq!(report.lesson, "nested-control-flow-relational-expressions");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "detect-relational-expressions"
    );
}

#[test]
fn nested_control_grading_classifies_basic_depth_from_flat_program() {
    let program = parse_fixture("flat-control", flat_control_xml());
    let report = grade_nested_control_flow(all_ready_input(Some(program)));

    assert!(
        report.passed,
        "flat control fixture should still satisfy basic grading"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "grade-basic-nesting"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Blocked,
        "grade-intermediate-nesting"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Blocked,
        "grade-advanced-nesting"
    );
}

#[test]
fn nested_control_grading_classifies_intermediate_depth() {
    let program = parse_fixture("nested-control", nested_control_xml());
    let report = grade_nested_control_flow(all_ready_input(Some(program)));

    assert!(
        report.passed,
        "nested control fixture should satisfy grading"
    );
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "grade-basic-nesting"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "grade-intermediate-nesting"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Blocked,
        "grade-advanced-nesting"
    );
}

#[test]
fn nested_control_grading_classifies_advanced_depth() {
    let program = parse_fixture("advanced-control", deep_nested_control_xml());
    let report = grade_nested_control_flow(all_ready_input(Some(program)));

    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "grade-basic-nesting"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "grade-intermediate-nesting"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "grade-advanced-nesting"
    );
}

#[test]
fn nested_control_ast_survives_json_round_trip() {
    let program = parse_fixture("nested-control-round-trip", deep_nested_control_xml());
    let json = serde_json::to_string(&program).expect("serialize program");
    let restored: Program = serde_json::from_str(&json).expect("deserialize program");
    assert_eq!(program, restored);
}

#[test]
fn real_alice_nested_control_flow_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice nested-control integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/nested-control-real");
    let run_id = format!(
        "real-nested-control-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let manifest = eatme_alice::run_launch_smoke(&eatme_alice::LaunchSmokeOptions {
        alice_home: alice_home(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 90,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: eatme_alice::LaunchSmokeScenario::new("relational-expressions-comparison-lab"),
    })
    .expect("run_launch_smoke should succeed for nested-control scenario");

    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category, got: {:?}",
        manifest.failure_category,
    );
    for key in ["dependencies_available", "process_started"] {
        let result = manifest
            .assertions
            .get(key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(result.passed, "assertion {key} failed: {}", result.detail);
    }

    let a3p_path = starter_project_path("amazonMinimum");
    let mut student_program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));
    assert!(
        !student_program.procedures.is_empty(),
        "parsed starter project should have at least one procedure"
    );

    if let Some(Procedure { body, .. }) = student_program.procedures.first_mut() {
        body.push(Statement::CountLoop {
            count: 3,
            body: vec![
                Statement::IfElse {
                    condition: "score less than 10 and lives greater than 0".into(),
                    if_body: vec![Statement::CountLoop {
                        count: 2,
                        body: vec![Statement::MethodCall {
                            object: "this.hero".into(),
                            method: "move".into(),
                            arguments: vec!["FORWARD".into(), "0.5".into()],
                        }],
                    }],
                    else_body: vec![Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "say".into(),
                        arguments: vec!["\"Stop\"".into()],
                    }],
                },
                Statement::CountLoop {
                    count: 2,
                    body: vec![Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "turn".into(),
                        arguments: vec!["LEFT".into()],
                    }],
                },
            ],
        });
        body.push(Statement::IfElse {
            condition: "round equals 2 or bonus equals 1".into(),
            if_body: vec![Statement::CountLoop {
                count: 2,
                body: vec![Statement::IfElse {
                    condition: "coins greater than 3 and timer less than 30".into(),
                    if_body: vec![Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "jump".into(),
                        arguments: vec![],
                    }],
                    else_body: vec![Statement::MethodCall {
                        object: "this.hero".into(),
                        method: "think".into(),
                        arguments: vec!["\"Wait\"".into()],
                    }],
                }],
            }],
            else_body: vec![Statement::MethodCall {
                object: "this.hero".into(),
                method: "say".into(),
                arguments: vec!["\"Retry\"".into()],
            }],
        });
    }

    let report = grade_nested_control_flow(all_ready_input(Some(student_program)));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "nested-control-flow-relational-expressions");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "detect-relational-expressions"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "grade-advanced-nesting"
    );
    assert!(
        report.passed,
        "augmented starter should grade as advanced nested control"
    );

    let manifest_dir = runs_dir
        .join("relational-expressions-comparison-lab")
        .join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
