use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
mod support;
use support::{
    DesktopFixture, PixelObservationFixture, assert_contains, overwrite_modernized_pixel_boundary,
    overwrite_modernized_pixel_observation, write_manifest,
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
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
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
fn vm_execution_sentinel_alone_is_not_visible_desktop_proof() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: false,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
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
}

#[test]
fn invalid_pixel_observation_evidence_is_reported_explicitly() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
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
}

#[test]
fn run_frame_prerequisite_is_preserved_when_screenshot_exists() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: false,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Blocked,
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
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_contains(
        &report.issues,
        "modernized launch_manifest assertion \"run_world_desktop_execution_observed\" must pass before first-lesson readiness is evidence-ready",
    );
}
