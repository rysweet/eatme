use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_readiness_reporting/support.rs"]
mod readiness_reporting_support;
#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use readiness_reporting_support::{
    complete_desktop_evidence_manifest, overwrite_ui_action_contracts, ready_ui_action_contract,
    rewrite_target_failure_categories, write_complete_next_action_evidence,
};
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture,
    overwrite_modernized_first_lesson_next_action, overwrite_modernized_pixel_observation,
    write_manifest,
};

#[test]
fn readiness_report_separates_user_facing_shown_missing_and_unproven_claims() {
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
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["save-project"],
  "requiresNextEvidence":["Collect explicit Save completion evidence before reporting Save completion."],
  "evidence_boundaries":[
    {"id":"select_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Select Project scenario evidence is present.","claim":"The Select Project boundary has auditable scenario evidence.","does_not_prove":["full Alice UI automation","first-lesson completion"]},
    {"id":"save_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Save option/action scenario evidence is present.","claim":"Save action evidence is present for this scenario boundary.","does_not_prove":["Save completion","grading","creative assessment","first-lesson completion"]},
    {"id":"visible_rendering","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Visible rendering scenario evidence is present.","claim":"Visible rendering was observed for this scenario boundary.","does_not_prove":["visible rendering correctness","creative assessment","first-lesson completion"]},
    {"id":"grading","status":"missing","source":"rabbithole","detail":"Grading scenario evidence is missing."},
    {"id":"creative_assessment","status":"missing","source":"rabbithole","detail":"Creative assessment scenario evidence is missing."},
    {"id":"first_lesson_completion","status":"missing","source":"rabbithole","detail":"First-lesson completion scenario evidence is missing."}
  ],
  "doesNotClaim":["full Alice UI automation","Save completion","grading","creative assessment","visible rendering correctness","first-lesson completion"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    let save = readiness_item(&report_json, "shown_evidence", "save_project");
    assert_eq!(save["state"], "present");
    assert_summary_contains(save, "shown");
    assert_summary_contains(save, "observed option/action only");
    assert_array_contains(&save["does_not_prove"], "Save completion");
    assert_array_contains(&save["does_not_prove"], "first-lesson completion");

    let grading = readiness_item(&report_json, "not_yet_shown", "grading");
    assert_eq!(grading["state"], "missing");
    assert_eq!(grading["summary"], "Grading is not yet shown.");
    assert_not_yet_shown_wording_is_user_facing(grading);

    let completion = readiness_item(&report_json, "not_yet_shown", "first_lesson_completion");
    assert_eq!(
        completion["summary"],
        "First-lesson completion is not yet shown."
    );
    assert_not_yet_shown_wording_is_user_facing(completion);

    assert_unproven_claims(&report_json);
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn desktop_next_action_summary_is_conditional_and_observational() {
    let missing_manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Missing,
        hook_actions_passed: &[],
    });
    let missing_report = check_lesson_session_readiness(&missing_manifest_path).unwrap();
    let missing_json = serde_json::to_value(&missing_report).unwrap();

    assert!(
        missing_json.get("desktop_next_action").is_none(),
        "missing desktop next-action artifact must not produce a desktop_next_action claim: {missing_json}"
    );
    assert!(
        readiness_items(&missing_json, "not_yet_shown")
            .iter()
            .any(|item| item["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("not yet shown"))),
        "missing desktop evidence should still be reported as not_yet_shown: {missing_json}"
    );

    let present_manifest_path = write_manifest(DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    });
    let present_report = check_lesson_session_readiness(&present_manifest_path).unwrap();
    let present_json = serde_json::to_value(&present_report).unwrap();

    let desktop = present_json.get("desktop_next_action").unwrap_or_else(|| {
        panic!("valid RabbitHole next-action evidence should be summarized: {present_json}")
    });
    assert_eq!(desktop["status"], "blocked");
    assert_array_contains(&desktop["candidate_actions"], "desktop_save_menu_action");
    assert_array_contains(
        &desktop["requires_next_evidence"],
        "desktop Save menu readiness or invocation artifact",
    );
    assert_array_contains(&desktop["does_not_prove"], "Save completion");
    assert_array_contains(&desktop["does_not_prove"], "first-lesson completion");
    assert!(
        desktop["observations"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "desktop_next_action should expose observations without implying completion: {desktop}"
    );
    assert_no_unsupported_success_claims(&present_json);
}

#[test]
fn no_go_contracts_block_readiness_even_when_failure_category_is_null() {
    let manifest_path = complete_desktop_evidence_manifest();
    rewrite_target_failure_categories(&manifest_path, serde_json::Value::Null, "passed");
    write_complete_next_action_evidence(&manifest_path, "blocked");

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        !report.no_go_contracts.is_empty(),
        "fixture must expose unsupported-action no-go contracts: {report_json}"
    );
    assert!(
        report.issues.is_empty(),
        "failure_category:null plus explicit no-go contracts should be a blocker, not an incomplete evidence issue: {:?}",
        report.issues
    );
    assert!(!report.passed);
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    assert_eq!(report.status, "blocked");
    assert_eq!(
        report.blocked_reason.as_deref(),
        Some("blocked_until_ui_automation")
    );
    assert!(
        report.evidence_gap_message.is_some(),
        "blocked no-go contracts must keep the evidence gap visible"
    );
    assert_eq!(
        report.lesson_session_readiness.no_go_contracts.len(),
        report.no_go_contracts.len()
    );
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn complete_evidence_with_null_failure_category_and_no_no_go_contracts_is_ready() {
    let manifest_path = complete_desktop_evidence_manifest();
    rewrite_target_failure_categories(&manifest_path, serde_json::Value::Null, "passed");
    write_complete_next_action_evidence(&manifest_path, "ready");
    overwrite_ui_action_contracts(&manifest_path, ready_ui_action_contract());

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        report.issues.is_empty(),
        "complete evidence with failure_category:null should not produce readiness issues: {:?}",
        report.issues
    );
    assert!(
        report.no_go_contracts.is_empty(),
        "ready evidence must not carry unsupported-action no-go contracts: {report_json}"
    );
    assert!(report.passed);
    assert_eq!(report.readiness_status, "ready");
    assert_eq!(report.status, "ready");
    assert_eq!(report.blocked_reason, None);
    assert_eq!(report.evidence_gap_message, None);
    assert_eq!(report.evidence_progress.missing, 0);
    assert_eq!(report.evidence_progress.invalid, 0);
    assert_eq!(report.evidence_progress.not_observed, 0);
    assert_eq!(report.evidence_progress.blocked, 0);
    assert!(report.not_yet_shown.is_empty());
    assert_eq!(report.lesson_session_readiness.status, "ready");
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn incomplete_progress_gap_fails_closed_even_without_issues_or_no_go_contracts() {
    let manifest_path = complete_desktop_evidence_manifest();
    rewrite_target_failure_categories(&manifest_path, serde_json::Value::Null, "passed");
    write_complete_next_action_evidence(&manifest_path, "ready");
    overwrite_ui_action_contracts(&manifest_path, ready_ui_action_contract());
    overwrite_modernized_pixel_observation(
        &manifest_path,
        r#"{"schema_version":"eatme.alice-desktop-run-pixel-observation/v1","status":"not_observed","reason":"Run target was present, but no visible pixel sample was observed."}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();

    assert!(
        report.issues.is_empty(),
        "fixture isolates progress-gap handling from issue-based failures: {:?}",
        report.issues
    );
    assert!(
        report.no_go_contracts.is_empty(),
        "fixture isolates progress-gap handling from no-go blockers: {report_json}"
    );
    assert!(!report.passed);
    assert_eq!(report.readiness_status, "incomplete");
    assert_eq!(report.status, "not_ready");
    assert!(
        report.evidence_gap_message.is_some(),
        "not-observed evidence must keep the gap explicit"
    );
    assert_eq!(report.evidence_progress.not_observed, 1);
    assert_eq!(report.lesson_session_readiness.status, "not_ready");
    assert_no_unsupported_success_claims(&report_json);
}

#[test]
fn unsupported_desktop_next_action_status_is_invalid_evidence() {
    let manifest_path = complete_desktop_evidence_manifest();
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"complete",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["save-project"],
  "doesNotClaim":["full Alice UI automation","Save completion","grading","creative assessment","first-lesson completion"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .expect("modernized target evidence");
    let next_action = modernized
        .desktop_first_lesson_next_action
        .as_ref()
        .expect("desktop next-action evidence");

    assert_eq!(next_action.status, "invalid");
    assert!(
        next_action
            .detail
            .contains("unsupported desktop next-action status"),
        "invalid status should explain the unsupported status: {}",
        next_action.detail
    );
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.contains("unsupported desktop next-action status")),
        "unsupported status must be surfaced as a readiness issue: {:?}",
        report.issues
    );
    assert!(!report.passed);
    assert_eq!(report.status, "not_ready");
    assert!(
        report.desktop_next_action.is_none(),
        "invalid desktop next-action evidence must not produce a desktop_next_action observation: {report_json}"
    );
    assert_no_unsupported_success_claims(&report_json);
}

fn readiness_items<'a>(report_json: &'a serde_json::Value, field: &str) -> &'a [serde_json::Value] {
    report_json[field]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level {field}[] in {report_json}"))
}

