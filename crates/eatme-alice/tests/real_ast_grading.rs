#![allow(unexpected_cfgs)]
//! Real-Alice e2e grading integration tests.
//!
//! Loads real `.a3p` starter projects from `ALICE_HOME`, constructs `Program`
//! structures from actual Alice scene data via regex-based XML extraction,
//! and verifies grading pipelines produce correct results against real programs.
//!
//! **Gated behind `EATME_REAL_ALICE=1`** — requires an actual Alice installation
//! with starter projects on disk. CI sets the env var; local devs skip by default.
//!
//! # Covered pipelines
//!
//! | Lesson | Pipeline              | Status                                  |
//! |--------|-----------------------|-----------------------------------------|
//! | 3      | Loops & Conditionals  | ✅ Active                               |
//! | 4      | Events & Collision    | ✅ Active                               |
//! | 5      | Functions             | 🔴 TDD contract (behind feature gate)  |
//! | 6      | Variables             | 🔴 TDD contract (behind feature gate)  |
//! | 7      | Parameters            | 🔴 TDD contract (behind feature gate)  |
//! | 8      | Creative Project      | 🔴 TDD contract (behind feature gate)  |

use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use eatme_assets::grading_report::{LoopsGradingInput, StepStatus, grade_loops_and_conditionals};
use eatme_assets::{EventsGradingInput, grade_events_and_collision};
use eatme_core::ast::{Procedure, Program, Statement};
use regex::Regex;

// ---------------------------------------------------------------------------
// Compiled regex cache — each pattern compiled once across all test runs
// ---------------------------------------------------------------------------

fn re_user_method_type_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"UserMethod"[^>]*name\s*=\s*"([^"]+)""#).unwrap())
}

fn re_user_method_name_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"name\s*=\s*"([^"]+)"[^>]*type\s*=\s*"UserMethod""#).unwrap())
}

fn re_method_invocation() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"type\s*=\s*"MethodInvocation"[^>]*method\s*=\s*"([^"]*)"#).unwrap()
    })
}

fn re_conditional() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"ConditionalStatement""#).unwrap())
}

fn re_count_loop() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"CountLoop""#).unwrap())
}

fn re_event_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"type\s*=\s*"AddEventListener"[^>]*event\s*=\s*"([^"]*)"#).unwrap()
    })
}

fn re_collision_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"CollisionStart(?:Event)?Listener""#).unwrap())
}

// ---------------------------------------------------------------------------
// Environment helpers (matching launch_smoke_real.rs pattern)
// ---------------------------------------------------------------------------

fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> PathBuf {
    PathBuf::from(env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()))
}

fn starter_project_path(name: &str) -> PathBuf {
    alice_home()
        .join("starter-projects")
        .join(format!("{name}.a3p"))
}

// ---------------------------------------------------------------------------
// .a3p ZIP parser — lightweight regex-based XML extraction
// ---------------------------------------------------------------------------

/// Parse an Alice 3 `.a3p` project file (ZIP) into a `Program`.
///
/// Opens the ZIP archive in memory, collects all XML content, and uses regex
/// to extract AST-relevant constructs:
///
/// | Alice XML type            | Maps to                       |
/// |---------------------------|-------------------------------|
/// | `UserMethod`              | `Procedure`                   |
/// | `MethodInvocation`        | `Statement::MethodCall`       |
/// | `ConditionalStatement`    | `Statement::IfElse`           |
/// | `CountLoop`               | `Statement::CountLoop`        |
/// | `AddEventListener`        | `Statement::EventListener`    |
/// | `CollisionStartListener`  | `Statement::CollisionListener`|
fn parse_a3p_program(path: &Path) -> Option<Program> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let mut all_xml = String::with_capacity(128 * 1024);
    let mut content_buf = String::new();
    for i in 0..archive.len() {
        // Skip entries that can't be read (e.g., corrupt binary assets) rather
        // than aborting the entire parse — we only need the XML content.
        let Ok(mut entry) = archive.by_index(i) else {
            continue;
        };
        let name = entry.name().to_string();
        if name.ends_with(".xml") {
            content_buf.clear();
            if entry.read_to_string(&mut content_buf).is_ok() {
                all_xml.push_str(&content_buf);
                all_xml.push('\n');
            }
        }
    }

    if all_xml.is_empty() {
        return None;
    }

    let procedures = extract_procedures(&all_xml);
    Some(Program {
        procedures,
        functions: vec![],
    })
}

