// Student lesson E2E tests: validates the student-facing contract of
// the first-lesson readiness sequence and desktop evidence chain.

use eatme_alice::{FirstLessonReadinessOptions, run_first_lesson_readiness_sequence};
use std::fs;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod desktop_evidence_support;
use desktop_evidence_support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, write_manifest,
};

const REQUIRED_BOUNDARY_IDS: [&str; 7] = [
    "select_project",
    "procedure_edit",
    "save_project",
    "visible_rendering",
    "grading",
    "creative_assessment",
    "first_lesson_completion",
];

// -------------------------------------------------------------------
// Test 1: Full sequence with fake targets reports student-facing contract
// -------------------------------------------------------------------

#[test]
fn run_first_lesson_readiness_sequence_reports_student_facing_contract() {
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
        run_id: "student-lesson-e2e-sequence".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        execute: true,
        starter_project: None,
    })
    .unwrap();

    // Contract envelope fields the student CLI depends on
    assert!(
        !report.schema_version.is_empty(),
        "schema_version must be set"
    );
    assert_eq!(report.scenario_id, "first-lessons-real-ui-actions");
    assert_eq!(report.run_id, "student-lesson-e2e-sequence");
    assert!(report.execute_requested, "execute_requested must be true");

    // Student-facing: sequence must not claim pass
    assert!(!report.passed);
    assert_eq!(report.readiness_status, "incomplete");

    // Evidence progress summary must be consistent between sequence and readiness
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

    // Unproven claims must include these student-visible assertions
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|c| c == "Full Alice UI automation is not proven.")
    );
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|c| c == "Visible rendering correctness is not proven.")
    );
    assert!(
        report
            .unproven_claims
            .iter()
            .any(|c| c == "First-lesson completion is not proven.")
    );

    // shown_evidence and not_yet_shown items must have non-empty id
    for item in &report.shown_evidence {
        assert!(
            !item.id.is_empty(),
            "shown_evidence item id must not be empty"
        );
    }
    for item in &report.not_yet_shown {
        assert!(
            !item.id.is_empty(),
            "not_yet_shown item id must not be empty"
        );
    }

    // Target statuses should reflect UI action automation failure
    for role in ["baseline", "modernized"] {
        let target = report.target_statuses.get(role).unwrap();
        assert_eq!(
            target.failure_category.as_deref(),
            Some("ui_action_automation_unimplemented")
        );
        assert!(target.launch_manifest_present);
        assert!(target.ui_action_contract_path.is_some());
    }

    // No overclaiming in full report
    let report_text = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
    assert_no_unsupported_readiness_claims(&report_text);
}

// -------------------------------------------------------------------
// Test 2: Desktop fixture with all hooks passed validates student contract
// -------------------------------------------------------------------

#[test]
fn check_lesson_session_readiness_validates_student_desktop_fixture() {
    let manifest_path = write_manifest(DesktopFixture {
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
    });

    let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();

    // Schema version and desktop proof contract are student-visible envelope fields
    assert!(
        !report.schema_version.is_empty(),
        "schema_version must be set"
    );
    assert!(
        !report.desktop_proof_contract.status.is_empty(),
        "desktop_proof_contract.status must be non-empty"
    );
    assert!(
        !report.desktop_proof_contract.detail.is_empty(),
        "desktop_proof_contract.detail must be non-empty"
    );

    // Required evidence must be a non-empty list students can read
    assert!(!report.required_evidence.is_empty());
    assert_eq!(
        report.required_evidence,
        report.lesson_session_readiness.required_evidence
    );

    // Evidence progress items must each have the evidence field
    for item in &report.evidence_progress.items {
        assert!(
            !item.evidence.is_empty(),
            "progress item evidence must not be empty"
        );
    }

    // Unproven claims are always present even when all hooks pass
    assert!(!report.unproven_claims.is_empty());

    // Student-visible wording: summary must contain "required evidence items"
    assert!(
        report
            .evidence_progress
            .summary
            .contains("required evidence items are present")
    );

    let report_text = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
    assert_no_unsupported_readiness_claims(&report_text);
}

// -------------------------------------------------------------------
// Test 3: Evidence progress tracks the student hook chain progression
// -------------------------------------------------------------------

