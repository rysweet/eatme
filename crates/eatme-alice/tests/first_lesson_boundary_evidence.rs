use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture, assert_contains,
    overwrite_modernized_first_lesson_next_action, write_manifest,
    write_modernized_run_window_evidence_file,
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

#[test]
fn missing_rabbithole_evidence_boundaries_are_plain_scenario_blockers() {
    let manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: false,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Missing,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_boundary_ids(&report_json);
    for id in REQUIRED_BOUNDARY_IDS {
        let boundary = evidence_boundary(&report_json, id);
        assert_ne!(
            boundary["status"], "present",
            "missing or uncertain {id} evidence must be a blocker: {boundary}"
        );
        assert_boundary_text_is_scenario_focused(boundary);
    }
    assert_boundary_status(&report_json, "select_project", "missing");
    assert_boundary_status(&report_json, "procedure_edit", "missing");
    assert_boundary_status(&report_json, "save_project", "missing");
    assert_boundary_status(&report_json, "visible_rendering", "missing");
    assert_boundary_status(&report_json, "grading", "missing");
    assert_boundary_status(&report_json, "creative_assessment", "missing");
    assert_boundary_status(&report_json, "first_lesson_completion", "missing");
    assert_boundary_detail_contains(
        &report_json,
        "select_project",
        "Select Project scenario evidence is missing.",
    );
    assert_boundary_detail_contains(
        &report_json,
        "procedure_edit",
        "Procedure/edit scenario evidence is missing.",
    );
    assert_boundary_detail_contains(
        &report_json,
        "save_project",
        "Save scenario evidence is missing.",
    );
    assert_boundary_detail_contains(
        &report_json,
        "first_lesson_completion",
        "First-lesson completion scenario evidence is missing.",
    );
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn explicit_rabbithole_boundary_evidence_is_reported_without_cross_claims() {
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
    write_modernized_run_window_evidence_file(
        &manifest_path,
        "save-project-completion.json",
        r#"{"status":"present","bounded_save_completion":true}"#,
    );
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "evidence_boundaries":[
    {"id":"select_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Select Project scenario evidence is present.","claim":"The Select Project boundary has auditable scenario evidence.","does_not_prove":["full Alice UI automation","first-lesson completion"]},
    {"id":"procedure_edit","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Procedure/edit scenario evidence is present.","claim":"The procedure/edit boundary has auditable scenario evidence.","does_not_prove":["code correctness","grading","first-lesson completion"]},
    {"id":"save_project","status":"present","source":"rabbithole","metadata_state":"observed","artifact":{"path":"run-window-evidence/save-project-completion.json"},"detail":"Save scenario evidence is present.","claim":"Bounded Save completion evidence is present for this scenario.","does_not_prove":["grading","creative assessment","first-lesson completion"]},
    {"id":"visible_rendering","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Visible rendering scenario evidence is present.","claim":"Visible rendering was observed for this scenario boundary.","does_not_prove":["visible rendering correctness","creative assessment","first-lesson completion"]},
    {"id":"grading","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Grading scenario evidence is present.","claim":"The grading boundary has auditable scenario evidence.","does_not_prove":["creative assessment","first-lesson completion"]},
    {"id":"creative_assessment","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Creative assessment scenario evidence is present.","claim":"The creative assessment boundary has auditable scenario evidence.","does_not_prove":["instructor judgment","first-lesson completion"]},
    {"id":"first_lesson_completion","status":"present","source":"rabbithole","metadata_state":"observed","detail":"First-lesson completion scenario evidence is present.","claim":"The first-lesson completion boundary has auditable scenario evidence.","does_not_prove":["full Alice UI automation","creative quality"]}
  ],
  "doesNotClaim":["full Alice UI automation","visible rendering correctness beyond the visible_rendering boundary"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_boundary_ids(&report_json);
    for id in REQUIRED_BOUNDARY_IDS {
        let boundary = evidence_boundary(&report_json, id);
        assert_eq!(
            boundary["status"], "present",
            "{id} boundary was not present: {boundary}"
        );
        assert_eq!(boundary["source"], "rabbithole");
        assert_eq!(boundary["metadata_state"], "observed");
        assert_boundary_text_is_scenario_focused(boundary);
    }
    assert_does_not_prove(
        evidence_boundary(&report_json, "save_project"),
        "first-lesson completion",
    );
    assert_does_not_prove(evidence_boundary(&report_json, "save_project"), "grading");
    assert_does_not_prove(
        evidence_boundary(&report_json, "visible_rendering"),
        "visible rendering correctness",
    );
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn observed_or_declared_boundary_metadata_does_not_prove_completion() {
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
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "evidence_boundaries":[
    {"id":"save_project","status":"declared","source":"rabbithole","detail":"Save boundary metadata was declared by RabbitHole."},
    {"id":"first_lesson_completion","status":"observed","source":"rabbithole","detail":"First-lesson completion boundary metadata was observed by RabbitHole."}
  ],
  "doesNotClaim":["desktop Save completion","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save_boundary = evidence_boundary(&report_json, "save_project");
    let completion_boundary = evidence_boundary(&report_json, "first_lesson_completion");

    assert_eq!(save_boundary["status"], "missing", "{save_boundary}");
    assert_eq!(
        save_boundary["metadata_state"], "declared",
        "{save_boundary}"
    );
    assert_detail_contains(save_boundary, "Save scenario metadata was declared");
    assert_detail_contains(save_boundary, "bounded Save completion evidence is missing");
    assert_eq!(
        completion_boundary["status"], "missing",
        "{completion_boundary}"
    );
    assert_eq!(
        completion_boundary["metadata_state"], "observed",
        "{completion_boundary}"
    );
    assert_detail_contains(
        completion_boundary,
        "First-lesson completion scenario metadata was observed",
    );
    assert_detail_contains(
        completion_boundary,
        "first-lesson completion scenario evidence is missing",
    );
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn invalid_boundary_evidence_blocks_readiness_without_success_claims() {
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
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "evidence_boundaries":[
    {"id":"grading","status":"complete","source":"rabbithole","detail":"unsupported grading status"},
    {"id":"creative_assessment","status":"present","source":"rabbithole","detail":"creative assessment artifact outside evidence root","artifact":{"path":"/outside/evidence/creative.json"}}
  ],
  "doesNotClaim":["grading","creative assessment","first-lesson completion"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert_boundary_status(&report_json, "grading", "invalid");
    assert_boundary_detail_contains(
        &report_json,
        "grading",
        "Grading scenario evidence is invalid",
    );
    assert_boundary_status(&report_json, "creative_assessment", "invalid");
    assert_boundary_detail_contains(
        &report_json,
        "creative_assessment",
        "Creative assessment scenario evidence is invalid",
    );
    assert_contains(&report.issues, "Grading scenario evidence is invalid");
    assert_contains(
        &report.issues,
        "Creative assessment scenario evidence is invalid",
    );
    assert_no_unsupported_success_claims(&report_json);
}

fn evidence_boundaries(report_json: &serde_json::Value) -> &[serde_json::Value] {
    report_json["evidence_boundaries"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level evidence_boundaries[] in {report_json}"))
}

fn evidence_boundary<'a>(report_json: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    evidence_boundaries(report_json)
        .iter()
        .find(|boundary| boundary["id"] == id)
        .unwrap_or_else(|| panic!("missing evidence boundary {id} in {report_json}"))
}

fn assert_boundary_ids(report_json: &serde_json::Value) {
    let actual = evidence_boundaries(report_json)
        .iter()
        .map(|boundary| boundary["id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(actual, REQUIRED_BOUNDARY_IDS, "unexpected boundary order");
}

fn assert_boundary_status(report_json: &serde_json::Value, id: &str, expected: &str) {
    let boundary = evidence_boundary(report_json, id);
    assert_eq!(
        boundary["status"], expected,
        "unexpected status for {id}: {boundary}"
    );
}

fn assert_boundary_detail_contains(report_json: &serde_json::Value, id: &str, expected: &str) {
    assert_detail_contains(evidence_boundary(report_json, id), expected);
}

fn assert_detail_contains(item: &serde_json::Value, expected: &str) {
    let detail = item["detail"].as_str().unwrap_or_default();
    assert!(
        detail.contains(expected),
        "expected detail to contain {expected:?}; item was {item}"
    );
}

fn assert_boundary_text_is_scenario_focused(boundary: &serde_json::Value) {
    let label = boundary["label"].as_str().unwrap_or_default();
    let detail = boundary["detail"].as_str().unwrap_or_default();
    assert!(
        label.contains("scenario evidence"),
        "boundary label must use scenario-focused wording: {boundary}"
    );
    assert!(
        detail.contains("scenario") || detail.contains("automation scenarios"),
        "boundary detail must use scenario-focused wording: {boundary}"
    );
    for forbidden in [
        "proof artifact",
        "ui-action-contract",
        "desktop-run-pixel",
        "desktop-first-lesson-next-action",
        "action_id",
        "no_go",
    ] {
        assert!(
            !label.contains(forbidden) && !detail.contains(forbidden),
            "primary boundary text leaked implementation detail {forbidden:?}: {boundary}"
        );
    }
}

fn assert_does_not_prove(boundary: &serde_json::Value, expected: &str) {
    let claims = boundary["does_not_prove"]
        .as_array()
        .unwrap_or_else(|| panic!("boundary missing does_not_prove[]: {boundary}"));
    assert!(
        claims.iter().any(|claim| claim == expected),
        "expected {expected:?} in does_not_prove for {boundary}"
    );
}

fn assert_no_unsupported_success_claims(value: &serde_json::Value) {
    let text = serde_json::to_string(value).unwrap().to_ascii_lowercase();
    for forbidden in [
        "full alice ui automation is proven",
        "ui automation succeeded",
        "visible rendering correctness is proven",
        "desktop save completion is proven",
        "bounded save completion is proven",
        "grading is complete",
        "creative assessment passed",
        "first-lesson completion is proven",
        "lesson completed",
    ] {
        assert!(
            !text.contains(forbidden),
            "readiness output must not claim {forbidden:?}: {text}"
        );
    }
}
