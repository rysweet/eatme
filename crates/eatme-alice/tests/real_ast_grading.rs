// Real-Alice desktop execution tests for all grading pipelines.
//
// These tests are gated behind `EATME_REAL_ALICE=1` and require `ALICE_HOME`
// to point at a built Alice 3 checkout containing starter project .a3p files.
//
// When the env gate is not set, all integration tests are #[ignore]'d.
// Parser unit tests (at the bottom) always run.

use eatme_assets::{
    EventsGradingInput, FunctionsGradingInput, LoopsGradingInput, ParametersGradingInput,
    StepStatus, VariablesGradingInput, grade_events_and_collision, grade_functions,
    grade_loops_and_conditionals, grade_parameters, grade_variables,
};
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement, VariableDeclaration};
use std::io::Read;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// A3P Parser — regex-based extraction of AST from Alice .a3p ZIP/XML files
// ---------------------------------------------------------------------------

/// Locate the Alice starter project directory from ALICE_HOME.
fn alice_starter_projects_dir() -> PathBuf {
    let alice_home = std::env::var("ALICE_HOME").expect("ALICE_HOME must be set");
    let base = PathBuf::from(alice_home);
    // Common locations: gallery/starterProjects or installed/share/alice3/gallery
    let candidates = [
        base.join("gallery/starterProjects"),
        base.join("installed/share/alice3/gallery/starterProjects"),
    ];
    for candidate in &candidates {
        if candidate.is_dir() {
            return candidate.clone();
        }
    }
    panic!(
        "Could not find starterProjects directory under ALICE_HOME. Tried: {:?}",
        candidates
    );
}

