// Arrays + arithmetic E2E tests: validates parsed A3P fixtures and grading.

use eatme_assets::{ArraysArithmeticGradingInput, StepStatus, grade_arrays_and_arithmetic};
use eatme_core::ast::{ArithmeticOperator, Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::{parse_a3p_program, write_synthetic_a3p};

fn arrays_arithmetic_xml() -> &'static str {
    r#"
    <root>
      <element type="UserMethod" name="myFirstMethod" />
      <node type="ArrayDeclaration" name="pets" elementType="Biped" elements="this.cat,this.dog,this.bunny" />
      <node type="ArrayAccess" array="pets" index="0" target="leader" />
      <node type="ForEachArray" item="pet" array="pets" />
      <node type="ArithmeticExpression" operator="add" left="score" right="1" result="scorePlusOne" />
      <node type="ArithmeticExpression" operator="subtract" left="scorePlusOne" right="2" result="scoreMinusTwo" />
      <node type="ArithmeticExpression" operator="multiply" left="scoreMinusTwo" right="3" result="tripleScore" />
      <node type="ArithmeticExpression" operator="divide" left="tripleScore" right="2" result="averageScore" />
    </root>
    "#
}

fn parsed_arrays_program() -> Program {
    let path = write_synthetic_a3p("arrays-arithmetic", arrays_arithmetic_xml());
    parse_a3p_program(&path).unwrap_or_else(|| panic!("failed to parse {}", path.display()))
}

fn all_ready_input(program: Option<Program>) -> ArraysArithmeticGradingInput {
    ArraysArithmeticGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

#[test]
fn parsed_a3p_extracts_array_and_arithmetic_constructs() {
    let program = parsed_arrays_program();
    assert_eq!(program.procedures.len(), 1);

    let body = &program.procedures[0].body;
    assert!(body.iter().any(|statement| matches!(
        statement,
        Statement::ArrayDeclaration {
            name,
            element_type,
            elements,
        } if name == "pets" && element_type == "Biped" && elements.len() == 3
    )));
    assert!(body.iter().any(|statement| matches!(
        statement,
        Statement::ArrayAccess { array, index, target }
            if array == "pets" && index == "0" && target == "leader"
    )));
    assert!(body.iter().any(|statement| matches!(
        statement,
        Statement::ForEachArray { item_name, array, .. }
            if item_name == "pet" && array == "pets"
    )));

    let operators: Vec<_> = body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ArithmeticExpression { operator, .. } => Some(*operator),
            _ => None,
        })
        .collect();
    assert_eq!(
        operators,
        vec![
            ArithmeticOperator::Add,
            ArithmeticOperator::Subtract,
            ArithmeticOperator::Multiply,
            ArithmeticOperator::Divide,
        ]
    );
}

#[test]
fn arrays_and_arithmetic_grading_all_ready_with_parsed_a3p() {
    let report = grade_arrays_and_arithmetic(all_ready_input(Some(parsed_arrays_program())));
    assert!(report.passed);
    assert_eq!(report.lesson, "arrays-collection-choreography");
    for step in &report.steps {
        assert_eq!(step.status, StepStatus::Ready, "step '{}'", step.name);
    }
}

#[test]
fn arrays_and_arithmetic_grading_missing_operator_blocks() {
    let program = Program::new(vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::ArrayDeclaration {
                name: "pets".into(),
                element_type: "Biped".into(),
                elements: vec!["this.cat".into(), "this.dog".into()],
            },
            Statement::ArrayAccess {
                array: "pets".into(),
                index: "0".into(),
                target: "leader".into(),
            },
            Statement::ForEachArray {
                item_name: "pet".into(),
                array: "pets".into(),
                body: vec![],
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Add,
                left: "score".into(),
                right: "1".into(),
                result: "scorePlusOne".into(),
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Subtract,
                left: "scorePlusOne".into(),
                right: "2".into(),
                result: "scoreMinusTwo".into(),
            },
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Multiply,
                left: "scoreMinusTwo".into(),
                right: "3".into(),
                result: "tripleScore".into(),
            },
        ],
    }]);

    let report = grade_arrays_and_arithmetic(all_ready_input(Some(program)));
    assert!(!report.passed);
    let arithmetic = report
        .steps
        .iter()
        .find(|step| step.name == "use-arithmetic-operators")
        .unwrap();
    assert_eq!(arithmetic.status, StepStatus::Blocked);
}

#[test]
fn arrays_ast_survives_json_round_trip() {
    let program = parsed_arrays_program();
    let json = serde_json::to_string_pretty(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored);
}
