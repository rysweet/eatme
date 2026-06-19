use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn deps_check_cli_json_reports_dependency_surface() {
    let output = Command::new(eatme_bin())
        .args(["deps", "check", "--json"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme deps check --json");

    assert_exit_code(&output, 0);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("dependency report is JSON");

    assert!(
        report["tools"].is_object(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    for tool in [
        "git", "java", "mvn", "Xvfb", "xdpyinfo", "wmctrl", "xwininfo", "xdotool",
    ] {
        assert!(
            report["tools"][tool].is_boolean(),
            "missing boolean tool entry {tool:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    assert!(report["screenshot_available"].is_boolean());
    assert!(report["all_required_available"].is_boolean());
    assert_eq!(
        report["tools"]["git"], true,
        "git should be available in the test environment"
    );
}

#[test]
fn committed_assets_validate_and_gadugi_check_pass_end_to_end() {
    let scenario_path =
        workspace_root().join("assets/scenarios/eatme/events-collision-proximity-game.yaml");

    let validate = Command::new(eatme_bin())
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets validate --json --path");

    assert_exit_code(&validate, 0);
    let validation_report: serde_json::Value =
        serde_json::from_slice(&validate.stdout).expect("asset validation report is JSON");
    assert_eq!(
        validation_report["passed"],
        true,
        "stdout: {}",
        String::from_utf8_lossy(&validate.stdout)
    );

    let gadugi_check = Command::new(eatme_bin())
        .args(["assets", "generate-gadugi", "--check", "--json", "--root"])
        .arg(workspace_root())
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets generate-gadugi --check --json");

    assert_exit_code(&gadugi_check, 0);
    let gadugi_report: serde_json::Value =
        serde_json::from_slice(&gadugi_check.stdout).expect("gadugi check report is JSON");
    assert_eq!(
        gadugi_report["passed"],
        true,
        "stdout: {}",
        String::from_utf8_lossy(&gadugi_check.stdout)
    );
    assert!(gadugi_report["changed"].is_array());
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
