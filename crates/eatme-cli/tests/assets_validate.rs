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
    assert_stdout_contains(&output.stdout, "\"passed\": true");
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
        .args(["assets", "validate", "--path"])
        .arg(&scenario_path)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, "\"passed\": false");
}

#[test]
fn missing_scenario_root_exits_nonzero() {
    let root = scratch_root("missing-scenario-root");
    copy_committed_persona_asset(&root);

    let output = Command::new(eatme_bin())
        .args(["assets", "validate"])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, "\"passed\": false");
    assert_stdout_contains(&output.stdout, "assets/scenarios");
}

#[test]
fn empty_scenario_root_exits_nonzero() {
    let root = scratch_root("empty-scenario-root");
    copy_committed_persona_asset(&root);
    fs::create_dir_all(root.join("assets/scenarios")).unwrap();

    let output = Command::new(eatme_bin())
        .args(["assets", "validate"])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    assert!(!output.status.success());
    assert_stdout_contains(&output.stdout, "\"passed\": false");
    assert_stdout_contains(
        &output.stdout,
        "must contain at least one .yaml or .yml scenario asset",
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
