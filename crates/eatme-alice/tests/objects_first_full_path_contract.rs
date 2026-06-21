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
fn full_path_contract_requires_transform_and_movement_procedure_evidence() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    write_full_path_hooks(&fixture);
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "contract-run".into(),
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
        "full path must pass when all fake action hooks return structured proof"
    );
    let contract = read_json(fixture.root.join(format!(
        "runs/{SCENARIO_ID}/contract-run/objects-first-full-path-contract.json"
    )));

    assert_eq!(
        contract["schema_version"],
        "eatme.alice-objects-first-full-path/v1"
    );
    assert_phase_passed(&contract, "place-object");
    assert_phase_passed(&contract, "transform-object");
    assert_eq!(
        contract["phases"]["transform-object"]["transform"]["x"], 1.25,
        "transform evidence should be loaded from the transform artifact without reparsing stdout later"
    );
    assert_phase_passed(&contract, "edit-movement-procedure");
    assert_eq!(
        contract["phases"]["edit-movement-procedure"]["procedure_selector"],
        "world.myFirstMethod"
    );
    assert_eq!(
        contract["phases"]["edit-movement-procedure"]["movement"]["object_id"],
        contract["phases"]["place-object"]["object_id"],
        "movement procedure must target the placed object identity"
    );
    assert_eq!(
        contract["phases"]["edit-movement-procedure"]["movement"]["operation"],
        "move"
    );
    assert!(
        contract["phases"]["edit-movement-procedure"]["movement"]["distance_meters"]
            .as_f64()
            .is_some_and(|distance| distance > 0.0),
        "movement procedure must encode positive motion semantics"
    );
}

#[test]
fn comment_only_procedure_edit_is_not_enough_for_full_path() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    write_transform_hook(&fixture);
    write_comment_only_edit_hook(&fixture);
    write_run_save_reopen_hooks(&fixture);
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "comment-only-run".into(),
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
        Some("movement_procedure_missing"),
        "comment-only edits must be a hard failure for the full executable path"
    );
    assert!(
        !manifest.assertions["edit_movement_procedure"].passed,
        "manifest must expose the failed movement procedure assertion"
    );
}

fn assert_phase_passed(contract: &Value, phase: &str) {
    assert_eq!(
        contract["phases"][phase]["status"], "passed",
        "phase {phase} must pass in contract: {contract}"
    );
}

fn write_full_path_hooks(fixture: &TestFixture) {
    write_transform_hook(fixture);
    write_movement_edit_hook(fixture);
    write_run_save_reopen_hooks(fixture);
}

fn write_transform_hook(fixture: &TestFixture) {
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
}

fn write_movement_edit_hook(fixture: &TestFixture) {
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
}

fn write_comment_only_edit_hook(fixture: &TestFixture) {
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
printf '%s\n' '{"comment":"not movement"}' > "$evidence_dir/procedure-diff.json"
printf '%s\n' '{"schema_version":"eatme.alice-procedure-edit-result/v1","status":"edited","procedure_selector":"scene.myFirstMethod","edited_project_artifact":"edited-project.a3p","procedure_or_code_diff":"procedure-diff.json"}'
"#,
    );
}

fn write_run_save_reopen_hooks(fixture: &TestFixture) {
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
printf '%s\n' '{"ran":true,"object_id":"bunny-1","position_after":{"x":2.25,"y":0.0,"z":-0.5}}' > "$evidence_dir/world-run.json"
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
printf '%s\n' '{"object_id":"bunny-1","procedure_selector":"world.myFirstMethod","transform":{"x":1.25,"y":0.0,"z":-0.5},"movement":{"operation":"move","distance_meters":1.0}}' > "$evidence_dir/project-state-before-reopen.json"
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
printf '%s\n' '{"object_id":"bunny-1","procedure_selector":"world.myFirstMethod","transform":{"x":1.25,"y":0.0,"z":-0.5},"movement":{"operation":"move","distance_meters":1.0}}' > "$evidence_dir/project-state-after-reopen.json"
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
