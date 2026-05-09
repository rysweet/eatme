use eatme_alice::{FirstLessonReadinessOptions, run_first_lesson_readiness_sequence};
use std::fs;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod desktop_evidence_support;
use desktop_evidence_support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture,
    overwrite_modernized_first_lesson_next_action, write_manifest,
};

#[test]
fn sequence_executes_fake_targets_until_ui_action_blocker() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-sequence".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    assert_eq!(report.readiness_status, "incomplete");
    assert_eq!(
        report.evidence_progress.summary,
        report.readiness_report.evidence_progress.summary
    );
    assert!(
        report
            .evidence_progress
            .summary
            .contains("required evidence items are present")
    );
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|claim| claim == "Full Alice UI automation is not proven.")
    );
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|claim| claim == "Visible rendering correctness is not proven.")
    );
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|claim| claim == "First-lesson completion is not proven.")
    );
    assert!(report.issues.iter().any(|issue| issue.contains(
        "missing visible desktop rendering evidence after Run-frame and VM statement execution"
    )));
    for role in ["baseline", "modernized"] {
        let target = report.target_statuses.get(role).unwrap();
        assert_eq!(
            target.failure_category.as_deref(),
            Some("ui_action_automation_unimplemented")
        );
        assert!(target.launch_manifest_present);
        assert!(target.ui_action_contract_path.is_some());
    }
}

#[test]
fn sequence_reports_action_progress_when_earlier_window_detection_fails() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_unrelated_window_tool();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-action-progress".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    assert!(
        report
            .evidence_progress
            .items
            .iter()
            .any(
                |item| item.evidence == "screenshot, log, and window artifacts"
                    && item.state == "missing"
            )
    );
    let modernized = report
        .readiness_report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .unwrap();
    assert_eq!(
        modernized.failure_category.as_deref(),
        Some("alice_window_not_detected")
    );
    assert_action(modernized, "verify-specific-alice-window", false);
    assert_action(modernized, "activate-specific-alice-window", false);
    assert_action(modernized, "place-object", true);
    assert_action(modernized, "edit-procedure-or-code-block", false);
    assert_action(modernized, "run-world", false);
    assert_action(modernized, "save-project", false);
}

#[test]
fn sequence_distinguishes_wrong_alice_like_window_from_absent_window() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_alice_like_license_window_tool();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-wrong-alice-like-window".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    let modernized = report
        .readiness_report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .unwrap();
    assert_eq!(
        modernized.failure_category.as_deref(),
        Some("alice_like_window_not_main")
    );
    assert_action_detail_contains(
        modernized,
        "verify-specific-alice-window",
        false,
        "Alice-like window",
    );
    assert_action_detail_contains(
        modernized,
        "activate-specific-alice-window",
        false,
        "Alice-like window",
    );
    assert_action(modernized, "place-object", true);
}

#[test]
fn sequence_distinguishes_unsupported_activation_after_window_detection() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_unsupported_activation_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let registry_path = fixture.root.join("targets.yaml");
    fs::write(
        &registry_path,
        format!(
            r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: {}
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: {}
"#,
            fixture.alice_home.display(),
            fixture.alice_home.display()
        ),
    )
    .unwrap();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
        registry_path,
        baseline_target: "baseline".into(),
        modernized_target: "modernized".into(),
        baseline_home_override: None,
        modernized_home_override: None,
        run_id: "fake-first-lesson-unsupported-activation".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    assert!(!report.passed);
    let modernized = report
        .readiness_report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .unwrap();
    assert_eq!(
        modernized.failure_category.as_deref(),
        Some("alice_window_activation_unsupported")
    );
    assert_eq!(
        report
            .evidence_progress
            .next_missing_real_desktop_proof
            .as_deref(),
        Some(
            "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions."
        )
    );
    let progress_json = serde_json::to_value(&report.evidence_progress).unwrap();
    assert_eq!(
        progress_json["next_missing_real_desktop_proof"],
        "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions."
    );
    assert_action(modernized, "verify-specific-alice-window", true);
    assert_action_detail_contains(
        modernized,
        "activate-specific-alice-window",
        false,
        "unsupported",
    );
    assert_action(modernized, "place-object", true);
}

#[test]
fn first_lesson_readiness_missing_next_action_artifact_names_missing_proof_artifact() {
    let report = eatme_alice::check_lesson_session_readiness(&write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[
            "place_object_ui_action",
            "edit_procedure_ui_action",
            "run_world_ui_action",
            "save_project_ui_action",
        ],
    }))
    .unwrap();

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect("missing next-action artifact should be the next proof blocker");
    assert!(next_proof.contains("missing desktop next-action evidence"));
    assert!(!next_proof.contains("run-window-evidence/desktop-first-lesson-next-action.json"));
    assert!(next_proof.contains("Save Project proof artifact"));
    let report_text = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
    assert_no_unsupported_readiness_claims(&report_text);
}