/// Read a specific XML entry from a .a3p ZIP file by entry name.
fn read_a3p_entry(a3p_path: &Path, entry_name: &str) -> Option<String> {
    let file = std::fs::File::open(a3p_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut entry = archive.by_name(entry_name).ok()?;
    let mut contents = String::new();
    entry.read_to_string(&mut contents).ok()?;
    Some(contents)
}

/// Parse a .a3p file into our AST Program representation.
///
/// Strategy: read the project XML, extract procedure/function/variable
/// definitions using regex, and build the AST structs.
fn parse_a3p_program(a3p_path: &Path) -> Option<Program> {
    // The main project XML is typically at the root of the ZIP
    let xml = read_a3p_entry(a3p_path, "project.xml").or_else(|| {
        // Try to find any .xml entry
        let file = std::fs::File::open(a3p_path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;
        let xml_index = (0..archive.len()).find(|&i| {
            archive
                .by_index(i)
                .ok()
                .is_some_and(|e| e.name().ends_with(".xml"))
        })?;
        let mut entry = archive.by_index(xml_index).ok()?;
        let mut contents = String::new();
        entry.read_to_string(&mut contents).ok()?;
        Some(contents)
    })?;

    let procedures = extract_procedures(&xml);
    let functions = extract_functions(&xml);
    let variable_declarations = extract_variable_declarations(&xml);

    Some(Program {
        procedures,
        functions,
        variable_declarations,
    })
}

/// Extract procedure definitions from Alice XML using regex.
fn extract_procedures(xml: &str) -> Vec<Procedure> {
    let proc_re =
        regex::Regex::new(r#"<procedure\s+name="([^"]+)"[^>]*>([\s\S]*?)</procedure>"#).unwrap();

    let param_re = regex::Regex::new(r#"<parameter\s+name="([^"]+)"\s+type="([^"]+)""#).unwrap();

    let mut procedures = Vec::new();
    for cap in proc_re.captures_iter(xml) {
        let name = cap[1].to_string();
        let body_xml = &cap[2];

        let parameters: Vec<Parameter> = param_re
            .captures_iter(body_xml)
            .map(|p| Parameter {
                name: p[1].to_string(),
                param_type: p[2].to_string(),
            })
            .collect();

        let body = extract_statements(body_xml);

        procedures.push(Procedure {
            name,
            parameters,
            body,
        });
    }
    procedures
}

/// Extract function definitions from Alice XML using regex.
fn extract_functions(xml: &str) -> Vec<Function> {
    let func_re = regex::Regex::new(
        r#"<function\s+name="([^"]+)"[^>]*returnType="([^"]+)"[^>]*>([\s\S]*?)</function>"#,
    )
    .unwrap();

    let mut functions = Vec::new();
    for cap in func_re.captures_iter(xml) {
        let name = cap[1].to_string();
        let return_type = cap[2].to_string();
        let body_xml = &cap[3];
        let body = extract_statements(body_xml);
        functions.push(Function {
            name,
            return_type,
            body,
        });
    }
    functions
}

/// Extract variable declarations from Alice XML using regex.
fn extract_variable_declarations(xml: &str) -> Vec<VariableDeclaration> {
    let var_re = regex::Regex::new(
        r#"<localDeclaration\s+name="([^"]+)"\s+type="([^"]+)"[^>]*initialValue="([^"]*)"#,
    )
    .unwrap();

    var_re
        .captures_iter(xml)
        .map(|cap| VariableDeclaration {
            name: cap[1].to_string(),
            var_type: cap[2].to_string(),
            initial_value: cap[3].to_string(),
        })
        .collect()
}

/// Extract statements from an XML fragment using regex patterns.
/// Only extracts top-level statements; nested statements are handled recursively
/// by container elements (countLoop, ifElse, eventListener, collisionListener).
fn extract_statements(xml: &str) -> Vec<Statement> {
    let mut statements: Vec<(usize, Statement)> = Vec::new();

    // Container elements: extract them and record their byte ranges so we can
    // skip simple statements that fall inside a container.
    let mut container_ranges: Vec<(usize, usize)> = Vec::new();

    // CountLoop
    let loop_re =
        regex::Regex::new(r#"<countLoop\s+count="(\d+)"[^>]*>([\s\S]*?)</countLoop>"#).unwrap();
    for cap in loop_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        container_ranges.push((m.start(), m.end()));
        let count: u32 = cap[1].parse().unwrap_or(0);
        let body = extract_statements(&cap[2]);
        statements.push((m.start(), Statement::CountLoop { count, body }));
    }

    // IfElse
    let if_re =
        regex::Regex::new(r#"<ifElse\s+condition="([^"]+)"[^>]*>([\s\S]*?)</ifElse>"#).unwrap();
    for cap in if_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        container_ranges.push((m.start(), m.end()));
        statements.push((
            m.start(),
            Statement::IfElse {
                condition: cap[1].to_string(),
                if_body: extract_statements(&cap[2]),
                else_body: vec![],
            },
        ));
    }

    // EventListener
    let event_re =
        regex::Regex::new(r#"<eventListener\s+event="([^"]+)"[^>]*>([\s\S]*?)</eventListener>"#)
            .unwrap();
    for cap in event_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        container_ranges.push((m.start(), m.end()));
        statements.push((
            m.start(),
            Statement::EventListener {
                event: cap[1].to_string(),
                body: extract_statements(&cap[2]),
            },
        ));
    }

    // CollisionListener
    let collision_re = regex::Regex::new(
        r#"<collisionListener\s+objectA="([^"]+)"\s+objectB="([^"]+)"[^>]*>([\s\S]*?)</collisionListener>"#
    ).unwrap();
    for cap in collision_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        container_ranges.push((m.start(), m.end()));
        statements.push((
            m.start(),
            Statement::CollisionListener {
                object_a: cap[1].to_string(),
                object_b: cap[2].to_string(),
                body: extract_statements(&cap[3]),
            },
        ));
    }

    let is_inside_container = |pos: usize| -> bool {
        container_ranges
            .iter()
            .any(|&(start, end)| pos > start && pos < end)
    };

    // Simple (leaf) statements — only match if not inside a container element
    let method_re =
        regex::Regex::new(r#"<methodInvocation\s+object="([^"]+)"\s+method="([^"]+)"[^>]*/>"#)
            .unwrap();
    for cap in method_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        if !is_inside_container(m.start()) {
            statements.push((
                m.start(),
                Statement::MethodCall {
                    object: cap[1].to_string(),
                    method: cap[2].to_string(),
                    arguments: vec![],
                },
            ));
        }
    }

    let func_call_re = regex::Regex::new(r#"<functionInvocation\s+name="([^"]+)"[^>]*/>"#).unwrap();
    for cap in func_call_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        if !is_inside_container(m.start()) {
            statements.push((
                m.start(),
                Statement::FunctionCall {
                    function_name: cap[1].to_string(),
                },
            ));
        }
    }

    let return_re = regex::Regex::new(r#"<returnStatement\s+value="([^"]+)"[^>]*/>"#).unwrap();
    for cap in return_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        if !is_inside_container(m.start()) {
            statements.push((
                m.start(),
                Statement::ReturnStatement {
                    value: cap[1].to_string(),
                },
            ));
        }
    }

    let assign_re =
        regex::Regex::new(r#"<assignmentExpression\s+variable="([^"]+)"\s+value="([^"]+)"[^>]*/>"#)
            .unwrap();
    for cap in assign_re.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        if !is_inside_container(m.start()) {
            statements.push((
                m.start(),
                Statement::VariableAssignment {
                    variable: cap[1].to_string(),
                    value: cap[2].to_string(),
                },
            ));
        }
    }

    // Sort by position to preserve document order
    statements.sort_by_key(|(pos, _)| *pos);
    statements.into_iter().map(|(_, s)| s).collect()
}

// ---------------------------------------------------------------------------
// Helper: should this test run?
// ---------------------------------------------------------------------------

fn should_run_real_alice() -> bool {
    std::env::var("EATME_REAL_ALICE").map_or(false, |v| v == "1")
}

fn skip_unless_real_alice() {
    if !should_run_real_alice() {
        eprintln!("EATME_REAL_ALICE not set — skipping real-Alice test");
    }
}

fn find_a3p(name: &str) -> PathBuf {
    let dir = alice_starter_projects_dir();
    let path = dir.join(name);
    assert!(
        path.exists(),
        "Starter project not found: {}",
        path.display()
    );
    path
}

fn grading_input_ready() -> (bool, String, bool, String) {
    (
        true,
        "All 93 scenario assets passed validation".into(),
        true,
        "All required tools available".into(),
    )
}

// ---------------------------------------------------------------------------
// Integration tests — gated behind EATME_REAL_ALICE=1
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn real_alice_a3p_parses_without_error() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path);
    assert!(
        program.is_some(),
        "Failed to parse amazonMinimum.a3p into a Program"
    );
    let program = program.unwrap();
    assert!(
        !program.procedures.is_empty(),
        "Parsed program should have at least one procedure"
    );
}

#[test]
#[ignore]
fn real_alice_a3p_round_trip() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored, "Program should survive JSON round-trip");
}