#[test]
fn evidence_progress_tracks_student_hook_chain() {
    let configs: Vec<(&[&str], &str)> = vec![
        (&[], "place-object"),
        (&["place_object_ui_action"], "edit-procedure-or-code-block"),
        (
            &["place_object_ui_action", "edit_procedure_ui_action"],
            "run-world",
        ),
        (
            &[
                "place_object_ui_action",
                "edit_procedure_ui_action",
                "run_world_ui_action",
            ],
            "save-project",
        ),
    ];

    for (hooks_passed, expected_next_hook) in configs {
        let manifest_path = write_manifest(DesktopFixture {
            run_frame_present: true,
            vm_statement_execution_present: true,
            visible_desktop_screenshot_present: true,
            pixel_boundary_present: true,
            pixel_observation: PixelObservationFixture::Observed,
            first_lesson_next_action: FirstLessonNextActionFixture::Missing,
            hook_actions_passed: hooks_passed,
        });

        let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();

        let next_proof = report
            .evidence_progress
            .next_missing_real_desktop_proof
            .as_deref()
            .unwrap_or_else(|| {
                panic!(
                    "next_missing_real_desktop_proof should be set when hooks {:?} passed",
                    hooks_passed
                )
            });
        assert!(
            next_proof.contains(expected_next_hook),
            "with hooks {:?}, expected next hook {expected_next_hook:?} in: {next_proof:?}",
            hooks_passed
        );
        assert!(
            next_proof.contains("does not prove full UI automation"),
            "automation limit must appear in: {next_proof:?}"
        );
    }
}

// -------------------------------------------------------------------
// Test 4: Boundaries disallow completion, grading, creative assessment
// -------------------------------------------------------------------

#[test]
fn readiness_boundaries_disallow_completion_grading_creative_assessment() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: false,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Missing,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    let boundaries = report_json["evidence_boundaries"]
        .as_array()
        .expect("evidence_boundaries must be an array");
    let boundary_ids: Vec<&str> = boundaries.iter().filter_map(|b| b["id"].as_str()).collect();
    for id in &REQUIRED_BOUNDARY_IDS {
        assert!(
            boundary_ids.contains(id),
            "missing required boundary {id:?}; found: {boundary_ids:?}"
        );
    }

    // grading, creative_assessment, and first_lesson_completion must NOT be "present"
    for id in ["grading", "creative_assessment", "first_lesson_completion"] {
        let boundary = boundaries
            .iter()
            .find(|b| b["id"] == id)
            .unwrap_or_else(|| panic!("boundary {id} not found"));
        assert_ne!(
            boundary["status"], "present",
            "{id} must not be 'present' when evidence is absent"
        );
    }

    // No unsupported success claims
    let text = serde_json::to_string(&report_json)
        .unwrap()
        .to_ascii_lowercase();
    for forbidden in [
        "grading is complete",
        "creative assessment passed",
        "first-lesson completion is proven",
        "lesson completed",
    ] {
        assert!(
            !text.contains(forbidden),
            "report must not claim {forbidden:?}"
        );
    }
}

// -------------------------------------------------------------------
// Test 5: No overclaiming across fixture configurations
// -------------------------------------------------------------------

#[test]
fn student_readiness_does_not_overclaim_across_fixture_configurations() {
    let configs = vec![
        // Minimal: no screenshot, no pixel evidence, no hooks
        DesktopFixture {
            run_frame_present: true,
            vm_statement_execution_present: true,
            visible_desktop_screenshot_present: false,
            pixel_boundary_present: false,
            pixel_observation: PixelObservationFixture::Missing,
            first_lesson_next_action: FirstLessonNextActionFixture::Missing,
            hook_actions_passed: &[],
        },
        // Pixel observation blocked
        DesktopFixture {
            run_frame_present: true,
            vm_statement_execution_present: true,
            visible_desktop_screenshot_present: true,
            pixel_boundary_present: true,
            pixel_observation: PixelObservationFixture::Blocked,
            first_lesson_next_action: FirstLessonNextActionFixture::Missing,
            hook_actions_passed: &[],
        },
        // All hooks passed, pixel observed, but next-action missing
        DesktopFixture {
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
        },
    ];

    for fixture in configs {
        let manifest_path = write_manifest(fixture);
        let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();
        let report_text = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
        assert_no_unsupported_readiness_claims(&report_text);
    }
}

// -------------------------------------------------------------------
// Test 6: Required evidence covers student session steps
// -------------------------------------------------------------------

#[test]
fn required_evidence_covers_student_session_steps() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();

    // Action assertions must cover the four core student actions
    let modernized = report
        .target_evidence
        .iter()
        .find(|t| t.role == "modernized")
        .expect("modernized target evidence should exist");
    let action_ids: Vec<&str> = modernized
        .action_assertions
        .iter()
        .map(|a| a.action_id.as_str())
        .collect();
    for expected in [
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project",
    ] {
        assert!(
            action_ids.contains(&expected),
            "action assertions should include {expected:?}; found: {action_ids:?}"
        );
    }

    // Limitations must be non-empty (the system never claims full completion)
    assert!(
        !report.limitations.is_empty(),
        "limitations should always be non-empty"
    );
}

// -------------------------------------------------------------------
// Shared helpers
// -------------------------------------------------------------------

fn assert_no_unsupported_readiness_claims(text: &str) {
    let snippet = if text.len() > 300 {
        format!("{}…(truncated {} chars)", &text[..300], text.len() - 300)
    } else {
        text.to_string()
    };
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
            "readiness output must not claim {forbidden:?}: {snippet}"
        );
    }
}
