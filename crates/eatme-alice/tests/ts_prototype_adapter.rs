//! TS prototype adapter — runs the student journey silver thread against
//! the alice-web-prototype REST API instead of desktop Alice.
//!
//! Gated behind EATME_TS_PROTOTYPE=1. Requires the TS server to be running.
//! Set ALICE_WEB_URL to override the default http://localhost:3099.

use eatme_assets::{SequencingGradingInput, StepStatus, grade_sequencing};
use eatme_core::ast::{SequenceBlock, SequenceKind};
use serde::Deserialize;
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

fn ts_port_root() -> PathBuf {
    env::var("ALICE_WEB_PROTOTYPE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../alice-web-prototype")
        })
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
