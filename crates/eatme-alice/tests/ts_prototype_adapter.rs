//! TS prototype adapter — runs the student journey silver thread against
//! the alice-web-prototype REST API instead of desktop Alice.
//!
//! Gated behind EATME_TS_PROTOTYPE=1. Requires the TS server to be running.
//! Set ALICE_WEB_URL to override the default http://localhost:3099.

use eatme_assets::{SequencingGradingInput, StepStatus, grade_sequencing};
use eatme_core::ast::{
    ArithmeticOperator, Parameter, Procedure, SequenceBlock, SequenceKind, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

fn ts_enabled() -> bool {
    env::var("EATME_TS_PROTOTYPE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn base_url() -> String {
    env::var("ALICE_WEB_URL").unwrap_or_else(|_| "http://localhost:3099".into())
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    runtime: String,
}

#[derive(Debug, Deserialize)]
struct LaunchResponse {
    status: String,
    #[serde(rename = "sceneObjectCount")]
    scene_object_count: usize,
}

#[derive(Debug, Deserialize)]
struct AddObjectResponse {
    status: String,
    #[serde(rename = "sceneFieldCountAfter")]
    scene_field_count_after: usize,
}

#[derive(Debug, Deserialize)]
struct EditResponse {
    status: String,
    #[serde(rename = "evidenceArtifact")]
    evidence_artifact: String,
}

#[derive(Debug, Deserialize)]
struct RunResponse {
    status: String,
    #[serde(rename = "scene_object_count")]
    scene_object_count: usize,
}

#[derive(Debug, Deserialize)]
struct SaveResponse {
    status: String,
}

fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
}

#[derive(Debug, Deserialize)]
struct TsRoundTrip {
    source: String,
    ast: TsClassDecl,
}

#[derive(Debug, Deserialize)]
struct TsClassDecl {
    methods: Vec<TsMethodDecl>,
}

#[derive(Debug, Deserialize)]
struct TsMethodDecl {
    name: String,
    body: Vec<TsStatement>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum TsStatement {
    DoInOrder { body: Vec<TsStatement> },
    DoTogether { body: Vec<TsStatement> },
    ExpressionStatement { expression: TsExpression },
    Comment { text: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum TsExpression {
    Identifier {
        name: String,
    },
    This,
    MemberAccess {
        target: Box<TsExpression>,
        #[serde(rename = "memberName")]
        member_name: String,
    },
    MethodInvocation {
        target: Option<Box<TsExpression>>,
        #[serde(rename = "methodName")]
        method_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TsPortProcedure {
    name: String,
    parameters: Vec<TsPortParameter>,
    body: Vec<TsPortStatement>,
}

impl TsPortProcedure {
    fn from_ast(procedure: &Procedure) -> Self {
        Self {
            name: procedure.name.clone(),
            parameters: procedure
                .parameters
                .iter()
                .map(TsPortParameter::from_ast)
                .collect(),
            body: procedure
                .body
                .iter()
                .map(TsPortStatement::from_ast)
                .collect(),
        }
    }

    fn to_ast(&self) -> Procedure {
        Procedure {
            name: self.name.clone(),
            parameters: self
                .parameters
                .iter()
                .map(TsPortParameter::to_ast)
                .collect(),
            body: self.body.iter().map(TsPortStatement::to_ast).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TsPortParameter {
    name: String,
    #[serde(rename = "paramType")]
    param_type: String,
}

impl TsPortParameter {
    fn from_ast(parameter: &Parameter) -> Self {
        Self {
            name: parameter.name.clone(),
            param_type: parameter.param_type.clone(),
        }
    }

    fn to_ast(&self) -> Parameter {
        Parameter {
            name: self.name.clone(),
            param_type: self.param_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TsPortArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl TsPortArithmeticOperator {
    fn from_ast(operator: ArithmeticOperator) -> Self {
        match operator {
            ArithmeticOperator::Add => Self::Add,
            ArithmeticOperator::Subtract => Self::Subtract,
            ArithmeticOperator::Multiply => Self::Multiply,
            ArithmeticOperator::Divide => Self::Divide,
        }
    }

    fn to_ast(self) -> ArithmeticOperator {
        match self {
            Self::Add => ArithmeticOperator::Add,
            Self::Subtract => ArithmeticOperator::Subtract,
            Self::Multiply => ArithmeticOperator::Multiply,
            Self::Divide => ArithmeticOperator::Divide,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TsPortStatement {
    MethodCall {
        object: String,
        method: String,
        arguments: Vec<String>,
    },
    CountLoop {
        count: u32,
        body: Vec<TsPortStatement>,
    },
    IfElse {
        condition: String,
        #[serde(rename = "ifBody")]
        if_body: Vec<TsPortStatement>,
        #[serde(rename = "elseBody")]
        else_body: Vec<TsPortStatement>,
    },
    EventListener {
        event: String,
        body: Vec<TsPortStatement>,
    },
    CollisionListener {
        #[serde(rename = "objectA")]
        object_a: String,
        #[serde(rename = "objectB")]
        object_b: String,
        body: Vec<TsPortStatement>,
    },
    DoInOrder {
        body: Vec<TsPortStatement>,
    },
    ReturnStatement {
        expression: String,
    },
    FunctionCall {
        object: String,
        function: String,
        arguments: Vec<String>,
    },
    VariableDeclaration {
        name: String,
        #[serde(rename = "varType")]
        var_type: String,
        #[serde(rename = "initialValue")]
        initial_value: String,
    },
    VariableAssignment {
        name: String,
        value: String,
    },
    ArrayDeclaration {
        name: String,
        #[serde(rename = "elementType")]
        element_type: String,
        elements: Vec<String>,
    },
    ArrayAccess {
        array: String,
        index: String,
        target: String,
    },
    ForEachArray {
        #[serde(rename = "itemName")]
        item_name: String,
        array: String,
        body: Vec<TsPortStatement>,
    },
    ArithmeticExpression {
        operator: TsPortArithmeticOperator,
        left: String,
        right: String,
        result: String,
    },
    Comment {
        text: String,
    },
    UserTypeDeclaration {
        name: String,
        extends: Option<String>,
        methods: Vec<TsPortProcedure>,
    },
}

impl TsPortStatement {
    fn from_ast(statement: &Statement) -> Self {
        match statement {
            Statement::MethodCall {
                object,
                method,
                arguments,
            } => Self::MethodCall {
                object: object.clone(),
                method: method.clone(),
                arguments: arguments.clone(),
            },
            Statement::CountLoop { count, body } => Self::CountLoop {
                count: *count,
                body: body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => Self::IfElse {
                condition: condition.clone(),
                if_body: if_body.iter().map(TsPortStatement::from_ast).collect(),
                else_body: else_body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::EventListener { event, body } => Self::EventListener {
                event: event.clone(),
                body: body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::CollisionListener {
                object_a,
                object_b,
                body,
            } => Self::CollisionListener {
                object_a: object_a.clone(),
                object_b: object_b.clone(),
                body: body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::DoInOrder { body } => Self::DoInOrder {
                body: body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::ReturnStatement { expression } => Self::ReturnStatement {
                expression: expression.clone(),
            },
            Statement::FunctionCall {
                object,
                function,
                arguments,
            } => Self::FunctionCall {
                object: object.clone(),
                function: function.clone(),
                arguments: arguments.clone(),
            },
            Statement::VariableDeclaration {
                name,
                var_type,
                initial_value,
            } => Self::VariableDeclaration {
                name: name.clone(),
                var_type: var_type.clone(),
                initial_value: initial_value.clone(),
            },
            Statement::VariableAssignment { name, value } => Self::VariableAssignment {
                name: name.clone(),
                value: value.clone(),
            },
            Statement::ArrayDeclaration {
                name,
                element_type,
                elements,
            } => Self::ArrayDeclaration {
                name: name.clone(),
                element_type: element_type.clone(),
                elements: elements.clone(),
            },
            Statement::ArrayAccess {
                array,
                index,
                target,
            } => Self::ArrayAccess {
                array: array.clone(),
                index: index.clone(),
                target: target.clone(),
            },
            Statement::ForEachArray {
                item_name,
                array,
                body,
            } => Self::ForEachArray {
                item_name: item_name.clone(),
                array: array.clone(),
                body: body.iter().map(TsPortStatement::from_ast).collect(),
            },
            Statement::ArithmeticExpression {
                operator,
                left,
                right,
                result,
            } => Self::ArithmeticExpression {
                operator: TsPortArithmeticOperator::from_ast(*operator),
                left: left.clone(),
                right: right.clone(),
                result: result.clone(),
            },
            Statement::Comment { text } => Self::Comment { text: text.clone() },
            Statement::UserTypeDeclaration {
                name,
                extends,
                methods,
            } => Self::UserTypeDeclaration {
                name: name.clone(),
                extends: extends.clone(),
                methods: methods.iter().map(TsPortProcedure::from_ast).collect(),
            },
        }
    }

    fn to_ast(&self) -> Statement {
        match self {
            Self::MethodCall {
                object,
                method,
                arguments,
            } => Statement::MethodCall {
                object: object.clone(),
                method: method.clone(),
                arguments: arguments.clone(),
            },
            Self::CountLoop { count, body } => Statement::CountLoop {
                count: *count,
                body: body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::IfElse {
                condition,
                if_body,
                else_body,
            } => Statement::IfElse {
                condition: condition.clone(),
                if_body: if_body.iter().map(TsPortStatement::to_ast).collect(),
                else_body: else_body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::EventListener { event, body } => Statement::EventListener {
                event: event.clone(),
                body: body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::CollisionListener {
                object_a,
                object_b,
                body,
            } => Statement::CollisionListener {
                object_a: object_a.clone(),
                object_b: object_b.clone(),
                body: body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::DoInOrder { body } => Statement::DoInOrder {
                body: body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::ReturnStatement { expression } => Statement::ReturnStatement {
                expression: expression.clone(),
            },
            Self::FunctionCall {
                object,
                function,
                arguments,
            } => Statement::FunctionCall {
                object: object.clone(),
                function: function.clone(),
                arguments: arguments.clone(),
            },
            Self::VariableDeclaration {
                name,
                var_type,
                initial_value,
            } => Statement::VariableDeclaration {
                name: name.clone(),
                var_type: var_type.clone(),
                initial_value: initial_value.clone(),
            },
            Self::VariableAssignment { name, value } => Statement::VariableAssignment {
                name: name.clone(),
                value: value.clone(),
            },
            Self::ArrayDeclaration {
                name,
                element_type,
                elements,
            } => Statement::ArrayDeclaration {
                name: name.clone(),
                element_type: element_type.clone(),
                elements: elements.clone(),
            },
            Self::ArrayAccess {
                array,
                index,
                target,
            } => Statement::ArrayAccess {
                array: array.clone(),
                index: index.clone(),
                target: target.clone(),
            },
            Self::ForEachArray {
                item_name,
                array,
                body,
            } => Statement::ForEachArray {
                item_name: item_name.clone(),
                array: array.clone(),
                body: body.iter().map(TsPortStatement::to_ast).collect(),
            },
            Self::ArithmeticExpression {
                operator,
                left,
                right,
                result,
            } => Statement::ArithmeticExpression {
                operator: operator.to_ast(),
                left: left.clone(),
                right: right.clone(),
                result: result.clone(),
            },
            Self::Comment { text } => Statement::Comment { text: text.clone() },
            Self::UserTypeDeclaration {
                name,
                extends,
                methods,
            } => Statement::UserTypeDeclaration {
                name: name.clone(),
                extends: extends.clone(),
                methods: methods.iter().map(TsPortProcedure::to_ast).collect(),
            },
        }
    }
}

fn ts_port_root() -> PathBuf {
    if let Ok(root) = env::var("ALICE_WEB_PROTOTYPE_ROOT") {
        return PathBuf::from(root);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        let candidate = ancestor.join("alice-web-prototype");
        if candidate.join("package.json").exists() {
            return candidate;
        }
    }

    manifest_dir.join("../../../alice-web-prototype")
}

fn ensure_ts_port_server_build() {
    let root = ts_port_root();
    if root.join("dist-server/code-generation.js").exists()
        && root.join("dist-server/tweedle-parser.js").exists()
    {
        return;
    }

    let status = Command::new("npm")
        .arg("run")
        .arg("build:server")
        .current_dir(&root)
        .status()
        .expect("failed to build alice-web-prototype server artifacts");
    assert!(status.success(), "npm run build:server failed");
}

fn run_ts_round_trip(mode: &str) -> TsRoundTrip {
    ensure_ts_port_server_build();
    let root = ts_port_root();
    let script = r#"
import { pathToFileURL } from 'node:url';

const { createTweedleSource } = await import(pathToFileURL(process.env.TS_CODEGEN).href);
const { parseTweedle } = await import(pathToFileURL(process.env.TS_PARSER).href);

const body = process.env.TS_SEQUENCE_MODE === 'missing-parallel'
  ? [
      'doInOrder {',
      '  bunny.hop();',
      '  bunny.turn();',
      '}',
    ]
  : [
      'doInOrder {',
      '  bunny.hop();',
      '  bunny.turn();',
      '}',
      'doTogether {',
      '  bunny.jump();',
      '  bunny.say("done");',
      '}',
    ];

const source = createTweedleSource('Runner', [{
  name: 'myFirstMethod',
  body,
}]);
const ast = parseTweedle(source);
console.log(JSON.stringify({ source, ast }));
"#;

    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .env("TS_CODEGEN", root.join("dist-server/code-generation.js"))
        .env("TS_PARSER", root.join("dist-server/tweedle-parser.js"))
        .env("TS_SEQUENCE_MODE", mode)
        .output()
        .expect("failed to execute TS round-trip script");

    assert!(
        output.status.success(),
        "node round-trip failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("invalid TS round-trip JSON")
}

fn sequence_blocks_from_round_trip(round_trip: &TsRoundTrip) -> Vec<SequenceBlock> {
    let method = round_trip
        .ast
        .methods
        .iter()
        .find(|method| method.name == "myFirstMethod")
        .expect("expected myFirstMethod in parsed AST");

    method
        .body
        .iter()
        .filter_map(sequence_block_from_statement)
        .collect()
}

fn sequence_block_from_statement(statement: &TsStatement) -> Option<SequenceBlock> {
    match statement {
        TsStatement::DoInOrder { body } => Some(SequenceBlock {
            kind: SequenceKind::DoInOrder,
            steps: body.iter().filter_map(sequence_step_label).collect(),
        }),
        TsStatement::DoTogether { body } => Some(SequenceBlock {
            kind: SequenceKind::DoTogether,
            steps: body.iter().filter_map(sequence_step_label).collect(),
        }),
        _ => None,
    }
}

fn sequence_step_label(statement: &TsStatement) -> Option<String> {
    match statement {
        TsStatement::ExpressionStatement { expression } => {
            Some(render_expression_label(expression))
        }
        TsStatement::Comment { text } => Some(format!("// {text}")),
        _ => None,
    }
}

fn render_expression_label(expression: &TsExpression) -> String {
    match expression {
        TsExpression::Identifier { name } => name.clone(),
        TsExpression::This => "this".into(),
        TsExpression::MemberAccess {
            target,
            member_name,
        } => {
            format!("{}.{}", render_expression_label(target), member_name)
        }
        TsExpression::MethodInvocation {
            target,
            method_name,
        } => match target {
            Some(target) => format!("{}.{}", render_expression_label(target), method_name),
            None => method_name.clone(),
        },
    }
}

fn all_ready_input(sequence_blocks: Option<Vec<SequenceBlock>>) -> SequencingGradingInput {
    SequencingGradingInput {
        assets_valid: true,
        asset_reason: "TS round-trip succeeded".into(),
        deps_available: true,
        deps_reason: "TS parser + eatme grading available".into(),
        sequence_blocks,
    }
}

const ALL_STATEMENT_KINDS: [&str; 16] = [
    "MethodCall",
    "CountLoop",
    "IfElse",
    "EventListener",
    "CollisionListener",
    "DoInOrder",
    "ReturnStatement",
    "FunctionCall",
    "VariableDeclaration",
    "VariableAssignment",
    "ArrayDeclaration",
    "ArrayAccess",
    "ForEachArray",
    "ArithmeticExpression",
    "Comment",
    "UserTypeDeclaration",
];

fn statement_kind(statement: &Statement) -> &'static str {
    match statement {
        Statement::MethodCall { .. } => "MethodCall",
        Statement::CountLoop { .. } => "CountLoop",
        Statement::IfElse { .. } => "IfElse",
        Statement::EventListener { .. } => "EventListener",
        Statement::CollisionListener { .. } => "CollisionListener",
        Statement::DoInOrder { .. } => "DoInOrder",
        Statement::ReturnStatement { .. } => "ReturnStatement",
        Statement::FunctionCall { .. } => "FunctionCall",
        Statement::VariableDeclaration { .. } => "VariableDeclaration",
        Statement::VariableAssignment { .. } => "VariableAssignment",
        Statement::ArrayDeclaration { .. } => "ArrayDeclaration",
        Statement::ArrayAccess { .. } => "ArrayAccess",
        Statement::ForEachArray { .. } => "ForEachArray",
        Statement::ArithmeticExpression { .. } => "ArithmeticExpression",
        Statement::Comment { .. } => "Comment",
        Statement::UserTypeDeclaration { .. } => "UserTypeDeclaration",
    }
}

fn assert_statement_kinds_covered(statements: &[Statement]) {
    let actual = statements
        .iter()
        .map(statement_kind)
        .collect::<BTreeSet<_>>();
    let expected = ALL_STATEMENT_KINDS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "expected all statement kinds to be covered"
    );
}

fn ts_port_statement_cases() -> Vec<(Statement, Value)> {
    vec![
        (
            Statement::MethodCall {
                object: "bunny".into(),
                method: "hop".into(),
                arguments: vec!["forward".into(), "1".into()],
            },
            serde_json::json!({
                "type": "MethodCall",
                "object": "bunny",
                "method": "hop",
                "arguments": ["forward", "1"]
            }),
        ),
        (
            Statement::CountLoop {
                count: 3,
                body: vec![
                    Statement::Comment { text: "lap".into() },
                    Statement::MethodCall {
                        object: "bunny".into(),
                        method: "hop".into(),
                        arguments: vec!["1".into()],
                    },
                ],
            },
            serde_json::json!({
                "type": "CountLoop",
                "count": 3,
                "body": [
                    {"type": "Comment", "text": "lap"},
                    {
                        "type": "MethodCall",
                        "object": "bunny",
                        "method": "hop",
                        "arguments": ["1"]
                    }
                ]
            }),
        ),
        (
            Statement::IfElse {
                condition: "score > 10".into(),
                if_body: vec![Statement::VariableAssignment {
                    name: "score".into(),
                    value: "score + 1".into(),
                }],
                else_body: vec![Statement::Comment {
                    text: "too low".into(),
                }],
            },
            serde_json::json!({
                "type": "IfElse",
                "condition": "score > 10",
                "ifBody": [
                    {
                        "type": "VariableAssignment",
                        "name": "score",
                        "value": "score + 1"
                    }
                ],
                "elseBody": [
                    {"type": "Comment", "text": "too low"}
                ]
            }),
        ),
        (
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: "camera".into(),
                    method: "pointAt".into(),
                    arguments: vec!["bunny".into()],
                }],
            },
            serde_json::json!({
                "type": "EventListener",
                "event": "SceneActivated",
                "body": [
                    {
                        "type": "MethodCall",
                        "object": "camera",
                        "method": "pointAt",
                        "arguments": ["bunny"]
                    }
                ]
            }),
        ),
        (
            Statement::CollisionListener {
                object_a: "bunny".into(),
                object_b: "fox".into(),
                body: vec![Statement::DoInOrder {
                    body: vec![Statement::MethodCall {
                        object: "bunny".into(),
                        method: "say".into(),
                        arguments: vec!["ouch".into()],
                    }],
                }],
            },
            serde_json::json!({
                "type": "CollisionListener",
                "objectA": "bunny",
                "objectB": "fox",
                "body": [
                    {
                        "type": "DoInOrder",
                        "body": [
                            {
                                "type": "MethodCall",
                                "object": "bunny",
                                "method": "say",
                                "arguments": ["ouch"]
                            }
                        ]
                    }
                ]
            }),
        ),
        (
            Statement::DoInOrder {
                body: vec![
                    Statement::MethodCall {
                        object: "bunny".into(),
                        method: "turn".into(),
                        arguments: vec!["left".into()],
                    },
                    Statement::Comment {
                        text: "keep going".into(),
                    },
                ],
            },
            serde_json::json!({
                "type": "DoInOrder",
                "body": [
                    {
                        "type": "MethodCall",
                        "object": "bunny",
                        "method": "turn",
                        "arguments": ["left"]
                    },
                    {"type": "Comment", "text": "keep going"}
                ]
            }),
        ),
        (
            Statement::ReturnStatement {
                expression: "score".into(),
            },
            serde_json::json!({
                "type": "ReturnStatement",
                "expression": "score"
            }),
        ),
        (
            Statement::FunctionCall {
                object: "math".into(),
                function: "random".into(),
                arguments: vec!["1".into(), "6".into()],
            },
            serde_json::json!({
                "type": "FunctionCall",
                "object": "math",
                "function": "random",
                "arguments": ["1", "6"]
            }),
        ),
        (
            Statement::VariableDeclaration {
                name: "score".into(),
                var_type: "WholeNumber".into(),
                initial_value: "0".into(),
            },
            serde_json::json!({
                "type": "VariableDeclaration",
                "name": "score",
                "varType": "WholeNumber",
                "initialValue": "0"
            }),
        ),
        (
            Statement::VariableAssignment {
                name: "score".into(),
                value: "score + 10".into(),
            },
            serde_json::json!({
                "type": "VariableAssignment",
                "name": "score",
                "value": "score + 10"
            }),
        ),
        (
            Statement::ArrayDeclaration {
                name: "pets".into(),
                element_type: "Biped".into(),
                elements: vec!["this.cat".into(), "this.dog".into(), "this.bunny".into()],
            },
            serde_json::json!({
                "type": "ArrayDeclaration",
                "name": "pets",
                "elementType": "Biped",
                "elements": ["this.cat", "this.dog", "this.bunny"]
            }),
        ),
        (
            Statement::ArrayAccess {
                array: "pets".into(),
                index: "0".into(),
                target: "leader".into(),
            },
            serde_json::json!({
                "type": "ArrayAccess",
                "array": "pets",
                "index": "0",
                "target": "leader"
            }),
        ),
        (
            Statement::ForEachArray {
                item_name: "pet".into(),
                array: "pets".into(),
                body: vec![Statement::MethodCall {
                    object: "pet".into(),
                    method: "jump".into(),
                    arguments: vec![],
                }],
            },
            serde_json::json!({
                "type": "ForEachArray",
                "itemName": "pet",
                "array": "pets",
                "body": [
                    {
                        "type": "MethodCall",
                        "object": "pet",
                        "method": "jump",
                        "arguments": []
                    }
                ]
            }),
        ),
        (
            Statement::ArithmeticExpression {
                operator: ArithmeticOperator::Multiply,
                left: "score".into(),
                right: "2".into(),
                result: "doubleScore".into(),
            },
            serde_json::json!({
                "type": "ArithmeticExpression",
                "operator": "Multiply",
                "left": "score",
                "right": "2",
                "result": "doubleScore"
            }),
        ),
        (
            Statement::Comment {
                text: "adapter coverage".into(),
            },
            serde_json::json!({
                "type": "Comment",
                "text": "adapter coverage"
            }),
        ),
        (
            Statement::UserTypeDeclaration {
                name: "HelperBunny".into(),
                extends: Some("Bunny".into()),
                methods: vec![Procedure {
                    name: "wave".into(),
                    parameters: vec![Parameter {
                        name: "times".into(),
                        param_type: "WholeNumber".into(),
                    }],
                    body: vec![
                        Statement::MethodCall {
                            object: "this".into(),
                            method: "wave".into(),
                            arguments: vec!["times".into()],
                        },
                        Statement::Comment {
                            text: "method body".into(),
                        },
                    ],
                }],
            },
            serde_json::json!({
                "type": "UserTypeDeclaration",
                "name": "HelperBunny",
                "extends": "Bunny",
                "methods": [
                    {
                        "name": "wave",
                        "parameters": [
                            {"name": "times", "paramType": "WholeNumber"}
                        ],
                        "body": [
                            {
                                "type": "MethodCall",
                                "object": "this",
                                "method": "wave",
                                "arguments": ["times"]
                            },
                            {"type": "Comment", "text": "method body"}
                        ]
                    }
                ]
            }),
        ),
    ]
}

#[test]
fn eatme_ast_statements_serialize_to_expected_ts_port_shape() {
    let cases = ts_port_statement_cases();
    let statements = cases
        .iter()
        .map(|(statement, _)| statement.clone())
        .collect::<Vec<_>>();
    assert_statement_kinds_covered(&statements);

    for (statement, expected_json) in cases {
        let actual_json = serde_json::to_value(TsPortStatement::from_ast(&statement)).unwrap();
        assert_eq!(
            actual_json,
            expected_json,
            "serialized TS port shape mismatch for {}",
            statement_kind(&statement)
        );
    }
}

#[test]
fn ts_port_statements_deserialize_to_expected_eatme_ast_shape() {
    let cases = ts_port_statement_cases();
    let statements = cases
        .iter()
        .map(|(statement, _)| statement.clone())
        .collect::<Vec<_>>();
    assert_statement_kinds_covered(&statements);

    for (expected_statement, ts_json) in cases {
        let actual_statement = serde_json::from_value::<TsPortStatement>(ts_json)
            .unwrap()
            .to_ast();
        assert_eq!(
            actual_statement,
            expected_statement,
            "deserialized eatme AST mismatch for {}",
            statement_kind(&expected_statement)
        );
    }
}

#[test]
fn ts_port_statement_round_trip_preserves_all_statement_types() {
    let statements = ts_port_statement_cases()
        .into_iter()
        .map(|(statement, _)| statement)
        .collect::<Vec<_>>();
    assert_statement_kinds_covered(&statements);

    let restored = statements
        .iter()
        .map(|statement| TsPortStatement::from_ast(statement).to_ast())
        .collect::<Vec<_>>();
    assert_eq!(restored, statements);
}

#[test]
fn ts_port_arithmetic_operator_round_trips_all_variants() {
    for operator in [
        ArithmeticOperator::Add,
        ArithmeticOperator::Subtract,
        ArithmeticOperator::Multiply,
        ArithmeticOperator::Divide,
    ] {
        let ts_operator = TsPortArithmeticOperator::from_ast(operator);
        let restored = serde_json::from_value::<TsPortArithmeticOperator>(
            serde_json::to_value(ts_operator).unwrap(),
        )
        .unwrap()
        .to_ast();
        assert_eq!(restored, operator);
    }
}

#[test]
fn ts_port_round_trip_grades_complete_sequence_program() {
    let round_trip = run_ts_round_trip("complete");
    assert!(round_trip.source.contains("doInOrder"));
    assert!(round_trip.source.contains("doTogether"));

    let sequence_blocks = sequence_blocks_from_round_trip(&round_trip);
    assert_eq!(sequence_blocks.len(), 2);
    assert_eq!(sequence_blocks[0].kind, SequenceKind::DoInOrder);
    assert_eq!(sequence_blocks[0].steps, vec!["bunny.hop", "bunny.turn"]);
    assert_eq!(sequence_blocks[1].kind, SequenceKind::DoTogether);
    assert_eq!(sequence_blocks[1].steps, vec!["bunny.jump", "bunny.say"]);

    let report = grade_sequencing(all_ready_input(Some(sequence_blocks)));
    assert!(report.passed);
    assert_eq!(
        report.lesson,
        "procedure-sequencing-do-in-order-do-together"
    );
    for step in &report.steps {
        assert_eq!(step.status, StepStatus::Ready, "step '{}'", step.name);
    }
}

#[test]
fn ts_port_round_trip_blocks_when_parallel_sequence_is_missing() {
    let round_trip = run_ts_round_trip("missing-parallel");
    assert!(round_trip.source.contains("doInOrder"));
    assert!(!round_trip.source.contains("doTogether"));

    let report = grade_sequencing(all_ready_input(Some(sequence_blocks_from_round_trip(
        &round_trip,
    ))));
    assert!(!report.passed);

    let do_together = report
        .steps
        .iter()
        .find(|step| step.name == "use-do-together")
        .expect("missing use-do-together step");
    assert_eq!(do_together.status, StepStatus::Blocked);

    let combined = report
        .steps
        .iter()
        .find(|step| step.name == "combine-sequential-and-parallel-actions")
        .expect("missing combine-sequential-and-parallel-actions step");
    assert_eq!(combined.status, StepStatus::Blocked);
}

#[test]
fn ts_prototype_silver_thread_journey() {
    if !ts_enabled() {
        eprintln!("skipping TS prototype test (set EATME_TS_PROTOTYPE=1)");
        return;
    }
    let base = base_url();
    let client = http_client();

    // Step 1: Health check
    let health: HealthResponse = client
        .get(&format!("{base}/api/health"))
        .call()
        .expect("health check failed")
        .into_json()
        .expect("invalid health JSON");
    assert_eq!(health.status, "running");
    assert_eq!(health.runtime, "typescript-web-prototype");

    // Step 2: Launch
    let launch: LaunchResponse = client
        .post(&format!("{base}/api/launch"))
        .send_json(ureq::json!({}))
        .expect("launch failed")
        .into_json()
        .expect("invalid launch JSON");
    assert_eq!(launch.status, "launched");
    assert!(
        launch.scene_object_count >= 2,
        "expected >= 2 scene objects"
    );

    // Step 3: Add object
    let add: AddObjectResponse = client
        .post(&format!("{base}/api/scene/add-object"))
        .send_json(ureq::json!({
            "className": "org.lgna.story.SBiped",
            "name": "bunny"
        }))
        .expect("add object failed")
        .into_json()
        .expect("invalid add JSON");
    assert_eq!(add.status, "added");
    assert!(add.scene_field_count_after > 0);

    // Step 4: Edit procedure
    let edit: EditResponse = client
        .post(&format!("{base}/api/code/edit-procedure"))
        .send_json(ureq::json!({
            "procedureSelector": "scene.myFirstMethod",
            "editSpec": "append-comment:eatme TS adapter proof"
        }))
        .expect("edit failed")
        .into_json()
        .expect("invalid edit JSON");
    assert_eq!(edit.status, "proved");
    assert!(!edit.evidence_artifact.is_empty());

    // Step 5: Run world
    let run: RunResponse = client
        .post(&format!("{base}/api/world/run"))
        .send_json(ureq::json!({}))
        .expect("run failed")
        .into_json()
        .expect("invalid run JSON");
    assert_eq!(run.status, "completed");
    assert!(run.scene_object_count > 0);

    // Step 6: Save project
    let save: SaveResponse = client
        .post(&format!("{base}/api/project/save"))
        .send_json(ureq::json!({}))
        .expect("save failed")
        .into_json()
        .expect("invalid save JSON");
    assert_eq!(save.status, "saved");

    eprintln!("TS prototype silver thread: all 6 steps passed");
}
