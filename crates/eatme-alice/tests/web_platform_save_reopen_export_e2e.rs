//! Focused LookingGlass save/reopen/export parity checks.
//!
//! Offline tests verify the objects-first and starter-project persistence
//! contracts. Live tests, gated by `EATME_WEB_PLATFORM=1`, exercise the
//! LookingGlass REST API with real save-to-path, reopen, and export calls.

use serde::Deserialize;
use std::env;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;

const OBJECTS_FIRST_WORLD_SAVE_PATH: &str = "target/test-work/web-platform/objects-first-world.a3p";
const OBJECTS_FIRST_FULL_PATH_SAVE_PATH: &str =
    "target/test-work/web-platform/objects-first-full-path.a3p";
const FIRST_LESSON_ACTIONS_SAVE_PATH: &str =
    "target/test-work/web-platform/first-lessons-real-ui-actions.a3p";
const STARTER_PREFLIGHT_SAVE_PATH: &str =
    "target/test-work/web-platform/starter-project-open-save-export-preflight.a3p";
const STARTER_PROJECT_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/real/africaMinimum.a3p"
);

#[derive(Debug, Clone)]
enum Step {
    Health,
    Launch,
    LaunchProject {
        path: &'static str,
    },
    AddObject {
        class_name: &'static str,
        instance_name: &'static str,
    },
    EditProcedure {
        method_name: &'static str,
        edit_spec: &'static str,
    },
    RunWorld,
    Save {
        path: &'static str,
    },
    Reopen {
        path: &'static str,
    },
    AddUnsavedObject {
        class_name: &'static str,
        instance_name: &'static str,
    },
    ExportTypeScript,
    AssertMinObjects {
        min: usize,
    },
    AssertEditedProcedurePersisted,
}

#[derive(Debug)]
struct Scenario {
    id: &'static str,
    steps: Vec<Step>,
}

