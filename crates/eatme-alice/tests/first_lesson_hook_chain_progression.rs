use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, write_manifest,
};

#[test]
fn after_place_object_passes_next_proof_names_edit_procedure() {
    // When place-object is proven but edit-procedure-or-code-block is not,
    // next_missing_real_desktop_proof must advance to the second hook in the chain
    // and cite tools/eatme-edit-procedure as the exact path to wire.
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &["place_object_ui_action"],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect(
            "next_missing_real_desktop_proof should be set when edit-procedure hook is unproven",
        );

    assert!(
        next_proof.contains("edit-procedure-or-code-block"),
        "expected edit-procedure-or-code-block hook guidance; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("tools/eatme-edit-procedure"),
        "expected tools/eatme-edit-procedure path; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("does not prove full UI automation"),
        "expected explicit automation limit statement; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("place-object"),
        "place-object should not reappear once it has passed; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("run-world"),
        "run-world should not be the next step before edit-procedure; got: {next_proof:?}"
    );
}

#[test]
fn after_place_object_and_edit_procedure_pass_next_proof_names_run_world() {
    // When place-object and edit-procedure-or-code-block are both proven but
    // run-world is not, next_missing_real_desktop_proof must advance to run-world
    // and cite tools/eatme-run-world.
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &["place_object_ui_action", "edit_procedure_ui_action"],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect("next_missing_real_desktop_proof should be set when run-world hook is unproven");

    assert!(
        next_proof.contains("run-world"),
        "expected run-world hook guidance; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("tools/eatme-run-world"),
        "expected tools/eatme-run-world path; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("does not prove full UI automation"),
        "expected explicit automation limit statement; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("save-project"),
        "save-project should not be the next step before run-world; got: {next_proof:?}"
    );
}

#[test]
fn after_place_object_edit_procedure_and_run_world_pass_next_proof_names_save_project() {
    // When only save-project remains unproven, next_missing_real_desktop_proof
    // must advance to save-project and cite tools/eatme-save-project.
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
        ],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect("next_missing_real_desktop_proof should be set when save-project hook is unproven");

    assert!(
        next_proof.contains("save-project"),
        "expected save-project hook guidance; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("tools/eatme-save-project"),
        "expected tools/eatme-save-project path; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("does not prove full UI automation"),
        "expected explicit automation limit statement; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("missing desktop next-action evidence"),
        "next-action artifact should not be requested before save-project; got: {next_proof:?}"
    );
}

#[test]
fn after_all_four_hooks_pass_next_proof_names_missing_next_action_artifact() {
    // Once the hook action chain is proven, readiness still requires the
    // next-action proof artifact before reporting RabbitHole evidence as complete.
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

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let next_proof = report
        .evidence_progress
        .next_missing_real_desktop_proof
        .as_deref()
        .expect("missing next-action artifact should remain the next proof blocker");

    assert!(
        next_proof.contains("missing desktop next-action evidence"),
        "expected missing next-action artifact guidance; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("run-window-evidence/desktop-first-lesson-next-action.json"),
        "next proof must not leak the internal next-action evidence path; got: {next_proof:?}"
    );
    assert!(
        next_proof.contains("Save Project proof artifact"),
        "expected Save Project proof artifact guidance; got: {next_proof:?}"
    );
    assert!(
        !next_proof.contains("lesson completed"),
        "next proof must not claim lesson completion; got: {next_proof:?}"
    );
}
