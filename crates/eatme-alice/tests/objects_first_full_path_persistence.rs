use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[path = "launch_smoke_support.rs"]
#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

const SCENARIO_ID: &str = "alice-objects-first-full-path";

#[test]
fn save_reopen_verification_fails_when_object_state_changes_after_reopen() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    write_full_path_hooks(
        &fixture,
        reopened_state("bunny-2", "world.myFirstMethod", 1.25, 1.0),
    );
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "mismatch-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("persistence_assertion_failed"),
        "saved/reopened object mismatch must be a hard failure"
    );
    let contract = read_json(fixture.root.join(format!(
        "runs/{SCENARIO_ID}/mismatch-run/objects-first-full-path-contract.json"
    )));
    assert_eq!(
        assertion_status(&contract, "object_identity_persisted"),
        "failed"
    );
    assert_eq!(
        contract["project_state"]["before_reopen"]["object_id"],
        "bunny-1"
    );
    assert_eq!(
        contract["project_state"]["after_reopen"]["object_id"],
        "bunny-2"
    );
}

#[test]
fn save_reopen_verification_passes_only_when_object_transform_procedure_and_run_persist() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    write_full_path_hooks(
        &fixture,
        reopened_state("bunny-1", "world.myFirstMethod", 1.25, 1.0),
    );
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "persistence-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category, None,
        "complete fake hook chain with matching reopened state must pass"
    );
    for assertion in [
        "saved_project_artifact_exists",
        "reopened_same_saved_project",
        "object_identity_persisted",
        "object_transform_persisted",
        "movement_procedure_persisted",
        "world_run_completed_before_save",
    ] {
        assert!(
            manifest.assertions[assertion].passed,
            "manifest assertion {assertion} must pass: {:?}",
            manifest.assertions[assertion]
        );
    }

    let contract = read_json(fixture.root.join(format!(
        "runs/{SCENARIO_ID}/persistence-run/objects-first-full-path-contract.json"
    )));
    for assertion in [
        "object_identity_persisted",
        "object_transform_persisted",
        "movement_procedure_persisted",
    ] {
        assert_eq!(assertion_status(&contract, assertion), "passed");
    }
}

fn assertion_status<'a>(contract: &'a Value, id: &str) -> &'a str {
    contract["persistence_assertions"]
        .as_array()
        .expect("persistence assertions array")
        .iter()
        .find(|assertion| assertion["id"] == id)
        .unwrap_or_else(|| panic!("missing persistence assertion {id}: {contract}"))["status"]
        .as_str()
        .expect("assertion status is string")
}

fn reopened_state(
    object_id: &str,
    procedure_selector: &str,
    transform_x: f64,
    movement_distance: f64,
) -> Value {
    json!({
        "object_id": object_id,
        "procedure_selector": procedure_selector,
        "transform": {"x": transform_x, "y": 0.0, "z": -0.5, "yaw_degrees": 35.0, "scale": 1.2},
        "movement": {"operation": "move", "direction": "forward", "distance_meters": movement_distance},
        "last_run": {"completed": true}
    })
}

fn write_full_path_hooks(fixture: &TestFixture, reopened_state: Value) {
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
printf '%s\n' '{"object_id":"bunny-1","transform":{"x":1.25,"y":0.0,"z":-0.5,"yaw_degrees":35.0,"scale":1.2}}' > "$evidence_dir/object-transform.json"
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
printf '%s\n' '{"object_id":"bunny-1","procedure_selector":"world.myFirstMethod","transform":{"x":1.25,"y":0.0,"z":-0.5,"yaw_degrees":35.0,"scale":1.2},"movement":{"operation":"move","direction":"forward","distance_meters":1.0},"last_run":{"completed":true}}' > "$evidence_dir/project-state-before-reopen.json"
printf '%s\n' '{"schema_version":"eatme.alice-project-save-result/v1","status":"saved","save_selector":"objects-first-full-path","saved_project_artifact":"saved-project.a3p","save_artifact":"project-state-before-reopen.json"}'
"#,
    );

    let reopen_script = format!(
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
printf '%s\n' '{{"opened":true}}' > "$evidence_dir/reopen.json"
cat > "$evidence_dir/project-state-after-reopen.json" <<'JSON'
{}
JSON
printf '%s\n' '{{"schema_version":"eatme.alice-project-reopen-result/v1","status":"reopened","source_saved_project_artifact":"project-save/saved-project.a3p","reopen_selector":"objects-first-full-path","reopened_project_artifact":"reopened-project.a3p","reopen_artifact":"reopen.json","reopened_state_artifact":"project-state-after-reopen.json","state_verification":"matched"}}'
"#,
        serde_json::to_string_pretty(&reopened_state).unwrap()
    );
    write_tool(
        &fixture.alice_home.join("tools/eatme-reopen-project"),
        &reopen_script,
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