#[derive(Debug)]
struct StepResult {
    name: String,
    ok: bool,
    msg: String,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct LaunchResponse {
    status: String,
    project: Option<String>,
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
struct RunWorldResponse {
    status: String,
    #[serde(rename = "scene_object_count")]
    scene_object_count: usize,
}

fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM").is_ok_and(|value| value == "1")
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

fn alice_objects_first_world() -> Scenario {
    Scenario {
        id: "alice-objects-first-world",
        steps: vec![
            Step::Health,
            Step::Launch,
            Step::AddObject {
                class_name: "Biped",
                instance_name: "bunny",
            },
            Step::EditProcedure {
                method_name: "myFirstMethod",
                edit_spec: "append-comment:bunny.move(FORWARD, 1.0)",
            },
            Step::RunWorld,
            Step::Save {
                path: OBJECTS_FIRST_WORLD_SAVE_PATH,
            },
            Step::AddUnsavedObject {
                class_name: "Prop",
                instance_name: "unsavedAfterObjectsFirst",
            },
            Step::Reopen {
                path: OBJECTS_FIRST_WORLD_SAVE_PATH,
            },
            Step::AssertEditedProcedurePersisted,
            Step::AssertMinObjects { min: 3 },
        ],
    }
}

fn alice_objects_first_full_path() -> Scenario {
    Scenario {
        id: "alice-objects-first-full-path",
        steps: vec![
            Step::Health,
            Step::Launch,
            Step::AddObject {
                class_name: "Biped",
                instance_name: "bunny",
            },
            Step::AddObject {
                class_name: "Prop",
                instance_name: "marker",
            },
            Step::EditProcedure {
                method_name: "myFirstMethod",
                edit_spec: "append-comment:bunny.move(FORWARD, 1.25) | bunny.turn(LEFT, 0.25)",
            },
            Step::RunWorld,
            Step::Save {
                path: OBJECTS_FIRST_FULL_PATH_SAVE_PATH,
            },
            Step::AddUnsavedObject {
                class_name: "Prop",
                instance_name: "unsavedAfterFullPath",
            },
            Step::Reopen {
                path: OBJECTS_FIRST_FULL_PATH_SAVE_PATH,
            },
            Step::AssertEditedProcedurePersisted,
            Step::ExportTypeScript,
            Step::AssertMinObjects { min: 4 },
        ],
    }
}

fn first_lessons_real_ui_actions() -> Scenario {
    Scenario {
        id: "first-lessons-real-ui-actions",
        steps: vec![
            Step::Health,
            Step::Launch,
            Step::AddObject {
                class_name: "Biped",
                instance_name: "studentHero",
            },
            Step::EditProcedure {
                method_name: "myFirstMethod",
                edit_spec: "append-comment:studentHero.say(\"First lesson action recorded\")",
            },
            Step::RunWorld,
            Step::Save {
                path: FIRST_LESSON_ACTIONS_SAVE_PATH,
            },
            Step::AddUnsavedObject {
                class_name: "Prop",
                instance_name: "unsavedAfterFirstLesson",
            },
            Step::Reopen {
                path: FIRST_LESSON_ACTIONS_SAVE_PATH,
            },
            Step::AssertEditedProcedurePersisted,
            Step::AssertMinObjects { min: 3 },
        ],
    }
}

fn starter_project_open_save_export_preflight() -> Scenario {
    Scenario {
        id: "starter-project-open-save-export-preflight",
        steps: vec![
            Step::Health,
            Step::LaunchProject {
                path: STARTER_PROJECT_FIXTURE,
            },
            Step::AddObject {
                class_name: "Prop",
                instance_name: "starterMarker",
            },
            Step::RunWorld,
            Step::Save {
                path: STARTER_PREFLIGHT_SAVE_PATH,
            },
            Step::AddUnsavedObject {
                class_name: "Prop",
                instance_name: "unsavedAfterStarter",
            },
            Step::Reopen {
                path: STARTER_PREFLIGHT_SAVE_PATH,
            },
            Step::ExportTypeScript,
            Step::AssertMinObjects { min: 3 },
        ],
    }
}

fn scenarios() -> Vec<Scenario> {
    vec![
        alice_objects_first_world(),
        alice_objects_first_full_path(),
        first_lessons_real_ui_actions(),
        starter_project_open_save_export_preflight(),
    ]
}

#[test]
fn every_targeted_scenario_saves_reopens_and_verifies_after_reopen() {
    for scenario in scenarios() {
        assert_order(&scenario, "save", "reopen");
        assert_order(&scenario, "reopen", "verify");
        assert!(
            has_step(&scenario, "run"),
            "{} must run before persistence evidence is trusted",
            scenario.id
        );
    }
}

#[test]
fn export_rows_export_only_after_reopen() {
    for scenario in [
        alice_objects_first_full_path(),
        starter_project_open_save_export_preflight(),
    ] {
        assert_order(&scenario, "reopen", "export");
    }
}

#[test]
fn first_lesson_row_keeps_object_edit_run_save_reopen_evidence() {
    let scenario = first_lessons_real_ui_actions();
    for required in ["add-object", "edit", "run", "save", "reopen", "verify-edit"] {
        assert!(
            has_step(&scenario, required),
            "{} missing {required}",
            scenario.id
        );
    }
}

#[test]
fn live_save_reopen_export_parity_rows() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }

    let client = http_client();
    let base = web_base_url();
    let mut failures = Vec::new();
    for scenario in scenarios() {
        for result in execute(&base, &client, &scenario.steps) {
            if !result.ok {
                failures.push(format!("{}/{}: {}", scenario.id, result.name, result.msg));
            }
        }
    }

    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}

