use eatme_alice::check_lesson_session_readiness;
use serde_json::json;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture,
    overwrite_modernized_first_lesson_next_action, write_manifest,
};

const BOUNDARY_IDS: [&str; 7] = [
    "select_project",
    "procedure_edit",
    "save_project",
    "visible_rendering",
    "grading",
    "creative_assessment",
    "first_lesson_completion",
];

#[test]
fn boundary_evidence_chain_feeds_shown_and_next_missing_pipeline_stages() {
    for present_count in 1..=BOUNDARY_IDS.len() {
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
        overwrite_modernized_first_lesson_next_action(
            &manifest_path,
            &serde_json::to_string_pretty(&json!({
                "schema_version": "eatme.alice-desktop-first-lesson-next-action/v1",
                "status": "blocked",
                "source": "desktop_run_render_target_attachment",
                "evidence_boundaries": valid_boundary_sequence()[..present_count].to_vec(),
                "doesNotClaim": [
                    "full Alice UI automation",
                    "visible rendering correctness",
                    "Save completion",
                    "grading",
                    "creative assessment",
                    "first-lesson completion"
                ]
            }))
            .unwrap(),
        );

        let report = check_lesson_session_readiness(&manifest_path).unwrap();
        let shown_boundary_ids = report
            .shown_evidence
            .iter()
            .filter_map(|item| {
                BOUNDARY_IDS
                    .contains(&item.id.as_str())
                    .then_some(item.id.as_str())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shown_boundary_ids,
            BOUNDARY_IDS[..present_count].to_vec(),
            "present_count {present_count} should surface the consumed boundary prefix",
        );

        if present_count < BOUNDARY_IDS.len() {
            let next_boundary = report
                .not_yet_shown
                .iter()
                .find(|item| item.id == BOUNDARY_IDS[present_count])
                .unwrap_or_else(|| {
                    panic!(
                        "missing next boundary {} in not_yet_shown for step {}",
                        BOUNDARY_IDS[present_count], present_count
                    )
                });
            assert_eq!(next_boundary.state, "missing");
            assert!(
                next_boundary.summary.contains("not yet shown"),
                "next stage should remain user-visible pipeline work: {:?}",
                next_boundary.summary
            );
        }
    }
}

fn valid_boundary_sequence() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": "select_project",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Select Project scenario evidence is present.",
            "claim": "The Select Project boundary has auditable scenario evidence.",
            "does_not_prove": ["full Alice UI automation", "first-lesson completion"]
        }),
        json!({
            "id": "procedure_edit",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Procedure/edit scenario evidence is present.",
            "claim": "The procedure/edit boundary has auditable scenario evidence.",
            "does_not_prove": ["code correctness", "grading", "first-lesson completion"]
        }),
        json!({
            "id": "save_project",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Save scenario evidence is present.",
            "claim": "Save action evidence is present for this scenario boundary.",
            "does_not_prove": ["Save completion", "grading", "creative assessment", "first-lesson completion"]
        }),
        json!({
            "id": "visible_rendering",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Visible rendering scenario evidence is present.",
            "claim": "Visible rendering was observed for this scenario boundary.",
            "does_not_prove": ["visible rendering correctness", "creative assessment", "first-lesson completion"]
        }),
        json!({
            "id": "grading",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Grading scenario evidence is present.",
            "claim": "The grading boundary has auditable scenario evidence.",
            "does_not_prove": ["creative assessment", "first-lesson completion"]
        }),
        json!({
            "id": "creative_assessment",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "Creative assessment scenario evidence is present.",
            "claim": "The report can surface available evidence and suggest next steps for the learner's creative work in this scenario, but it does not grade creativity, judge quality, or mark the lesson complete.",
            "does_not_prove": ["instructor judgment", "first-lesson completion"]
        }),
        json!({
            "id": "first_lesson_completion",
            "status": "present",
            "source": "rabbithole",
            "metadata_state": "observed",
            "detail": "First-lesson completion scenario evidence is present.",
            "claim": "The first-lesson completion boundary has auditable scenario evidence.",
            "does_not_prove": ["full Alice UI automation", "creative quality"]
        }),
    ]
}
