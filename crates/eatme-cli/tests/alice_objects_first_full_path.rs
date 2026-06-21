use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn alice_help_discovers_objects_first_full_path_command() {
    let output = Command::new(eatme_bin())
        .args(["alice", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme alice --help");

    assert_exit_code(&output, 0);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("objects-first-full-path"),
        "alice help must expose the full executable objects-first path command; stdout:\n{stdout}"
    );
}

#[test]
fn objects_first_full_path_help_documents_canonical_scenario() {
    let output = Command::new(eatme_bin())
        .args(["alice", "objects-first-full-path", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme alice objects-first-full-path --help");

    assert_exit_code(&output, 0);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("alice-objects-first-full-path"),
        "command help must name the canonical scenario id; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("assets/scenarios/eatme/alice-objects-first-full-path.yaml"),
        "command help must point users at the canonical scenario asset; stdout:\n{stdout}"
    );
}

#[test]
fn objects_first_full_path_requires_real_alice_gate_before_launching() {
    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "objects-first-full-path",
            "--alice-home",
            "/definitely/not/alice",
            "--run-id",
            "gate-check",
            "--runs-dir",
            "target/test-work/alice-objects-first-cli",
            "--json",
            "--no-memory",
            "--offline-package",
        ])
        .env_remove("EATME_REAL_ALICE")
        .current_dir(workspace_root())
        .output()
        .expect("run gated objects-first command");

    assert_exit_code(&output, 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("EATME_REAL_ALICE=1"),
        "real Alice gate must fail before discovery/package work; stderr:\n{stderr}"
    );
}

#[test]
fn objects_first_full_path_command_binds_to_canonical_scenario() {
    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "objects-first-full-path",
            "--alice-home",
            "/definitely/not/alice",
            "--run-id",
            "canonical-scenario-check",
            "--runs-dir",
            "target/test-work/alice-objects-first-cli",
            "--json",
            "--no-memory",
            "--offline-package",
        ])
        .env("EATME_REAL_ALICE", "1")
        .current_dir(workspace_root())
        .output()
        .expect("run objects-first command with explicit gate");

    assert_exit_code(&output, 1);
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("alice-objects-first-full-path"),
        "command must delegate to scenario alice-objects-first-full-path even when Alice discovery fails; output:\n{combined}"
    );
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
