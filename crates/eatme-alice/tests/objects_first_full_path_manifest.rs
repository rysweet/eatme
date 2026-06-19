use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[path = "launch_smoke_support.rs"]
#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

const SCENARIO_ID: &str = "alice-objects-first-full-path";

#[test]
fn full_path_manifest_contains_command_scenario_phase_artifacts_and_screenshot_status() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    write_full_path_hooks(&fixture);
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "manifest-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .unwrap();

    assert_eq!(manifest.failure_category, None);
    let manifest_path = fixture
        .root
        .join(format!("runs/{SCENARIO_ID}/manifest-run/manifest.json"));
    let manifest_json = read_json(&manifest_path);

    assert_eq!(manifest_json["schema_version"], "eatme.launch-smoke/v1");
    assert_eq!(manifest_json["scenario_id"], SCENARIO_ID);
    assert_eq!(
        manifest_json["command"]["argv"],
        serde_json::json!(["eatme", "alice", "objects-first-full-path"])
    );
    assert_eq!(
        manifest_json["scenario"]["path"],
        "assets/scenarios/eatme/alice-objects-first-full-path.yaml"
    );
    assert_eq!(
        manifest_json["evidence"]["screenshot"]["status"], "captured",
        "screenshot availability must be recorded explicitly"
    );

    for artifact in [
        "ui_action_contract",
        "objects_first_full_path_contract",
        "object_placement",
        "object_transform",
        "procedure_edit",
        "world_run",
        "project_state_before_reopen",
        "project_reopen",
        "project_state_after_reopen",
    ] {
        assert!(
            manifest_json["evidence"]["phase_artifacts"][artifact]["path"].is_string(),
            "manifest evidence must include phase artifact {artifact}: {manifest_json}"
        );
    }

    assert_eq!(
        manifest_json["persistence_assertions"]["object_identity_persisted"]["passed"],
        true
    );
    assert_eq!(
        manifest_json["persistence_assertions"]["movement_procedure_persisted"]["passed"],
        true
    );
}

fn write_full_path_hooks(fixture: &TestFixture) {
    write_tool(
        &fixture.alice_home.join("tools/eatme-transform-object"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"object_id":"bunny-1","transform":{"x":1.25,"y":0.0,"z":-0.5}}' > "$evidence_dir/object-transform.json"
printf '%s\n' 'transformed project' > "$evidence_dir/transformed-object-project.a3p"
printf '%s\n' '{"schema_version":"eatme.alice-object-transform-result/v1","status":"transformed","object_id":"bunny-1","transform_artifact":"object-transform.json","transformed_project_artifact":"transformed-object-project.a3p"}'
"#,
    );
    write_tool(
        &fixture.alice_home.join("tools/eatme-edit-procedure"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' 'edited project' > "$evidence_dir/edited-project.a3p"
printf '%s\n' '{"procedure_selector":"world.myFirstMethod","movement":{"object_id":"bunny-1","operation":"move","direction":"forward","distance_meters":1.0}}' > "$evidence_dir/procedure-diff.json"
printf '%s\n' '{"schema_version":"eatme.alice-procedure-edit-result/v2","status":"edited","procedure_selector":"world.myFirstMethod","edit_kind":"movement","object_id":"bunny-1","movement":{"operation":"move","direction":"forward","distance_meters":1.0},"edited_project_artifact":"edited-project.a3p","procedure_or_code_diff":"procedure-diff.json"}'
"#,
    );
    write_tool(
        &fixture.alice_home.join("tools/eatme-run-world"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"completed":true,"object_id":"bunny-1"}' > "$evidence_dir/world-run.json"
printf '%s\n' 'runtime log' > "$evidence_dir/runtime.log"
printf '%s\n' '{"schema_version":"eatme.alice-world-run-result/v1","status":"ran","run_selector":"world.myFirstMethod","run_artifact":"world-run.json","runtime_or_log_evidence":"runtime.log"}'
"#,
    );
    write_tool(
        &fixture.alice_home.join("tools/eatme-save-project"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' 'saved project' > "$evidence_dir/saved-project.a3p"
printf '%s\n' '{"object_id":"bunny-1","procedure_selector":"world.myFirstMethod","transform":{"x":1.25,"y":0.0,"z":-0.5},"movement":{"operation":"move","direction":"forward","distance_meters":1.0},"last_run":{"completed":true}}' > "$evidence_dir/project-state-before-reopen.json"
printf '%s\n' '{"schema_version":"eatme.alice-project-save-result/v1","status":"saved","save_selector":"objects-first-full-path","saved_project_artifact":"saved-project.a3p","save_artifact":"project-state-before-reopen.json"}'
"#,
    );
    write_tool(
        &fixture.alice_home.join("tools/eatme-reopen-project"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
    --saved-project) shift; saved_project="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
cp "$saved_project" "$evidence_dir/reopened-project.a3p"
printf '%s\n' '{"opened":true}' > "$evidence_dir/reopen.json"
printf '%s\n' '{"object_id":"bunny-1","procedure_selector":"world.myFirstMethod","transform":{"x":1.25,"y":0.0,"z":-0.5},"movement":{"operation":"move","direction":"forward","distance_meters":1.0},"last_run":{"completed":true}}' > "$evidence_dir/project-state-after-reopen.json"
printf '%s\n' '{"schema_version":"eatme.alice-project-reopen-result/v1","status":"reopened","source_saved_project_artifact":"project-save/saved-project.a3p","reopen_selector":"objects-first-full-path","reopened_project_artifact":"reopened-project.a3p","reopen_artifact":"reopen.json","reopened_state_artifact":"project-state-after-reopen.json","state_verification":"matched"}'
"#,
    );
}

fn write_tool(path: &Path, script: &str) {
    fs::create_dir_all(path.parent().expect("tool has parent")).unwrap();
    fs::write(path, script).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn read_json(path: impl AsRef<Path>) -> Value {
    let path = path.as_ref();
    let content =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}
