use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

const SCENARIO_ID: &str = "alice-objects-first-world";

#[test]
fn objects_first_scenario_is_a_strict_real_workflow() {
    let scenario = LaunchSmokeScenario::new(SCENARIO_ID);

    assert!(
        scenario.requires_real_ui_actions(),
        "{SCENARIO_ID} must require the real Alice action workflow, not only process launch evidence"
    );
}

#[test]
fn fake_toolchain_objects_first_workflow_requires_all_user_steps_and_persistence() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    write_objects_first_hooks(&fixture.alice_home);
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "objects-first-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .expect("objects-first workflow should run against the fake Alice hooks");

    assert_eq!(manifest.scenario_id, SCENARIO_ID);
    assert_eq!(
        manifest.failure_category, None,
        "a complete fake objects-first run should pass once every workflow proof exists"
    );
    for assertion in [
        "create_or_open_project_ui_action",
        "place_object_ui_action",
        "transform_object_ui_action",
        "edit_movement_procedure_ui_action",
        "run_world_ui_action",
        "save_project_ui_action",
        "reopen_project_ui_action",
        "persisted_state_verified",
        "objects_first_evidence_recorded",
    ] {
        let result = manifest
            .assertions
            .get(assertion)
            .unwrap_or_else(|| panic!("missing objects-first assertion {assertion}"));
        assert!(
            result.passed,
            "{assertion} should pass after the fake hook chain: {}",
            result.detail
        );
    }

    let run_dir = fixture
        .root
        .join("runs")
        .join(SCENARIO_ID)
        .join("objects-first-run");
    let persisted_state = run_dir.join("project-reopen/persisted-state.json");
    assert!(
        persisted_state.is_file(),
        "objects-first workflow must write persisted-state.json under the run evidence directory"
    );
    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&persisted_state).unwrap())
            .expect("persisted-state.json should be valid JSON");
    assert_eq!(state["object"]["name"], "bunny");
    assert_eq!(state["object"]["visible"], true);
    assert_eq!(state["object"]["transform"]["position"]["x"], 1.5);
    assert_eq!(state["procedure"]["movement"]["object"], "bunny");
    assert_eq!(state["procedure"]["movement"]["method"], "move");
    assert_eq!(state["world_run"]["status"], "ran");
}

#[test]
fn objects_first_evidence_paths_stay_under_the_run_directory() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    write_objects_first_hooks(&fixture.alice_home);
    let _path_override = PathOverride::prepend(&fixture.bin);

    let _manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "objects-first-paths".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .expect("objects-first workflow should run against the fake Alice hooks");

    let run_dir = fixture
        .root
        .join("runs")
        .join(SCENARIO_ID)
        .join("objects-first-paths");
    let contract_path = run_dir.join("ui-action-contract.json");
    assert!(
        contract_path.is_file(),
        "objects-first workflow must write ui-action-contract.json at {}",
        contract_path.display()
    );
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap())
            .expect("ui-action-contract.json should be valid JSON");

    let mut failures = Vec::new();
    collect_artifact_paths(&contract, &mut |path| {
        let relative = Path::new(path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            failures.push(format!("{path} is not a simple relative artifact path"));
            return;
        }
        if !run_dir.join(relative).exists() {
            failures.push(format!("{path} does not exist under {}", run_dir.display()));
        }
    });

    assert!(
        failures.is_empty(),
        "objects-first evidence artifacts must be relative files under the run directory:\n{}",
        failures.join("\n")
    );
}

fn collect_artifact_paths(value: &serde_json::Value, visit: &mut impl FnMut(&str)) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(path) = map.get("path").and_then(serde_json::Value::as_str) {
                visit(path);
            }
            for nested in map.values() {
                collect_artifact_paths(nested, visit);
            }
        }
        serde_json::Value::Array(items) => {
            for nested in items {
                collect_artifact_paths(nested, visit);
            }
        }
        _ => {}
    }
}

