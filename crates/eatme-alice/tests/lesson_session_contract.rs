use eatme_alice::check_lesson_session_contract;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn missing_lesson_session_contract_emits_structured_diagnostic() {
    let root = scratch_root("missing-session-contract");
    let manifest_path = root.join("comparison-manifest.json");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "eatme.alice-comparison/v1",
            "scenario_id": "first-lessons-real-ui-actions"
        }))
        .unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_contract(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(report_json["passed"], false);
    let diagnostic = diagnostic_with_code(&report_json, "missing_lesson_session_contract");
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["field"], "lesson_session_contract");
    assert!(
        diagnostic["message"]
            .as_str()
            .is_some_and(|message| message.contains("missing lesson_session_contract")),
        "diagnostic should explain the missing contract: {diagnostic}"
    );
    assert_contract_evidence_state(&report_json, "lesson_session_contract", "missing");
}

#[test]
fn complete_first_lesson_session_contract_exposes_present_contract_evidence() {
    let root = scratch_root("complete-session-contract");
    let manifest_path = write_complete_first_lesson_contract_manifest(&root);

    let report = check_lesson_session_contract(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(report_json["passed"], true, "{report_json}");
    assert_no_error_diagnostics(&report_json);
    for id in [
        "lesson_session_contract.schema_version",
        "lesson_session_contract.scenario_id",
        "lesson_session_contract.required_session_steps",
        "lesson_session_contract.executable_evidence",
        "lesson_session_contract.boundaries",
    ] {
        assert_contract_evidence_state(&report_json, id, "present");
    }
}

fn write_complete_first_lesson_contract_manifest(root: &Path) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "eatme.alice-comparison/v1",
            "scenario_id": "first-lessons-real-ui-actions",
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
    diagnostics(report)
        .iter()
        .find(|diagnostic| diagnostic["code"] == code)
        .unwrap_or_else(|| panic!("missing diagnostic code {code:?}: {report}"))
}

fn diagnostics(report: &serde_json::Value) -> &[serde_json::Value] {
    report["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level diagnostics[]: {report}"))
}

fn assert_no_error_diagnostics(report: &serde_json::Value) {
    assert!(
        !diagnostics(report)
            .iter()
            .any(|diagnostic| diagnostic["severity"] == "error"),
        "valid contract should not emit error diagnostics: {report}"
    );
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
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/eatme-alice-contract-tests")
        .join(format!("{name}-{nonce}"))
}
