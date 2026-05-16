// Real-Alice desktop execution tests for all grading pipelines.
//
// These tests are gated behind `EATME_REAL_ALICE=1` and require `ALICE_HOME`
// to point at a built Alice 3 checkout containing starter project .a3p files.
//
// When the env gate is not set, all integration tests are #[ignore]'d.
// Parser unit tests (at the bottom) always run.

use eatme_assets::{
    EventsGradingInput, FunctionsGradingInput, GradingReport, LoopsGradingInput,
    ParametersGradingInput, StepStatus, VariablesGradingInput, grade_events_and_collision,
    grade_functions, grade_loops_and_conditionals, grade_parameters, grade_variables,
};
use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement, VariableDeclaration};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Cached regex patterns — compiled once, reused across all parser calls
// ---------------------------------------------------------------------------

struct Patterns {
    procedure: regex::Regex,
    parameter: regex::Regex,
    function: regex::Regex,
    local_decl: regex::Regex,
    count_loop: regex::Regex,
    if_else: regex::Regex,
    event_listener: regex::Regex,
    collision_listener: regex::Regex,
    method_call: regex::Regex,
    func_call: regex::Regex,
    return_stmt: regex::Regex,
    assignment: regex::Regex,
}

static RE: LazyLock<Patterns> = LazyLock::new(|| {
    Patterns {
    procedure: regex::Regex::new(
        r#"<procedure\s+name="([^"]+)"[^>]*>([\s\S]*?)</procedure>"#,
    ).unwrap(),
    parameter: regex::Regex::new(
        r#"<parameter\s+name="([^"]+)"\s+type="([^"]+)""#,
    ).unwrap(),
    function: regex::Regex::new(
        r#"<function\s+name="([^"]+)"[^>]*returnType="([^"]+)"[^>]*>([\s\S]*?)</function>"#,
    ).unwrap(),
    local_decl: regex::Regex::new(
        r#"<localDeclaration\s+name="([^"]+)"\s+type="([^"]+)"[^>]*initialValue="([^"]*)"#,
    ).unwrap(),
    count_loop: regex::Regex::new(
        r#"<countLoop\s+count="(\d+)"[^>]*>([\s\S]*?)</countLoop>"#,
    ).unwrap(),
    if_else: regex::Regex::new(
        r#"<ifElse\s+condition="([^"]+)"[^>]*>([\s\S]*?)</ifElse>"#,
    ).unwrap(),
    event_listener: regex::Regex::new(
        r#"<eventListener\s+event="([^"]+)"[^>]*>([\s\S]*?)</eventListener>"#,
    ).unwrap(),
    collision_listener: regex::Regex::new(
        r#"<collisionListener\s+objectA="([^"]+)"\s+objectB="([^"]+)"[^>]*>([\s\S]*?)</collisionListener>"#,
    ).unwrap(),
    method_call: regex::Regex::new(
        r#"<methodInvocation\s+object="([^"]+)"\s+method="([^"]+)"[^>]*/>"#,
    ).unwrap(),
    func_call: regex::Regex::new(
        r#"<functionInvocation\s+name="([^"]+)"[^>]*/>"#,
    ).unwrap(),
    return_stmt: regex::Regex::new(
        r#"<returnStatement\s+value="([^"]+)"[^>]*/>"#,
    ).unwrap(),
    assignment: regex::Regex::new(
        r#"<assignmentExpression\s+variable="([^"]+)"\s+value="([^"]+)"[^>]*/>"#,
    ).unwrap(),
}
});

// ---------------------------------------------------------------------------
// A3P Parser — regex-based extraction of AST from Alice .a3p ZIP/XML files
// ---------------------------------------------------------------------------

/// Cached starter projects directory — resolved once from ALICE_HOME.
static STARTER_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let alice_home = std::env::var("ALICE_HOME").expect("ALICE_HOME must be set");
    let base = PathBuf::from(alice_home);
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
});