#[test]
#[ignore]
fn real_alice_loops_grading() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let (assets_valid, asset_reason, deps_available, deps_reason) = grading_input_ready();

    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid,
        asset_reason,
        deps_available,
        deps_reason,
        student_program: Some(program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
    assert_eq!(report.steps.len(), 7);

    // Preconditions always pass with valid inputs
    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    // The actual grading results depend on amazonMinimum.a3p content.
    // We assert the report is structurally valid — actual pass/block
    // status depends on whether the starter project has loops/conditionals.
    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                StepStatus::Ready | StepStatus::Blocked | StepStatus::NotYetTested
            ),
            "step {} has unexpected status: {:?}",
            step.name,
            step.status
        );
    }
}

#[test]
#[ignore]
fn real_alice_events_grading() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let (assets_valid, asset_reason, deps_available, deps_reason) = grading_input_ready();

    let report = grade_events_and_collision(EventsGradingInput {
        assets_valid,
        asset_reason,
        deps_available,
        deps_reason,
        student_program: Some(program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "events-collision-proximity-game");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                StepStatus::Ready | StepStatus::Blocked | StepStatus::NotYetTested
            ),
            "step {} has unexpected status: {:?}",
            step.name,
            step.status
        );
    }
}

#[test]
#[ignore]
fn real_alice_functions_grading() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let (assets_valid, asset_reason, deps_available, deps_reason) = grading_input_ready();

    let report = grade_functions(FunctionsGradingInput {
        assets_valid,
        asset_reason,
        deps_available,
        deps_reason,
        student_program: Some(program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "functions-mini-challenge");
    assert_eq!(report.steps.len(), 8);

    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                StepStatus::Ready | StepStatus::Blocked | StepStatus::NotYetTested
            ),
            "step {} has unexpected status: {:?}",
            step.name,
            step.status
        );
    }
}

