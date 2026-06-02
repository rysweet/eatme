// Comments E2E tests: validates parsed comment nodes and grading.

use eatme_assets::{CommentsGradingInput, StepStatus, grade_comments};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::{parse_a3p_program, write_synthetic_a3p};

fn comments_xml() -> &'static str {
    r#"
    <root>
      <element type="UserMethod" name="myFirstMethod" />
      <node type="Comment" text="Explain why the array order matters for the dance" />
      <node type="Comment" text="Keep score in sync with the loop counter" />
    </root>
    "#
}

fn parsed_comments_program() -> Program {
    let path = write_synthetic_a3p("comments", comments_xml());
    parse_a3p_program(&path).unwrap_or_else(|| panic!("failed to parse {}", path.display()))
}

fn all_ready_input(program: Option<Program>) -> CommentsGradingInput {
    CommentsGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn parsed_a3p_extracts_comment_text() {
    let program = parsed_comments_program();
    let comments: Vec<_> = program.procedures[0]
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::Comment { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(
        comments,
        vec![
            "Explain why the array order matters for the dance",
            "Keep score in sync with the loop counter",
        ]
    );
}

#[test]
fn comments_grading_passes_with_meaningful_comments() {
    let mut program = parsed_comments_program();
    program.procedures[0].body.push(Statement::MethodCall {
        object: "this.cat".into(),
        method: "say".into(),
        arguments: vec!["\"Ready to perform\"".into()],
    });

    let report = grade_comments(all_ready_input(Some(program)));
    assert!(report.passed);
    assert_eq!(report.lesson, "comments-mini-challenge");
    for step in &report.steps {
        assert_eq!(step.status, StepStatus::Ready, "step '{}'", step.name);
    }
}

#[test]
fn comments_grading_blocks_without_meaningful_comment() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::Comment {
            text: "todo".into(),
        }],
    }]);

    let report = grade_comments(all_ready_input(Some(program)));
    assert!(!report.passed);
    let meaningful = report
        .steps
        .iter()
        .find(|step| step.name == "write-meaningful-comment")
        .unwrap();
    assert_eq!(meaningful.status, StepStatus::Blocked);
}

#[test]
fn comments_ast_survives_json_round_trip() {
    let program = parsed_comments_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
