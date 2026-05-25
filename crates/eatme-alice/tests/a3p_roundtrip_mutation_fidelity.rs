use eatme_core::ast::{Parameter, Procedure, Program, Statement};

mod structured_a3p_support;

use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

const FIXTURE_XML: &str = r#"
<program>
  <procedure name="sceneActivated">
    <body>
      <statement type="AddEventListener" event="SceneActivated">
        <body>
          <statement type="MethodInvocation" object="rabbit" method="hop">
            <argument value="1"/>
          </statement>
        </body>
      </statement>
      <statement type="CollisionStartListener" object_a="rabbit" object_b="tree">
        <body>
          <statement type="MethodInvocation" object="rabbit" method="turnLeft">
            <argument value="0.25"/>
          </statement>
        </body>
      </statement>
    </body>
  </procedure>
  <procedure name="hop">
    <parameter name="amount" type="Number"/>
    <body>
      <statement type="MethodInvocation" object="rabbit" method="move">
        <argument value="FORWARD"/>
        <argument value="amount"/>
      </statement>
    </body>
  </procedure>
</program>
"#;

fn parse_fixture(name: &str) -> Program {
    let path = write_structured_a3p(name, FIXTURE_XML);
    parse_structured_a3p_program(&path)
        .unwrap_or_else(|| panic!("failed to parse structured fixture {}", path.display()))
}

fn roundtrip(name: &str, program: &Program) -> Program {
    let path = write_structured_a3p(name, &program_to_xml(program));
    parse_structured_a3p_program(&path)
        .unwrap_or_else(|| panic!("failed to re-parse round-trip fixture {}", path.display()))
}

fn program_to_xml(program: &Program) -> String {
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
        push_statements_xml(&mut xml, &procedure.body);
        xml.push_str("</body></procedure>");
    }
    xml.push_str("</program>");
    xml
}

fn push_statements_xml(xml: &mut String, statements: &[Statement]) {
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
            Statement::EventListener { event, body } => {
                xml.push_str("<statement type=\"AddEventListener\" event=\"");
                xml.push_str(&escape_attr(event));
                xml.push_str("\"><body>");
                push_statements_xml(xml, body);
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
                push_statements_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            Statement::DoInOrder { body } => {
                xml.push_str("<statement type=\"DoInOrder\"><body>");
                push_statements_xml(xml, body);
                xml.push_str("</body></statement>");
            }
            unsupported => panic!("unsupported statement in round-trip test: {unsupported:?}"),
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

fn remove_event_statements(statements: &mut Vec<Statement>) {
    statements.retain(|statement| {
        !matches!(
            statement,
            Statement::EventListener { .. } | Statement::CollisionListener { .. }
        )
    });
    for statement in statements {
        match statement {
            Statement::DoInOrder { body }
            | Statement::CountLoop { body, .. }
            | Statement::ForEachArray { body, .. } => remove_event_statements(body),
            Statement::IfElse {
                if_body, else_body, ..
            } => {
                remove_event_statements(if_body);
                remove_event_statements(else_body);
            }
            Statement::EventListener { body, .. } | Statement::CollisionListener { body, .. } => {
                remove_event_statements(body)
            }
            _ => {}
        }
    }
}

fn contains_event_statements(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::EventListener { .. } | Statement::CollisionListener { .. } => true,
        Statement::DoInOrder { body }
        | Statement::CountLoop { body, .. }
        | Statement::ForEachArray { body, .. } => contains_event_statements(body),
        Statement::IfElse {
            if_body, else_body, ..
        } => contains_event_statements(if_body) || contains_event_statements(else_body),
        _ => false,
    })
}

#[test]
fn roundtrip_persists_added_procedure() {
    let mut program = parse_fixture("mutation-add-procedure");
    program.procedures.push(Procedure {
        name: "spin".into(),
        parameters: vec![Parameter {
            name: "speed".into(),
            param_type: "Number".into(),
        }],
        body: vec![Statement::MethodCall {
            object: "rabbit".into(),
            method: "turnLeft".into(),
            arguments: vec!["speed".into()],
        }],
    });

    let reparsed = roundtrip("mutation-add-procedure-roundtrip", &program);

    let added = reparsed
        .procedures
        .iter()
        .find(|procedure| procedure.name == "spin")
        .expect("added procedure should persist after round-trip");
    assert_eq!(added.parameters.len(), 1);
    assert_eq!(added.parameters[0].name, "speed");
    assert_eq!(
        added.body,
        vec![Statement::MethodCall {
            object: "rabbit".into(),
            method: "turnLeft".into(),
            arguments: vec!["speed".into()],
        }]
    );
}

#[test]
fn roundtrip_persists_removing_all_events() {
    let mut program = parse_fixture("mutation-remove-events");
    for procedure in &mut program.procedures {
        remove_event_statements(&mut procedure.body);
    }

    let reparsed = roundtrip("mutation-remove-events-roundtrip", &program);

    assert!(
        reparsed
            .procedures
            .iter()
            .all(|procedure| !contains_event_statements(&procedure.body)),
        "event statements should stay removed after round-trip"
    );
}

#[test]
fn roundtrip_persists_procedure_rename() {
    let mut program = parse_fixture("mutation-rename-procedure");
    let original = program
        .procedures
        .iter_mut()
        .find(|procedure| procedure.name == "hop")
        .expect("fixture should contain hop procedure");
    original.name = "jump".into();

    let reparsed = roundtrip("mutation-rename-procedure-roundtrip", &program);

    assert!(
        reparsed
            .procedures
            .iter()
            .any(|procedure| procedure.name == "jump")
    );
    assert!(
        !reparsed
            .procedures
            .iter()
            .any(|procedure| procedure.name == "hop")
    );
}