#[test]
#[ignore]
fn real_alice_variables_grading() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let (assets_valid, asset_reason, deps_available, deps_reason) = grading_input_ready();

    let report = grade_variables(VariablesGradingInput {
        assets_valid,
        asset_reason,
        deps_available,
        deps_reason,
        student_program: Some(program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "variables-scorekeeper-timekeeper");
    assert_eq!(report.steps.len(), 8);

    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                StepStatus::Ready | StepStatus::Blocked | StepStatus::NotYetTested
            ),
            "step {} has unexpected status: {:?}",
            step.name,
            step.status
        );
    }
}

#[test]
#[ignore]
fn real_alice_parameters_grading() {
    skip_unless_real_alice();
    if !should_run_real_alice() {
        return;
    }
    let a3p_path = find_a3p("amazonMinimum.a3p");
    let program = parse_a3p_program(&a3p_path).expect("Failed to parse .a3p");
    let (assets_valid, asset_reason, deps_available, deps_reason) = grading_input_ready();

    let report = grade_parameters(ParametersGradingInput {
        assets_valid,
        asset_reason,
        deps_available,
        deps_reason,
        student_program: Some(program),
    });

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "parameters-procedure-generalization");
    assert_eq!(report.steps.len(), 7);

    assert_eq!(report.steps[0].status, StepStatus::Ready);
    assert_eq!(report.steps[1].status, StepStatus::Ready);
    assert_eq!(report.steps[2].status, StepStatus::Ready);

    for step in &report.steps {
        assert!(
            matches!(
                step.status,
                StepStatus::Ready | StepStatus::Blocked | StepStatus::NotYetTested
            ),
            "step {} has unexpected status: {:?}",
            step.name,
            step.status
        );
    }
}

