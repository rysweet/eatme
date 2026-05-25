use eatme_assets::grading_report::{
    GradingReport, LoopsGradingInput, StepStatus, grade_loops_and_conditionals,
};
use eatme_core::ast::Program;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
mod a3p_parser_support;
#[allow(dead_code)]
mod structured_a3p_support;

use a3p_parser_support::parse_a3p_program;
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

fn ready_input(student_program: Option<Program>) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program,
    }
}

fn write_corrupt_a3p(name: &str, bytes: &[u8]) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-work/malformed-input");
    fs::create_dir_all(&root).expect("create malformed input dir");
    let path = root.join(format!("{name}.a3p"));
    fs::write(&path, bytes).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    path
}

fn find_step<'a>(
    report: &'a GradingReport,
    name: &str,
) -> &'a eatme_assets::grading_report::StepGrade {
    report
        .steps
        .iter()
        .find(|step| step.name == name)
        .unwrap_or_else(|| panic!("missing step {name}: {:?}", report.steps))
}

fn assert_no_program_chain(report: &GradingReport) {
    assert!(!report.passed);

    for step_name in [
        "build-counting-loop",
        "add-conditional-branch",
        "run-world",
        "save-project",
    ] {
        let step = find_step(report, step_name);
        assert_eq!(step.status, StepStatus::Blocked, "{step_name}");
        assert!(
            step.reason.contains("No student program provided"),
            "{step_name} should explain missing parsed program: {}",
            step.reason
        );
    }
}

#[test]
fn corrupt_a3p_blocks_grading_instead_of_crashing() {
    let path = write_corrupt_a3p("corrupt-archive", b"not a zip archive");

    let program = parse_a3p_program(&path);
    assert!(program.is_none(), "corrupt archive should fail closed");

    let report = grade_loops_and_conditionals(ready_input(program));
    assert_no_program_chain(&report);
}

#[test]
fn empty_xml_blocks_grading_instead_of_crashing() {
    let path = write_structured_a3p("empty-xml", "");

    let program = parse_structured_a3p_program(&path);
    assert!(program.is_none(), "empty XML should not produce a program");

    let report = grade_loops_and_conditionals(ready_input(program));
    assert_no_program_chain(&report);
}

#[test]
fn missing_procedures_yield_a_blocked_report_without_panicking() {
    let path = write_structured_a3p(
        "functions-only",
        "<program><function name=\"helper\" return_type=\"Number\"><body><statement type=\"ReturnStatement\" expression=\"1\" /></body></function></program>",
    );

    let program =
        parse_structured_a3p_program(&path).expect("function-only XML should still parse");
    assert!(
        program.procedures.is_empty(),
        "fixture should omit procedures"
    );

    let report = grade_loops_and_conditionals(ready_input(Some(program)));
    let step = find_step(&report, "build-counting-loop");
    assert_eq!(step.status, StepStatus::Blocked);
    assert!(
        step.reason.contains("No CountLoop found"),
        "{}",
        step.reason
    );
}

#[test]
fn null_student_program_field_blocks_interaction_steps_cleanly() {
    let report = grade_loops_and_conditionals(ready_input(None));
    assert_no_program_chain(&report);
}

#[test]
fn nullish_structured_fields_fail_closed_without_panicking() {
    let path = write_structured_a3p(
        "nullish-fields",
        "<program><procedure name=\"\"><body><statement type=\"MethodInvocation\" object=\"\" method=\"\" /></body></procedure></program>",
    );

    let program =
        parse_structured_a3p_program(&path).expect("nullish XML should still parse to a Program");
    assert_eq!(program.procedures.len(), 1);
    assert_eq!(program.procedures[0].name, "");

    let report = grade_loops_and_conditionals(ready_input(Some(program)));
    let step = find_step(&report, "build-counting-loop");
    assert_eq!(step.status, StepStatus::Blocked);
    assert!(
        step.reason.contains("No CountLoop found"),
        "{}",
        step.reason
    );
}
