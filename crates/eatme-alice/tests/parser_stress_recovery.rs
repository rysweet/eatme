use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
mod a3p_parser_support;
mod structured_a3p_support;

use a3p_parser_support::{parse_a3p_program, write_synthetic_a3p};
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};
use zip::write::SimpleFileOptions;

fn large_regex_program_xml(statement_count: usize) -> String {
    let mut xml = String::from("<root><element type=\"UserMethod\" name=\"myFirstMethod\" />");
    for index in 0..statement_count {
        xml.push_str(&format!(
            "<node type=\"MethodInvocation\" method=\"step{index}\" />"
        ));
    }
    xml.push_str("</root>");
    xml
}

fn large_structured_program_xml(statement_count: usize) -> String {
    let mut xml = String::from("<program><procedure name=\"myFirstMethod\"><body>");
    for index in 0..statement_count {
        xml.push_str(&format!(
            "<statement type=\"MethodInvocation\" object=\"this\" method=\"step{index}\"><argument value=\"{index}\" /></statement>"
        ));
    }
    xml.push_str("</body></procedure></program>");
    xml
}

fn write_multi_xml_a3p(name: &str, entries: &[(&str, &str)]) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-work/parser-stress");
    fs::create_dir_all(&root).expect("create parser stress dir");

    let path = root.join(format!("{name}.a3p"));
    let file =
        fs::File::create(&path).unwrap_or_else(|err| panic!("create {}: {err}", path.display()));
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    for (entry_name, xml) in entries {
        writer
            .start_file(*entry_name, options)
            .unwrap_or_else(|err| panic!("start {entry_name}: {err}"));
        writer
            .write_all(xml.as_bytes())
            .unwrap_or_else(|err| panic!("write {entry_name}: {err}"));
    }

    writer.finish().expect("finish parser stress archive");
    path
}

#[test]
fn regex_a3p_parser_handles_more_than_one_thousand_statements() {
    let statement_count = 1_500;
    let path = write_synthetic_a3p(
        "regex-parser-stress",
        &large_regex_program_xml(statement_count),
    );

    let program = parse_a3p_program(&path).expect("large regex-driven A3P should parse");

    assert_eq!(program.procedures.len(), 1);
    assert_eq!(program.procedures[0].body.len(), statement_count);
}

#[test]
fn structured_a3p_parser_handles_more_than_one_thousand_statements_under_one_second() {
    let statement_count = 1_500;
    let path = write_structured_a3p(
        "structured-parser-stress",
        &large_structured_program_xml(statement_count),
    );

    let start = Instant::now();
    let program = parse_structured_a3p_program(&path).expect("large structured A3P should parse");
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "expected structured parser to stay under 1s for {statement_count} statements, got {elapsed:?}"
    );
    assert_eq!(program.procedures.len(), 1);
    assert_eq!(program.procedures[0].body.len(), statement_count);
}

#[test]
fn structured_a3p_parser_recovers_from_malformed_xml_entries() {
    let path = write_multi_xml_a3p(
        "structured-parser-recovery",
        &[
            (
                "broken.xml",
                "<program><procedure name=\"oops\"><body><statement type=\"MethodInvocation\"",
            ),
            (
                "program.xml",
                "<program><procedure name=\"myFirstMethod\"><body><statement type=\"MethodInvocation\" object=\"this\" method=\"say\" /></body></procedure></program>",
            ),
        ],
    );

    let program = parse_structured_a3p_program(&path)
        .expect("parser should recover with a later valid XML entry");

    assert_eq!(program.procedures.len(), 1);
    assert_eq!(program.procedures[0].name, "myFirstMethod");
    assert_eq!(program.procedures[0].body.len(), 1);
}
