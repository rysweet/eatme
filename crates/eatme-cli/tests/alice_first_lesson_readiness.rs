use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn run_first_lesson_readiness_cli_reports_incomplete_manifest_only_sequence() {
    let root = scratch_root("first-lesson-readiness-cli");
    let registry_path = root.join("targets.yaml");
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-sequence",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    assert_eq!(
        report["schema_version"],
        "eatme.first-lesson-readiness-sequence/v1"
    );
    assert_eq!(report["passed"], false);
    assert_eq!(report["readiness_status"], "incomplete");
    assert_eq!(report["evidence_progress"]["total_required"], 7);
    assert!(
        report["evidence_progress"]["summary"]
            .as_str()
            .unwrap()
            .contains("required evidence items are present")
    );
    assert!(report["readiness_report"]["evidence_progress"]["items"].is_array());
    assert!(
        report["comparison_manifest_path"]
            .as_str()
            .unwrap()
            .ends_with("comparison-manifest.json")
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
