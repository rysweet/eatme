#![allow(unexpected_cfgs)]

use std::path::{Path, PathBuf};

use eatme_assets::grading_report::{LoopsGradingInput, grade_loops_and_conditionals};
use eatme_assets::{
    CreativeProjectGradingInput, FunctionsGradingInput, ParametersGradingInput, StepStatus,
    VariablesGradingInput, grade_creative_project, grade_functions, grade_parameters,
    grade_variables,
};
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement};

#[allow(dead_code)]
mod a3p_content_support;
#[allow(dead_code)]
mod a3p_parser_support;
mod structured_a3p_support;

use a3p_content_support::extract_all_xml;
use a3p_parser_support::parse_a3p_program;
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

struct RealFixture<'a> {
    name: &'a str,
    source: &'a str,
    xml_markers: &'a [&'a str],
}

const REAL_FIXTURES: &[RealFixture<'_>] = &[
    RealFixture {
        name: "amazonMinimum",
        source: "alice/core/resources/src/application/resources/starter-projects/amazonMinimum.a3p",
        xml_markers: &[
            "UserMethod",
            "MethodInvocation",
            "ConditionalStatement",
            "UserParameter",
            "LocalDeclarationStatement",
        ],
    },
    RealFixture {
        name: "indiaMinimum",
        source: "alice/core/ide/src/test/resources/starters/indiaMinimum.a3p",
        xml_markers: &[
            "UserMethod",
            "MethodInvocation",
            "ConditionalStatement",
            "UserParameter",
            "LocalDeclarationStatement",
        ],
    },
    RealFixture {
        name: "magicMinimum",
        source: "alice/core/resources/src/application/resources/starter-projects/magicMinimum.a3p",
        xml_markers: &["UserMethod", "MethodInvocation", "ReturnStatement"],
    },
    RealFixture {
        name: "lagoonMinimum",
        source: "alice/core/resources/src/application/resources/starter-projects/lagoonMinimum.a3p",
        xml_markers: &["UserMethod", "MethodInvocation"],
    },
    RealFixture {
        name: "africaMinimum",
        source: "alice/core/resources/src/application/resources/starter-projects/africaMinimum.a3p",
        xml_markers: &["UserMethod", "MethodInvocation"],
    },
    RealFixture {
        name: "iceFull",
        source: "alice/core/resources/src/application/resources/starter-projects/iceFull.a3p",
        xml_markers: &[
            "UserMethod",
            "MethodInvocation",
            "ReturnStatement",
            "org.lgna.project.ast.Comment",
        ],
    },
    RealFixture {
        name: "snowFull",
        source: "alice/core/resources/src/application/resources/starter-projects/snowFull.a3p",
        xml_markers: &["UserMethod", "MethodInvocation"],
    },
];

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real")
        .join(format!("{name}.a3p"))
}

fn fixture_xml(name: &str) -> String {
    extract_all_xml(&fixture_path(name))
}

fn parse_fixture_program(name: &str) -> Program {
    let path = fixture_path(name);
    parse_a3p_program(&path).unwrap_or_else(|| panic!("failed to parse {}", path.display()))
}

fn loops_input(program: Program) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "real Alice fixture parsed".into(),
        deps_available: true,
        deps_reason: "fixture-driven grading".into(),
        student_program: Some(program),
    }
}

fn variables_input(program: Program) -> VariablesGradingInput {
    VariablesGradingInput {
        assets_valid: true,
        asset_reason: "real Alice fixture parsed".into(),
        deps_available: true,
        deps_reason: "fixture-driven grading".into(),
        student_program: Some(program),
    }
}

fn parameters_input(program: Program) -> ParametersGradingInput {
    ParametersGradingInput {
        assets_valid: true,
        asset_reason: "real Alice fixture parsed".into(),
        deps_available: true,
        deps_reason: "fixture-driven grading".into(),
        student_program: Some(program),
    }
}

fn functions_input(program: Program) -> FunctionsGradingInput {
    FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "real Alice fixture parsed".into(),
        deps_available: true,
        deps_reason: "fixture-driven grading".into(),
        student_program: Some(program),
    }
}

