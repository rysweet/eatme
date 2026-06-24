//! Web platform curriculum scenario tests.
//!
//! These tests verify that the same curriculum scenarios used for desktop
//! Alice can also be executed against the TypeScript web port's REST API.
//!
//! Tier 1 (offline, always run): validate scenario structure/step counts.
//! Tier 2 (gated behind EATME_WEB_PLATFORM=1): hit the live TS server.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::time::Duration;

// ── Configuration ───────────────────────────────────────────────────

fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn web_base_url() -> String {
    normalize_web_base_url(env::var("ALICE_WEB_URL").ok())
}

fn local_api_token() -> String {
    env::var("ALICE_LOCAL_API_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .unwrap_or_else(|| "gadugi-local-api-token".into())
}

fn normalize_web_base_url(raw_url: Option<String>) -> String {
    raw_url
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "http://localhost:3099".into())
}

fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
}

fn authed_post(client: &ureq::Agent, url: &str) -> ureq::Request {
    client
        .post(url)
        .set("X-Alice-Local-Api-Token", &local_api_token())
}

fn authed_get(client: &ureq::Agent, url: &str) -> ureq::Request {
    client
        .get(url)
        .set("X-Alice-Local-Api-Token", &local_api_token())
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
    status: Option<String>,
    #[serde(rename = "registrationId")]
    registration_id: Option<String>,
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
    DesignProcessEvidence,
    RegisterEvent {
        event_type: String,
        handler_name: String,
    },
    ExpectError {
        name: String,
        endpoint: String,
        body: Value,
        expected_status: u16,
        expected_message: String,
    },
    CameraComfortEvidence,
    VrNativeBoundaryEvidence,
    AccessibilityCaptionEvidence,
    GalleryWalkRubricEvidence,
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
                        Ok(h) => StepResult { name: "health".into(), ok: matches!(h.status.as_str(), "ok" | "running"), msg: "ok".into() },
                        Err(e) => StepResult { name: "health".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "health".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::Launch { template } => {
                match authed_post(client, &format!("{base}/api/launch")).send_json(ureq::json!({ "template": template })) {
                    Ok(resp) => match resp.into_json::<LaunchResponse>() {
                        Ok(r) => { last_count = r.scene_object_count; StepResult { name: format!("launch({template})"), ok: matches!(r.status.as_str(), "ok" | "launched"), msg: format!("objects={}", r.scene_object_count) } },
                        Err(e) => StepResult { name: format!("launch({template})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("launch({template})"), ok: false, msg: e.to_string() },
                }
            }
            Step::AddObject { class_name, instance_name } => {
                match authed_post(client, &format!("{base}/api/scene/add-object")).send_json(ureq::json!({ "className": class_name, "name": instance_name })) {
                    Ok(resp) => match resp.into_json::<AddObjectResponse>() {
                        Ok(r) => { last_count = r.scene_field_count_after; StepResult { name: format!("add({class_name})"), ok: matches!(r.status.as_str(), "ok" | "added"), msg: format!("after={}", r.scene_field_count_after) } },
                        Err(e) => StepResult { name: format!("add({class_name})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("add({class_name})"), ok: false, msg: e.to_string() },
                }
            }
            Step::EditProcedure { class_name, method_name, statements } => {
                match authed_post(client, &format!("{base}/api/code/edit-procedure")).send_json(ureq::json!({ "procedureSelector": format!("scene.{method_name}"), "editSpec": build_edit_spec(class_name, method_name, statements) })) {
                    Ok(resp) => match resp.into_json::<EditProcedureResponse>() {
                        Ok(r) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: matches!(r.status.as_str(), "ok" | "proved"), msg: "ok".into() },
                        Err(e) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("edit({class_name}.{method_name})"), ok: false, msg: e.to_string() },
                }
            }
            Step::RunWorld => {
                match authed_post(client, &format!("{base}/api/world/run")).send_json(ureq::json!({})) {
                    Ok(resp) => match resp.into_json::<RunWorldResponse>() {
                        Ok(r) => { last_count = r.scene_object_count; StepResult { name: "run".into(), ok: matches!(r.status.as_str(), "ok" | "completed"), msg: format!("objects={}", r.scene_object_count) } },
                        Err(e) => StepResult { name: "run".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "run".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::Save { path } => {
                match authed_post(client, &format!("{base}/api/project/save")).send_json(ureq::json!({ "targetPath": path })) {
                    Ok(resp) => match resp.into_json::<SaveResponse>() {
                        Ok(r) => {
                            if matches!(r.status.as_str(), "ok" | "saved") {
                                saved_count = Some(last_count);
                                saved_path = Some(path.clone());
                            }
                            StepResult { name: format!("save({path})"), ok: matches!(r.status.as_str(), "ok" | "saved"), msg: "ok".into() }
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
            Step::DesignProcessEvidence => {
                match authed_post(client, &format!("{base}/api/design-process/story-or-game/evidence")).send_json(design_process_evidence_payload()) {
                    Ok(resp) => match resp.into_json::<Value>() {
                        Ok(value) => {
                            let phases: Vec<_> = value
                                .get("phases")
                                .and_then(Value::as_array)
                                .map(|items| {
                                    items
                                        .iter()
                                        .filter_map(Value::as_str)
                                        .collect()
                                })
                                .unwrap_or_default();
                            let authored_objects: Vec<_> = value
                                .pointer("/journeyEvidence/build/authoredObjectNames")
                                .and_then(Value::as_array)
                                .map(|items| {
                                    items
                                        .iter()
                                        .filter_map(Value::as_str)
                                        .collect()
                                })
                                .unwrap_or_default();
                            let does_not_claim: Vec<_> = value
                                .get("doesNotClaim")
                                .and_then(Value::as_array)
                                .map(|items| {
                                    items
                                        .iter()
                                        .filter_map(Value::as_str)
                                        .collect()
                                })
                                .unwrap_or_default();
                            let run_count = value
                                .pointer("/journeyEvidence/playtest/runCount")
                                .and_then(Value::as_u64)
                                .unwrap_or_default();
                            let revision_note = value
                                .pointer("/journeyEvidence/revise/revisionNote")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let ok = value.get("schema_version").and_then(Value::as_str)
                                    == Some("lookingglass.design-process-story-or-game-evidence/v1")
                                && value.get("status").and_then(Value::as_str) == Some("evidence-recorded")
                                && ["plan", "build", "playtest", "revise", "review"]
                                    .iter()
                                    .all(|phase| phases.contains(phase))
                                && authored_objects.contains(&"prototypeHero")
                                && run_count >= 2
                                && revision_note.contains("second narration line")
                                && does_not_claim.contains(&"automated creative assessment");
                            StepResult { name: "design-process-evidence".into(), ok, msg: value.to_string() }
                        }
                        Err(e) => StepResult { name: "design-process-evidence".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "design-process-evidence".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::RegisterEvent { event_type, handler_name } => {
                let mut payload = serde_json::json!({ "eventType": event_type, "handlerName": handler_name });
                if event_type == "keyPress" || event_type == "keyPressed" {
                    payload["key"] = serde_json::json!("SPACE");
                }
                match authed_post(client, &format!("{base}/api/events/register")).send_json(payload) {
                    Ok(resp) => match resp.into_json::<EventResponse>() {
                        Ok(r) => StepResult { name: format!("register({event_type})"), ok: r.status.as_deref() == Some("ok") || r.registration_id.is_some(), msg: "ok".into() },
                        Err(e) => StepResult { name: format!("register({event_type})"), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: format!("register({event_type})"), ok: false, msg: e.to_string() },
                }
            }
            Step::ExpectError { name, endpoint, body, expected_status, expected_message } => {
                match authed_post(client, &format!("{base}{endpoint}")).send_json(body.clone()) {
                    Ok(resp) => StepResult {
                        name: name.clone(),
                        ok: false,
                        msg: format!("expected status {expected_status}, got {}", resp.status()),
                    },
                    Err(ureq::Error::Status(code, resp)) => {
                        let message = resp
                            .into_json::<Value>()
                            .ok()
                            .and_then(|value| value.get("error").and_then(Value::as_str).map(str::to_string))
                            .unwrap_or_default();
                        StepResult {
                            name: name.clone(),
                            ok: code == *expected_status && message.contains(expected_message),
                            msg: format!("status={code} message={message}"),
                        }
                    }
                    Err(e) => StepResult { name: name.clone(), ok: false, msg: e.to_string() },
                }
            }
            Step::CameraComfortEvidence => {
                match authed_get(client, &format!("{base}/api/vr/camera-comfort")).call() {
                    Ok(resp) => match resp.into_json::<Value>() {
                        Ok(value) => {
                            let ok = value.get("schema_version").and_then(Value::as_str)
                                    == Some("alice.camera-vr-comfort-evidence/v1")
                                && value.get("desktopCameraAvailable").and_then(Value::as_bool) == Some(true)
                                && value.get("trueHeadsetVrSupported").and_then(Value::as_bool) == Some(false)
                                && value.get("nativeVrSupported").and_then(Value::as_bool) == Some(false);
                            StepResult { name: "camera-comfort-evidence".into(), ok, msg: value.to_string() }
                        }
                        Err(e) => StepResult { name: "camera-comfort-evidence".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "camera-comfort-evidence".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::VrNativeBoundaryEvidence => {
                match authed_get(client, &format!("{base}/api/vr/camera-comfort")).call() {
                    Ok(resp) => match resp.into_json::<Value>() {
                        Ok(value) => {
                            let browser_session = value
                                .get("browserWebXrSession")
                                .and_then(Value::as_object);
                            let playtest = value
                                .get("playerComfortPlaytest")
                                .and_then(Value::as_object);
                            let locomotion_mode = browser_session
                                .and_then(|session| session.get("locomotionMode"))
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let evidence_codes: Vec<_> = value
                                .get("evidenceCodes")
                                .and_then(Value::as_array)
                                .map(|items| items.iter().filter_map(Value::as_str).collect())
                                .unwrap_or_default();
                            let comfort_checks = value.get("comfortChecks").and_then(Value::as_object);
                            let browser_status = value
                                .get("browserWebXrStatus")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let browser_boundary_ok = browser_session
                                .map(|_| {
                                    matches!(
                                        locomotion_mode,
                                        "combined" | "controller-smooth" | "click-move" | "point-click" | "disabled" | "unknown"
                                    )
                                })
                                .unwrap_or(false);
                            let playtest_boundary_ok = playtest
                                .map(|boundary| {
                                    boundary
                                        .get("truePlayerComfortPlaytestSupported")
                                        .and_then(Value::as_bool)
                                        == Some(false)
                                        && boundary
                                            .get("revisionLoopEvidence")
                                            .and_then(Value::as_str)
                                            == Some("not-observed")
                                })
                                .unwrap_or(false);
                            let comfort_boundary_ok = comfort_checks
                                .map(|checks| {
                                    checks.get("noForcedHeadset").and_then(Value::as_bool) == Some(true)
                                        && checks.get("stableHorizon").and_then(Value::as_bool) == Some(true)
                                })
                                .unwrap_or(false);
                            let ok = browser_boundary_ok
                                && playtest_boundary_ok
                                && comfort_boundary_ok
                                && matches!(browser_status, "available" | "unavailable" | "unknown")
                                && evidence_codes.contains(&"true-vr-unsupported")
                                && value.get("trueHeadsetVrSupported").and_then(Value::as_bool) == Some(false)
                                && value.get("nativeVrSupported").and_then(Value::as_bool) == Some(false);
                            StepResult { name: "vr-native-boundary-evidence".into(), ok, msg: value.to_string() }
                        }
                        Err(e) => StepResult { name: "vr-native-boundary-evidence".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "vr-native-boundary-evidence".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::AccessibilityCaptionEvidence => {
                match authed_get(client, &format!("{base}/api/accessibility/rescue-camera-captions")).call() {
                    Ok(resp) => match resp.into_json::<Value>() {
                        Ok(value) => {
                            let caption_ids: Vec<_> = value
                                .get("captionChecks")
                                .and_then(Value::as_array)
                                .map(|checks| {
                                    checks
                                        .iter()
                                        .filter_map(|check| check.get("id").and_then(Value::as_str))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let ok = value.get("schema_version").and_then(Value::as_str)
                                    == Some("alice.accessibility-rescue-camera-captions/v1")
                                && value.get("cameraCaption").and_then(Value::as_str).unwrap_or_default().contains("Camera")
                                && value.get("objectCaption").and_then(Value::as_str).unwrap_or_default().contains("captionGuide")
                                && value.get("keyboardReviewAvailable").and_then(Value::as_bool) == Some(true)
                                && value.get("highContrastReviewAvailable").and_then(Value::as_bool) == Some(true)
                                && caption_ids.contains(&"aria-live-status")
                                && caption_ids.contains(&"camera-caption")
                                && caption_ids.contains(&"scene-object-caption");
                            StepResult { name: "accessibility-caption-evidence".into(), ok, msg: value.to_string() }
                        }
                        Err(e) => StepResult { name: "accessibility-caption-evidence".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "accessibility-caption-evidence".into(), ok: false, msg: e.to_string() },
                }
            }
            Step::GalleryWalkRubricEvidence => {
                match authed_get(client, &format!("{base}/api/review/gallery-walk-rubric")).call() {
                    Ok(resp) => match resp.into_json::<Value>() {
                        Ok(value) => {
                            let rubric_ids: Vec<_> = value
                                .get("rubric")
                                .and_then(Value::as_array)
                                .map(|criteria| {
                                    criteria
                                        .iter()
                                        .filter_map(|criterion| criterion.get("id").and_then(Value::as_str))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let has_review_prompt = value
                                .get("galleryItems")
                                .and_then(Value::as_array)
                                .map(|items| items.iter().any(|item| {
                                    item.get("title").and_then(Value::as_str) == Some("reviewCheckpoint")
                                        && item.get("reviewPrompt").and_then(Value::as_str).unwrap_or_default().contains("reviewCheckpoint")
                                }))
                                .unwrap_or(false);
                            let ok = value.get("schema_version").and_then(Value::as_str)
                                    == Some("alice.gallery-walk-rubric-evidence/v1")
                                && value.get("reviewWorkflowSupported").and_then(Value::as_bool) == Some(true)
                                && value.get("rubricRecordingSupported").and_then(Value::as_bool) == Some(false)
                                && value.get("liveStudioSupported").and_then(Value::as_bool) == Some(true)
                                && value.get("galleryItemCount").and_then(Value::as_u64).unwrap_or_default() >= 1
                                && has_review_prompt
                                && rubric_ids.contains(&"visible-world")
                                && rubric_ids.contains(&"camera-framing")
                                && rubric_ids.contains(&"accessibility-captions");
                            StepResult { name: "gallery-walk-rubric-evidence".into(), ok, msg: value.to_string() }
                        }
                        Err(e) => StepResult { name: "gallery-walk-rubric-evidence".into(), ok: false, msg: e.to_string() },
                    },
                    Err(e) => StepResult { name: "gallery-walk-rubric-evidence".into(), ok: false, msg: e.to_string() },
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

fn assert_live_scenario(name: &str, steps: Vec<Step>) {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let client = http_client();
    let base = web_base_url();
    let failures = execute(&base, &client, &steps)
        .into_iter()
        .filter(|result| !result.ok)
        .map(|result| format!("{name}/{}: {}", result.name, result.msg))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
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
                path: HELLO_WORLD_SAVE_PATH.into(),
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

fn vr_camera_locomotion_journey() -> (&'static str, Vec<Step>) {
    (
        "vr-camera-locomotion-journey",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "comfortGuide".into(),
            },
            Step::CameraComfortEvidence,
            Step::VrNativeBoundaryEvidence,
            Step::RunWorld,
        ],
    )
}

fn vr_player_comfort_playtest() -> (&'static str, Vec<Step>) {
    (
        "vr-player-comfort-playtest",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "playerTester".into(),
            },
            Step::CameraComfortEvidence,
            Step::VrNativeBoundaryEvidence,
            Step::RunWorld,
        ],
    )
}

fn accessibility_rescue_camera_captions() -> (&'static str, Vec<Step>) {
    (
        "accessibility-rescue-camera-captions",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "captionGuide".into(),
            },
            Step::AccessibilityCaptionEvidence,
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

const HELLO_WORLD_SAVE_PATH: &str = "target/test-work/web-platform/hello-world-save.a3p";
const PROJECT_IO_SAVE_PATH: &str = "target/test-work/web-platform/project-io-reload.a3p";
const FULL_STUDENT_JOURNEY_SAVE_PATH: &str =
    "target/test-work/web-platform/full-student-journey.a3p";
const INSTRUCTOR_GRADING_SAVE_PATH: &str = "target/test-work/web-platform/instructor-grading.a3p";

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

fn say_think() -> (&'static str, Vec<Step>) {
    (
        "say-think",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "speaker".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("speaker.say".into()),
                        args: vec!["\"Welcome to the bubble lab\"".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("speaker.think".into()),
                        args: vec!["\"I should keep this plan quiet\"".into()],
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
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("prototypeHero.say".into()),
                    args: vec!["\"Revision: show win feedback\"".into()],
                }],
            },
            Step::RunWorld,
            Step::DesignProcessEvidence,
        ],
    )
}

fn vehicle_parenting() -> (&'static str, Vec<Step>) {
    (
        "vehicle-parenting",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "driver".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("camera.setVehicle".into()),
                        args: vec!["driver".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("driver.walk".into()),
                        args: vec!["1.0".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn joint_manipulation() -> (&'static str, Vec<Step>) {
    (
        "joint-manipulation",
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
                        kind: "methodCall".into(),
                        method: Some("dancer.rightShoulder.turn".into()),
                        args: vec!["FORWARD".into(), "0.25".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("dancer.leftKnee.turn".into()),
                        args: vec!["BACKWARD".into(), "0.15".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn scene_transition() -> (&'static str, Vec<Step>) {
    (
        "scene-transition",
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
                        kind: "sceneDeclaration".into(),
                        method: None,
                        args: vec!["introScene".into()],
                    },
                    StatementSpec {
                        kind: "sceneDeclaration".into(),
                        method: None,
                        args: vec!["creditsScene".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("setActiveScene".into()),
                        args: vec!["creditsScene".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn property_animation() -> (&'static str, Vec<Step>) {
    (
        "property-animation",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "overlay".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "animateProperty".into(),
                        method: None,
                        args: vec![
                            "overlay.opacity".into(),
                            "1.0".into(),
                            "0.25".into(),
                            "1.5".into(),
                        ],
                    },
                    StatementSpec {
                        kind: "animateProperty".into(),
                        method: None,
                        args: vec![
                            "overlay.color".into(),
                            "Color.WHITE".into(),
                            "Color.BLUE".into(),
                            "1.5".into(),
                        ],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn nested_control_flow() -> (&'static str, Vec<Step>) {
    (
        "nested-control-flow",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "logicHero".into(),
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
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["logicHero.isNear(target)".into()],
                    },
                    StatementSpec {
                        kind: "countLoop".into(),
                        method: None,
                        args: vec!["2".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("logicHero.say".into()),
                        args: vec!["\"Looping in branch\"".into()],
                    },
                ],
            },
            Step::RunWorld,
        ],
    )
}

fn full_student_journey() -> (&'static str, Vec<Step>) {
    (
        "full-student-journey",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "studentHero".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "goal".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "hazard".into(),
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
                        method: Some("studentHero.walk".into()),
                        args: vec!["1.0".into()],
                    },
                    StatementSpec {
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["studentHero.distanceTo(goal) < 2.0".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("studentHero.say".into()),
                        args: vec!["\"Goal reached\"".into()],
                    },
                ],
            },
            Step::RegisterEvent {
                event_type: "collision".into(),
                handler_name: "onStudentCollision".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "onStudentCollision".into(),
                statements: vec![StatementSpec {
                    kind: "methodCall".into(),
                    method: Some("studentHero.say".into()),
                    args: vec!["\"Careful!\"".into()],
                }],
            },
            Step::RunWorld,
            Step::Save {
                path: FULL_STUDENT_JOURNEY_SAVE_PATH.into(),
            },
            Step::AssertMinObjects { min: 5 },
        ],
    )
}

fn instructor_grading() -> (&'static str, Vec<Step>) {
    (
        "instructor-grading",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "learner".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "checkpoint".into(),
            },
            Step::EditProcedure {
                class_name: "Scene".into(),
                method_name: "myFirstMethod".into(),
                statements: vec![
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("learner.walk".into()),
                        args: vec!["2.0".into()],
                    },
                    StatementSpec {
                        kind: "ifElse".into(),
                        method: None,
                        args: vec!["learner.distanceTo(checkpoint) < 1.5".into()],
                    },
                    StatementSpec {
                        kind: "methodCall".into(),
                        method: Some("learner.say".into()),
                        args: vec!["\"Rubric ready\"".into()],
                    },
                ],
            },
            Step::RunWorld,
            Step::Save {
                path: INSTRUCTOR_GRADING_SAVE_PATH.into(),
            },
            Step::Load {
                path: INSTRUCTOR_GRADING_SAVE_PATH.into(),
            },
            Step::AssertMinObjects { min: 4 },
        ],
    )
}

fn classroom_gallery_walk_and_rubric() -> (&'static str, Vec<Step>) {
    (
        "classroom-gallery-walk-and-rubric",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "reviewHero".into(),
            },
            Step::AddObject {
                class_name: "Prop".into(),
                instance_name: "reviewCheckpoint".into(),
            },
            Step::GalleryWalkRubricEvidence,
        ],
    )
}

fn error_recovery() -> (&'static str, Vec<Step>) {
    (
        "error-recovery",
        vec![
            Step::Health,
            Step::Launch {
                template: "blank".into(),
            },
            Step::ExpectError {
                name: "missing-class-name".into(),
                endpoint: "/api/scene/add-object".into(),
                body: serde_json::json!({}),
                expected_status: 400,
                expected_message: "className is required".into(),
            },
            Step::ExpectError {
                name: "unknown-event-type".into(),
                endpoint: "/api/events/register".into(),
                body: serde_json::json!({ "eventType": "madeUpEvent", "handlerName": "onMadeUpEvent" }),
                expected_status: 400,
                expected_message: "unknown eventType".into(),
            },
            Step::AddObject {
                class_name: "Biped".into(),
                instance_name: "resilientHero".into(),
            },
            Step::RunWorld,
            Step::AssertMinObjects { min: 3 },
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
        say_think(),
        design_process(),
        camera_viewpoint(),
        vr_camera_locomotion_journey(),
        vr_player_comfort_playtest(),
        accessibility_rescue_camera_captions(),
        audio(),
        vehicle_parenting(),
        joint_manipulation(),
        scene_transition(),
        property_animation(),
        nested_control_flow(),
        full_student_journey(),
        instructor_grading(),
        classroom_gallery_walk_and_rubric(),
        error_recovery(),
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

fn build_edit_spec(class_name: &str, method_name: &str, statements: &[StatementSpec]) -> String {
    let summary = statements
        .iter()
        .map(|statement| {
            let target = statement
                .method
                .as_deref()
                .unwrap_or(statement.kind.as_str());
            if statement.args.is_empty() {
                target.to_string()
            } else {
                format!("{}({})", target, statement.args.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(" | ");
    format!("append-comment:{class_name}.{method_name}: {summary}")
}

fn design_process_evidence_payload() -> Value {
    serde_json::json!({
        "scenario": "design-process-story-or-game",
        "mode": "game",
        "designBrief": "Game: guide the hero from goal setup to win feedback.",
        "sceneSketches": [
            {
                "name": "setup-state",
                "character": "prototypeHero",
                "action": "explains the win goal"
            },
            {
                "name": "win-state",
                "character": "prototypeHero",
                "action": "reports the successful revision"
            }
        ],
        "bridgeMappings": [
            {
                "scene": "setup-state",
                "aliceConcept": "myFirstMethod",
                "controls": "prototypeHero goal narration"
            },
            {
                "scene": "win-state",
                "aliceConcept": "conditional",
                "controls": "revised win feedback"
            }
        ],
        "playtestObservation": "First playtest showed the goal, but the win feedback was missing.",
        "revisionNote": "Added a second narration line after playtest so the player sees the win feedback.",
        "reviewNote": "Review confirms plan, build, playtest, revise, and review evidence are present.",
        "accessibilityChoice": "Use narrated text instead of extra characters."
    })
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
fn vr_camera_locomotion_records_bounded_comfort_evidence() {
    let (_, steps) = vr_camera_locomotion_journey();
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::CameraComfortEvidence)),
        "VR camera journey should prove web camera comfort evidence"
    );
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::VrNativeBoundaryEvidence)),
        "VR camera journey should record browser WebXR session/locomotion evidence boundaries"
    );
    assert!(
        !steps
            .iter()
            .any(|step| matches!(step, Step::GalleryWalkRubricEvidence)),
        "VR camera journey must not claim unrelated review tooling"
    );
}

#[test]
fn vr_player_comfort_keeps_true_headset_playtest_unsupported_until_observed() {
    let (_, steps) = vr_player_comfort_playtest();
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::CameraComfortEvidence)),
        "VR player comfort should use the bounded camera/WebXR evidence endpoint"
    );
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::VrNativeBoundaryEvidence)),
        "VR player comfort must require explicit unsupported headset/revision-loop boundaries"
    );
    assert!(steps.iter().any(|step| matches!(step, Step::AddObject { instance_name, .. } if instance_name == "playerTester")));
}

#[test]
fn blank_alice_web_url_uses_default_base_url() {
    assert_eq!(normalize_web_base_url(None), "http://localhost:3099");
    assert_eq!(
        normalize_web_base_url(Some("   \n\t  ".into())),
        "http://localhost:3099"
    );
    assert_eq!(
        normalize_web_base_url(Some(" http://127.0.0.1:4000/ ".into())),
        "http://127.0.0.1:4000/"
    );
}

#[test]
fn live_vr_camera_locomotion_exercises_camera_comfort_api() {
    let (name, steps) = vr_camera_locomotion_journey();
    assert_live_scenario(name, steps);
}

#[test]
fn live_vr_player_comfort_exercises_vr_boundary_api() {
    let (name, steps) = vr_player_comfort_playtest();
    assert_live_scenario(name, steps);
}

#[test]
fn accessibility_rescue_camera_captions_records_caption_evidence() {
    let (_, steps) = accessibility_rescue_camera_captions();
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::AccessibilityCaptionEvidence)),
        "accessibility rescue scenario should prove browser caption evidence"
    );
    assert!(steps.iter().any(|step| matches!(step, Step::AddObject { instance_name, .. } if instance_name == "captionGuide")));
}

#[test]
fn live_accessibility_rescue_camera_captions_exercises_caption_api() {
    let (name, steps) = accessibility_rescue_camera_captions();
    assert_live_scenario(name, steps);
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

#[path = "support/web_platform_curriculum_tail.rs"]
mod web_platform_curriculum_tail;
