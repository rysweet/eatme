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
            "missing or uncertain {id} evidence must be reported as a blocker, not success: {boundary}"
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
fn explicit_rabbithole_boundary_evidence_is_reported_per_boundary_without_cross_claims() {
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
        assert!(
            boundary["claim"]
                .as_str()
                .unwrap_or_default()
                .contains("boundary")
                || boundary["claim"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("scenario"),
            "boundary claim should stay bounded and scenario-specific: {boundary}"
        );
    }

    let save_boundary = evidence_boundary(&report_json, "save_project");
    assert_does_not_prove(save_boundary, "first-lesson completion");
    assert_does_not_prove(save_boundary, "grading");
    let rendering_boundary = evidence_boundary(&report_json, "visible_rendering");
    assert_does_not_prove(rendering_boundary, "visible rendering correctness");
    assert_does_not_prove(rendering_boundary, "first-lesson completion");
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
