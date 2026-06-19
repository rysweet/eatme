use eatme_alice::check_lesson_session_readiness;

#[path = "first_lesson_desktop_evidence/support.rs"]
#[allow(dead_code)]
mod support;
use support::{
    DesktopFixture, FirstLessonNextActionFixture, PixelObservationFixture,
    overwrite_modernized_first_lesson_next_action, write_manifest,
    write_modernized_run_window_evidence_file,
};

#[test]
fn malformed_next_action_shape_fails_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":"desktop_save_menu_action",
  "requiresNextEvidence":[],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        !report.passed,
        "malformed next-action shape must block readiness: {report_json}"
    );
    assert_eq!(next_action["status"], "invalid", "{next_action}");
    assert_report_issue_contains(&report, "desktop next-action evidence is invalid");
    assert_report_issue_contains(&report, "candidate_actions");
    assert_report_issue_contains(&report, "requiresNextEvidence");
}

#[test]
fn missing_required_boundary_fields_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[
    {
      "id":"grading",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "claim":"The grading boundary has auditable scenario evidence.",
      "does_not_prove":["creative assessment","first-lesson completion"]
    }
  ],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let grading = evidence_boundary(&report_json, "grading");

    assert!(
        !report.passed,
        "boundary entries missing required fields must block readiness: {report_json}"
    );
    assert_eq!(grading["status"], "invalid", "{grading}");
    assert_detail_contains(grading, "Grading scenario evidence is invalid");
    assert_detail_contains(grading, "detail");
    assert_report_issue_contains(&report, "Grading scenario evidence is invalid");
    assert_report_issue_contains(&report, "detail");
}

#[test]
fn wrong_or_empty_boundary_collections_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[
    {
      "id":"save_project",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "detail":"Save action evidence is present for this scenario boundary.",
      "claim":"Save action evidence is present for this scenario boundary.",
      "does_not_prove":[]
    },
    {
      "id":"visible_rendering",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "detail":"Visible rendering scenario evidence is present.",
      "claim":"Visible rendering was observed for this scenario boundary.",
      "does_not_prove":"first-lesson completion"
    }
  ],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save = evidence_boundary(&report_json, "save_project");
    let rendering = evidence_boundary(&report_json, "visible_rendering");

    assert!(
        !report.passed,
        "wrong or empty evidence-boundary collections must block readiness: {report_json}"
    );
    assert_eq!(save["status"], "invalid", "{save}");
    assert_eq!(rendering["status"], "invalid", "{rendering}");
    assert_report_issue_contains(&report, "does_not_prove");
}

#[test]
fn invalid_camel_does_not_prove_alias_fails_even_when_snake_alias_is_valid() {
    assert_invalid_does_not_prove_aliases(
        r#"["Save completion","grading","first-lesson completion"]"#,
        r#"["TODO placeholder limitation"]"#,
        "doesNotProve",
    );
}

#[test]
fn invalid_snake_does_not_prove_alias_fails_even_when_camel_alias_is_valid() {
    assert_invalid_does_not_prove_aliases(
        r#"["TODO placeholder limitation"]"#,
        r#"["Save completion","grading","first-lesson completion"]"#,
        "does_not_prove",
    );
}

fn assert_invalid_does_not_prove_aliases(snake_claims: &str, camel_claims: &str, issue: &str) {
    let manifest_path = write_manifest(ready_desktop_fixture());
    let next_action = format!(
        r#"{{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[{{"id":"save_project","status":"present","source":"rabbithole","metadata_state":"observed","detail":"Save action evidence is present for this scenario boundary.","claim":"Save action evidence is present for this scenario boundary.","does_not_prove":{snake_claims},"doesNotProve":{camel_claims}}}],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}}"#
    );
    overwrite_modernized_first_lesson_next_action(&manifest_path, &next_action);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let save = evidence_boundary(&report_json, "save_project");

    assert!(
        !report.passed,
        "invalid snake limitation alias must block readiness: {report_json}"
    );
    assert_eq!(save["status"], "invalid", "{save}");
    assert_report_issue_contains(&report, issue);
    assert_report_issue_contains(&report, "filler");
}

#[test]
fn malformed_boundary_section_fails_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":{"id":"grading","status":"present"},
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        !report.passed,
        "malformed evidence_boundaries section must block readiness: {report_json}"
    );
    assert_eq!(next_action["status"], "invalid", "{next_action}");
    assert_report_issue_contains(&report, "evidence_boundaries");
}

