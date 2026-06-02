use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eatme_assets::{ParametersGradingInput, grade_parameters};
use eatme_core::ast::{Parameter, Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

fn new_fixture_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-work/real-a3p-pipeline")
        .join(format!("{name}-{nonce}"));
    fs::create_dir_all(&dir).expect("fixture directory should be created");
    dir
}

fn write_a3p_with_entry(name: &str, entry_name: &str, xml: &str) -> PathBuf {
    let dir = new_fixture_dir(name);
    let path = dir.join(format!("{name}.a3p"));
    let file = File::create(&path).expect("fixture archive should be created");
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    writer
        .start_file(entry_name, options)
        .expect("zip entry should be created");
    writer
        .write_all(xml.as_bytes())
        .expect("xml should be written to archive");
    writer.finish().expect("zip archive should finish cleanly");
    path
}

fn read_zip_entry(path: &Path, entry_name: &str) -> String {
    let file = File::open(path).expect("fixture archive should open");
    let mut archive = zip::ZipArchive::new(file).expect("fixture archive should be readable");
    let mut entry = archive
        .by_name(entry_name)
        .expect("expected program.xml entry to exist");
    let mut xml = String::new();
    entry
        .read_to_string(&mut xml)
        .expect("program.xml should be readable as utf-8");
    xml
}

fn render_statement_xml(statement: &Statement) -> String {
    match statement {
        Statement::MethodCall { method, .. } => {
            format!(r#"<node type="MethodInvocation" method="{method}" />"#)
        }
        Statement::CountLoop { .. } => r#"<node type="CountLoop" />"#.to_string(),
        Statement::IfElse { .. } => r#"<node type="ConditionalStatement" />"#.to_string(),
        Statement::EventListener { event, .. } => {
            format!(r#"<node type="AddEventListener" event="{event}" />"#)
        }
        Statement::CollisionListener { .. } => {
            r#"<node type="CollisionStartListener" />"#.to_string()
        }
        other => panic!("unsupported statement for XML round-trip fixture: {other:?}"),
    }
}

fn render_program_xml(program: &Program) -> String {
    let mut xml = String::from("<alice version=\"3.6\">");
    for procedure in &program.procedures {
        xml.push_str(&format!(
            r#"<node type="UserMethod" name="{}" />"#,
            procedure.name
        ));
        for statement in &procedure.body {
            xml.push_str(&render_statement_xml(statement));
        }
    }
    xml.push_str("</alice>");
    xml
}

fn grading_input(student_program: Program) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "fixture archive parsed successfully".into(),
        deps_available: true,
        deps_reason: "integration test does not require Alice runtime".into(),
        student_program: Some(student_program),
    }
}

fn parameter_score(report: &eatme_assets::GradingReport) -> u8 {
    report
        .quality_scores
        .iter()
        .find(|score| score.dimension == "parameter_types")
        .expect("parameter score should be present")
        .score
}

#[test]
fn parses_program_xml_from_real_a3p_archive_and_finds_expected_ast_nodes() {
    let xml = r#"
        <alice version="3.6">
          <node type="UserMethod" name="myFirstMethod" />
          <node type="MethodInvocation" method="move" />
          <node type="ConditionalStatement" />
          <node type="CountLoop" />
          <node type="AddEventListener" event="SceneActivated" />
        </alice>
    "#;
    let path = write_a3p_with_entry("real-a3p-parse", "program.xml", xml);

    let extracted_xml = read_zip_entry(&path, "program.xml");
    assert!(extracted_xml.contains("MethodInvocation"));
    assert!(extracted_xml.contains("ConditionalStatement"));
    assert!(extracted_xml.contains("CountLoop"));

    let program = parse_a3p_program(&path).expect("archive should parse into an AST program");
    let body = &program.procedures[0].body;

    assert_eq!(program.procedures[0].name, "myFirstMethod");
    assert!(
        body.iter()
            .any(|statement| matches!(statement, Statement::MethodCall { method, .. } if method == "move")),
        "expected parsed program to contain a method invocation"
    );
    assert!(
        body.iter()
            .any(|statement| matches!(statement, Statement::IfElse { .. })),
        "expected parsed program to contain a conditional"
    );
    assert!(
        body.iter()
            .any(|statement| matches!(statement, Statement::CountLoop { .. })),
        "expected parsed program to contain a count loop"
    );
    assert!(
        body.iter().any(|statement| matches!(statement, Statement::EventListener { event, .. } if event == "SceneActivated")),
        "expected parsed program to contain an event listener"
    );
}

#[test]
fn program_ast_roundtrips_through_program_xml_archive() {
    let original = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::MethodCall {
                object: "this".into(),
                method: "move".into(),
                arguments: vec![],
            },
            Statement::IfElse {
                condition: String::new(),
                if_body: vec![],
                else_body: vec![],
            },
            Statement::CountLoop {
                count: 1,
                body: vec![],
            },
        ],
    }]);
    let path = write_a3p_with_entry(
        "real-a3p-roundtrip",
        "program.xml",
        &render_program_xml(&original),
    );

    let reparsed = parse_a3p_program(&path).expect("round-trip archive should parse");

    assert_eq!(reparsed, original);
}

#[test]
fn grading_pipeline_reports_expected_quality_scores_for_good_and_bad_programs() {
    let known_good = Program {
        procedures: vec![
            Procedure {
                name: "moveAnimal".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
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
                    method: "moveAnimal".into(),
                    arguments: vec!["2.0".into()],
                }],
            },
        ],
        functions: vec![],
    };
    let known_bad = Program {
        procedures: vec![
            Procedure {
                name: "moveAnimal".into(),
                parameters: vec![Parameter {
                    name: "distance".into(),
                    param_type: "Object".into(),
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
                    method: "moveAnimal".into(),
                    arguments: vec!["2.0".into()],
                }],
            },
        ],
        functions: vec![],
    };

    let good_report = grade_parameters(grading_input(known_good));
    let bad_report = grade_parameters(grading_input(known_bad));

    assert!(good_report.passed, "known-good grading fixture should pass");
    assert!(
        bad_report.passed,
        "known-bad quality fixture should still satisfy required steps"
    );
    assert_eq!(parameter_score(&good_report), 100);
    assert_eq!(parameter_score(&bad_report), 0);
}
