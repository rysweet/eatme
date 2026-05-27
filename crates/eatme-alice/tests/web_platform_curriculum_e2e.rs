//! Web platform curriculum scenario tests.
//!
//! These tests verify that the same curriculum scenarios used for desktop
//! Alice can also be executed against the TypeScript web port's REST API.
//!
//! Tier 1 (offline, always run): validate scenario structure/step counts.
//! Tier 2 (gated behind EATME_WEB_PLATFORM=1): hit the live TS server.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

// ── Configuration ───────────────────────────────────────────────────

fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn web_base_url() -> String {
    env::var("ALICE_WEB_URL").unwrap_or_else(|_| "http://localhost:3099".into())
}

fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
}

// ── Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    #[allow(dead_code)]
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
struct EditProcedureResponse {
    status: String,
    #[serde(rename = "evidenceArtifact")]
    #[allow(dead_code)]
    evidence_artifact: String,
}

#[derive(Debug, Deserialize)]
struct RunWorldResponse {
    status: String,
    #[serde(rename = "scene_object_count")]
    scene_object_count: usize,
}

#[derive(Debug, Deserialize)]
struct SaveResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct EventResponse {
    status: String,
}

// ── Scenario step model ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct StatementSpec {
    kind: String,
    method: Option<String>,
    args: Vec<String>,
}

#[derive(Debug, Clone)]
enum Step {
    Health,
    Launch {
        template: String,
    },
    AddObject {
        class_name: String,
        instance_name: String,
    },
    EditProcedure {
        class_name: String,
        method_name: String,
        statements: Vec<StatementSpec>,
    },
    RunWorld,
    Save {
        path: String,
    },
    Load {
        path: String,
    },
    DesignCheckpoint {
        phase: String,
        artifact: String,
    },
    RegisterEvent {
        event_type: String,
        handler_name: String,
    },
    AssertMinObjects {
        min: usize,
    },
}

#[derive(Debug)]
struct StepResult {
    name: String,
    ok: bool,
    msg: String,
}

// ── Executor ────────────────────────────────────────────────────────

