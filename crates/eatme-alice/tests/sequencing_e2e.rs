// Sequencing E2E tests: validates doInOrder/doTogether parsing and grading.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use eatme_assets::{SequencingGradingInput, StepStatus, grade_sequencing};
use eatme_core::ast::{SequenceBlock, SequenceKind};
use zip::write::SimpleFileOptions;

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_sequences;

fn do_in_order_only_xml() -> &'static str {
    r#"
    <root>
        <element type="UserMethod" name="myFirstMethod" />
        <block type="DoInOrder">
            <step method="move" />
            <step method="turn" />
        </block>
    </root>
    "#
}

fn mixed_sequencing_xml() -> &'static str {
    r#"
    <root>
        <element type="UserMethod" name="myFirstMethod" />
        <block type="DoInOrder">
            <step method="move" />
            <step method="turn" />
        </block>
        <block type="DoTogether">
            <step method="say" />
            <step method="think" />
        </block>
    </root>
    "#
}

fn write_test_a3p(name: &str, xml: &str) -> PathBuf {
    let work_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-work/sequencing-e2e");
    std::fs::create_dir_all(&work_dir).expect("create sequencing test work dir");

    let path = work_dir.join(format!(
        "{}-{}-{}.a3p",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let cursor = Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(cursor);
    let options = SimpleFileOptions::default();

    zip_writer
        .start_file("programType.xml", options)
        .expect("start xml entry");
    zip_writer
        .write_all(xml.as_bytes())
        .expect("write sequencing xml");

    let bytes = zip_writer
        .finish()
        .expect("finish sequencing zip")
        .into_inner();
    std::fs::write(&path, bytes).expect("write sequencing a3p");
    path
}

fn all_ready_input(sequence_blocks: Option<Vec<SequenceBlock>>) -> SequencingGradingInput {
    SequencingGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        sequence_blocks,
    }
}

#[test]
fn sequencing_parser_extracts_do_in_order_blocks() {
    let path = write_test_a3p("do-in-order-only", do_in_order_only_xml());
    let sequence_blocks = parse_a3p_sequences(&path).expect("sequence blocks should parse");
    let _ = std::fs::remove_file(&path);

    assert_eq!(sequence_blocks.len(), 1);
    assert!(matches!(sequence_blocks[0].kind, SequenceKind::DoInOrder));
    assert_eq!(sequence_blocks[0].steps, ["move", "turn"]);
}

#[test]
fn sequencing_parser_distinguishes_sequential_and_parallel_execution() {
    let path = write_test_a3p("mixed-sequencing", mixed_sequencing_xml());
    let sequence_blocks = parse_a3p_sequences(&path).expect("sequence blocks should parse");
    let _ = std::fs::remove_file(&path);

    assert_eq!(sequence_blocks.len(), 2);
    assert!(matches!(sequence_blocks[0].kind, SequenceKind::DoInOrder));
    assert!(matches!(sequence_blocks[1].kind, SequenceKind::DoTogether));
    assert_eq!(sequence_blocks[0].steps, ["move", "turn"]);
    assert_eq!(sequence_blocks[1].steps, ["say", "think"]);
}

#[test]
fn sequencing_grading_full_marks_require_both_constructs() {
    let path = write_test_a3p("mixed-sequencing", mixed_sequencing_xml());
    let sequence_blocks = parse_a3p_sequences(&path).expect("sequence blocks should parse");
    let _ = std::fs::remove_file(&path);

    let report = grade_sequencing(all_ready_input(Some(sequence_blocks)));
    assert!(report.passed);

    for name in [
        "use-do-in-order",
        "use-do-together",
        "combine-sequential-and-parallel-actions",
        "save-project",
    ] {
        let step = report.steps.iter().find(|step| step.name == name).unwrap();
        assert_eq!(step.status, StepStatus::Ready, "{name} should be ready");
    }
}

#[test]
fn sequencing_grading_partial_when_only_one_construct_present() {
    let path = write_test_a3p("do-in-order-only", do_in_order_only_xml());
    let sequence_blocks = parse_a3p_sequences(&path).expect("sequence blocks should parse");
    let _ = std::fs::remove_file(&path);

    let report = grade_sequencing(all_ready_input(Some(sequence_blocks)));
    assert!(!report.passed);

    let do_in_order = report
        .steps
        .iter()
        .find(|step| step.name == "use-do-in-order")
        .unwrap();
    assert_eq!(do_in_order.status, StepStatus::Ready);

    let do_together = report
        .steps
        .iter()
        .find(|step| step.name == "use-do-together")
        .unwrap();
    assert_eq!(do_together.status, StepStatus::Blocked);

    let combine = report
        .steps
        .iter()
        .find(|step| step.name == "combine-sequential-and-parallel-actions")
        .unwrap();
    assert_eq!(combine.status, StepStatus::Blocked);
}

#[test]
fn sequencing_schema_and_round_trip() {
    let path = write_test_a3p("mixed-sequencing", mixed_sequencing_xml());
    let sequence_blocks = parse_a3p_sequences(&path).expect("sequence blocks should parse");
    let _ = std::fs::remove_file(&path);

    let report = grade_sequencing(all_ready_input(Some(sequence_blocks.clone())));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(
        report.lesson,
        "procedure-sequencing-do-in-order-do-together"
    );

    let json = serde_json::to_string_pretty(&sequence_blocks).unwrap();
    let restored: Vec<SequenceBlock> = serde_json::from_str(&json).unwrap();
    assert_eq!(sequence_blocks, restored);
}