fn write_objects_first_hooks(alice_home: &Path) {
    let tools = alice_home.join("tools");
    fs::create_dir_all(&tools).unwrap();
    write_tool(
        &tools.join("eatme-place-object"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"object":"bunny","visible":true}' > "$evidence_dir/placement.json"
printf '%s\n' '{"added":["bunny"]}' > "$evidence_dir/scene.diff.json"
printf 'placed project\n' > "$evidence_dir/placed-project.a3p"
printf '%s\n' '{"schema_version":"eatme.alice-object-placement-result/v1","status":"placed","object_identifier":"alice-gallery://animals/bunny","placement_artifact":"placement.json","scene_or_project_diff":"scene.diff.json"}'
"#,
    );
    write_tool(
        &tools.join("eatme-transform-object"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"object":"bunny","visible":true,"transform":{"position":{"x":1.5,"y":0.0,"z":-2.0},"scale":1.25,"rotation":{"y":45.0}}}' > "$evidence_dir/object-transform.json"
printf 'transformed project\n' > "$evidence_dir/transformed-project.a3p"
printf '%s\n' '{"schema_version":"eatme.alice-object-transform-result/v1","status":"transformed","object_identifier":"alice-gallery://animals/bunny","transform_artifact":"object-transform.json","transformed_project_artifact":"transformed-project.a3p"}'
"#,
    );
    write_tool(
        &tools.join("eatme-edit-procedure"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"procedure":"scene.myFirstMethod","movement":{"object":"bunny","method":"move","direction":"FORWARD","amount":1.0}}' > "$evidence_dir/procedure-movement.json"
printf 'edited project\n' > "$evidence_dir/edited-project.a3p"
printf '%s\n' '{"schema_version":"eatme.alice-first-lesson-code-editor-action-proof-result/v1","status":"edited","procedure_selector":"scene.myFirstMethod","edited_project_artifact":"edited-project.a3p","procedure_or_code_diff":"procedure-movement.json"}'
"#,
    );
    write_tool(
        &tools.join("eatme-run-world"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf '%s\n' '{"status":"ran","procedure":"scene.myFirstMethod","moved_object":"bunny"}' > "$evidence_dir/world-run.json"
printf '%s\n' 'runtime reached bunny.move' > "$evidence_dir/runtime.log"
printf '%s\n' '{"schema_version":"eatme.alice-world-run-result/v1","status":"ran","run_selector":"scene.myFirstMethod","run_artifact":"world-run.json","runtime_or_log_evidence":"runtime.log"}'
"#,
    );
    write_tool(
        &tools.join("eatme-save-project"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf 'saved objects-first world\n' > "$evidence_dir/saved-project.a3p"
printf '%s\n' '{"status":"saved","object":"bunny"}' > "$evidence_dir/project-save.json"
printf '%s\n' '{"schema_version":"eatme.alice-project-save-result/v1","status":"saved","save_selector":"scene.myFirstMethod","saved_project_artifact":"saved-project.a3p","save_artifact":"project-save.json"}'
"#,
    );
    write_tool(
        &tools.join("eatme-reopen-project"),
        r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
printf 'reopened objects-first world\n' > "$evidence_dir/reopened-project.a3p"
printf '%s\n' '{"status":"reopened"}' > "$evidence_dir/project-reopen.json"
printf '%s\n' '{"object":{"name":"bunny","visible":true,"transform":{"position":{"x":1.5,"y":0.0,"z":-2.0},"scale":1.25}},"procedure":{"name":"scene.myFirstMethod","movement":{"object":"bunny","method":"move","direction":"FORWARD","amount":1.0}},"world_run":{"status":"ran"}}' > "$evidence_dir/persisted-state.json"
printf '%s\n' '{"schema_version":"eatme.alice-project-reopen-result/v1","status":"reopened","source_saved_project_artifact":"project-save/saved-project.a3p","reopen_selector":"scene.myFirstMethod","reopened_project_artifact":"reopened-project.a3p","reopen_artifact":"project-reopen.json","reopened_state_artifact":"persisted-state.json","state_verification":"passed"}'
"#,
    );
}

fn write_tool(path: &Path, script: &str) {
    fs::write(path, script).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
