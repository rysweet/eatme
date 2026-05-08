use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, assert_contains,
    overwrite_modernized_first_lesson_next_action, write_manifest,
    write_modernized_run_window_evidence_file,
};

#[test]
fn blocked_first_lesson_next_action_artifact_reports_missing_ui_action_targets() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::BlockedWithNextAction,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let next_action = modernized
        .desktop_first_lesson_next_action
        .as_ref()
        .expect("modernized target should report first-lesson next-action evidence");
    assert_eq!(next_action.status, "blocked");
    assert_eq!(
        next_action.candidate_actions,
        vec![
            "desktop_save_menu_action",
            "desktop_code_editor_or_procedure_action"
        ]
    );
    assert!(
        next_action
            .requires_next_evidence
            .iter()
            .any(|evidence| evidence.contains("desktop Save menu readiness"))
    );

    let next_blocker = report
        .evidence_progress
        .next_actionable_blocker
        .as_deref()
        .expect("blocked next-action artifact should name the missing UI actions");
    assert!(next_blocker.contains("desktop first-lesson next action is blocked"));
    assert!(next_blocker.contains("desktop Save menu readiness or invocation artifact"));
    assert!(next_blocker.contains("code editor/procedure action readiness or invocation artifact"));
    assert!(next_blocker.contains("desktop_save_menu_action_not_bound"));
    assert!(next_blocker.contains("procedure_editor_action_not_bound"));
    assert!(!next_blocker.contains("desktop save-menu completion is proven"));
    assert!(!next_blocker.contains("first-lesson completion is proven"));
}

#[test]
fn missing_first_lesson_next_action_artifact_is_reported_without_replacing_pixel_blocker() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::BlockedWithNextAction,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let next_action = modernized
        .desktop_first_lesson_next_action
        .as_ref()
        .expect("modernized target should report missing first-lesson next-action evidence");
    assert_eq!(next_action.status, "missing");
    assert!(
        next_action
            .detail
            .contains("desktop-first-lesson-next-action.json")
    );
    assert!(
        report
            .evidence_progress
            .next_actionable_blocker
            .as_deref()
            .unwrap()
            .contains("desktop Run pixel observation is blocked")
    );
}

#[test]
fn invalid_first_lesson_next_action_artifact_is_reported_explicitly() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","source":"desktop_run_render_target_attachment"}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "desktop first-lesson next-action evidence is missing status field",
    );
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let next_action = modernized
        .desktop_first_lesson_next_action
        .as_ref()
        .expect("modernized target should report invalid first-lesson next-action evidence");
    assert_eq!(next_action.status, "invalid");
}

#[test]
fn save_and_select_project_proof_artifacts_present_from_next_action_metadata() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    write_modernized_run_window_evidence_file(
        &manifest_path,
        "save-project-proof.json",
        r#"{"proof":"save-project"}"#,
    );
    write_modernized_run_window_evidence_file(
        &manifest_path,
        "select-project-proof.json",
        r#"{"proof":"select-project"}"#,
    );
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","blocker":{"reason":"Save and Select Project proof artifacts are available, but deterministic follow-on UI actions remain separate evidence."},"save_project_proof_artifact":{"artifact":{"path":"run-window-evidence/save-project-proof.json","size_bytes":24,"sha256":"sha256-save-project-proof","metadata":{"producer":"rabbithole-desktop-proof"}}},"select_project_proof_artifact":{"artifact":{"path":"run-window-evidence/select-project-proof.json","size_bytes":26,"sha256":"sha256-select-project-proof","metadata":{"producer":"rabbithole-desktop-proof"}}},"doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = &report_json["target_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| target["role"] == "modernized")
        .unwrap()["desktop_first_lesson_next_action"];

    assert_eq!(
        next_action["save_project_proof_artifact"]["status"],
        "present"
    );
    assert_eq!(
        next_action["select_project_proof_artifact"]["status"],
        "present"
    );
    let save_item = progress_item(&report_json, "save_project_proof_artifact");
    let select_item = progress_item(&report_json, "select_project_proof_artifact");
    assert_eq!(save_item["state"], "present");
    assert_eq!(select_item["state"], "present");
    assert_eq!(save_item["evidence"], "Save Project proof artifact");
    assert_eq!(select_item["evidence"], "Select Project proof artifact");
    assert_detail_contains(save_item, "run-window-evidence/save-project-proof.json");
    assert_detail_contains(save_item, "24 bytes");
    assert_detail_contains(save_item, "sha256-save-project-proof");
    assert_detail_contains(select_item, "run-window-evidence/select-project-proof.json");
    assert_detail_contains(select_item, "26 bytes");
    assert_detail_contains(select_item, "sha256-select-project-proof");
    assert_no_project_proof_success_claims(save_item);
    assert_no_project_proof_success_claims(select_item);
    let report_text = serde_json::to_string(&report_json).unwrap();
    assert!(
        !report_text.contains("rabbithole-desktop-proof"),
        "project proof metadata must not copy arbitrary metadata values into readiness output: {report_text}"
    );
}