#[test]
fn boundary_filler_text_fails_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[
    {
      "id":"procedure_edit",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "detail":"TODO sample dummy scenario evidence placeholder; lorem ipsum.",
      "claim":"Dummy sample scenario evidence is present.",
      "does_not_prove":["code correctness","grading","first-lesson completion"]
    }
  ],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let boundary = evidence_boundary(&report_json, "procedure_edit");

    assert!(
        !report.passed,
        "placeholder/filler boundary wording must block readiness: {report_json}"
    );
    assert_eq!(boundary["status"], "invalid", "{boundary}");
    assert_detail_contains(boundary, "filler");
}

#[test]
fn unsupported_affirmative_boundary_claims_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[
    {
      "id":"first_lesson_completion",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "detail":"First-lesson completion scenario evidence is present.",
      "claim":"This proves first-lesson completion, grading is complete, creative assessment passed, and full Alice UI automation succeeded.",
      "does_not_prove":["full Alice UI automation","creative assessment"]
    }
  ],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let boundary = evidence_boundary(&report_json, "first_lesson_completion");

    assert!(
        !report.passed,
        "unsupported affirmative boundary claims must block readiness: {report_json}"
    );
    assert_eq!(boundary["status"], "invalid", "{boundary}");
    assert_report_issue_contains(&report, "unsupported claim");
    assert_report_issue_contains(&report, "first-lesson completion");
}

#[test]
fn unsupported_affirmative_next_action_claims_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "blocker":{"reason":"Full Alice UI automation succeeded; lesson completed; grading is complete; creative assessment passed."},
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        !report.passed,
        "unsupported affirmative next-action claims must block readiness: {report_json}"
    );
    assert_eq!(next_action["status"], "invalid", "{next_action}");
    assert_report_issue_contains(&report, "unsupported claim");
    assert_report_issue_contains(&report, "full Alice UI automation");
}

#[test]
fn unsupported_world_execution_and_sharing_next_action_claims_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "blocker":{"reason":"Full world execution succeeded; deployed sharing succeeded; platform success was confirmed."},
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        !report.passed,
        "unsupported full-world-execution and sharing/platform claims must block readiness: {report_json}"
    );
    assert_eq!(next_action["status"], "invalid", "{next_action}");
    assert_report_issue_contains(&report, "unsupported claim");
    assert_report_issue_contains(&report, "full world execution");
    assert_report_issue_contains(&report, "deployed sharing");
    assert_report_issue_contains(&report, "platform success");
}

#[test]
fn unsupported_world_execution_and_sharing_boundary_claims_fail_validation() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["desktop Save menu readiness or invocation artifact"],
  "evidence_boundaries":[
    {
      "id":"first_lesson_completion",
      "status":"present",
      "source":"rabbithole",
      "metadata_state":"observed",
      "detail":"First-lesson completion scenario evidence is present.",
      "claim":"The first lesson finished with full world execution, deployed sharing success, and platform success.",
      "does_not_prove":["full Alice UI automation","creative quality"]
    }
  ],
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let boundary = evidence_boundary(&report_json, "first_lesson_completion");

    assert!(
        !report.passed,
        "unsupported full-world-execution and sharing/platform boundary claims must block readiness: {report_json}"
    );
    assert_eq!(boundary["status"], "invalid", "{boundary}");
    assert_report_issue_contains(&report, "unsupported claim");
    assert_report_issue_contains(&report, "full world execution");
    assert_report_issue_contains(&report, "deployed sharing");
    assert_report_issue_contains(&report, "platform success");
}

#[test]
fn restrained_limitation_wording_remains_valid() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "source":"desktop_run_render_target_attachment",
  "candidate_actions":["desktop_save_menu_action"],
  "blocker":{"reason":"UI automation is not complete; this does not prove completion. Grading is not assessed. Creative assessment is not assessed."},
  "requiresNextEvidence":["Collect explicit Save finish-state evidence before reporting Save completion."],
  "evidence_boundaries":[
    {
      "id":"first_lesson_completion",
      "status":"missing",
      "source":"rabbithole",
      "metadata_state":"missing",
      "detail":"First-lesson completion scenario evidence is missing; this does not prove completion.",
      "claim":"First-lesson completion is not proven; grading is not assessed and creative assessment is not assessed.",
      "does_not_prove":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
    }
  ],
  "doesNotClaim":["full Alice UI automation is not complete","first-lesson completion is not proven","grading is not assessed","creative assessment is not assessed"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        report.issues.is_empty(),
        "restrained limitation wording must remain valid: {:?}",
        report.issues
    );
    assert_eq!(next_action["status"], "blocked", "{next_action}");
    assert_report_text_contains(&report_json, "does not prove completion");
    assert_report_text_contains(&report_json, "grading is not assessed");
    assert_report_text_contains(&report_json, "UI automation is not complete");
}

