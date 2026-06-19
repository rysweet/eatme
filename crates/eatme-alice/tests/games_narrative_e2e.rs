// Games and interactive narrative E2E tests.
// Exercises: structured .a3p parsing → AST inspection → grading report.

use eatme_assets::{GamesNarrativeGradingInput, StepStatus, grade_games_and_narrative};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
#[allow(dead_code)]
mod launch_smoke_support;
mod structured_a3p_support;

use a3p_parser_support::parse_a3p_program;
use launch_smoke_support::{alice_home, real_alice_enabled, starter_project_path};
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

fn all_ready_input(program: Option<Program>) -> GamesNarrativeGradingInput {
    GamesNarrativeGradingInput {
        assets_valid: true,
        asset_reason: "All scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: program,
    }
}

fn game_project_xml() -> &'static str {
    r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="VariableDeclaration" name="score" var_type="WholeNumber" initial_value="0" />
          <statement type="AddEventListener" event="SceneActivated">
            <body>
              <statement type="ConditionalStatement" condition="score less than 10 and lives greater than 0">
                <ifBody>
                  <statement type="MethodInvocation" object="this.hero" method="move">
                    <argument value="FORWARD" />
                    <argument value="1.0" />
                  </statement>
                </ifBody>
                <elseBody>
                  <statement type="MethodInvocation" object="this.hero" method="say">
                    <argument value='"Game over"' />
                  </statement>
                </elseBody>
              </statement>
            </body>
          </statement>
          <statement type="CollisionStartEventListener" object_a="this.hero" object_b="this.coin">
            <body>
              <statement type="VariableAssignment" name="score" value="score + 10" />
              <statement type="MethodInvocation" object="this.scoreBoard" method="setText">
                <argument value="score" />
              </statement>
            </body>
          </statement>
        </body>
      </procedure>
    </program>
    "#
}

fn narrative_project_xml() -> &'static str {
    r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="DoInOrder">
            <body>
              <statement type="MethodInvocation" object="this.narrator" method="say">
                <argument value='"Welcome, traveler."' />
              </statement>
              <statement type="MethodInvocation" object="this.guardian" method="think">
                <argument value='"Choose wisely."' />
              </statement>
              <statement type="MethodInvocation" object="this.narrator" method="say">
                <argument value='"The gate opens."' />
              </statement>
            </body>
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

