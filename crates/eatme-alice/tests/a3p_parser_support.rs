//! Shared `.a3p` ZIP parser for integration tests.
//!
//! Extracts AST-relevant constructs from real Alice `.a3p` project files
//! (ZIP archives containing XML scene data) using regex-based XML parsing.
//!
//! Included as a module by test files that need to parse real starter projects:
//! - `real_ast_grading.rs`
//! - `loops_and_conditionals_e2e.rs`

use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

use eatme_core::ast::{Procedure, Program, Statement};
use regex::Regex;

// ---------------------------------------------------------------------------
// Compiled regex cache — each pattern compiled once across all test runs
// ---------------------------------------------------------------------------

pub fn re_user_method_type_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"UserMethod"[^>]*name\s*=\s*"([^"]+)""#).unwrap())
}

pub fn re_user_method_name_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"name\s*=\s*"([^"]+)"[^>]*type\s*=\s*"UserMethod""#).unwrap())
}

pub fn re_method_invocation() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"type\s*=\s*"MethodInvocation"[^>]*method\s*=\s*"([^"]*)"#).unwrap()
    })
}

pub fn re_conditional() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"ConditionalStatement""#).unwrap())
}

pub fn re_count_loop() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"CountLoop""#).unwrap())
}

pub fn re_event_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"type\s*=\s*"AddEventListener"[^>]*event\s*=\s*"([^"]*)"#).unwrap()
    })
}

pub fn re_collision_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"CollisionStart(?:Event)?Listener""#).unwrap())
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
pub fn parse_a3p_program(path: &Path) -> Option<Program> {
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
pub fn extract_procedures(xml: &str) -> Vec<Procedure> {
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
pub fn extract_statements(xml: &str) -> Vec<Statement> {
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
// Parser unit tests
// ---------------------------------------------------------------------------

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
    use zip::write::SimpleFileOptions;

    // Build a minimal .a3p ZIP in memory with both XML and binary content
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

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