// ---------------------------------------------------------------------------
// Parser unit tests — always run (no env gate)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn extract_procedures_from_xml_snippet() {
        let xml = r#"
            <procedure name="myFirstMethod">
                <methodInvocation object="this.cat" method="walk"/>
            </procedure>
        "#;
        let procs = extract_procedures(xml);
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].name, "myFirstMethod");
        assert!(!procs[0].body.is_empty());
    }

    #[test]
    fn extract_procedures_with_parameters() {
        let xml = r#"
            <procedure name="greet">
                <parameter name="message" type="String"/>
                <methodInvocation object="this.cat" method="say"/>
            </procedure>
        "#;
        let procs = extract_procedures(xml);
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].name, "greet");
        assert_eq!(procs[0].parameters.len(), 1);
        assert_eq!(procs[0].parameters[0].name, "message");
        assert_eq!(procs[0].parameters[0].param_type, "String");
    }

    #[test]
    fn extract_functions_from_xml_snippet() {
        let xml = r#"
            <function name="getScore" returnType="WholeNumber">
                <returnStatement value="42"/>
            </function>
        "#;
        let funcs = extract_functions(xml);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "getScore");
        assert_eq!(funcs[0].return_type, "WholeNumber");
        assert!(!funcs[0].body.is_empty());
    }

    #[test]
    fn extract_variable_declarations_from_xml_snippet() {
        let xml = r#"
            <localDeclaration name="score" type="WholeNumber" initialValue="0"/>
        "#;
        let vars = extract_variable_declarations(xml);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "score");
        assert_eq!(vars[0].var_type, "WholeNumber");
        assert_eq!(vars[0].initial_value, "0");
    }

    #[test]
    fn extract_method_call_statements() {
        let xml = r#"<methodInvocation object="this.cat" method="walk"/>"#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(
            matches!(&stmts[0], Statement::MethodCall { object, method, .. }
            if object == "this.cat" && method == "walk")
        );
    }

    #[test]
    fn extract_count_loop_statements() {
        let xml = r#"
            <countLoop count="3">
                <methodInvocation object="this.cat" method="walk"/>
            </countLoop>
        "#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(&stmts[0], Statement::CountLoop { count: 3, body } if !body.is_empty()));
    }

    #[test]
    fn extract_event_listener_statements() {
        let xml = r#"
            <eventListener event="SceneActivated">
                <methodInvocation object="this.cat" method="say"/>
            </eventListener>
        "#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(&stmts[0], Statement::EventListener { event, body }
                if event == "SceneActivated" && !body.is_empty()));
    }

    #[test]
    fn extract_collision_listener_statements() {
        let xml = r#"
            <collisionListener objectA="this.cat" objectB="this.dog">
                <methodInvocation object="this.cat" method="say"/>
            </collisionListener>
        "#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(
            matches!(&stmts[0], Statement::CollisionListener { object_a, object_b, body }
                if object_a == "this.cat" && object_b == "this.dog" && !body.is_empty())
        );
    }

    #[test]
    fn extract_function_call_statements() {
        let xml = r#"<functionInvocation name="getGreeting"/>"#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(
            matches!(&stmts[0], Statement::FunctionCall { function_name }
                if function_name == "getGreeting")
        );
    }

    #[test]
    fn extract_return_statements() {
        let xml = r#"<returnStatement value="42"/>"#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(&stmts[0], Statement::ReturnStatement { value } if value == "42"));
    }

    #[test]
    fn extract_variable_assignment_statements() {
        let xml = r#"<assignmentExpression variable="score" value="score + 1"/>"#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(
            matches!(&stmts[0], Statement::VariableAssignment { variable, value }
                if variable == "score" && value == "score + 1")
        );
    }

    #[test]
    fn extract_empty_xml_produces_empty_program() {
        let xml = "";
        assert!(extract_procedures(xml).is_empty());
        assert!(extract_functions(xml).is_empty());
        assert!(extract_variable_declarations(xml).is_empty());
        assert!(extract_statements(xml).is_empty());
    }

    #[test]
    fn extract_multiple_procedures() {
        let xml = r#"
            <procedure name="methodOne">
                <methodInvocation object="this.cat" method="walk"/>
            </procedure>
            <procedure name="methodTwo">
                <methodInvocation object="this.dog" method="run"/>
            </procedure>
        "#;
        let procs = extract_procedures(xml);
        assert_eq!(procs.len(), 2);
        assert_eq!(procs[0].name, "methodOne");
        assert_eq!(procs[1].name, "methodTwo");
    }

    #[test]
    fn extract_if_else_statements() {
        let xml = r#"
            <ifElse condition="this.cat isCloseTo this.dog">
                <methodInvocation object="this.cat" method="say"/>
            </ifElse>
        "#;
        let stmts = extract_statements(xml);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(
            &stmts[0],
            Statement::IfElse {
                condition,
                if_body,
                ..
            } if condition == "this.cat isCloseTo this.dog" && !if_body.is_empty()
        ));
    }
}