/// Extract `Procedure` definitions from Alice XML content.
fn extract_procedures(xml: &str) -> Vec<Procedure> {
    let mut procedures = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for re in [re_user_method_type_first(), re_user_method_name_first()] {
        for cap in re.captures_iter(xml) {
            let name = cap[1].to_string();
            if seen_names.insert(name.clone()) {
                procedures.push(Procedure {
                    name,
                    parameters: vec![],
                    body: Vec::new(),
                });
            }
        }
    }

    let stmts = extract_statements(xml);

    // Flat model: assign all statements to the first procedure.
    if let Some(first) = procedures.first_mut() {
        first.body = stmts;
    } else if !stmts.is_empty() {
        procedures.push(Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: stmts,
        });
    }

    procedures
}

/// Extract all `Statement` nodes from Alice XML content.
fn extract_statements(xml: &str) -> Vec<Statement> {
    let mut stmts = Vec::new();

    // MethodInvocation → MethodCall
    for cap in re_method_invocation().captures_iter(xml) {
        stmts.push(Statement::MethodCall {
            object: "this".into(),
            method: cap[1].to_string(),
            arguments: vec![],
        });
    }

    // ConditionalStatement → IfElse
    for _ in re_conditional().find_iter(xml) {
        stmts.push(Statement::IfElse {
            condition: String::new(),
            if_body: vec![],
            else_body: vec![],
        });
    }

    // CountLoop → CountLoop
    for _ in re_count_loop().find_iter(xml) {
        stmts.push(Statement::CountLoop {
            count: 1,
            body: vec![],
        });
    }

    // AddEventListener → EventListener
    for cap in re_event_listener().captures_iter(xml) {
        stmts.push(Statement::EventListener {
            event: cap[1].to_string(),
            body: vec![],
        });
    }

    // CollisionStartListener → CollisionListener
    for _ in re_collision_listener().find_iter(xml) {
        stmts.push(Statement::CollisionListener {
            object_a: "unknown".into(),
            object_b: "unknown".into(),
            body: vec![],
        });
    }

    stmts
}

// ---------------------------------------------------------------------------
// Helpers: build grading inputs from a parsed program
// ---------------------------------------------------------------------------

fn loops_input(program: Program) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

fn events_input(program: Program) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "Real .a3p starter project parsed successfully".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

// ===========================================================================
// Lesson 3: Loops & Conditionals — real .a3p grading
// ===========================================================================

#[test]
fn real_alice_loops_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice loops grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    // Parser must extract at least one procedure from real data
    assert!(
        !program.procedures.is_empty(),
        "parsed program should have at least one procedure"
    );

    let report = grade_loops_and_conditionals(loops_input(program));

    // --- Schema contract ---
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps.len(), 7);

    // --- Precondition steps: all Ready ---
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // --- AST checks against real starter data ---
    // amazonMinimum.a3p has NO CountLoop → build-counting-loop = Blocked
    assert_eq!(report.steps[3].name, "build-counting-loop");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "starter project has no CountLoop — should be Blocked"
    );
    assert!(
        report.steps[3].reason.contains("No CountLoop found"),
        "reason should explain missing construct: {}",
        report.steps[3].reason
    );

    // Cascade: build-counting-loop blocked → add-conditional-branch blocked
    assert_eq!(report.steps[4].name, "add-conditional-branch");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "blocked by upstream build-counting-loop"
    );

    // Cascade: add-conditional-branch blocked → run-world blocked
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);

    // Cascade: run-world blocked → save-project blocked
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    // Overall: not passed (blocked steps)
    assert!(!report.passed);
}

// ===========================================================================
// Lesson 4: Events & Collision — real .a3p grading
// ===========================================================================

