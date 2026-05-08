use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, assert_contains,
    overwrite_modernized_pixel_boundary, overwrite_modernized_pixel_observation, write_manifest,
};

const MISSING_DESKTOP_PROOF: &str = "missing visible desktop rendering evidence after Run-frame and VM statement execution; expected screenshots/run-window-after-dispatch.png under the comparison evidence root";

#[test]
fn readiness_passes_with_visible_run_window_screenshot() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    assert_eq!(
        report.desktop_proof_contract.status,
        "launched_but_unverified"
    );
    assert_eq!(
        report.desktop_proof_contract.reason_code,
        "desktop_pixel_observation_blocked"
    );
    assert!(
        report
            .desktop_proof_contract
            .detail
            .contains("desktop Run pixel-observation evidence is blocked")
    );
    assert_eq!(
        report.required_evidence,
        vec![
            "comparison-manifest.json with baseline and modernized targets",
            "launch evidence for each target",
            "modernized Run-window evidence",
            "modernized desktop-run-pixel-boundary.json status",
            "modernized desktop-run-pixel-observation.json status",
            "modernized desktop execution evidence",
            "screenshot, log, and window artifacts",
            "ui-action-contract.json",
        ]
    );
    assert_eq!(
        report.lesson_session_readiness.required_evidence,
        report.required_evidence
    );
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let pixel_boundary = modernized
        .desktop_run_pixel_boundary
        .as_ref()
        .expect("modernized target should report pixel-boundary evidence status");
    assert_eq!(pixel_boundary.status, "not_observed");
    let pixel_observation = modernized
        .desktop_run_pixel_observation
        .as_ref()
        .expect("modernized target should report pixel-observation evidence status");
    assert_eq!(pixel_observation.status, "blocked");
    assert!(pixel_observation.blocker.is_some());
    assert!(pixel_observation.component_state.is_some());
    assert_eq!(report.evidence_progress.total_required, 8);
    assert_eq!(report.evidence_progress.present, 6);
    assert_eq!(report.evidence_progress.not_observed, 1);
    assert_eq!(report.evidence_progress.blocked, 1);
    assert!(
        report
            .evidence_progress
            .summary
            .contains("6 of 8 required evidence items are present")
    );
    let next_blocker = report
        .evidence_progress
        .next_actionable_blocker
        .as_deref()
        .expect("blocked pixel observation should name the next blocker");
    assert!(next_blocker.contains("desktop Run pixel observation is blocked"));
    assert!(next_blocker.contains("run Alice with a non-headless graphics environment"));
    assert!(next_blocker.contains("make the Run render target displayable"));
    assert!(next_blocker.contains("render_target_not_displayable"));
    assert!(next_blocker.contains("graphicsEnvironmentHeadless=true"));
    assert!(next_blocker.contains("renderTargetWidth=0"));
    assert!(
        pixel_boundary
            .detail
            .contains("does not inspect screenshots or pixel output")
    );
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-observation.json status"
                    && item.state == "blocked"
                    && item.detail.contains("render_target_not_displayable")
            )
    );
}

#[test]
fn blocked_pixel_observation_reports_explicit_next_action_as_next_fix() {
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
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-observation.json status"
                    && item.state == "blocked"
            )
    );
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let pixel_observation = modernized
        .desktop_run_pixel_observation
        .as_ref()
        .expect("modernized target should report pixel-observation evidence status");
    assert!(pixel_observation.next_action.is_some());
    let next_blocker = report
        .evidence_progress
        .next_actionable_blocker
        .as_deref()
        .expect("blocked pixel observation should name the next action");
    assert!(next_blocker.contains(
        "fix next: rerun RabbitHole with DISPLAY backed by a visible desktop and capture desktop-run-render-target.png"
    ));
    assert!(!next_blocker.contains("first-lesson completion"));
    assert!(!next_blocker.contains("visible rendering correctness is proven"));
}

#[test]
fn vm_execution_sentinel_alone_is_not_visible_desktop_proof() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: false,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(&report.issues, MISSING_DESKTOP_PROOF);
}

#[test]
fn missing_pixel_boundary_evidence_is_reported_explicitly() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: false,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "missing desktop Run pixel-boundary evidence; expected run-window-evidence/desktop-run-pixel-boundary.json under the comparison evidence root",
    );
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let pixel_boundary = modernized
        .desktop_run_pixel_boundary
        .as_ref()
        .expect("modernized target should report missing pixel-boundary evidence");
    assert_eq!(pixel_boundary.status, "missing");
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-boundary.json status"
                    && item.state == "missing"
            )
    );
}