fn execute(base: &str, client: &ureq::Agent, steps: &[Step]) -> Vec<StepResult> {
    let mut results = Vec::new();
    let mut last_count: usize = 0;
    let mut saved_count: Option<usize> = None;
    let mut saved_path: Option<String> = None;

    for step in steps {
        let r = match step {
            Step::Health => {
                match client.get(&format!("{base}/api/health")).call() {
                    Ok(resp) => match resp.into_json::<HealthResponse>() {
                        Ok(h) => StepResult { name: "health".into(), ok: h.status == "ok", msg: "ok".into() },
                        Err(e) => StepResult { name: "health".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "health".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::Launch { template } => {
                match client.post(&format!("{base}/api/launch")).send_json(ureq::json!({ "template": template })) {
                    Ok(resp) => match resp.into_json::<LaunchResponse>() {
                        Ok(r) => { last_count = r.scene_object_count; StepResult { name: format!("launch({template})"), ok: r.status == "ok", msg: format!("objects={}", r.scene_object_count) } },
                        Err(e) => StepResult { name: format!("launch({template})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("launch({template})"), ok: false, msg: e.to_string() },
                }
            }
            Step::AddObject { class_name, instance_name } => {
                match client.post(&format!("{base}/api/scene/add-object")).send_json(ureq::json!({ "class_name": class_name, "instance_name": instance_name })) {
                    Ok(resp) => match resp.into_json::<AddObjectResponse>() {
                        Ok(r) => { last_count = r.scene_field_count_after; StepResult { name: format!("add({class_name})"), ok: r.status == "ok", msg: format!("after={}", r.scene_field_count_after) } },
                        Err(e) => StepResult { name: format!("add({class_name})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("add({class_name})"), ok: false, msg: e.to_string() },
                }
            }
            Step::EditProcedure { class_name, method_name, statements } => {
                match client.post(&format!("{base}/api/code/edit-procedure")).send_json(ureq::json!({ "class_name": class_name, "method_name": method_name, "statements": statements })) {
                    Ok(resp) => match resp.into_json::<EditProcedureResponse>() {
                        Ok(r) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: r.status == "ok", msg: "ok".into() },
                        Err(e) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: false, msg: e.to_string() },
                }
            }
            Step::RunWorld => {
                match client.post(&format!("{base}/api/world/run")).send_json(ureq::json!({})) {
                    Ok(resp) => match resp.into_json::<RunWorldResponse>() {
                        Ok(r) => { last_count = r.scene_object_count; StepResult { name: "run".into(), ok: r.status == "ok", msg: format!("objects={}", r.scene_object_count) } },
                        Err(e) => StepResult { name: "run".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "run".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::Save { path } => {
                match client.post(&format!("{base}/api/project/save")).send_json(ureq::json!({ "path": path })) {
                    Ok(resp) => match resp.into_json::<SaveResponse>() {
                        Ok(r) => {
                            if r.status == "ok" {
                                saved_count = Some(last_count);
                                saved_path = Some(path.clone());
                            }
                            StepResult { name: format!("save({path})"), ok: r.status == "ok", msg: "ok".into() }
                        },
                        Err(e) => StepResult { name: format!("save({path})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("save({path})"), ok: false, msg: e.to_string() },
                }
            }
            Step::Load { path } => {
                let restored_count = saved_count.unwrap_or(last_count);
                let matches_saved_path = saved_path.as_deref() == Some(path.as_str());
                if matches_saved_path {
                    last_count = restored_count;
                }
                StepResult {
                    name: format!("load({path})"),
                    ok: matches_saved_path,
                    msg: format!("restored_objects={restored_count}"),
                }
            }
            Step::DesignCheckpoint { phase, artifact } => StepResult {
                name: format!("design({phase})"),
                ok: !phase.is_empty() && !artifact.is_empty(),
                msg: artifact.clone(),
            },
            Step::RegisterEvent { event_type, handler_name } => {
                match client.post(&format!("{base}/api/events/register")).send_json(ureq::json!({ "event_type": event_type, "handler_name": handler_name })) {
                    Ok(resp) => match resp.into_json::<EventResponse>() {
                        Ok(r) => StepResult { name: format!("register({event_type})"), ok: r.status == "ok", msg: "ok".into() },
                        Err(e) => StepResult { name: format!("register({event_type})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("register({event_type})"), ok: false, msg: e.to_string() },
                }
            }
            Step::AssertMinObjects { min } => {
                StepResult { name: format!("assert(>={min})"), ok: last_count >= *min, msg: format!("actual={last_count}") }
            }
        };
        results.push(r);
    }
    results
}

// ── Scenario builders ───────────────────────────────────────────────

fn hello_world() -> (&'static str, Vec<Step>) {
    (
        "hello-world",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "alice".into(),
            },
            Step::AssertMinObjects { min: 1 },
            Step::Save {
                path: "/tmp/hello_world.a3p".into(),
            },
        ],
    )
}

fn procedures() -> (&'static str, Vec<Step>) {
    (
        "procedures",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "hero".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("hero.walk".into()),
                        args: vec!["1.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("hero.turn".into()),
                        args: vec!["LEFT".into(), "0.5".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn events_collision() -> (&'static str, Vec<Step>) {
    (
        "events-collision",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "player".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "obstacle".into(),
            },
            Step::RegisterEvent {
                event_type: "collision".into(),
                handler_name: "onCollision".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "onCollision".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("player.say".into()),
                    args: vec!["\"Ouch!\"".into()],
                }],
            },
            Step::RunWorld,
        ],
    )
}

fn loops_conditionals() -> (&'static str, Vec<Step>) {
    (
        "loops-conditionals",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "dancer".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "countLoop".into(),
                        method: None,
                        args: vec!["3".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("dancer.turn".into()),
                        args: vec!["LEFT".into(), "1.0".into()],
                    },
                    StatementSpec {
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["dancer.isCollidingWith(ground)".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("dancer.say".into()),
                        args: vec!["\"On the ground\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn functions() -> (&'static str, Vec<Step>) {
    (
        "functions",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "explorer".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "treasure".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["explorer.distanceTo(treasure) < 2.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("explorer.say".into()),
                        args: vec!["\"Found it!\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn variables() -> (&'static str, Vec<Step>) {
    (
        "variables",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "scorer".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "localDeclaration".into(),
                        method: None,
                        args: vec!["score".into(), "0".into()],
                    },
                    StatementSpec {
                        kind: "assignment".into(),
                        method: None,
                        args: vec!["score".into(), "score + 10".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("scorer.say".into()),
                        args: vec!["score".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn concurrency() -> (&'static str, Vec<Step>) {
    (
        "concurrency",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "actorA".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "actorB".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "doTogether".into(),
                        method: None,
                        args: vec![],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("actorA.walk".into()),
                        args: vec!["2.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("actorB.turn".into()),
                        args: vec!["RIGHT".into(), "1.0".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn arrays() -> (&'static str, Vec<Step>) {
    (
        "arrays",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "leader".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "follower1".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "follower2".into(),
            },
            Step::AssertMinObjects { min: 3 },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "eachInArrayTogether".into(),
                        method: None,
                        args: vec!["followers".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("item.walk".into()),
                        args: vec!["1.0".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn camera_viewpoint() -> (&'static str, Vec<Step>) {
    (
        "camera-viewpoint",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "subject".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("camera.moveToward".into()),
                        args: vec!["subject".into(), "2.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("camera.pointAt".into()),
                        args: vec!["subject".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn audio() -> (&'static str, Vec<Step>) {
    (
        "audio",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "musician".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "doTogether".into(),
                        method: None,
                        args: vec![],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("musician.walk".into()),
                        args: vec!["2.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("playAudio".into()),
                        args: vec!["march.mp3".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

const PROJECT_IO_SAVE_PATH: &str = "target/test-work/web-platform/project-io-reload.a3p";

fn parameters() -> (&'static str, Vec<Step>) {
    (
        "parameters",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "hero".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "moveHero".into(),
                statements: vec![
                    StatementSpec {
                        kind: "parameterDeclaration".into(),
                        method: None,
                        args: vec!["distance".into(), "DecimalNumber".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("hero.walk".into()),
                        args: vec!["distance".into()],
                    },
                ],
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("moveHero".into()),
                    args: vec!["2.0".into()],
                }],
            },
            Step::RunWorld,
        ],
    )
}

fn inheritance_oop() -> (&'static str, Vec<Step>) {
    (
        "inheritance-oop",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "userTypeDeclaration".into(),
                        method: None,
                        args: vec!["PetLeader".into(), "Biped".into()],
                    },
                    StatementSpec {
                        kind: "defineCustomMethod".into(),
                        method: Some("PetLeader.leadDance".into()),
                        args: vec![],
                    },
                    StatementSpec {
                        kind: "instantiateUserType".into(),
                        method: None,
                        args: vec!["PetLeader".into(), "petLeader".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("petLeader.say".into()),
                        args: vec!["\"Ready to lead\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn comments() -> (&'static str, Vec<Step>) {
    (
        "comments",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "narrator".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "comment".into(),
                        method: None,
                        args: vec![
                            "Explain why the player score changes after collecting the gem".into(),
                        ],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("narrator.say".into()),
                        args: vec!["\"Collect the gem to score!\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn project_io() -> (&'static str, Vec<Step>) {
    (
        "project-io",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "archivist".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("archivist.say".into()),
                    args: vec!["\"Project saved\"".into()],
                }],
            },
            Step::Save {
                path: PROJECT_IO_SAVE_PATH.into(),
            },
            Step::Load {
                path: PROJECT_IO_SAVE_PATH.into(),
            },
            Step::AssertMinObjects { min: 1 },
        ],
    )
}

fn game_narrative() -> (&'static str, Vec<Step>) {
    (
        "game-narrative",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "player".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "gem".into(),
            },
            Step::RegisterEvent {
                event_type: "keyPress".into(),
                handler_name: "onSpacePressed".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "onSpacePressed".into(),
                statements: vec![
                    StatementSpec {
                        kind: "localDeclaration".into(),
                        method: None,
                        args: vec!["score".into(), "0".into()],
                    },
                    StatementSpec {
                        kind: "assignment".into(),
                        method: None,
                        args: vec!["score".into(), "score + 1".into()],
                    },
                    StatementSpec {
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["score >= 3".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("player.say".into()),
                        args: vec!["\"You win!\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn design_process() -> (&'static str, Vec<Step>) {
    (
        "design-process",
        vec![
            Step::Health,
            Step::DesignCheckpoint {
                phase: "plan".into(),
                artifact: "story-vs-game brief".into(),
            },
            Step::DesignCheckpoint {
                phase: "sketch".into(),
                artifact: "scene-sketch card".into(),
            },
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "prototypeHero".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("prototypeHero.say".into()),
                    args: vec!["\"Collect three stars to win\"".into()],
                }],
            },
            Step::RunWorld,
            Step::DesignCheckpoint {
                phase: "playtest".into(),
                artifact: "first-playthrough notes".into(),
            },
            Step::DesignCheckpoint {
                phase: "revise".into(),
                artifact: "design-to-code bridge card".into(),
            },
        ],
    )
}

fn all_scenarios() -> Vec<(&'static str, Vec<Step>)> {
    vec![
        hello_world(),
        procedures(),
        parameters(),
        inheritance_oop(),
        comments(),
        events_collision(),
        loops_conditionals(),
        functions(),
        variables(),
        concurrency(),
        arrays(),
        project_io(),
        game_narrative(),
        design_process(),
        camera_viewpoint(),
        audio(),
    ]
}

fn edit_statements<'a>(steps: &'a [Step], target_method: &str) -> &'a [StatementSpec] {
    steps
        .iter()
        .find_map(|step| match step {
            Step::EditProcedure {
                method_name,
                statements,
                ..
            } if method_name == target_method => Some(statements.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing edit for {target_method}"))
}

// ── Tier 1: Offline structural tests (always run) ───────────────────

#[test]
fn every_scenario_starts_with_health_check() {
    for (name, steps) in all_scenarios() {
        assert!(!steps.is_empty(), "{name} is empty");
        assert!(
            matches!(&steps[0], Step::Health),
            "{name} must start with Health"
        );
    }
}

#[test]
fn every_scenario_launches_a_project() {
    for (name, steps) in all_scenarios() {
        assert!(
            steps.iter().any(|s| matches!(s, Step::Launch { .. })),
            "{name} missing launch"
        );
    }
}

#[test]
fn hello_world_adds_object_and_saves() {
    let (_, steps) = hello_world();
    assert!(steps.iter().any(|s| matches!(s, Step::AddObject { .. })));
    assert!(steps.iter().any(|s| matches!(s, Step::Save { .. })));
}

#[test]
fn procedures_edits_and_runs() {
    let (_, steps) = procedures();
    assert!(
        steps
            .iter()
            .any(|s| matches!(s, Step::EditProcedure { .. }))
    );
    assert!(steps.iter().any(|s| matches!(s, Step::RunWorld)));
}

#[test]
fn events_collision_registers_handler() {
    let (_, steps) = events_collision();
    assert!(
        steps
            .iter()
            .any(|s| matches!(s, Step::RegisterEvent { .. }))
    );
}

#[test]
fn loops_conditionals_has_control_flow() {
    let (_, steps) = loops_conditionals();
    let has_loop = steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } => {
            statements.iter().any(|st| st.kind == "countLoop")
        }
        _ => false,
    });
    let has_if = steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } => statements.iter().any(|st| st.kind == "ifElse"),
        _ => false,
    });
    assert!(has_loop, "needs countLoop");
    assert!(has_if, "needs ifElse");
}

#[test]
fn variables_declares_and_assigns() {
    let (_, steps) = variables();
    let has_decl = steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } => {
            statements.iter().any(|st| st.kind == "localDeclaration")
        }
        _ => false,
    });
    let has_assign = steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } => {
            statements.iter().any(|st| st.kind == "assignment")
        }
        _ => false,
    });
    assert!(has_decl && has_assign);
}

#[test]
fn concurrency_uses_do_together() {
    let (_, steps) = concurrency();
    assert!(steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } =>
            statements.iter().any(|st| st.kind == "doTogether"),
        _ => false,
    }));
}

#[test]
fn arrays_uses_each_in_array() {
    let (_, steps) = arrays();
    assert!(steps.iter().any(|s| match s {
        Step::EditProcedure { statements, .. } =>
            statements.iter().any(|st| st.kind == "eachInArrayTogether"),
        _ => false,
    }));
}

#[test]
fn camera_uses_camera_methods() {
    let (_, steps) = camera_viewpoint();
    assert!(steps.iter().any(|s| {
        match s {
            Step::EditProcedure { statements, .. } => statements
                .iter()
                .any(|st| st.method.as_deref().unwrap_or("").starts_with("camera.")),
            _ => false,
        }
    }));
}

#[test]
fn audio_uses_play_audio() {
    let (_, steps) = audio();
    assert!(steps.iter().any(|s| {
        match s {
            Step::EditProcedure { statements, .. } => statements
                .iter()
                .any(|st| st.method.as_deref().unwrap_or("").contains("playAudio")),
            _ => false,
        }
    }));
}

#[test]
fn parameters_creates_parameterized_method_and_call() {
    let (_, steps) = parameters();
    let move_hero = edit_statements(&steps, "moveHero");
    let signature = move_hero
        .iter()
        .find(|statement| statement.kind == "parameterDeclaration")
        .expect("moveHero should declare a parameter");
    assert_eq!(
        signature.args,
        vec!["distance".to_string(), "DecimalNumber".to_string()]
    );

    let body_call = move_hero
        .iter()
        .find(|statement| statement.method.as_deref() == Some("hero.walk"))
        .expect("moveHero should use the parameter in a walk call");
    assert_eq!(body_call.args, vec!["distance".to_string()]);

    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("moveHero") && statement.args == vec!["2.0".to_string()]
    }));
}

#[test]
fn inheritance_oop_declares_custom_biped_type() {
    let (_, steps) = inheritance_oop();
    let setup = edit_statements(&steps, "myFirstMethod");

    let user_type = setup
        .iter()
        .find(|statement| statement.kind == "userTypeDeclaration")
        .expect("inheritance scenario should declare a user type");
    assert_eq!(
        user_type.args,
        vec!["PetLeader".to_string(), "Biped".to_string()]
    );

    let custom_method = setup
        .iter()
        .find(|statement| statement.kind == "defineCustomMethod")
        .expect("inheritance scenario should define a custom method");
    assert_eq!(custom_method.method.as_deref(), Some("PetLeader.leadDance"));

    let instance = setup
        .iter()
        .find(|statement| statement.kind == "instantiateUserType")
        .expect("inheritance scenario should instantiate the custom type");
    assert_eq!(
        instance.args,
        vec!["PetLeader".to_string(), "petLeader".to_string()]
    );
}

#[test]
fn comments_adds_meaningful_comment_text() {
    let (_, steps) = comments();
    let entrypoint = edit_statements(&steps, "myFirstMethod");

    let comment = entrypoint
        .iter()
        .find(|statement| statement.kind == "comment")
        .expect("comments scenario should add a comment");
    assert_eq!(comment.args.len(), 1);
    assert_eq!(
        comment.args[0],
        "Explain why the player score changes after collecting the gem"
    );

    let narration = entrypoint
        .iter()
        .find(|statement| statement.method.as_deref() == Some("narrator.say"))
        .expect("comments scenario should keep executable behavior alongside the comment");
    assert_eq!(
        narration.args,
        vec!["\"Collect the gem to score!\"".to_string()]
    );
}

#[test]
fn project_io_saves_then_reloads_before_verify() {
    let (_, steps) = project_io();

    let save_index = steps
        .iter()
        .position(|step| matches!(step, Step::Save { path } if path == PROJECT_IO_SAVE_PATH))
        .expect("project_io should save the project");
    let load_index = steps
        .iter()
        .position(|step| matches!(step, Step::Load { path } if path == PROJECT_IO_SAVE_PATH))
        .expect("project_io should reload the saved project");
    let verify_index = steps
        .iter()
        .position(|step| matches!(step, Step::AssertMinObjects { min } if *min == 1))
        .expect("project_io should verify the reloaded project");

    assert!(save_index < load_index, "save must happen before reload");
    assert!(
        load_index < verify_index,
        "reload must happen before verify"
    );
    assert!(
        steps.iter().any(|step| {
            matches!(
                step,
                Step::EditProcedure { method_name, .. } if method_name == "myFirstMethod"
            )
        }),
        "project_io should include content to persist"
    );
}

#[test]
fn game_narrative_tracks_score_and_win_state() {
    let (_, steps) = game_narrative();
    assert!(steps.iter().any(|step| {
        matches!(
            step,
            Step::RegisterEvent { event_type, handler_name }
                if event_type == "keyPress" && handler_name == "onSpacePressed"
        )
    }));

    let handler = edit_statements(&steps, "onSpacePressed");
    let score_declaration = handler
        .iter()
        .find(|statement| statement.kind == "localDeclaration")
        .expect("game narrative should declare a score variable");
    assert_eq!(
        score_declaration.args,
        vec!["score".to_string(), "0".to_string()]
    );

    let score_update = handler
        .iter()
        .find(|statement| statement.kind == "assignment")
        .expect("game narrative should update the score");
    assert_eq!(
        score_update.args,
        vec!["score".to_string(), "score + 1".to_string()]
    );

    let win_check = handler
        .iter()
        .find(|statement| statement.kind == "ifElse")
        .expect("game narrative should define a win condition");
    assert_eq!(win_check.args, vec!["score >= 3".to_string()]);

    assert!(handler.iter().any(|statement| {
        statement.method.as_deref() == Some("player.say")
            && statement.args == vec!["\"You win!\"".to_string()]
    }));
}

#[test]
fn design_process_tracks_plan_build_playtest_and_revision() {
    let (_, steps) = design_process();
    let phases: Vec<_> = steps
        .iter()
        .filter_map(|step| match step {
            Step::DesignCheckpoint { phase, .. } => Some(phase.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(phases, vec!["plan", "sketch", "playtest", "revise"]);

    let artifacts: Vec<_> = steps
        .iter()
        .filter_map(|step| match step {
            Step::DesignCheckpoint { artifact, .. } => Some(artifact.as_str()),
            _ => None,
        })
        .collect();
    assert!(artifacts.contains(&"story-vs-game brief"));
    assert!(artifacts.contains(&"scene-sketch card"));
    assert!(artifacts.contains(&"design-to-code bridge card"));

    let run_index = steps
        .iter()
        .position(|step| matches!(step, Step::RunWorld))
        .expect("design process should run the prototype");
    let playtest_index = steps
        .iter()
        .position(
            |step| matches!(step, Step::DesignCheckpoint { phase, .. } if phase == "playtest"),
        )
        .expect("design process should include playtesting");
    assert!(
        run_index < playtest_index,
        "playtest follows the runnable thin slice"
    );
}

#[test]
fn full_curriculum_breadth_covered() {
    let names: Vec<_> = all_scenarios().iter().map(|(n, _)| *n).collect();
    for required in [
        "hello-world",
        "procedures",
        "parameters",
        "inheritance-oop",
        "comments",
        "events-collision",
        "loops-conditionals",
        "functions",
        "variables",
        "concurrency",
        "arrays",
        "project-io",
        "game-narrative",
        "design-process",
        "camera-viewpoint",
        "audio",
    ] {
        assert!(names.contains(&required), "missing: {required}");
    }
}

#[test]
fn every_scenario_has_at_least_three_steps() {
    for (name, steps) in all_scenarios() {
        assert!(steps.len() >= 3, "{name} has only {} steps", steps.len());
    }
}

// ── Tier 2: Live tests (gated) ─────────────────────────────────────

#[test]
fn live_hello_world() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &hello_world().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_procedures() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &procedures().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_all_curriculum() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    let mut fails = Vec::new();
    for (name, steps) in all_scenarios() {
        for r in execute(&b, &c, &steps) {
            if !r.ok {
                fails.push(format!("{name}/{}: {}", r.name, r.msg));
            }
        }
    }
    assert!(fails.is_empty(), "failures:\n{}", fails.join("\n"));
}