fn readiness_item<'a>(
    report_json: &'a serde_json::Value,
    field: &str,
    id: &str,
) -> &'a serde_json::Value {
    readiness_items(report_json, field)
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing {field} item {id} in {report_json}"))
}

fn assert_summary_contains(item: &serde_json::Value, expected: &str) {
    let summary = item["summary"].as_str().unwrap_or_default();
    assert!(
        summary.contains(expected),
        "expected summary to contain {expected:?}; item was {item}"
    );
}

fn assert_not_yet_shown_wording_is_user_facing(item: &serde_json::Value) {
    let summary = item["summary"].as_str().unwrap_or_default();
    let detail = item["detail"].as_str().unwrap_or_default();
    assert!(
        summary.contains("not yet shown"),
        "not_yet_shown summaries must use plain user-facing wording: {item}"
    );
    assert!(
        detail.contains("not yet shown"),
        "not_yet_shown details must use plain user-facing wording: {item}"
    );
    let text = format!("{summary}\n{detail}").to_ascii_lowercase();
    for forbidden in [
        "blocker",
        "blocked",
        "invalid",
        "missing",
        "proof artifact",
        "ui-action-contract",
        "desktop-run-pixel",
        "desktop-first-lesson-next-action",
        "no_go",
    ] {
        assert!(
            !text.contains(forbidden),
            "not_yet_shown item leaked internal wording {forbidden:?}: {item}"
        );
    }
}

fn assert_unproven_claims(report_json: &serde_json::Value) {
    let actual = readiness_items(report_json, "unproven_claims")
        .iter()
        .map(|claim| claim.as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            "Full Alice UI automation is not proven.",
            "Grading is not proven.",
            "Creative assessment is not proven.",
            "Visible rendering correctness is not proven.",
            "Save completion is not proven.",
            "First-lesson completion is not proven.",
        ],
        "unproven_claims must be the canonical user-facing non-claims"
    );
}

fn assert_array_contains(value: &serde_json::Value, expected: &str) {
    let values = value
        .as_array()
        .unwrap_or_else(|| panic!("expected array containing {expected:?}, got {value}"));
    assert!(
        values.iter().any(|value| value == expected),
        "expected array to contain {expected:?}, got {value}"
    );
}

fn assert_no_unsupported_success_claims(value: &serde_json::Value) {
    let text = serde_json::to_string(value).unwrap().to_ascii_lowercase();
    for forbidden in [
        "full alice ui automation is proven",
        "ui automation succeeded",
        "visible rendering correctness is proven",
        "save completion evidence",
        "save completed",
        "save project succeeded",
        "desktop save completion is proven",
        "bounded save completion is proven",
        "bounded save completion evidence",
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
