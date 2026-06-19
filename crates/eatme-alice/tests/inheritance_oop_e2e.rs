// Inheritance + OOP E2E tests: validates parsed user types and grading.

use eatme_assets::{InheritanceOopGradingInput, StepStatus, grade_inheritance_oop};
use eatme_core::ast::{Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::{parse_a3p_program, write_synthetic_a3p};

fn inheritance_xml() -> &'static str {
    r#"
    <root>
      <element type="UserMethod" name="myFirstMethod" />
      <node type="UserType" name="PetLeader" extends="Biped" methods="leadDance,resetPose" />
    </root>
    "#
}

fn parsed_inheritance_program() -> Program {
    let path = write_synthetic_a3p("inheritance-oop", inheritance_xml());
    parse_a3p_program(&path).unwrap_or_else(|| panic!("failed to parse {}", path.display()))
}

fn all_ready_input(program: Option<Program>) -> InheritanceOopGradingInput {
    InheritanceOopGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn parsed_a3p_extracts_user_type_hierarchy_and_methods() {
    let program = parsed_inheritance_program();
    let user_type = program.procedures[0]
        .body
        .iter()
        .find_map(|statement| match statement {
            Statement::UserTypeDeclaration {
                name,
                extends,
                methods,
            } => Some((name, extends, methods)),
            _ => None,
        })
        .expect("expected parsed user type");

    assert_eq!(user_type.0, "PetLeader");
    assert_eq!(user_type.1.as_deref(), Some("Biped"));
    let method_names: Vec<_> = user_type
        .2
        .iter()
        .map(|method| method.name.as_str())
        .collect();
    assert_eq!(method_names, vec!["leadDance", "resetPose"]);
}

#[test]
fn inheritance_oop_grading_passes_with_custom_type_and_methods() {
    let report = grade_inheritance_oop(all_ready_input(Some(parsed_inheritance_program())));
    assert!(report.passed);
    assert_eq!(report.lesson, "inheritance-oop-mini-challenge");
    for step in &report.steps {
        assert_eq!(step.status, StepStatus::Ready, "step '{}'", step.name);
    }
}

#[test]
fn inheritance_oop_grading_blocks_without_custom_method() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::UserTypeDeclaration {
            name: "PetLeader".into(),
            extends: Some("Biped".into()),
            methods: vec![],
        }],
    }]);

    let report = grade_inheritance_oop(all_ready_input(Some(program)));
    assert!(!report.passed);
    let custom_method = report
        .steps
        .iter()
        .find(|step| step.name == "define-custom-method")
        .unwrap();
    assert_eq!(custom_method.status, StepStatus::Blocked);
}

#[test]
fn inheritance_ast_survives_json_round_trip() {
    let program = parsed_inheritance_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
