use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn committed_assets_validation_exits_zero() {
    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json"])
        .current_dir(workspace_root())
        .output()
        .unwrap();

    assert_exit_code(&output, 0);
    assert_stdout_contains(&output.stdout, r#""passed": true"#);
}

#[test]
fn malformed_scenario_asset_exits_nonzero() {
    let root = scratch_root("malformed-scenario-asset");
    let scenario_path = root.join("assets/scenarios/eatme/malformed.yaml");
    fs::create_dir_all(scenario_path.parent().unwrap()).unwrap();
    fs::write(
        &scenario_path,
        r#"
schema_version: eatme.scenario/v1
id: "not valid"
title: ""
"#,
    )
    .unwrap();

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, r#""passed": false"#);
}

#[test]
fn missing_scenario_root_exits_nonzero() {
    let root = scratch_root("missing-scenario-root");
    copy_committed_persona_asset(&root);

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, r#""passed": false"#);
    assert_stdout_contains(&output.stdout, "assets/scenarios");
}

#[test]
fn empty_scenario_root_exits_nonzero() {
    let root = scratch_root("empty-scenario-root");
    copy_committed_persona_asset(&root);
    fs::create_dir_all(root.join("assets/scenarios")).unwrap();

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, r#""passed": false"#);
    assert_stdout_contains(
        &output.stdout,
        "must contain at least one .yaml or .yml scenario asset",
    );
}

#[test]
fn assets_validate_rejects_bad_persona_reference_from_real_path() {
    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/persona-root/scenarios/eatme/test_bad_persona.yaml");

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .expect("run eatme-cli assets validate");

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validation output is JSON");
    assert_eq!(report["passed"], false);

    let errors = report["errors"]
        .as_array()
        .expect("errors is an array")
        .iter()
        .filter_map(|error| error.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        errors.contains("missing instructor persona nonexistent-instructor"),
        "errors: {errors}"
    );
}

#[test]
fn assets_validate_accepts_custom_crew_filename_outside_scenarios_dir() {
    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/flexible-root/flows/eatme/test_custom_persona.yaml");

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .expect("run eatme-cli assets validate");

    assert_exit_code(&output, 0);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validation output is JSON");
    assert_eq!(
        report["passed"],
        true,
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn assets_validate_combines_multiple_persona_crews() {
    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/split-root/flows/eatme/test_split_personas.yaml");

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .expect("run eatme-cli assets validate");

    assert_exit_code(&output, 0);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validation output is JSON");
    assert_eq!(
        report["passed"],
        true,
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn assets_validate_reports_malformed_discovered_persona_yaml() {
    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/malformed-root/flows/eatme/test_malformed_persona.yaml");

    let output = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .expect("run eatme-cli assets validate");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parsing persona crew YAML") && stderr.contains("bad-crew.yaml"),
        "stderr: {stderr}"
    );
}

fn scratch_root(name: &str) -> PathBuf {
    let root = workspace_root()
        .join("target/eatme-cli-integration-tests")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn copy_committed_persona_asset(root: &Path) {
    let target = root.join("assets/personas/alice-user-crew.yaml");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(
        workspace_root().join("assets/personas/alice-user-crew.yaml"),
        target,
    )
    .unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn eatme_bin() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_eatme-cli") {
        return PathBuf::from(path);
    }
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("eatme-cli")
}

fn assert_exit_code(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "unexpected status {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stdout_contains(stdout: &[u8], needle: &str) {
    let stdout = String::from_utf8_lossy(stdout);
    assert!(
        stdout.contains(needle),
        "stdout did not contain {needle:?}: {stdout}"
    );
}
