use eatme_alice::check_lesson_session_readiness;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, write_manifest,
};

#[test]
fn manifest_only_readiness_emits_contract_evidence_and_diagnostics() {
    let root = scratch_root("manifest-only-readiness-contract");
    let manifest_path = write_manifest_only_first_lesson_contract(&root);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(report_json["passed"], false);
    assert_eq!(report_json["status"], "not_ready");
    assert_contract_evidence_state(&report_json, "comparison_manifest", "present");
    assert_contract_evidence_state(&report_json, "execute_requested", "missing");
    assert_contract_evidence_state(&report_json, "baseline.launch_manifest", "missing");
    assert_contract_evidence_state(&report_json, "modernized.launch_manifest", "missing");
    assert_diagnostic_code(&report_json, "execution_not_requested");
    assert_diagnostic_code(&report_json, "missing_target_evidence");
}

#[test]
fn blocked_but_structurally_valid_readiness_has_no_error_diagnostics() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[
            "place_object_ui_action",
            "edit_procedure_ui_action",
            "run_world_ui_action",
            "save_project_ui_action",
        ],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(report_json["passed"], true, "{report_json}");
    assert_eq!(report_json["status"], "blocked");
    assert_no_error_diagnostics(&report_json);
    for id in [
        "comparison_manifest",
        "baseline.launch_manifest",
        "modernized.launch_manifest",
        "baseline.ui_action_contract",
        "modernized.ui_action_contract",
        "modernized.desktop_pixel_observation",
    ] {
        assert_contract_evidence_state(&report_json, id, "present");
    }
    assert_contract_evidence_state(&report_json, "first_lesson_completion", "blocked");
}

#[test]
fn incomplete_ui_action_contract_reports_stable_missing_action_diagnostic() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    remove_required_action(&manifest_path, "modernized", "save-project");

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(report_json["passed"], false);
    let diagnostic = diagnostic_with_code(&report_json, "missing_required_action");
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(
        diagnostic["field"],
        "targets.modernized.ui_action_contract.required_actions"
    );
    assert_eq!(diagnostic["expected"], "save-project");
    assert_contract_evidence_state(
        &report_json,
        "modernized.required_action.save-project",
        "missing",
    );
}

fn write_manifest_only_first_lesson_contract(root: &Path) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    let manifest_path = root.join("comparison-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "eatme.alice-comparison/v1",
            "scenario_id": "first-lessons-real-ui-actions",
            "execute_requested": false,
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

fn remove_required_action(manifest_path: &Path, role: &str, action_id: &str) {
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let contract_path = manifest["targets"][role]["launch_manifest"]["ui_action_contract"]["path"]
        .as_str()
        .unwrap();
    let mut contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).unwrap()).unwrap();
    contract["required_actions"]
        .as_array_mut()
        .unwrap()
        .retain(|action| action["id"] != action_id);
    fs::write(contract_path, serde_json::to_vec_pretty(&contract).unwrap()).unwrap();
}

fn assert_diagnostic_code(report: &serde_json::Value, code: &str) {
    diagnostic_with_code(report, code);
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
        "valid blocked report should not emit error diagnostics: {report}"
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