#[test]
fn real_alice_events_grading_with_starter_project() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice events grading test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let a3p_path = starter_project_path("amazonMinimum");
    assert!(
        a3p_path.exists(),
        "starter project not found at {}",
        a3p_path.display()
    );

    let program = parse_a3p_program(&a3p_path)
        .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

    let report = grade_events_and_collision(events_input(program));

    // --- Schema contract ---
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "events-collision-proximity-game");
    assert_eq!(report.steps.len(), 7);

    // --- Precondition steps: all Ready ---
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // --- AST checks against real starter data ---
    // amazonMinimum.a3p has NO EventListener → add-event-listener = Blocked
    assert_eq!(report.steps[3].name, "add-event-listener");
    assert_eq!(
        report.steps[3].status,
        StepStatus::Blocked,
        "starter project has no EventListener — should be Blocked"
    );
    assert!(
        report.steps[3].reason.contains("No EventListener found"),
        "reason should explain missing construct: {}",
        report.steps[3].reason
    );

    // Cascade: add-event-listener blocked → add-collision-listener blocked
    assert_eq!(report.steps[4].name, "add-collision-listener");
    assert_eq!(
        report.steps[4].status,
        StepStatus::Blocked,
        "blocked by upstream add-event-listener"
    );

    // Cascade: add-collision-listener blocked → run-world blocked
    assert_eq!(report.steps[5].name, "run-world");
    assert_eq!(report.steps[5].status, StepStatus::Blocked);

    // Cascade: run-world blocked → save-project blocked
    assert_eq!(report.steps[6].name, "save-project");
    assert_eq!(report.steps[6].status, StepStatus::Blocked);

    // Overall: not passed
    assert!(!report.passed);
}

// ===========================================================================
// Parser unit tests (run without EATME_REAL_ALICE)
// ===========================================================================

#[test]
fn parse_a3p_returns_none_for_missing_file() {
    let result = parse_a3p_program(Path::new("/nonexistent/path/to/project.a3p"));
    assert!(result.is_none(), "should return None for missing file");
}

#[test]
fn extract_statements_finds_method_invocations() {
    let xml = r#"<node type="MethodInvocation" method="walk" />"#;
    let stmts = extract_statements(xml);
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Statement::MethodCall { method, .. } => assert_eq!(method, "walk"),
        other => panic!("expected MethodCall, got {other:?}"),
    }
}

#[test]
fn extract_statements_finds_conditional_statements() {
    let xml = r#"<node type="ConditionalStatement" />"#;
    let stmts = extract_statements(xml);
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Statement::IfElse { .. }));
}

#[test]
fn extract_statements_finds_count_loops() {
    let xml = r#"<node type="CountLoop" count="3" />"#;
    let stmts = extract_statements(xml);
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Statement::CountLoop { .. }));
}

#[test]
fn extract_statements_finds_event_listeners() {
    let xml = r#"<node type="AddEventListener" event="SceneActivated" />"#;
    let stmts = extract_statements(xml);
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Statement::EventListener { event, .. } => assert_eq!(event, "SceneActivated"),
        other => panic!("expected EventListener, got {other:?}"),
    }
}

#[test]
fn extract_statements_finds_collision_listeners() {
    let xml = r#"<node type="CollisionStartListener" />"#;
    let stmts = extract_statements(xml);
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Statement::CollisionListener { .. }));
}

#[test]
fn extract_procedures_finds_user_methods() {
    let xml = r#"
        <element type="UserMethod" name="myFirstMethod" />
        <element name="helperMethod" type="UserMethod" />
        <node type="MethodInvocation" method="walk" />
    "#;
    let procs = extract_procedures(xml);
    assert_eq!(procs.len(), 2, "should find both UserMethod definitions");
    assert_eq!(procs[0].name, "myFirstMethod");
    assert_eq!(procs[1].name, "helperMethod");
    // Statements assigned to first procedure
    assert!(!procs[0].body.is_empty());
}

#[test]
fn extract_procedures_creates_synthetic_when_no_user_methods() {
    let xml = r#"<node type="MethodInvocation" method="say" />"#;
    let procs = extract_procedures(xml);
    assert_eq!(procs.len(), 1);
    assert_eq!(procs[0].name, "myFirstMethod");
    assert_eq!(procs[0].body.len(), 1);
}

#[test]
fn extract_statements_returns_empty_for_no_constructs() {
    let xml = r#"<root><child attr="value" /></root>"#;
    let stmts = extract_statements(xml);
    assert!(stmts.is_empty());
}

#[test]
fn starter_project_path_uses_alice_home() {
    let path = starter_project_path("amazonMinimum");
    assert!(
        path.to_string_lossy().contains("amazonMinimum.a3p"),
        "path should include project name: {}",
        path.display()
    );
    assert!(
        path.to_string_lossy().contains("starter-projects"),
        "path should include starter-projects directory: {}",
        path.display()
    );
}