fn execute(base: &str, client: &ureq::Agent, steps: &[Step]) -> Vec<StepResult> {
    let mut results = Vec::new();
    let mut last_count = 0;
    let mut saved_count = None;
    let mut saved_path = None;
    let mut edited_snippets = Vec::new();

    for step in steps {
        let result = match step {
            Step::Health => get_status(client, &format!("{base}/api/health")),
            Step::Launch => post_launch(client, &format!("{base}/api/launch"), &mut last_count),
            Step::LaunchProject { path } => {
                post_launch_project(client, base, path, &mut last_count)
            }
            Step::AddObject {
                class_name,
                instance_name,
            } => post_add_object(
                client,
                &format!("{base}/api/scene/add-object"),
                class_name,
                instance_name,
                &mut last_count,
            ),
            Step::AddUnsavedObject {
                class_name,
                instance_name,
            } => post_add_object(
                client,
                &format!("{base}/api/scene/add-object"),
                class_name,
                instance_name,
                &mut last_count,
            ),
            Step::EditProcedure {
                method_name,
                edit_spec,
            } => {
                let result = post_edit(client, base, method_name, edit_spec);
                if result.ok {
                    edited_snippets.push(persisted_snippet_from_edit_spec(edit_spec));
                }
                result
            }
            Step::RunWorld => post_run_world(client, base, &mut last_count),
            Step::Save { path } => {
                let result = post_save(client, base, path);
                if result.ok {
                    saved_count = Some(last_count);
                    saved_path = Some(*path);
                }
                result
            }
            Step::Reopen { path } => post_reopen(
                client,
                base,
                path,
                saved_path,
                saved_count.unwrap_or(last_count),
                &mut last_count,
            ),
            Step::ExportTypeScript => get_typescript_export(client, base),
            Step::AssertMinObjects { min } => StepResult {
                name: format!("assert-min-objects({min})"),
                ok: last_count >= *min,
                msg: format!("actual={last_count}"),
            },
            Step::AssertEditedProcedurePersisted => {
                get_typescript_export_contains(client, base, &edited_snippets)
            }
        };
        results.push(result);
    }

    results
}

fn get_status(client: &ureq::Agent, url: &str) -> StepResult {
    match client.get(url).call() {
        Ok(resp) => match resp.into_json::<StatusResponse>() {
            Ok(body) => StepResult {
                name: "health".into(),
                ok: matches!(body.status.as_str(), "ok" | "running"),
                msg: body.status,
            },
            Err(error) => failed("health", error),
        },
        Err(error) => failed("health", error),
    }
}

fn post_launch(client: &ureq::Agent, url: &str, last_count: &mut usize) -> StepResult {
    match client.post(url).send_json(ureq::json!({})) {
        Ok(resp) => match resp.into_json::<LaunchResponse>() {
            Ok(body) => {
                *last_count = body.scene_object_count;
                StepResult {
                    name: "launch".into(),
                    ok: matches!(body.status.as_str(), "ok" | "launched"),
                    msg: format!("objects={}", body.scene_object_count),
                }
            }
            Err(error) => failed("launch", error),
        },
        Err(error) => failed("launch", error),
    }
}

fn post_launch_project(
    client: &ureq::Agent,
    base: &str,
    path: &str,
    last_count: &mut usize,
) -> StepResult {
    match client
        .post(&format!("{base}/api/launch"))
        .send_json(ureq::json!({ "project": path }))
    {
        Ok(resp) => match resp.into_json::<LaunchResponse>() {
            Ok(body) => {
                *last_count = body.scene_object_count;
                let opened_requested_project = body
                    .project
                    .as_deref()
                    .is_some_and(|project| project.ends_with(path));
                StepResult {
                    name: format!("launch-project({path})"),
                    ok: matches!(body.status.as_str(), "ok" | "launched")
                        && opened_requested_project,
                    msg: format!(
                        "project={:?} objects={}",
                        body.project, body.scene_object_count
                    ),
                }
            }
            Err(error) => failed("launch-project", error),
        },
        Err(error) => failed("launch-project", error),
    }
}