#[test]
fn missing_save_and_select_project_proof_artifacts_are_visible_in_shared_progress() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save_item = progress_item(&report_json, "save_project_proof_artifact");
    let select_item = progress_item(&report_json, "select_project_proof_artifact");

    assert_eq!(save_item["state"], "missing");
    assert_eq!(select_item["state"], "missing");
    assert_detail_contains(save_item, "Save Project proof artifact is missing");
    assert_detail_contains(select_item, "Select Project proof artifact is missing");
    assert_no_project_proof_success_claims(save_item);
    assert_no_project_proof_success_claims(select_item);
}

#[test]
fn blocked_project_proof_artifacts_reuse_known_blocker_details() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","save_project_proof_artifact":{"status":"blocked","blocker":{"reason":"Save dialog owner does not expose a stable proof-artifact handoff yet.","codes":["save_project_artifact_handoff_not_bound"],"debug_secret":"proof-handoff-token"}},"select_project_proof_artifact":{"status":"missing"},"doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save_item = progress_item(&report_json, "save_project_proof_artifact");
    let select_item = progress_item(&report_json, "select_project_proof_artifact");

    assert_eq!(save_item["state"], "blocked");
    assert_eq!(select_item["state"], "missing");
    assert_detail_contains(
        save_item,
        "Save dialog owner does not expose a stable proof-artifact handoff yet.",
    );
    assert_detail_contains(save_item, "save_project_artifact_handoff_not_bound");
    assert_detail_contains(
        select_item,
        "Select Project proof artifact is missing; artifact availability was declared missing.",
    );
    assert_no_project_proof_success_claims(save_item);
    let report_text = serde_json::to_string(&report_json).unwrap();
    assert!(
        !report_text.contains("proof-handoff-token"),
        "project proof blockers must not copy arbitrary blocker fields into readiness output: {report_text}"
    );
}

#[test]
fn blocked_project_proof_artifacts_without_details_report_plain_blocked_state() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","save_project_proof_artifact":{"status":"missing"},"select_project_proof_artifact":{"status":"blocked"},"doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save_item = progress_item(&report_json, "save_project_proof_artifact");
    let select_item = progress_item(&report_json, "select_project_proof_artifact");

    assert_eq!(save_item["state"], "missing");
    assert_eq!(select_item["state"], "blocked");
    assert_detail_contains(
        save_item,
        "Save Project proof artifact is missing; artifact availability was declared missing.",
    );
    assert_detail_contains(select_item, "Select Project proof artifact is blocked");
    assert_no_project_proof_success_claims(select_item);
}

fn progress_item<'a>(report_json: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report_json["evidence_progress"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing evidence_progress item {id}"))
}

fn assert_detail_contains(item: &serde_json::Value, expected: &str) {
    let detail = item["detail"].as_str().unwrap_or_default();
    assert!(
        detail.contains(expected),
        "expected detail to contain {expected:?}; item was {item}"
    );
}

fn assert_no_project_proof_success_claims(item: &serde_json::Value) {
    let text = serde_json::to_string(item).unwrap().to_ascii_lowercase();
    for forbidden in [
        "ui automation succeeded",
        "automation passed",
        "lesson completed",
        "grading occurred",
        "creative assessment passed",
        "creative quality assessed",
        "save project succeeded",
        "select project succeeded",
    ] {
        assert!(
            !text.contains(forbidden),
            "project proof artifact output must not claim {forbidden:?}; item was {item}"
        );
    }
}