fn contains_game_loop_pattern(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::EventListener { body, .. } | Statement::CollisionListener { body, .. } => {
            body.iter().any(|nested| {
                matches!(nested, Statement::IfElse { if_body, else_body, .. }
                    if !if_body.is_empty() || !else_body.is_empty())
            })
        }
        Statement::CountLoop { body, .. }
        | Statement::DoInOrder { body }
        | Statement::ForEachArray { body, .. } => contains_game_loop_pattern(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_game_loop_pattern(if_body) || contains_game_loop_pattern(else_body),
        Statement::UserTypeDeclaration { methods, .. } => methods
            .iter()
            .any(|method| contains_game_loop_pattern(&method.body)),
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

fn count_do_in_order_dialogue(stmts: &[Statement]) -> usize {
    stmts.iter()
        .map(|stmt| match stmt {
            Statement::DoInOrder { body } => body
                .iter()
                .filter(|nested| {
                    matches!(nested, Statement::MethodCall { method, .. } if matches!(method.as_str(), "say" | "think"))
                })
                .count(),
            Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::ForEachArray { body, .. } => count_do_in_order_dialogue(body),
            Statement::IfElse {
                if_body, else_body, ..
            } => count_do_in_order_dialogue(if_body) + count_do_in_order_dialogue(else_body),
            Statement::UserTypeDeclaration { methods, .. } => methods
                .iter()
                .map(|method| count_do_in_order_dialogue(&method.body))
                .sum(),
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => 0,
        })
        .sum()
}

#[test]
fn game_a3p_parses_event_collision_and_score_tracking() {
    let program = parse_fixture("games-mechanics", game_project_xml());
    let body = &program.procedures[0].body;

    assert!(
        body.iter()
            .any(|stmt| matches!(stmt, Statement::EventListener { .. })),
        "fixture should parse an event listener"
    );
    assert!(
        body.iter()
            .any(|stmt| matches!(stmt, Statement::CollisionListener { .. })),
        "fixture should parse a collision handler"
    );
    assert!(
        body.iter().any(
            |stmt| matches!(stmt, Statement::VariableDeclaration { name, .. } if name == "score")
        ),
        "fixture should parse score tracking state"
    );
    assert!(
        contains_game_loop_pattern(body),
        "fixture should parse event → condition → action game loop evidence"
    );
}

#[test]
fn games_narrative_grading_detects_game_project() {
    let program = parse_fixture("games-grading", game_project_xml());
    let report = grade_games_and_narrative(all_ready_input(Some(program)));

    assert!(report.passed, "game fixture should satisfy grading report");
    assert_eq!(report.lesson, "games-and-interactive-narrative");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Ready,
        "detect-event-listener"
    );
    assert_eq!(
        report.steps[4].status,
        StepStatus::Ready,
        "detect-collision-handler"
    );
    assert_eq!(
        report.steps[5].status,
        StepStatus::Ready,
        "detect-state-tracking"
    );
    assert_eq!(
        report.steps[6].status,
        StepStatus::Ready,
        "detect-game-loop-pattern"
    );
    assert_eq!(
        report.steps[7].status,
        StepStatus::Ready,
        "grade-game-project"
    );
    assert_eq!(
        report.steps[10].status,
        StepStatus::Blocked,
        "grade-narrative-project"
    );
}

#[test]
fn narrative_a3p_parses_do_in_order_dialogue_sequence() {
    let program = parse_fixture("narrative-sequence", narrative_project_xml());
    let body = &program.procedures[0].body;

    assert!(
        body.iter()
            .any(|stmt| matches!(stmt, Statement::DoInOrder { .. })),
        "fixture should parse a DoInOrder sequence"
    );
    assert!(
        count_do_in_order_dialogue(body) >= 2,
        "fixture should parse dialogue-like beats inside DoInOrder"
    );
}

#[test]
fn games_narrative_grading_detects_narrative_project() {
    let program = parse_fixture("narrative-grading", narrative_project_xml());
    let report = grade_games_and_narrative(all_ready_input(Some(program)));

    assert!(
        report.passed,
        "narrative fixture should satisfy grading report"
    );
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(
        report.steps[8].status,
        StepStatus::Ready,
        "detect-do-in-order"
    );
    assert_eq!(
        report.steps[9].status,
        StepStatus::Ready,
        "detect-dialogue-sequence"
    );
    assert_eq!(
        report.steps[10].status,
        StepStatus::Ready,
        "grade-narrative-project"
    );
    assert_eq!(
        report.steps[7].status,
        StepStatus::Blocked,
        "grade-game-project"
    );
}

#[test]
fn games_narrative_ast_survives_json_round_trip() {
    let program = parse_fixture("games-narrative-round-trip", narrative_project_xml());
    let json = serde_json::to_string(&program).expect("serialize program");
    let restored: Program = serde_json::from_str(&json).expect("deserialize program");
    assert_eq!(program, restored);
}

#[test]
fn real_alice_games_narrative_grading_integration() {
    if !real_alice_enabled() {
        eprintln!(
            "skipping real-Alice games/narrative integration test (set EATME_REAL_ALICE=1 to enable)"
        );
        return;
    }

    let runs_dir = std::env::current_dir()
        .unwrap()
        .join("target/test-work/games-narrative-real");
    let run_id = format!(
        "real-games-narrative-{}",
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
        scenario: eatme_alice::LaunchSmokeScenario::new("mythic-choice-event-tree"),
    })
    .expect("run_launch_smoke should succeed for games/narrative scenario");

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
        body.push(Statement::VariableDeclaration {
            name: "score".into(),
            var_type: "WholeNumber".into(),
            initial_value: "0".into(),
        });
        body.push(Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![Statement::IfElse {
                condition: "score less than 10 and lives greater than 0".into(),
                if_body: vec![Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "move".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
                else_body: vec![Statement::MethodCall {
                    object: "this.hero".into(),
                    method: "say".into(),
                    arguments: vec!["\"Game over\"".into()],
                }],
            }],
        });
        body.push(Statement::CollisionListener {
            object_a: "this.hero".into(),
            object_b: "this.coin".into(),
            body: vec![Statement::VariableAssignment {
                name: "score".into(),
                value: "score + 10".into(),
            }],
        });
        body.push(Statement::DoInOrder {
            body: vec![
                Statement::MethodCall {
                    object: "this.narrator".into(),
                    method: "say".into(),
                    arguments: vec!["\"Welcome, traveler.\"".into()],
                },
                Statement::MethodCall {
                    object: "this.guardian".into(),
                    method: "think".into(),
                    arguments: vec!["\"Choose wisely.\"".into()],
                },
            ],
        });
    }

    let report = grade_games_and_narrative(all_ready_input(Some(student_program)));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "games-and-interactive-narrative");
    assert_eq!(
        report.steps[7].status,
        StepStatus::Ready,
        "grade-game-project"
    );
    assert_eq!(
        report.steps[10].status,
        StepStatus::Ready,
        "grade-narrative-project"
    );
    assert!(
        report.passed,
        "augmented starter should grade as game+narrative"
    );

    let manifest_dir = runs_dir.join("mythic-choice-event-tree").join(&run_id);
    assert!(
        manifest_dir.is_dir(),
        "run directory should exist at {}",
        manifest_dir.display()
    );
}
