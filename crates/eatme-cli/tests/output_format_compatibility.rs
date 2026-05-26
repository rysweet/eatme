use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn deps_check_json_stdout_stays_machine_readable_for_ci() {
    let output = Command::new(eatme_bin())
        .args(["deps", "check", "--json"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme deps check --json");

    assert_exit_code(&output, 0);
    assert_json_stdout_contract(&output.stdout);
    assert!(
        output.stderr.is_empty(),
        "successful machine-readable output should keep stderr empty: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_single_json_document(&output.stdout);
    assert!(report["tools"].is_object(), "stdout: {report}");
    assert!(
        report["all_required_available"].is_boolean(),
        "stdout: {report}"
    );
}

#[test]
fn grading_report_failure_keeps_stdout_json_for_dashboards() {
    let output = Command::new(eatme_bin())
        .args(["assets", "grading-report", "--json", "--path"])
        .arg(workspace_root())
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets grading-report --json --path");

    assert_exit_code(&output, 1);
    assert_json_stdout_contract(&output.stdout);
    assert_no_ansi(&output.stderr, "stderr");

    let report = parse_single_json_document(&output.stdout);
    assert_eq!(report["schema_version"], "eatme.assets/grading/v1");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("not all steps ready"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn first_lesson_plain_output_stays_newline_delimited_for_dashboards() {
    let root = scratch_root("output-format-compatibility");
    let registry_path = write_registry(&root);

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "output-format-compatibility",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
        ])
        .output()
        .expect("run eatme alice run-first-lesson-readiness");

    assert_exit_code(&output, 1);
    assert_plain_stdout_contract(&output.stdout);
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with("First-lesson/grading gap report:"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Shown:\n"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("first-lesson readiness sequence incomplete"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn first_lesson_json_output_stays_machine_readable_for_dashboards() {
    let root = scratch_root("output-format-json-compatibility");
    let registry_path = write_registry(&root);

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--json",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "output-format-json-compatibility",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
        ])
        .output()
        .expect("run eatme alice run-first-lesson-readiness --json");

    assert_exit_code(&output, 1);
    assert_json_stdout_contract(&output.stdout);
    assert_no_ansi(&output.stderr, "stderr");

    let report = parse_single_json_document(&output.stdout);
    assert_eq!(
        report["schema_version"],
        serde_json::Value::String("eatme.first-lesson-readiness-sequence/v1".into())
    );
    assert!(report["not_yet_shown"].is_array(), "stdout: {report}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("first-lesson readiness sequence incomplete"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_json_stdout_contract(stdout: &[u8]) {
    assert!(stdout.ends_with(b"\n"), "stdout should end with a newline");
    assert_no_ansi(stdout, "stdout");
    let _ = parse_single_json_document(stdout);
}

fn assert_plain_stdout_contract(stdout: &[u8]) {
    assert!(stdout.ends_with(b"\n"), "stdout should end with a newline");
    assert_no_ansi(stdout, "stdout");
    let text = String::from_utf8(stdout.to_vec()).expect("stdout must be UTF-8");
    assert!(
        !text.trim_start().starts_with('{'),
        "plain-text contract should not start with JSON: {text}"
    );
}

fn assert_no_ansi(stream: &[u8], label: &str) {
    let text = String::from_utf8_lossy(stream);
    assert!(
        !text.contains('\u{1b}'),
        "{label} should not contain ANSI escapes: {text}"
    );
}

fn parse_single_json_document(stdout: &[u8]) -> serde_json::Value {
    let text = String::from_utf8(stdout.to_vec()).expect("stdout must be UTF-8");
    let mut documents = serde_json::Deserializer::from_str(&text).into_iter::<serde_json::Value>();
    let value = documents
        .next()
        .expect("stdout should contain one JSON document")
        .expect("stdout should contain valid JSON");
    assert!(
        documents.next().is_none(),
        "stdout should contain a single JSON document: {text}"
    );
    value
}

fn write_registry(root: &Path) -> PathBuf {
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
    registry_path
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