fn creative_input(program: Program) -> CreativeProjectGradingInput {
    CreativeProjectGradingInput {
        assets_valid: true,
        asset_reason: "real Alice fixture parsed".into(),
        deps_available: true,
        deps_reason: "fixture-driven grading".into(),
        student_program: Some(program),
    }
}

fn roundtrip_program(name: &str, program: &Program) -> Program {
    let xml = program_to_structured_xml(program);
    let path = write_structured_a3p(name, &xml);
    parse_structured_a3p_program(&path)
        .unwrap_or_else(|| panic!("failed to re-parse round-trip fixture {}", path.display()))
}

fn program_to_structured_xml(program: &Program) -> String {
    let mut xml = String::from("<program>");
    for procedure in &program.procedures {
        xml.push_str("<procedure name=\"");
        xml.push_str(&escape_attr(&procedure.name));
        xml.push_str("\">");
        for parameter in &procedure.parameters {
            xml.push_str("<parameter name=\"");
            xml.push_str(&escape_attr(&parameter.name));
            xml.push_str("\" type=\"");
            xml.push_str(&escape_attr(&parameter.param_type));
            xml.push_str("\"/>");
        }
        xml.push_str("<body>");
        push_statement_xml(&mut xml, &procedure.body);
        xml.push_str("</body></procedure>");
    }
    for function in &program.functions {
        xml.push_str("<function name=\"");
        xml.push_str(&escape_attr(&function.name));
        xml.push_str("\" return_type=\"");
        xml.push_str(&escape_attr(&function.return_type));
        xml.push_str("\"><body>");
        push_statement_xml(&mut xml, &function.body);
        xml.push_str("</body></function>");
    }
    xml.push_str("</program>");
    xml
}

