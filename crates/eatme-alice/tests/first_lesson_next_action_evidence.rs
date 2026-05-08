use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, assert_contains,
    overwrite_modernized_first_lesson_next_action, write_manifest,
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