fn post_add_object(
    client: &ureq::Agent,
    url: &str,
    class_name: &str,
    instance_name: &str,
    last_count: &mut usize,
) -> StepResult {
    match client
        .post(url)
        .send_json(ureq::json!({ "className": class_name, "name": instance_name }))
    {
        Ok(resp) => match resp.into_json::<AddObjectResponse>() {
            Ok(body) => {
                *last_count = body.scene_field_count_after;
                StepResult {
                    name: format!("add({instance_name})"),
                    ok: matches!(body.status.as_str(), "ok" | "added"),
                    msg: format!("objects={}", body.scene_field_count_after),
                }
            }
            Err(error) => failed("add-object", error),
        },
        Err(error) => failed("add-object", error),
    }
}

fn post_edit(client: &ureq::Agent, base: &str, method_name: &str, edit_spec: &str) -> StepResult {
    match client
        .post(&format!("{base}/api/code/edit-procedure"))
        .send_json(ureq::json!({
            "procedureSelector": format!("scene.{method_name}"),
            "editSpec": edit_spec,
        })) {
        Ok(resp) => match resp.into_json::<StatusResponse>() {
            Ok(body) => StepResult {
                name: format!("edit({method_name})"),
                ok: matches!(body.status.as_str(), "ok" | "proved"),
                msg: body.status,
            },
            Err(error) => failed("edit", error),
        },
        Err(error) => failed("edit", error),
    }
}

fn post_run_world(client: &ureq::Agent, base: &str, last_count: &mut usize) -> StepResult {
    match client
        .post(&format!("{base}/api/world/run"))
        .send_json(ureq::json!({}))
    {
        Ok(resp) => match resp.into_json::<RunWorldResponse>() {
            Ok(body) => {
                *last_count = body.scene_object_count;
                StepResult {
                    name: "run".into(),
                    ok: matches!(body.status.as_str(), "ok" | "completed"),
                    msg: format!("objects={}", body.scene_object_count),
                }
            }
            Err(error) => failed("run", error),
        },
        Err(error) => failed("run", error),
    }
}

fn post_save(client: &ureq::Agent, base: &str, path: &str) -> StepResult {
    match client
        .post(&format!("{base}/api/project/save"))
        .send_json(ureq::json!({ "targetPath": path }))
    {
        Ok(resp) => match resp.into_json::<StatusResponse>() {
            Ok(body) => StepResult {
                name: format!("save({path})"),
                ok: matches!(body.status.as_str(), "ok" | "saved"),
                msg: body.status,
            },
            Err(error) => failed("save", error),
        },
        Err(error) => failed("save", error),
    }
}

fn post_reopen(
    client: &ureq::Agent,
    base: &str,
    path: &str,
    saved_path: Option<&str>,
    expected_count: usize,
    last_count: &mut usize,
) -> StepResult {
    match client
        .post(&format!("{base}/api/project/reopen"))
        .send_json(ureq::json!({ "project": path }))
    {
        Ok(resp) => match resp.into_json::<LaunchResponse>() {
            Ok(body) => {
                *last_count = body.scene_object_count;
                StepResult {
                    name: format!("reopen({path})"),
                    ok: matches!(body.status.as_str(), "ok" | "reopened" | "launched")
                        && saved_path == Some(path)
                        && body
                            .project
                            .as_deref()
                            .is_some_and(|project| project.ends_with(path))
                        && body.scene_object_count == expected_count,
                    msg: format!(
                        "project={:?} restored_objects={} expected_objects={expected_count}",
                        body.project, body.scene_object_count
                    ),
                }
            }
            Err(error) => failed("reopen", error),
        },
        Err(error) => failed("reopen", error),
    }
}
fn get_typescript_export(client: &ureq::Agent, base: &str) -> StepResult {
    match get_typescript_export_bytes(client, base) {
        Ok((content_type, bytes)) => StepResult {
            name: "export-typescript".into(),
            ok: content_type.contains("application/zip") && !bytes.is_empty(),
            msg: format!("content_type={content_type} bytes={}", bytes.len()),
        },
        Err(error) => StepResult {
            name: "export-typescript".into(),
            ok: false,
            msg: error,
        },
    }
}

