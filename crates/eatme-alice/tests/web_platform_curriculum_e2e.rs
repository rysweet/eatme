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
                        Ok(r) => StepResult { name: format!("save({path})"), ok: r.status == "ok", msg: "ok".into() },
                        Err(e) => StepResult { name: format!("save({path})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("save({path})"), ok: false, msg: e.to_string() },
                }
            }
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

fn all_scenarios() -> Vec<(&'static str, Vec<Step>)> {
    vec![
        hello_world(),
        procedures(),
        events_collision(),
        loops_conditionals(),
        functions(),
        variables(),
        concurrency(),
        arrays(),
        camera_viewpoint(),
        audio(),
    ]
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
fn full_curriculum_breadth_covered() {
    let names: Vec<_> = all_scenarios().iter().map(|(n, _)| *n).collect();
    for required in [
        "hello-world",
        "procedures",
        "events-collision",
        "loops-conditionals",
        "functions",
        "variables",
        "concurrency",
        "arrays",
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
