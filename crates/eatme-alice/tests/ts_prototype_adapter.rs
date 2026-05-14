//! TS prototype adapter — runs the student journey silver thread against
//! the alice-web-prototype REST API instead of desktop Alice.
//!
//! Gated behind EATME_TS_PROTOTYPE=1. Requires the TS server to be running.
//! Set ALICE_WEB_URL to override the default http://localhost:3099.

use serde::Deserialize;
use std::env;
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
