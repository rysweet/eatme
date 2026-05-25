//! Shared `.a3p` ZIP parser for integration tests.
//!
//! Extracts AST-relevant constructs from real Alice `.a3p` project files
//! (ZIP archives containing XML scene data) using regex-based XML parsing.
//!
//! Included as a module by test files that need to parse real starter projects:
//! - `real_ast_grading.rs`
//! - `loops_and_conditionals_e2e.rs`

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use eatme_core::ast::{
    ArithmeticOperator, CameraPose, Procedure, Program, SceneLayout, SceneObject, SequenceBlock,
    SequenceKind, Statement, Vec3,
};
use regex::Regex;

// ---------------------------------------------------------------------------
// Compiled regex cache — each pattern compiled once across all test runs
// ---------------------------------------------------------------------------

pub fn re_user_method_type_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)type\s*=\s*"(?:[^"]+\.)?UserMethod"[^>]*?(?:name\s*=\s*"([^"]+)"|.*?<property\s+name\s*=\s*"name">\s*<value[^>]*>([^<]+)</value>)"#,
        )
        .unwrap()
    })
}

pub fn re_user_method_name_first() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"name\s*=\s*"([^"]+)"[^>]*type\s*=\s*"(?:[^"]+\.)?UserMethod""#).unwrap()
    })
}

pub fn re_user_method_any() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"(?:[^"]+\.)?UserMethod""#).unwrap())
}

pub fn re_method_invocation() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)type\s*=\s*"(?:[^"]+\.)?MethodInvocation"[^>]*?(?:method\s*=\s*"([^"]*)"|.*?<method[^>]*name\s*=\s*"([^"]+)")"#,
        )
        .unwrap()
    })
}

pub fn re_conditional() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"(?:[^"]+\.)?ConditionalStatement""#).unwrap())
}

pub fn re_count_loop() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"(?:[^"]+\.)?CountLoop""#).unwrap())
}

pub fn re_event_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)type\s*=\s*"(?:[^"]+\.)?AddEventListener"[^>]*?(?:event\s*=\s*"([^"]*)"|.*?<property\s+name\s*=\s*"event"[^>]*>\s*<value[^>]*>([^<]+)</value>)"#,
        )
        .unwrap()
    })
}

pub fn re_collision_listener() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"type\s*=\s*"(?:[^"]+\.)?CollisionStart(?:Event)?Listener""#).unwrap()
    })
}

pub fn re_array_declaration() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"ArrayDeclaration"[^>]*name\s*=\s*"([^"]+)"[^>]*elementType\s*=\s*"([^"]+)"[^>]*elements\s*=\s*"([^"]*)""#,
        )
        .unwrap()
    })
}

pub fn re_array_access() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"ArrayAccess"[^>]*array\s*=\s*"([^"]+)"[^>]*index\s*=\s*"([^"]+)"[^>]*target\s*=\s*"([^"]+)""#,
        )
        .unwrap()
    })
}

pub fn re_for_each_array() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"ForEachArray"[^>]*item\s*=\s*"([^"]+)"[^>]*array\s*=\s*"([^"]+)""#,
        )
        .unwrap()
    })
}

pub fn re_arithmetic_expression() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"ArithmeticExpression"[^>]*operator\s*=\s*"([^"]+)"[^>]*left\s*=\s*"([^"]+)"[^>]*right\s*=\s*"([^"]+)"[^>]*result\s*=\s*"([^"]+)""#,
        )
        .unwrap()
    })
}

pub fn re_comment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"Comment"[^>]*text\s*=\s*"([^"]+)""#).unwrap())
}

pub fn re_user_type() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"UserType"[^>]*name\s*=\s*"([^"]+)"(?:[^>]*extends\s*=\s*"([^"]*)")?(?:[^>]*methods\s*=\s*"([^"]*)")?"#,
        )
        .unwrap()
    })
}

pub fn re_scene_object() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"type\s*=\s*"SceneObject"[^>]*name\s*=\s*"([^"]+)"[^>]*kind\s*=\s*"([^"]+)"(?:[^>]*position\s*=\s*"([^"]+)")?(?:[^>]*size\s*=\s*"([^"]+)")?(?:[^>]*color\s*=\s*"([^"]+)")?(?:[^>]*opacity\s*=\s*"([^"]+)")?"#,
        )
        .unwrap()
    })
}