#[test]
fn first_lesson_readiness_reports_first_unavailable_desktop_proof_artifact() {
    for (next_action_json, save_state, select_state, first_detail, later_detail) in [
        (
            r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","save_project_proof_artifact":{"status":"blocked","blocker":{"reason":"Save dialog owner does not expose a stable proof-artifact handoff yet.","codes":["save_project_artifact_handoff_not_bound"]}},"select_project_proof_artifact":{"status":"missing"},"doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]}"#,
            "blocked",
            "missing",
            "blocked Save Project proof artifact",
            "missing Select Project proof artifact",
        ),
        (
            r#"{"schema_version":"eatme.alice-desktop-first-lesson-next-action/v1","status":"blocked","source":"desktop_run_render_target_attachment","save_project_proof_artifact":{"status":"missing","reason":"Save Project proof artifact is not available from the desktop next-action evidence yet."},"select_project_proof_artifact":{"status":"blocked","blocker":{"reason":"Select dialog owner does not expose a stable proof-artifact handoff yet.","codes":["select_project_artifact_handoff_not_bound"]}},"doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]}"#,
            "missing",
            "blocked",
            "missing Save Project proof artifact",
            "blocked Select Project proof artifact",
        ),
    ] {
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
        overwrite_modernized_first_lesson_next_action(&manifest_path, next_action_json);
        let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();
        let mut save_index = None;
        let mut select_index = None;
        for (index, item) in report.evidence_progress.items.iter().enumerate() {
            match item.evidence.as_str() {
                "Save Project proof artifact" => save_index = Some(index),
                "Select Project proof artifact" => select_index = Some(index),
                _ => {}
            }
            if save_index.is_some() && select_index.is_some() {
                break;
            }
        }
        let save_index = save_index.expect("Save Project proof artifact progress item");
        let select_index = select_index.expect("Select Project proof artifact progress item");
        assert!(save_index < select_index);
        let save_item = &report.evidence_progress.items[save_index];
        let select_item = &report.evidence_progress.items[select_index];
        assert_eq!(save_item.state, save_state);
        assert_eq!(select_item.state, select_state);
        assert!(save_item.detail.contains(first_detail));
        assert!(save_item.detail.contains("desktop next-action evidence"));
        assert!(
            !save_item
                .detail
                .contains("run-window-evidence/desktop-first-lesson-next-action.json")
        );
        let next_proof = report
            .evidence_progress
            .next_missing_real_desktop_proof
            .as_deref()
            .expect("expected first unavailable desktop proof artifact");
        let next_proof_detail = next_proof
            .strip_prefix("next missing real-desktop proof: ")
            .expect("next proof should include the real-desktop proof prefix");
        assert_eq!(next_proof_detail, save_item.detail);
        assert!(
            !next_proof.contains(later_detail),
            "later unavailable proof artifacts must not be surfaced before the first one"
        );
        let progress_json = serde_json::to_value(&report.evidence_progress).unwrap();
        assert_eq!(progress_json["items"][save_index]["state"], save_state);
        assert_eq!(progress_json["items"][select_index]["state"], select_state);
        assert_eq!(progress_json["next_missing_real_desktop_proof"], next_proof);
        let report_text = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
        assert_no_unsupported_readiness_claims(&report_text);
    }
}

fn assert_action(
    target: &eatme_alice::compare::LessonTargetEvidence,
    action_id: &str,
    expected_passed: bool,
) {
    let action = target
        .action_assertions
        .iter()
        .find(|action| action.action_id == action_id)
        .unwrap_or_else(|| panic!("missing action evidence for {action_id}"));
    assert_eq!(action.passed, expected_passed, "{action:?}");
}

fn assert_action_detail_contains(
    target: &eatme_alice::compare::LessonTargetEvidence,
    action_id: &str,
    expected_passed: bool,
    expected_detail: &str,
) {
    let action = target
        .action_assertions
        .iter()
        .find(|action| action.action_id == action_id)
        .unwrap_or_else(|| panic!("missing action evidence for {action_id}"));
    assert_eq!(action.passed, expected_passed, "{action:?}");
    assert!(
        action.detail.contains(expected_detail),
        "expected {expected_detail:?} in {action:?}"
    );
}

fn assert_no_unsupported_readiness_claims(text: &str) {
    for forbidden in [
        "save completion evidence",
        "save completed",
        "save project succeeded",
        "lesson completed",
        "ui automation succeeded",
        "grading occurred",
        "creative assessment passed",
        "creative quality assessed",
    ] {
        assert!(
            !text.contains(forbidden),
            "readiness output must not claim {forbidden:?}: {text}"
        );
    }
}