fn push_statement_xml(xml: &mut String, statements: &[Statement]) {
    for statement in statements {
        match statement {
            Statement::MethodCall {
                object,
                method,
                arguments,
            } => {
                xml.push_str("<statement type=\"MethodInvocation\" object=\"");
                xml.push_str(&escape_attr(object));
                xml.push_str("\" method=\"");
                xml.push_str(&escape_attr(method));
                xml.push_str("\">");
                for argument in arguments {
                    xml.push_str("<argument value=\"");
                    xml.push_str(&escape_attr(argument));
                    xml.push_str("\"/>");
                }
                xml.push_str("</statement>");
            }
            Statement::CountLoop { count, body } => {
                xml.push_str("<statement type=\"CountLoop\" count=\"");
                xml.push_str(&count.to_string());
                xml.push_str("\"><body>");
                push_statement_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                xml.push_str("<statement type=\"ConditionalStatement\" condition=\"");
                xml.push_str(&escape_attr(condition));
                xml.push_str("\"><ifBody>");
                push_statement_xml(xml, if_body);
                xml.push_str("</ifBody><elseBody>");
                push_statement_xml(xml, else_body);
                xml.push_str("</elseBody></statement>");
            }
            Statement::EventListener { event, body } => {
                xml.push_str("<statement type=\"AddEventListener\" event=\"");
                xml.push_str(&escape_attr(event));
                xml.push_str("\"><body>");
                push_statement_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            Statement::CollisionListener {
                object_a,
                object_b,
                body,
            } => {
                xml.push_str("<statement type=\"CollisionStartListener\" object_a=\"");
                xml.push_str(&escape_attr(object_a));
                xml.push_str("\" object_b=\"");
                xml.push_str(&escape_attr(object_b));
                xml.push_str("\"><body>");
                push_statement_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            Statement::DoInOrder { body } => {
                xml.push_str("<statement type=\"DoInOrder\"><body>");
                push_statement_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            Statement::ReturnStatement { expression } => {
                xml.push_str("<statement type=\"ReturnStatement\" expression=\"");
                xml.push_str(&escape_attr(expression));
                xml.push_str("\"/>");
            }
            Statement::FunctionCall {
                object,
                function,
                arguments,
            } => {
                xml.push_str("<statement type=\"FunctionCall\" object=\"");
                xml.push_str(&escape_attr(object));
                xml.push_str("\" function=\"");
                xml.push_str(&escape_attr(function));
                xml.push_str("\">");
                for argument in arguments {
                    xml.push_str("<argument value=\"");
                    xml.push_str(&escape_attr(argument));
                    xml.push_str("\"/>");
                }
                xml.push_str("</statement>");
            }
            Statement::VariableDeclaration {
                name,
                var_type,
                initial_value,
            } => {
                xml.push_str("<statement type=\"VariableDeclaration\" name=\"");
                xml.push_str(&escape_attr(name));
                xml.push_str("\" var_type=\"");
                xml.push_str(&escape_attr(var_type));
                xml.push_str("\" initial_value=\"");
                xml.push_str(&escape_attr(initial_value));
                xml.push_str("\"/>");
            }
            Statement::VariableAssignment { name, value } => {
                xml.push_str("<statement type=\"VariableAssignment\" name=\"");
                xml.push_str(&escape_attr(name));
                xml.push_str("\" value=\"");
                xml.push_str(&escape_attr(value));
                xml.push_str("\"/>");
            }
            Statement::Comment { text } => {
                xml.push_str("<statement type=\"Comment\" text=\"");
                xml.push_str(&escape_attr(text));
                xml.push_str("\"/>");
            }
            Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ForEachArray { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::UserTypeDeclaration { .. } => {
                panic!("unsupported statement in real-fixture round-trip: {statement:?}");
            }
        }
    }
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[test]
fn copied_real_fixtures_parse_and_expose_expected_lesson_markers() {
    for fixture in REAL_FIXTURES {
        let path = fixture_path(fixture.name);
        assert!(
            path.exists(),
            "missing copied fixture {} from {}",
            path.display(),
            fixture.source
        );

        let xml = fixture_xml(fixture.name);
        assert!(!xml.is_empty(), "{} should contain XML", fixture.name);
        for marker in fixture.xml_markers {
            assert!(
                xml.contains(marker),
                "{} should contain XML marker {marker} from {}",
                fixture.name,
                fixture.source
            );
        }

        let program = parse_fixture_program(fixture.name);
        assert!(
            !program.procedures.is_empty(),
            "{} should parse into at least one procedure",
            fixture.name
        );
        assert!(
            program
                .procedures
                .iter()
                .flat_map(|procedure| procedure.body.iter())
                .any(|statement| matches!(statement, Statement::MethodCall { .. })),
            "{} should expose at least one MethodCall through the parser",
            fixture.name
        );
    }
}

#[test]
fn amazon_minimum_fixture_grades_loops_baseline() {
    let program = parse_fixture_program("amazonMinimum");
    assert!(
        program
            .procedures
            .iter()
            .flat_map(|procedure| procedure.body.iter())
            .any(|statement| matches!(statement, Statement::IfElse { .. })),
        "amazonMinimum should expose at least one IfElse through the parser"
    );

    let report = grade_loops_and_conditionals(loops_input(program));
    assert!(!report.passed);
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps[3].name, "build-counting-loop");
    assert_eq!(report.steps[3].status, StepStatus::Blocked);
    assert_eq!(report.steps[4].name, "add-conditional-branch");
    assert_eq!(report.steps[4].status, StepStatus::Blocked);
}

#[test]
fn amazon_minimum_roundtrip_grades_variables_lesson() {
    let mut program = parse_fixture_program("amazonMinimum");
    let first = program
        .procedures
        .first_mut()
        .expect("amazonMinimum should parse with at least one procedure");
    first.body.push(Statement::VariableDeclaration {
        name: "speed".into(),
        var_type: "DecimalNumber".into(),
        initial_value: "0.5".into(),
    });
    first.body.push(Statement::MethodCall {
        object: "this.rabbit".into(),
        method: "move".into(),
        arguments: vec!["FORWARD".into(), "speed".into()],
    });
    first.body.push(Statement::VariableAssignment {
        name: "speed".into(),
        value: "1.0".into(),
    });

    let roundtripped = roundtrip_program("amazon-minimum-variables", &program);
    let report = grade_variables(variables_input(roundtripped.clone()));

    assert!(
        report.passed,
        "variables lesson should pass after round-trip"
    );
    assert!(roundtripped.procedures.iter().any(|procedure| {
        procedure.body.iter().any(|statement| matches!(statement, Statement::VariableDeclaration { name, .. } if name == "speed"))
    }));
    assert!(roundtripped.procedures.iter().any(|procedure| {
        procedure.body.iter().any(|statement| matches!(statement, Statement::VariableAssignment { name, .. } if name == "speed"))
    }));
}

#[test]
fn india_minimum_roundtrip_grades_parameters_lesson() {
    let mut program = parse_fixture_program("indiaMinimum");
    program.procedures.push(Procedure {
        name: "moveAnimal".into(),
        parameters: vec![Parameter {
            name: "distance".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.camel".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "distance".into()],
        }],
    });
    program
        .procedures
        .first_mut()
        .expect("indiaMinimum should parse with at least one procedure")
        .body
        .push(Statement::MethodCall {
            object: "this".into(),
            method: "moveAnimal".into(),
            arguments: vec!["2.0".into()],
        });

    let roundtripped = roundtrip_program("india-minimum-parameters", &program);
    let report = grade_parameters(parameters_input(roundtripped.clone()));

    assert!(
        report.passed,
        "parameters lesson should pass after round-trip"
    );
    assert!(
        roundtripped
            .procedures
            .iter()
            .any(|procedure| !procedure.parameters.is_empty())
    );
}

#[test]
fn magic_minimum_roundtrip_grades_functions_lesson() {
    let mut program = parse_fixture_program("magicMinimum");
    program.functions.push(Function {
        name: "computeDistance".into(),
        return_type: "DecimalNumber".into(),
        body: vec![
            Statement::MethodCall {
                object: "this.dragon".into(),
                method: "getDistanceTo".into(),
                arguments: vec!["this.wizard".into()],
            },
            Statement::ReturnStatement {
                expression: "this.dragon getDistanceTo this.wizard".into(),
            },
        ],
    });
    program
        .procedures
        .first_mut()
        .expect("magicMinimum should parse with at least one procedure")
        .body
        .push(Statement::FunctionCall {
            object: "this".into(),
            function: "computeDistance".into(),
            arguments: vec!["this.dragon".into(), "this.wizard".into()],
        });

    let roundtripped = roundtrip_program("magic-minimum-functions", &program);
    let report = grade_functions(functions_input(roundtripped.clone()));

    assert!(
        report.passed,
        "functions lesson should pass after round-trip"
    );
    assert_eq!(roundtripped.functions.len(), 1);
    assert!(roundtripped.procedures.iter().any(|procedure| {
        procedure.body.iter().any(|statement| matches!(statement, Statement::FunctionCall { function, .. } if function == "computeDistance"))
    }));
}

#[test]
fn ice_full_roundtrip_grades_creative_lesson() {
    let mut program = parse_fixture_program("iceFull");
    let first = program
        .procedures
        .first_mut()
        .expect("iceFull should parse with at least one procedure");
    first.body.push(Statement::MethodCall {
        object: "this.penguin".into(),
        method: "say".into(),
        arguments: vec!["\"Welcome!\"".into()],
    });
    first.body.push(Statement::MethodCall {
        object: "this.seal".into(),
        method: "walk".into(),
        arguments: vec!["FORWARD".into(), "1.0".into()],
    });
    first.body.push(Statement::CountLoop {
        count: 3,
        body: vec![Statement::MethodCall {
            object: "this.penguin".into(),
            method: "turn".into(),
            arguments: vec!["LEFT".into(), "0.25".into()],
        }],
    });
    first.body.push(Statement::EventListener {
        event: "SceneActivated".into(),
        body: vec![Statement::MethodCall {
            object: "this.penguin".into(),
            method: "say".into(),
            arguments: vec!["\"Game on!\"".into()],
        }],
    });
    program.procedures.push(Procedure {
        name: "doSpecialMove".into(),
        parameters: vec![Parameter {
            name: "speed".into(),
            param_type: "DecimalNumber".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "this.seal".into(),
            method: "move".into(),
            arguments: vec!["FORWARD".into(), "speed".into()],
        }],
    });

    let roundtripped = roundtrip_program("ice-full-creative", &program);
    let report = grade_creative_project(creative_input(roundtripped));

    assert!(
        report.passed,
        "creative lesson should pass after round-trip"
    );
    for step in report.steps.iter().skip(3) {
        assert_eq!(
            step.status,
            StepStatus::Ready,
            "creative step {}",
            step.name
        );
    }
}