#[test]
fn extract_procedures_handles_realistic_nested_alice_xml() {
    // Simulates real Alice 3 XML with nested methods, invocations, conditionals,
    // loops, and event listeners — all interleaved the way Alice serialises them.
    let xml = r#"
        <root>
          <element type="UserMethod" isDefault="true" name="myFirstMethod">
            <children>
              <child type="MethodInvocation" method="say" isParameter="false" />
              <child type="ConditionalStatement">
                <condition type="RelationalTest" />
                <ifBody>
                  <child type="MethodInvocation" method="turn" />
                </ifBody>
              </child>
              <child type="CountLoop" count="5">
                <body>
                  <child type="MethodInvocation" method="move" />
                </body>
              </child>
            </children>
          </element>
          <element type="UserMethod" name="helperProcedure" isDefault="false" />
          <element type="AddEventListener" event="SceneActivated" />
          <element type="CollisionStartEventListener" />
        </root>
    "#;

    let procs = extract_procedures(xml);
    assert_eq!(
        procs.len(),
        2,
        "should find myFirstMethod and helperProcedure"
    );
    assert_eq!(procs[0].name, "myFirstMethod");
    assert_eq!(procs[1].name, "helperProcedure");

    // All statements go to the first procedure (flat model)
    let body = &procs[0].body;

    let method_calls: Vec<_> = body
        .iter()
        .filter(|s| matches!(s, Statement::MethodCall { .. }))
        .collect();
    assert!(
        method_calls.len() >= 3,
        "should find at least 3 MethodInvocations (say, turn, move): got {}",
        method_calls.len()
    );

    assert!(
        body.iter().any(|s| matches!(s, Statement::IfElse { .. })),
        "should find ConditionalStatement"
    );
    assert!(
        body.iter()
            .any(|s| matches!(s, Statement::CountLoop { .. })),
        "should find CountLoop"
    );
    assert!(
        body.iter().any(
            |s| matches!(s, Statement::EventListener { event, .. } if event == "SceneActivated")
        ),
        "should find AddEventListener"
    );
    assert!(
        body.iter()
            .any(|s| matches!(s, Statement::CollisionListener { .. })),
        "should find CollisionStartEventListener"
    );
}

#[test]
fn parse_a3p_parses_in_memory_zip_with_xml_and_binary_entries() {
    use std::io::{Cursor, Write};
    use zip::write::FileOptions;

    // Build a minimal .a3p ZIP in memory with both XML and binary content
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);

    let options = FileOptions::default();

    // XML entry with Alice-like content
    zip_writer
        .start_file("programType.xml", options)
        .expect("start xml file");
    zip_writer
        .write_all(
            br#"<root>
                <element type="UserMethod" name="testMethod" />
                <child type="MethodInvocation" method="walk" />
            </root>"#,
        )
        .expect("write xml");

    // Binary entry (image placeholder) — parser should skip this
    zip_writer
        .start_file("textures/grass.png", options)
        .expect("start binary file");
    zip_writer
        .write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A])
        .expect("write binary");

    let finished = zip_writer.finish().expect("finish zip");
    let bytes = finished.into_inner();

    // Write to a unique temp file (avoid predictable names in shared /tmp)
    let tmp = std::env::temp_dir().join(format!("test_parse_a3p_{}.a3p", std::process::id()));
    std::fs::write(&tmp, &bytes).expect("write temp zip");

    let program = parse_a3p_program(&tmp);
    let _ = std::fs::remove_file(&tmp);

    let program = program.expect("should parse ZIP despite binary entries");
    assert!(!program.procedures.is_empty());
    assert_eq!(program.procedures[0].name, "testMethod");
    assert!(
        !program.procedures[0].body.is_empty(),
        "should extract MethodInvocation from XML"
    );
}