fn get_typescript_export_contains(
    client: &ureq::Agent,
    base: &str,
    snippets: &[String],
) -> StepResult {
    if snippets.is_empty() {
        return StepResult {
            name: "assert-edited-procedure-persisted".into(),
            ok: false,
            msg: "no edited procedure snippets were recorded before reopen".into(),
        };
    }
    let (_, bytes) = match get_typescript_export_bytes(client, base) {
        Ok(export) => export,
        Err(error) => {
            return StepResult {
                name: "assert-edited-procedure-persisted".into(),
                ok: false,
                msg: error,
            };
        }
    };
    match zip_text(&bytes) {
        Ok(text) => {
            let missing = snippets
                .iter()
                .filter(|snippet| !text.contains(snippet.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            StepResult {
                name: "assert-edited-procedure-persisted".into(),
                ok: missing.is_empty(),
                msg: format!("missing={missing:?}"),
            }
        }
        Err(error) => StepResult {
            name: "assert-edited-procedure-persisted".into(),
            ok: false,
            msg: error,
        },
    }
}

fn get_typescript_export_bytes(
    client: &ureq::Agent,
    base: &str,
) -> Result<(String, Vec<u8>), String> {
    match client
        .get(&format!("{base}/api/projects/current/export/typescript"))
        .call()
    {
        Ok(resp) => {
            let content_type = resp.header("content-type").unwrap_or("").to_string();
            let mut bytes = Vec::new();
            resp.into_reader()
                .read_to_end(&mut bytes)
                .map_err(|error| error.to_string())?;
            Ok((content_type, bytes))
        }
        Err(error) => Err(error.to_string()),
    }
}

fn zip_text(bytes: &[u8]) -> Result<String, String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    let mut combined = String::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if !entry.name().ends_with(".ts") && !entry.name().ends_with(".js") {
            continue;
        }
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|error| error.to_string())?;
        combined.push_str(&text);
        combined.push('\n');
    }
    Ok(combined)
}

fn persisted_snippet_from_edit_spec(edit_spec: &str) -> String {
    let method_name = edit_spec_identifier(
        edit_spec
            .strip_prefix("append-comment:")
            .unwrap_or(edit_spec),
    );
    format!("scene.call(\"this\", \"{}\"", method_name)
}

fn edit_spec_identifier(value: &str) -> String {
    value
        .strip_prefix("append-comment:")
        .unwrap_or(value)
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                lower_first(part)
            } else {
                upper_first(part)
            }
        })
        .collect::<String>()
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn upper_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn failed(label: impl Into<String>, error: impl std::fmt::Display) -> StepResult {
    StepResult {
        name: label.into(),
        ok: false,
        msg: error.to_string(),
    }
}

fn assert_order(scenario: &Scenario, before: &str, after: &str) {
    let before_index = step_index(scenario, before);
    let after_index = step_index(scenario, after);
    assert!(
        before_index < after_index,
        "{} should perform {before} before {after}",
        scenario.id
    );
}

fn step_index(scenario: &Scenario, label: &str) -> usize {
    scenario
        .steps
        .iter()
        .position(|step| step_matches(step, label))
        .unwrap_or_else(|| panic!("{} missing {label}", scenario.id))
}

fn has_step(scenario: &Scenario, label: &str) -> bool {
    scenario.steps.iter().any(|step| step_matches(step, label))
}

fn step_matches(step: &Step, label: &str) -> bool {
    matches!(
        (step, label),
        (Step::AddObject { .. }, "add-object")
            | (Step::EditProcedure { .. }, "edit")
            | (Step::RunWorld, "run")
            | (Step::Save { .. }, "save")
            | (Step::Reopen { .. }, "reopen")
            | (Step::ExportTypeScript, "export")
            | (Step::AssertEditedProcedurePersisted, "verify-edit")
            | (Step::AssertMinObjects { .. }, "verify")
    )
}
