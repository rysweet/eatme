use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn check_lesson_session_cli_json_reports_structured_contract_diagnostics() {
    let root = scratch_root("lesson-session-contract-cli-diagnostics");
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "eatme.alice-comparison/v1",
            "scenario_id": "first-lessons-real-ui-actions"
        }))
        .unwrap(),
    )
    .unwrap();

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "check-lesson-session",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("lesson-session report is JSON");
    assert_eq!(report["passed"], false);
    let diagnostic = diagnostic_with_code(&report, "missing_lesson_session_contract");
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["field"], "lesson_session_contract");
    assert_contract_evidence_state(&report, "lesson_session_contract", "missing");
}

#[test]
fn check_lesson_readiness_cli_json_reports_contract_evidence_for_manifest_only_gap() {
    let root = scratch_root("lesson-readiness-contract-cli-manifest-only");
    let manifest_path = write_complete_first_lesson_contract_manifest(&root, false);

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "check-lesson-readiness",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("lesson-readiness report is JSON");
    assert_eq!(report["passed"], false);
    assert_eq!(report["status"], "not_ready");
    assert_contract_evidence_state(&report, "comparison_manifest", "present");
    assert_contract_evidence_state(&report, "execute_requested", "missing");
    assert_contract_evidence_state(&report, "baseline.launch_manifest", "missing");
    assert_contract_evidence_state(&report, "modernized.launch_manifest", "missing");
    diagnostic_with_code(&report, "execution_not_requested");
    diagnostic_with_code(&report, "missing_target_evidence");
}

#[test]
fn check_lesson_readiness_cli_json_reports_malformed_manifest_diagnostic() {
    let root = scratch_root("lesson-readiness-contract-cli-malformed");
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(&manifest_path, "{not valid json").unwrap();

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "check-lesson-readiness",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("malformed manifest diagnostic is JSON");
    assert_eq!(report["passed"], false);
    let diagnostic = diagnostic_with_code(&report, "malformed_comparison_manifest");
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["field"], "manifest");
}

fn write_complete_first_lesson_contract_manifest(root: &Path, execute_requested: bool) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "eatme.alice-comparison/v1",
            "scenario_id": "first-lessons-real-ui-actions",
            "execute_requested": execute_requested,
            "lesson_session_contract": {
                "schema_version": "eatme.alice-lesson-session-contract/v1",
                "scenario_id": "first-lessons-real-ui-actions",
                "session_kind": "first_lesson_action_contract",
                "automation_status": "action_contract_blocked_until_ui_automation",
                "actor_roles": [
                    "instructor prepares the Alice classroom task",
                    "student opens, changes, runs, saves, and reflects on an Alice project"
                ],
                "required_session_steps": [
                    "instructor selects an Alice lesson objective and starter project",
                    "student opens the configured starter project in Alice",
                    "student places or modifies an object in the scene",
                    "student edits a procedure or code block",
                    "student runs the world and observes the visible result",
                    "student saves the project and records one next revision"
                ],
                "executable_evidence": [
                    "comparison manifest records both target runs under the same scenario id",
                    "target launch manifests record dependency, package, display, window, screenshot, log, and assertion evidence",
                    "automation scenarios name the required actions that are not automated yet"
                ],
                "boundaries": [
                    "does not automate complete instructor assignment creation",
                    "does not automate complete student lesson consumption",
                    "does not perform creative assessment",
                    "does not grade student worlds",
                    "does not prove broad Alice compatibility beyond the selected scenario"
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    manifest_path
}

fn diagnostic_with_code<'a>(report: &'a serde_json::Value, code: &str) -> &'a serde_json::Value {
    report["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level diagnostics[]: {report}"))
        .iter()
        .find(|diagnostic| diagnostic["code"] == code)
        .unwrap_or_else(|| panic!("missing diagnostic code {code:?}: {report}"))
}

fn assert_contract_evidence_state(report: &serde_json::Value, id: &str, expected_state: &str) {
    let evidence = report["contract_evidence"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level contract_evidence[]: {report}"))
        .iter()
        .find(|evidence| evidence["id"] == id)
        .unwrap_or_else(|| panic!("missing contract evidence {id:?}: {report}"));
    assert_eq!(
        evidence["state"], expected_state,
        "unexpected contract evidence state for {id}: {evidence}"
    );
    assert_eq!(evidence["required"], true, "{evidence}");
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