#[test]
fn present_invalid_pixel_boundary_status_is_reported_as_evidence_status() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });
    overwrite_modernized_pixel_boundary(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-run-pixel-boundary/v1","status":"invalid","reason":"producer reported invalid pixel evidence"}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(&report.issues, "producer reported invalid pixel evidence");
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let pixel_boundary = modernized
        .desktop_run_pixel_boundary
        .as_ref()
        .expect("modernized target should report invalid pixel-boundary evidence");
    assert_eq!(pixel_boundary.status, "invalid");
    assert_eq!(
        pixel_boundary.detail,
        "producer reported invalid pixel evidence"
    );
    assert_eq!(report.evidence_progress.invalid, 1);
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-boundary.json status"
                    && item.state == "invalid"
            )
    );
}

#[test]
fn present_observed_pixel_observation_is_reported_as_desktop_pixel_status() {
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

    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence should exist");
    let pixel_observation = modernized
        .desktop_run_pixel_observation
        .as_ref()
        .expect("modernized target should report observed pixel-observation evidence");
    assert_eq!(pixel_observation.status, "observed");
    assert_eq!(report.desktop_proof_contract.status, "verified");
    assert_eq!(
        report.desktop_proof_contract.reason_code,
        "desktop_pixel_observation_verified"
    );
    assert!(
        report
            .desktop_proof_contract
            .detail
            .contains("does not prove full lesson automation")
    );
    assert!(
        report
            .desktop_proof_contract
            .artifact
            .as_deref()
            .is_some_and(|artifact| artifact.ends_with("desktop-run-pixel-observation.json"))
    );
    assert!(pixel_observation.screenshot.is_some());
    assert!(pixel_observation.sample.is_some());
    assert!(
        pixel_observation
            .detail
            .contains("center pixel: 0xFF336699")
    );
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-observation.json status"
                    && item.state == "present"
                    && item.detail.contains("desktop-run-render-target.png")
            )
    );
}

#[test]
fn missing_pixel_observation_evidence_is_reported_explicitly() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Missing,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "missing desktop Run pixel-observation evidence; expected run-window-evidence/desktop-run-pixel-observation.json under the comparison evidence root",
    );
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-observation.json status"
                    && item.state == "missing"
            )
    );
    assert!(report.evidence_progress.next_actionable_blocker.is_none());
}

#[test]
fn invalid_pixel_observation_evidence_is_reported_explicitly() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });
    overwrite_modernized_pixel_observation(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-run-pixel-observation/v1","claim":"missing status"}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "desktop Run pixel-observation evidence is missing status field",
    );
    assert_eq!(report.evidence_progress.invalid, 1);
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "modernized desktop-run-pixel-observation.json status"
                    && item.state == "invalid"
            )
    );
    assert!(report.evidence_progress.next_actionable_blocker.is_none());
}

#[test]
fn run_frame_prerequisite_is_preserved_when_screenshot_exists() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: false,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"run_world_desktop_toolbar_window_observed\" must pass before first-lesson readiness is evidence-ready",
    );
}

#[test]
fn vm_statement_prerequisite_is_preserved_when_screenshot_exists() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: false,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"run_world_desktop_execution_observed\" must pass before first-lesson readiness is evidence-ready",
    );
}

#[test]
fn after_full_desktop_pixel_chain_next_proof_names_first_missing_rabbithole_hook() {
    // When all Run-window/pixel-chain evidence is present but the RabbitHole hook
    // actions are still unproven, next_missing_real_desktop_proof should name the
    // first missing hook (place-object) and cite the specific tools/ path to wire.
    // This gives a plain user an exact next step rather than silence.
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

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect("next_missing_real_desktop_proof should be set when RabbitHole hooks are unproven");

    assert!(
        next_proof.contains("place-object"),
        "expected place-object hook guidance; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("tools/eatme-place-object"),
        "expected tools/eatme-place-object path; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("does not prove full UI automation"),
        "expected explicit automation limit statement; got: {next_proof:?}"
    );
    // The run-world and save-project hooks come after place-object in the chain;
    // they should not appear as the next step until place-object is wired.
    assert!(
        !next_proof.contains("run-world"),
        "run-world should not be the next step before place-object; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("save-project"),
        "save-project should not be the next step before place-object; got: {next_proof:?}"
    );
}