/// Parse a .a3p file into our AST Program representation.
///
/// Opens the ZIP once, tries "project.xml" then falls back to first .xml entry.
fn parse_a3p_program(a3p_path: &Path) -> Option<Program> {
    // Cap decompressed XML at 50 MB — Alice projects are typically < 5 MB.
    const MAX_XML_SIZE: u64 = 50 * 1024 * 1024;

    let file = std::fs::File::open(a3p_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let xml = if let Ok(mut entry) = archive.by_name("project.xml") {
        if entry.size() > MAX_XML_SIZE {
            return None;
        }
        let mut buf = String::with_capacity(entry.size() as usize);
        entry.read_to_string(&mut buf).ok()?;
        buf
    } else {
        let xml_index = (0..archive.len()).find(|&i| {
            archive
                .by_index(i)
                .ok()
                .is_some_and(|e| e.name().ends_with(".xml"))
        })?;
        let mut entry = archive.by_index(xml_index).ok()?;
        if entry.size() > MAX_XML_SIZE {
            return None;
        }
        let mut buf = String::with_capacity(entry.size() as usize);
        entry.read_to_string(&mut buf).ok()?;
        buf
    };

    Some(Program {
        procedures: extract_procedures(&xml),
        functions: extract_functions(&xml),
        variable_declarations: extract_variable_declarations(&xml),
    })
}

/// Extract procedure definitions from Alice XML.
fn extract_procedures(xml: &str) -> Vec<Procedure> {
    RE.procedure
        .captures_iter(xml)
        .map(|cap| {
            let body_xml = &cap[2];
            Procedure {
                name: cap[1].to_string(),
                parameters: RE
                    .parameter
                    .captures_iter(body_xml)
                    .map(|p| Parameter {
                        name: p[1].to_string(),
                        param_type: p[2].to_string(),
                    })
                    .collect(),
                body: extract_statements(body_xml),
            }
        })
        .collect()
}

/// Extract function definitions from Alice XML.
fn extract_functions(xml: &str) -> Vec<Function> {
    RE.function
        .captures_iter(xml)
        .map(|cap| Function {
            name: cap[1].to_string(),
            return_type: cap[2].to_string(),
            body: extract_statements(&cap[3]),
        })
        .collect()
}

/// Extract variable declarations from Alice XML.
fn extract_variable_declarations(xml: &str) -> Vec<VariableDeclaration> {
    RE.local_decl
        .captures_iter(xml)
        .map(|cap| VariableDeclaration {
            name: cap[1].to_string(),
            var_type: cap[2].to_string(),
            initial_value: cap[3].to_string(),
        })
        .collect()
}

/// Extract statements from an XML fragment.
/// Top-level only; nested statements are handled recursively via containers.
fn extract_statements(xml: &str) -> Vec<Statement> {
    let mut statements: Vec<(usize, Statement)> = Vec::new();
    let mut container_ranges: Vec<(usize, usize)> = Vec::new();

    // --- Container elements (record ranges to exclude nested leaves) ---

    for cap in RE.count_loop.captures_iter(xml) {
        let m = cap.get(0).unwrap();
        container_ranges.push((m.start(), m.end()));
        statements.push((
            m.start(),
            Statement::CountLoop {
                count: cap[1].parse().unwrap_or(0),
                body: extract_statements(&cap[2]),
            },
        ));
    }

    for cap in RE.if_else.captures_iter(xml) {
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

    for cap in RE.event_listener.captures_iter(xml) {
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

    for cap in RE.collision_listener.captures_iter(xml) {
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

    container_ranges.sort_unstable();
    let is_inside_container = |pos: usize| -> bool {
        // Binary search: skip ranges starting at/after pos, check remainder in reverse
        let idx = container_ranges.partition_point(|&(start, _)| start < pos);
        container_ranges[..idx]
            .iter()
            .rev()
            .any(|&(_, end)| pos < end)
    };

    // --- Leaf statements (skip if inside a container) ---

    for cap in RE.method_call.captures_iter(xml) {
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

    for cap in RE.func_call.captures_iter(xml) {
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

    for cap in RE.return_stmt.captures_iter(xml) {
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

    for cap in RE.assignment.captures_iter(xml) {
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

    statements.sort_by_key(|(pos, _)| *pos);
    statements.into_iter().map(|(_, s)| s).collect()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn should_run_real_alice() -> bool {
    std::env::var("EATME_REAL_ALICE").is_ok_and(|v| v == "1")
}

fn find_a3p(name: &str) -> PathBuf {
    let path = STARTER_DIR.join(name);
    assert!(
        path.exists(),
        "Starter project not found: {}",
        path.display()
    );
    path
}

/// Parse amazonMinimum.a3p, grade it, and assert structural validity.
fn assert_grading_report_valid(report: &GradingReport, lesson: &str, step_count: usize) {
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, lesson);
    assert_eq!(report.steps.len(), step_count);

    // Preconditions always pass with valid inputs
    for i in 0..3 {
        assert_eq!(report.steps[i].status, StepStatus::Ready);
    }

    // All steps must have a valid status
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

/// Cached parse of amazonMinimum.a3p — parsed once, cloned per test.
static AMAZON_MINIMUM: LazyLock<Program> = LazyLock::new(|| {
    let path = find_a3p("amazonMinimum.a3p");
    parse_a3p_program(&path).expect("Failed to parse amazonMinimum.a3p")
});

fn load_amazon_minimum() -> Program {
    AMAZON_MINIMUM.clone()
}

// ---------------------------------------------------------------------------
// Integration tests — gated behind EATME_REAL_ALICE=1
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn real_alice_a3p_parses_without_error() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    assert!(
        !program.procedures.is_empty(),
        "Parsed program should have at least one procedure"
    );
}

#[test]
#[ignore]
fn real_alice_a3p_round_trip() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let json = serde_json::to_string(&program).unwrap();
    let restored: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, restored, "Program should survive JSON round-trip");
}

#[test]
#[ignore]
fn real_alice_loops_grading() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let report = grade_loops_and_conditionals(LoopsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: Some(program),
    });
    assert_grading_report_valid(&report, "loops-and-conditionals-mini-challenge", 7);
}

#[test]
#[ignore]
fn real_alice_events_grading() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let report = grade_events_and_collision(EventsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: Some(program),
    });
    assert_grading_report_valid(&report, "events-collision-proximity-game", 7);
}

#[test]
#[ignore]
fn real_alice_functions_grading() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let report = grade_functions(FunctionsGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: Some(program),
    });
    assert_grading_report_valid(&report, "functions-mini-challenge", 8);
}

#[test]
#[ignore]
fn real_alice_variables_grading() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let report = grade_variables(VariablesGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: Some(program),
    });
    assert_grading_report_valid(&report, "variables-scorekeeper-timekeeper", 8);
}

#[test]
#[ignore]
fn real_alice_parameters_grading() {
    if !should_run_real_alice() {
        return;
    }
    let program = load_amazon_minimum();
    let report = grade_parameters(ParametersGradingInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required tools available".into(),
        student_program: Some(program),
    });
    assert_grading_report_valid(&report, "parameters-procedure-generalization", 7);
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