pub fn re_camera() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"type\s*=\s*"Camera"[^>]*position\s*=\s*"([^"]+)""#).unwrap())
}

pub fn re_do_in_order() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<block\s+type\s*=\s*"DoInOrder"[^>]*>(.*?)</block>"#).unwrap()
    })
}

pub fn re_do_together() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?s)<block\s+type\s*=\s*"DoTogether"[^>]*>(.*?)</block>"#).unwrap()
    })
}

pub fn re_sequence_step() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"method\s*=\s*"([^"]+)""#).unwrap())
}

// ---------------------------------------------------------------------------
// .a3p ZIP parser — lightweight regex-based XML extraction
// ---------------------------------------------------------------------------

fn capture_text<'a>(captures: &'a regex::Captures<'a>, groups: &[usize]) -> Option<&'a str> {
    groups
        .iter()
        .find_map(|&index| captures.get(index).map(|value| value.as_str()))
}

fn read_all_xml(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let mut all_xml = String::with_capacity(128 * 1024);
    let mut content_buf = String::new();
    for i in 0..archive.len() {
        let Ok(mut entry) = archive.by_index(i) else {
            continue;
        };
        if entry.name().ends_with(".xml") {
            content_buf.clear();
            if entry.read_to_string(&mut content_buf).is_ok() {
                all_xml.push_str(&content_buf);
                all_xml.push('\n');
            }
        }
    }

    if all_xml.is_empty() {
        None
    } else {
        Some(all_xml)
    }
}

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
    let all_xml = read_all_xml(path)?;
    let procedures = extract_procedures(&all_xml);
    Some(Program {
        procedures,
        functions: vec![],
    })
}

pub fn write_synthetic_a3p(fixture_name: &str, xml: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-work/synthetic-a3p")
        .join(format!("{fixture_name}-{nonce}"));
    std::fs::create_dir_all(&root).unwrap();

    let path = root.join(format!("{fixture_name}.a3p"));
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    writer.start_file("programType.xml", options).unwrap();
    writer.write_all(xml.as_bytes()).unwrap();
    writer.finish().unwrap();

    path
}

pub fn parse_a3p_scene(path: &Path) -> Option<SceneLayout> {
    let all_xml = read_all_xml(path)?;
    extract_scene_layout(&all_xml)
}

pub fn parse_a3p_sequences(path: &Path) -> Option<Vec<SequenceBlock>> {
    let all_xml = read_all_xml(path)?;
    Some(extract_sequence_blocks(&all_xml))
}

