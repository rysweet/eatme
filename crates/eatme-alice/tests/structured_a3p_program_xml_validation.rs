#[allow(dead_code)]
mod structured_a3p_support;

use eatme_core::ast::Statement;
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

#[test]
fn structured_fixture_parser_validates_program_xml_root_and_statement_structure() {
    let path = write_structured_a3p(
        "program-xml-structure",
        r#"
        <program>
            <procedure name="intro">
                <parameter name="target" type="Object" />
                <body>
                    <statement type="MethodInvocation" object="this.camera" method="setVehicle">
                        <argument value="target" />
                    </statement>
                    <statement type="MethodInvocation" object="this.camera" method="pointAt" arguments="target" />
                    <statement type="CountLoop" count="2">
                        <body>
                            <statement type="MethodInvocation" object="this.hero" method="moveToward">
                                <argument value="this.stage" />
                                <argument value="0.5" />
                            </statement>
                        </body>
                    </statement>
                </body>
            </procedure>
        </program>
        "#,
    );

    let program =
        parse_structured_a3p_program(&path).expect("program.xml should parse as a program");

    assert_eq!(program.procedures.len(), 1);
    let intro = &program.procedures[0];
    assert_eq!(intro.name, "intro");
    assert_eq!(intro.parameters.len(), 1);
    assert_eq!(intro.parameters[0].name, "target");
    assert_eq!(intro.body.len(), 3);

    match &intro.body[0] {
        Statement::MethodCall {
            object,
            method,
            arguments,
        } => {
            assert_eq!(object, "this.camera");
            assert_eq!(method, "setVehicle");
            assert_eq!(arguments, &vec!["target".to_string()]);
        }
        other => panic!("expected first statement to be MethodCall, got {other:?}"),
    }

    match &intro.body[2] {
        Statement::CountLoop { count, body } => {
            assert_eq!(*count, 2);
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::MethodCall {
                    method, arguments, ..
                } => {
                    assert_eq!(method, "moveToward");
                    assert_eq!(arguments.len(), 2);
                }
                other => panic!("expected loop body to contain MethodCall, got {other:?}"),
            }
        }
        other => panic!("expected third statement to be CountLoop, got {other:?}"),
    }
}

#[test]
fn structured_fixture_parser_rejects_non_program_xml_even_when_named_program_xml() {
    let path = write_structured_a3p(
        "invalid-program-xml-root",
        r#"
        <scene>
            <procedure name="intro">
                <body>
                    <statement type="MethodInvocation" object="this.camera" method="pointAt" arguments="this.hero" />
                </body>
            </procedure>
        </scene>
        "#,
    );

    assert!(
        parse_structured_a3p_program(&path).is_none(),
        "parser should reject program.xml entries that do not use a program root"
    );
}