// ===========================================================================
// TDD CONTRACTS — Lessons 5-8 grading pipelines
// ===========================================================================
//
// These tests define the expected API and behavior for grading pipelines
// that have NOT been implemented yet. They are gated behind the Cargo feature
// `grading-l5-l8`, which does not currently exist. To activate:
//
//   1. Implement the grading functions + AST extensions in eatme-assets/eatme-core
//   2. Add `grading-l5-l8 = []` to [features] in crates/eatme-alice/Cargo.toml
//   3. Export the new types from eatme-assets lib.rs
//   4. Run: cargo test -p eatme-alice --test real_ast_grading --features grading-l5-l8
//
// Expected new types to implement:
//
//   eatme-core/src/ast.rs:
//     - Statement::VariableDeclaration { name: String, value: String }
//     - Statement::VariableAssignment { name: String, value: String }
//     - Statement::FunctionCall { name: String, arguments: Vec<String> }
//     - Statement::ReturnStatement { value: String }
//     - Procedure { name, parameters: Vec<Parameter>, body }  (add parameters field)
//     - Function { name: String, parameters: Vec<Parameter>, return_type: Option<String>, body: Vec<Statement> }
//     - Parameter { name: String, parameter_type: String }
//     - Program { procedures, functions: Vec<Function> }  (add functions field)
//
//   eatme-assets grading modules:
//     - FunctionsGradingInput + grade_functions() → GradingReport
//     - VariablesGradingInput + grade_variables() → GradingReport
//     - ParametersGradingInput + grade_parameters() → GradingReport
//     - CreativeGradingInput + grade_creative_project() → GradingReport

#[cfg(feature = "grading-l5-l8")]
mod future_pipeline_contracts {
    use super::*;

    // These imports will fail until the types are implemented — that's the TDD contract.
    use eatme_assets::{
        CreativeGradingInput, FunctionsGradingInput, ParametersGradingInput, VariablesGradingInput,
        grade_creative_project, grade_functions, grade_parameters, grade_variables,
    };