#[test]
fn proof_artifact_contract_accepts_direct_declarations_and_blocked_aliases() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    write_modernized_run_window_evidence_file(
        &manifest_path,
        "save-project-proof.json",
        r#"{"proof":"save-project"}"#,
    );
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["Collect explicit Save finish-state evidence before reporting Save completion."],
  "save_project_proof_artifact":{
    "path":"run-window-evidence/save-project-proof.json",
    "size_bytes":24,
    "sha256":"sha256-save-project-proof"
  },
  "selectProjectProofArtifact":{
    "status":"blocked",
    "blocker":{
      "reason":"Select Project proof collection is blocked by an explicit desktop affordance boundary.",
      "codes":["select_project_proof_unavailable"]
    }
  },
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert_eq!(
        next_action["save_project_proof_artifact"]["status"],
        "present"
    );
    assert_eq!(
        next_action["select_project_proof_artifact"]["status"],
        "blocked"
    );
    assert_report_text_contains(&report_json, "run-window-evidence/save-project-proof.json");
    assert_report_text_contains(
        &report_json,
        "Select Project proof collection is blocked by an explicit desktop affordance boundary.",
    );
    assert_report_text_contains(&report_json, "select_project_proof_unavailable");
}

#[test]
fn proof_artifact_contract_rejects_empty_status_values() {
    let manifest_path = write_manifest(ready_desktop_fixture());
    overwrite_modernized_first_lesson_next_action(
        &manifest_path,
        r#"{
  "schema_version":"eatme.alice-desktop-first-lesson-next-action/v1",
  "status":"blocked",
  "candidate_actions":["desktop_save_menu_action"],
  "requiresNextEvidence":["Collect explicit Save finish-state evidence before reporting Save completion."],
  "save_project_proof_artifact":{"status":""},
  "doesNotClaim":["full Alice UI automation","first-lesson completion","grading","creative assessment"]
}"#,
    );

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let next_action = modernized_next_action(&report_json);

    assert!(
        !report.passed,
        "empty proof-artifact status must block readiness: {report_json}"
    );
    assert_eq!(
        next_action["save_project_proof_artifact"]["status"],
        "invalid"
    );
    assert_report_issue_contains(
        &report,
        "Save Project proof artifact status must be a non-empty string",
    );
}

fn ready_desktop_fixture() -> DesktopFixture {
    DesktopFixture {
        run_frame_present: true,
        vm_statement_execution_present: true,
        visible_desktop_screenshot_present: true,
        pixel_boundary_present: true,
        pixel_observation: PixelObservationFixture::Observed,
        first_lesson_next_action: FirstLessonNextActionFixture::Blocked,
        hook_actions_passed: &[],
    }
}

fn modernized_next_action(report_json: &serde_json::Value) -> &serde_json::Value {
    &report_json["target_evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| target["role"] == "modernized")
        .unwrap()["desktop_first_lesson_next_action"]
}

fn evidence_boundary<'a>(report_json: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report_json["evidence_boundaries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|boundary| boundary["id"] == id)
        .unwrap_or_else(|| panic!("missing evidence boundary {id} in {report_json}"))
}

fn assert_detail_contains(item: &serde_json::Value, expected: &str) {
    let detail = item["detail"].as_str().unwrap_or_default();
    assert!(
        detail.contains(expected),
        "expected detail to contain {expected:?}; item was {item}"
    );
}

fn assert_report_issue_contains(
    report: &eatme_alice::compare::LessonSessionReadinessReport,
    expected: &str,
) {
    let issues = report.issues.join("\n");
    assert!(
        issues.contains(expected),
        "expected report issues to contain {expected:?}; issues were {issues:?}"
    );
}

fn assert_report_text_contains(report_json: &serde_json::Value, expected: &str) {
    let text = serde_json::to_string(report_json).unwrap();
    assert!(
        text.contains(expected),
        "expected report JSON to contain {expected:?}; JSON was {text}"
    );
}