/// Extract `Procedure` definitions from Alice XML content.
pub fn extract_procedures(xml: &str) -> Vec<Procedure> {
    let mut procedures = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    if xml.contains("UserMethod") {
        for re in [re_user_method_type_first(), re_user_method_name_first()] {
            for cap in re.captures_iter(xml) {
                let Some(name) = capture_text(&cap, &[1, 2]).map(str::to_string) else {
                    continue;
                };
                if seen_names.insert(name.clone()) {
                    procedures.push(Procedure {
                        name,
                        parameters: vec![],
                        body: Vec::new(),
                    });
                }
            }
        }

        if procedures.is_empty() && re_user_method_any().is_match(xml) {
            procedures.push(Procedure {
                name: "myFirstMethod".into(),
                parameters: vec![],
                body: Vec::new(),
            });
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

    if xml.contains("MethodInvocation") {
        for cap in re_method_invocation().captures_iter(xml) {
            stmts.push(Statement::MethodCall {
                object: "this".into(),
                method: capture_text(&cap, &[1, 2]).unwrap_or("unknown").to_string(),
                arguments: vec![],
            });
        }
    }

    if xml.contains("ConditionalStatement") {
        for _ in re_conditional().find_iter(xml) {
            stmts.push(Statement::IfElse {
                condition: String::new(),
                if_body: vec![],
                else_body: vec![],
            });
        }
    }

    if xml.contains("CountLoop") {
        for _ in re_count_loop().find_iter(xml) {
            stmts.push(Statement::CountLoop {
                count: 1,
                body: vec![],
            });
        }
    }

    if xml.contains("AddEventListener") {
        for cap in re_event_listener().captures_iter(xml) {
            stmts.push(Statement::EventListener {
                event: capture_text(&cap, &[1, 2]).unwrap_or("unknown").to_string(),
                body: vec![],
            });
        }
    }

    if xml.contains("CollisionStart") {
        for _ in re_collision_listener().find_iter(xml) {
            stmts.push(Statement::CollisionListener {
                object_a: "unknown".into(),
                object_b: "unknown".into(),
                body: vec![],
            });
        }
    }

    if xml.contains("ArrayDeclaration") {
        for cap in re_array_declaration().captures_iter(xml) {
            let elements = cap[3]
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            stmts.push(Statement::ArrayDeclaration {
                name: cap[1].to_string(),
                element_type: cap[2].to_string(),
                elements,
            });
        }
    }

    if xml.contains("ArrayAccess") {
        for cap in re_array_access().captures_iter(xml) {
            stmts.push(Statement::ArrayAccess {
                array: cap[1].to_string(),
                index: cap[2].to_string(),
                target: cap[3].to_string(),
            });
        }
    }

    if xml.contains("ForEachArray") {
        for cap in re_for_each_array().captures_iter(xml) {
            stmts.push(Statement::ForEachArray {
                item_name: cap[1].to_string(),
                array: cap[2].to_string(),
                body: vec![],
            });
        }
    }

    if xml.contains("ArithmeticExpression") {
        for cap in re_arithmetic_expression().captures_iter(xml) {
            let Some(operator) = parse_arithmetic_operator(&cap[1]) else {
                continue;
            };
            stmts.push(Statement::ArithmeticExpression {
                operator,
                left: cap[2].to_string(),
                right: cap[3].to_string(),
                result: cap[4].to_string(),
            });
        }
    }

    if xml.contains("Comment") {
        for cap in re_comment().captures_iter(xml) {
            stmts.push(Statement::Comment {
                text: cap[1].to_string(),
            });
        }
    }

    if xml.contains("UserType") {
        for cap in re_user_type().captures_iter(xml) {
            let extends = cap.get(2).and_then(|value| {
                let text = value.as_str().trim();
                (!text.is_empty()).then(|| text.to_string())
            });
            let methods = cap
                .get(3)
                .map(|value| parse_user_type_methods(value.as_str()))
                .unwrap_or_default();
            stmts.push(Statement::UserTypeDeclaration {
                name: cap[1].to_string(),
                extends,
                methods,
            });
        }
    }

    stmts
}

fn parse_arithmetic_operator(value: &str) -> Option<ArithmeticOperator> {
    match value.trim().to_ascii_lowercase().as_str() {
        "add" => Some(ArithmeticOperator::Add),
        "subtract" => Some(ArithmeticOperator::Subtract),
        "multiply" => Some(ArithmeticOperator::Multiply),
        "divide" => Some(ArithmeticOperator::Divide),
        _ => None,
    }
}

fn parse_user_type_methods(value: &str) -> Vec<Procedure> {
    value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| Procedure {
            name: name.to_string(),
            parameters: vec![],
            body: vec![],
        })
        .collect()
}

pub fn extract_scene_layout(xml: &str) -> Option<SceneLayout> {
    let mut ground_present = false;
    let mut sky_present = false;
    let mut objects = Vec::new();

    for caps in re_scene_object().captures_iter(xml) {
        let kind = caps[2].to_string();
        match kind.as_str() {
            "ground" => {
                ground_present = true;
            }
            "sky" => {
                sky_present = true;
            }
            _ => {
                objects.push(SceneObject {
                    name: caps[1].to_string(),
                    kind,
                    position: caps.get(3).and_then(|m| parse_vec3(m.as_str())),
                    size: caps.get(4).and_then(|m| m.as_str().parse::<f32>().ok()),
                    color: caps.get(5).map(|m| m.as_str().to_string()),
                    opacity: caps.get(6).and_then(|m| m.as_str().parse::<f32>().ok()),
                });
            }
        }
    }

    let camera = re_camera()
        .captures(xml)
        .and_then(|caps| parse_vec3(&caps[1]).map(|position| CameraPose { position }));

    if !ground_present && !sky_present && objects.is_empty() && camera.is_none() {
        None
    } else {
        Some(SceneLayout {
            ground_present,
            sky_present,
            objects,
            camera,
        })
    }
}

pub fn extract_sequence_blocks(xml: &str) -> Vec<SequenceBlock> {
    let mut blocks = Vec::new();

    for caps in re_do_in_order().captures_iter(xml) {
        blocks.push(SequenceBlock {
            kind: SequenceKind::DoInOrder,
            steps: extract_step_names(&caps[1]),
        });
    }

    for caps in re_do_together().captures_iter(xml) {
        blocks.push(SequenceBlock {
            kind: SequenceKind::DoTogether,
            steps: extract_step_names(&caps[1]),
        });
    }

    blocks
}

fn extract_step_names(xml: &str) -> Vec<String> {
    re_sequence_step()
        .captures_iter(xml)
        .map(|caps| caps[1].to_string())
        .collect()
}

fn parse_vec3(value: &str) -> Option<Vec3> {
    let mut parts = value.split(',').map(str::trim);
    let x = parts.next()?.parse().ok()?;
    let y = parts.next()?.parse().ok()?;
    let z = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Vec3 { x, y, z })
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
fn extract_procedures_handles_fully_qualified_alice_ast_nodes() {
    let xml = r#"
        <root>
          <node type="org.lgna.project.ast.UserMethod" uuid="1">
            <property name="name"><value type="java.lang.String">main</value></property>
          </node>
          <node type="org.lgna.project.ast.ExpressionStatement">
            <property name="expression">
              <node type="org.lgna.project.ast.MethodInvocation" uuid="2">
                <property name="method">
                  <node type="org.lgna.project.ast.JavaMethod" uuid="3">
                    <method isVarArgs="false" name="initializeInFrame" />
                  </node>
                </property>
              </node>
            </property>
          </node>
          <node type="org.lgna.project.ast.ConditionalStatement" uuid="4" />
        </root>
    "#;

    let procs = extract_procedures(xml);
    assert_eq!(procs.len(), 1);
    assert_eq!(procs[0].name, "main");
    assert!(procs[0].body.iter().any(
        |stmt| matches!(stmt, Statement::MethodCall { method, .. } if method == "initializeInFrame")
    ));
    assert!(
        procs[0]
            .body
            .iter()
            .any(|stmt| matches!(stmt, Statement::IfElse { .. }))
    );
}

#[test]
fn extract_scene_layout_finds_objects_and_camera() {
    let xml = r##"
        <root>
            <node type="SceneObject" name="ground" kind="ground" />
            <node type="SceneObject" name="sky" kind="sky" />
            <node type="SceneObject" name="bunny" kind="Biped" position="1,0,-2" size="1.25" color="#ffaa00" opacity="0.8" />
            <node type="SceneObject" name="tree" kind="Prop" position="-3,0,4" size="2.5" color="#00aa44" opacity="1.0" />
            <node type="Camera" position="0,6,12" />
        </root>
    "##;

    let scene = extract_scene_layout(xml).expect("scene layout should be extracted");
    assert!(scene.ground_present);
    assert!(scene.sky_present);
    assert_eq!(scene.objects.len(), 2);
    assert_eq!(scene.objects[0].name, "bunny");
    assert_eq!(scene.camera.as_ref().unwrap().position.z, 12.0);
}

#[test]
fn extract_sequence_blocks_finds_do_in_order_and_do_together() {
    let xml = r#"
        <root>
            <block type="DoInOrder">
                <step method="move" />
                <step method="turn" />
            </block>
            <block type="DoTogether">
                <step method="say" />
                <step method="think" />
            </block>
        </root>
    "#;

    let blocks = extract_sequence_blocks(xml);
    assert_eq!(blocks.len(), 2);
    assert!(matches!(blocks[0].kind, SequenceKind::DoInOrder));
    assert!(matches!(blocks[1].kind, SequenceKind::DoTogether));
    assert_eq!(blocks[0].steps, ["move", "turn"]);
    assert_eq!(blocks[1].steps, ["say", "think"]);
}

#[test]
fn parse_a3p_parses_in_memory_zip_with_xml_and_binary_entries() {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

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

    zip_writer
        .start_file("textures/grass.png", options)
        .expect("start binary file");
    zip_writer
        .write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A])
        .expect("write binary");

    let finished = zip_writer.finish().expect("finish zip");
    let bytes = finished.into_inner();

    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-work/a3p-parser-support");
    std::fs::create_dir_all(&root).expect("create parser test work dir");
    let path = root.join(format!("test-parse-a3p-{}.a3p", std::process::id()));
    std::fs::write(&path, &bytes).expect("write test zip");

    let program = parse_a3p_program(&path);
    let _ = std::fs::remove_file(&path);

    let program = program.expect("should parse ZIP despite binary entries");
    assert!(!program.procedures.is_empty());
    assert_eq!(program.procedures[0].name, "testMethod");
    assert!(
        !program.procedures[0].body.is_empty(),
        "should extract MethodInvocation from XML"
    );
}