    fn functions_input(program: Program) -> FunctionsGradingInput {
        FunctionsGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn variables_input(program: Program) -> VariablesGradingInput {
        VariablesGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn parameters_input(program: Program) -> ParametersGradingInput {
        ParametersGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    fn creative_input(program: Program) -> CreativeGradingInput {
        CreativeGradingInput {
            assets_valid: true,
            asset_reason: "Real .a3p starter project parsed successfully".into(),
            deps_available: true,
            deps_reason: "Alice installation verified".into(),
            student_program: Some(program),
        }
    }

    // -----------------------------------------------------------------------
    // Lesson 5: Functions — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   create-function, call-function, run-world, save-project
    // Starter has NO user-defined Function entries → create-function = Blocked

    #[test]
    fn real_alice_functions_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice functions grading test (set EATME_REAL_ALICE=1 to enable)"
            );
            return;
        }

        let a3p_path = starter_project_path("amazonMinimum");
        assert!(
            a3p_path.exists(),
            "starter project not found at {}",
            a3p_path.display()
        );

        let program = parse_a3p_program(&a3p_path)
            .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

        let report = grade_functions(functions_input(program));

        assert_eq!(report.schema_version, "eatme.assets/grading/v1");
        assert_eq!(report.steps.len(), 7);

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has no user-defined functions → create-function = Blocked
        assert_eq!(report.steps[3].name, "create-function");
        assert_eq!(report.steps[3].status, StepStatus::Blocked);

        // Cascade blocks downstream
        assert_eq!(report.steps[4].name, "call-function");
        assert_eq!(report.steps[4].status, StepStatus::Blocked);
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::Blocked);
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Blocked);

        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 6: Variables — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   declare-variable, modify-variable, run-world, save-project
    // Starter has LocalDeclarationStatement → declare-variable = Ready
    // Starter has NO VariableAssignment → modify-variable = Blocked

    #[test]
    fn real_alice_variables_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice variables grading test (set EATME_REAL_ALICE=1 to enable)"
            );
            return;
        }

        let a3p_path = starter_project_path("amazonMinimum");
        assert!(
            a3p_path.exists(),
            "starter project not found at {}",
            a3p_path.display()
        );

        let program = parse_a3p_program(&a3p_path)
            .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

        let report = grade_variables(variables_input(program));

        assert_eq!(report.schema_version, "eatme.assets/grading/v1");
        assert_eq!(report.steps.len(), 7);

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has LocalDeclarationStatement → declare-variable = Ready
        assert_eq!(report.steps[3].name, "declare-variable");
        assert_eq!(
            report.steps[3].status,
            StepStatus::Ready,
            "starter has VariableDeclaration from LocalDeclarationStatement"
        );

        // No VariableAssignment in starter → modify-variable = Blocked
        assert_eq!(report.steps[4].name, "modify-variable");
        assert_eq!(
            report.steps[4].status,
            StepStatus::Blocked,
            "starter has no VariableAssignment"
        );

        // Cascade blocks downstream from modify-variable
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::Blocked);
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Blocked);

        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 7: Parameters — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   add-parameter, pass-argument, run-world, save-project
    // Starter has parameterized procedures → add-parameter = Ready
    // Starter has method calls with args → pass-argument = Ready

    #[test]
    fn real_alice_parameters_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice parameters grading test (set EATME_REAL_ALICE=1 to enable)"
            );
            return;
        }

        let a3p_path = starter_project_path("amazonMinimum");
        assert!(
            a3p_path.exists(),
            "starter project not found at {}",
            a3p_path.display()
        );

        let program = parse_a3p_program(&a3p_path)
            .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

        let report = grade_parameters(parameters_input(program));

        assert_eq!(report.schema_version, "eatme.assets/grading/v1");
        assert_eq!(report.steps.len(), 7);

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has UserParameter definitions → add-parameter = Ready
        assert_eq!(report.steps[3].name, "add-parameter");
        assert_eq!(
            report.steps[3].status,
            StepStatus::Ready,
            "starter has UserParameter definitions"
        );

        // Starter has MethodInvocations with arguments → pass-argument = Ready
        assert_eq!(report.steps[4].name, "pass-argument");
        assert_eq!(
            report.steps[4].status,
            StepStatus::Ready,
            "starter has MethodInvocations with arguments"
        );

        // Runtime step — requires execution
        assert_eq!(report.steps[5].name, "run-world");
        assert_eq!(report.steps[5].status, StepStatus::NotYetTested);

        // Save/reopen round-trip
        assert_eq!(report.steps[6].name, "save-project");
        assert_eq!(report.steps[6].status, StepStatus::Ready);

        // Not passed because run-world is not-yet-tested
        assert!(!report.passed);
    }

    // -----------------------------------------------------------------------
    // Lesson 8: Creative Project — real .a3p grading
    // -----------------------------------------------------------------------
    // Expected steps: validate-assets, check-dependencies, launch-smoke,
    //   build-scene, create-custom-procedure, add-control-structure,
    //   add-event-or-interaction
    // Starter has objects → build-scene = Ready
    // Starter has UserMethod → create-custom-procedure = Ready
    // Starter has IfElse → add-control-structure = Ready
    // Starter has NO events → add-event-or-interaction = Blocked

    #[test]
    fn real_alice_creative_grading_with_starter_project() {
        if !real_alice_enabled() {
            eprintln!(
                "skipping real-Alice creative grading test (set EATME_REAL_ALICE=1 to enable)"
            );
            return;
        }

        let a3p_path = starter_project_path("amazonMinimum");
        assert!(
            a3p_path.exists(),
            "starter project not found at {}",
            a3p_path.display()
        );

        let program = parse_a3p_program(&a3p_path)
            .unwrap_or_else(|| panic!("failed to parse {}", a3p_path.display()));

        let report = grade_creative_project(creative_input(program));

        assert_eq!(report.schema_version, "eatme.assets/grading/v1");
        assert_eq!(report.steps.len(), 7);

        // Preconditions Ready
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, StepStatus::Ready);

        // Starter has scene objects → build-scene = Ready
        assert_eq!(report.steps[3].name, "build-scene");
        assert_eq!(report.steps[3].status, StepStatus::Ready);

        // Starter has UserMethod definitions → create-custom-procedure = Ready
        assert_eq!(report.steps[4].name, "create-custom-procedure");
        assert_eq!(report.steps[4].status, StepStatus::Ready);

        // Starter has IfElse → add-control-structure = Ready
        assert_eq!(report.steps[5].name, "add-control-structure");
        assert_eq!(report.steps[5].status, StepStatus::Ready);

        // No events in starter → add-event-or-interaction = Blocked
        assert_eq!(report.steps[6].name, "add-event-or-interaction");
        assert_eq!(
            report.steps[6].status,
            StepStatus::Blocked,
            "starter has no EventListener or CollisionListener"
        );

        assert!(!report.passed);
    }
}
