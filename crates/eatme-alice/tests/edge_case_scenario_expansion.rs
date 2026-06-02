use eatme_assets::{NestedControlFlowGradingInput, StepStatus, grade_nested_control_flow};
use eatme_core::ast::{Program, Statement};

#[allow(dead_code)]
mod a3p_parser_support;
mod structured_a3p_support;

use a3p_parser_support::{parse_a3p_program, parse_a3p_scene, write_synthetic_a3p};
use structured_a3p_support::{parse_structured_a3p_program, write_structured_a3p};

fn ready_input(program: Option<Program>) -> NestedControlFlowGradingInput {
    NestedControlFlowGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_program: program,
    }
}

fn max_control_depth(statements: &[Statement]) -> usize {
    statements
        .iter()
        .map(|statement| match statement {
            Statement::CountLoop { body, .. }
            | Statement::EventListener { body, .. }
            | Statement::CollisionListener { body, .. }
            | Statement::ForEachArray { body, .. }
            | Statement::DoInOrder { body } => 1 + max_control_depth(body),
            Statement::IfElse {
                if_body, else_body, ..
            } => 1 + max_control_depth(if_body).max(max_control_depth(else_body)),
            Statement::UserTypeDeclaration { methods, .. } => methods
                .iter()
                .map(|method| max_control_depth(&method.body))
                .max()
                .unwrap_or(0),
            Statement::MethodCall { .. }
            | Statement::ReturnStatement { .. }
            | Statement::FunctionCall { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::VariableAssignment { .. }
            | Statement::ArrayDeclaration { .. }
            | Statement::ArrayAccess { .. }
            | Statement::ArithmeticExpression { .. }
            | Statement::Comment { .. } => 0,
        })
        .max()
        .unwrap_or(0)
}

fn detect_circular_inheritance(program: &Program) -> Result<(), String> {
    use std::collections::{HashMap, HashSet};

    let inheritance: HashMap<String, String> = program
        .procedures
        .iter()
        .flat_map(|procedure| procedure.body.iter())
        .filter_map(|statement| match statement {
            Statement::UserTypeDeclaration {
                name,
                extends: Some(parent),
                ..
            } => Some((name.clone(), parent.clone())),
            _ => None,
        })
        .collect();

    for node in inheritance.keys() {
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        let mut current = node.as_str();
        while let Some(parent) = inheritance.get(current) {
            path.push(current.to_string());
            if !seen.insert(current.to_string()) {
                path.push(parent.clone());
                return Err(format!(
                    "Circular inheritance detected: {}",
                    path.join(" -> ")
                ));
            }
            current = parent;
        }
    }

    Ok(())
}

#[test]
fn empty_project_is_handled_without_crashing() {
    let path = write_synthetic_a3p("empty-project", "<root />");

    let program =
        parse_a3p_program(&path).expect("empty projects should still decode to an empty program");
    let scene = parse_a3p_scene(&path);

    assert!(program.procedures.is_empty());
    assert!(program.functions.is_empty());
    assert!(
        scene.is_none(),
        "scene parser should fail closed for an empty project"
    );
}

#[test]
fn malformed_xml_yields_blocked_report_with_error_message() {
    let path = write_structured_a3p(
        "malformed-project",
        "<program><procedure name=\"broken\"><body><statement type=\"CountLoop\" count=\"3\"></program>",
    );

    let program = parse_structured_a3p_program(&path);
    let report = grade_nested_control_flow(ready_input(program));
    let blocked = report
        .steps
        .iter()
        .find(|step| step.name == "detect-relational-expressions")
        .expect("missing detect-relational-expressions step");

    assert!(!report.passed);
    assert_eq!(blocked.status, StepStatus::Blocked);
    assert!(
        blocked.reason.contains("No student program provided"),
        "expected actionable error message, got {}",
        blocked.reason
    );
}

#[test]
fn parser_handles_more_than_one_hundred_entities() {
    let mut xml =
        String::from("<root><node type=\"SceneObject\" name=\"ground\" kind=\"ground\" />");
    for index in 0..128 {
        xml.push_str(&format!(
            "<node type=\"SceneObject\" name=\"entity{index}\" kind=\"Prop\" position=\"{index},0,0\" size=\"1.0\" color=\"#ffffff\" opacity=\"1.0\" />"
        ));
    }
    xml.push_str("</root>");
    let path = write_synthetic_a3p("maximum-entities", &xml);

    let scene = parse_a3p_scene(&path).expect("large scene should parse");

    assert_eq!(scene.objects.len(), 128);
    assert_eq!(
        scene.objects.first().map(|object| object.name.as_str()),
        Some("entity0")
    );
    assert_eq!(
        scene.objects.last().map(|object| object.name.as_str()),
        Some("entity127")
    );
}

#[test]
fn long_method_names_are_preserved() {
    let long_name = format!("method_{}", "x".repeat(210));
    let xml = format!(
        "<program><procedure name=\"{long_name}\"><body><statement type=\"MethodInvocation\" object=\"this.hero\" method=\"say\"><argument value='\"ok\"' /></statement></body></procedure></program>"
    );
    let path = write_structured_a3p("long-method-name", &xml);

    let program =
        parse_structured_a3p_program(&path).expect("long method name fixture should parse");

    assert_eq!(program.procedures.len(), 1);
    assert_eq!(program.procedures[0].name, long_name);
    assert!(program.procedures[0].name.len() > 200);
}

#[test]
fn deeply_nested_control_flow_reaches_ten_levels() {
    let xml = r#"
    <program>
      <procedure name="myFirstMethod">
        <body>
          <statement type="CountLoop" count="1">
            <body>
              <statement type="ConditionalStatement" condition="c1">
                <ifBody>
                  <statement type="CountLoop" count="1">
                    <body>
                      <statement type="ConditionalStatement" condition="c2">
                        <ifBody>
                          <statement type="CountLoop" count="1">
                            <body>
                              <statement type="ConditionalStatement" condition="c3">
                                <ifBody>
                                  <statement type="CountLoop" count="1">
                                    <body>
                                      <statement type="ConditionalStatement" condition="c4">
                                        <ifBody>
                                          <statement type="CountLoop" count="1">
                                            <body>
                                              <statement type="ConditionalStatement" condition="c5">
                                                <ifBody>
                                                  <statement type="MethodInvocation" object="this.hero" method="say">
                                                    <argument value='"deep"' />
                                                  </statement>
                                                </ifBody>
                                              </statement>
                                            </body>
                                          </statement>
                                        </ifBody>
                                      </statement>
                                    </body>
                                  </statement>
                                </ifBody>
                              </statement>
                            </body>
                          </statement>
                        </ifBody>
                      </statement>
                    </body>
                  </statement>
                </ifBody>
              </statement>
            </body>
          </statement>
        </body>
      </procedure>
    </program>
    "#;
    let path = write_structured_a3p("deeply-nested-control-flow", xml);
    let program = parse_structured_a3p_program(&path).expect("deep nesting fixture should parse");

    assert_eq!(max_control_depth(&program.procedures[0].body), 10);
}

#[test]
fn circular_user_type_inheritance_is_detected() {
    let xml = r#"
    <root>
      <element type="UserMethod" name="myFirstMethod" />
      <node type="UserType" name="ClassA" extends="ClassB" methods="alpha" />
      <node type="UserType" name="ClassB" extends="ClassA" methods="beta" />
    </root>
    "#;
    let path = write_synthetic_a3p("circular-inheritance", xml);
    let program = parse_a3p_program(&path).expect("circular inheritance fixture should parse");

    let error = detect_circular_inheritance(&program).expect_err("cycle should be reported");

    assert!(error.contains("Circular inheritance detected"));
    assert!(error.contains("ClassA"));
    assert!(error.contains("ClassB"));
}
