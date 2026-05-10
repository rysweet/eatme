use super::lesson_session_helpers::{
    assert_safe_blocker_text, unique_test_dir, write_executable_blocked_first_lesson_manifest,
};
use super::*;
use std::fs;

#[test]
fn lesson_session_readiness_preserves_original_alice_action_evidence_blocker() {
    let root = unique_test_dir("original-alice-action-evidence-blocker");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["targets"]["baseline"]["launch_manifest"]["assertions"]
        .as_object_mut()
        .unwrap()
        .remove("save_project_ui_action");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(!report.passed);
    assert_eq!(
        (
            report.status.as_str(),
            report.readiness_status.as_str(),
            report.lesson_session_readiness.status.as_str()
        ),
        ("not_ready", "incomplete", "not_ready")
    );
    let report_json = serde_json::to_value(&report).unwrap();
    let baseline = target_evidence_json(&report_json, "baseline");
    let blockers = baseline["blockers"]
        .as_array()
        .unwrap_or_else(|| panic!("baseline target should expose blockers: {baseline}"));
    let blocker = blockers
        .iter()
        .find(|blocker| {
            blocker["code"] == "missing_real_action_evidence" && blocker["action"] == "save-project"
        })
        .unwrap_or_else(|| panic!("missing baseline action-evidence blocker: {blockers:?}"));
    assert_eq!(
        blocker["reason"],
        "Required original Alice action evidence is missing from automation scenarios."
    );
    assert_safe_blocker_text(blocker["reason"].as_str().unwrap_or_default());
    assert!(blocker.get("message").is_none(), "{blocker}");
}

#[test]
fn lesson_session_readiness_reports_missing_original_alice_action_evidence_as_structured_state() {
    let root = unique_test_dir("original-alice-action-evidence-structured-missing");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    // The branch's fail-closed readiness_status reports blocked manifests as
    // not-passed. Missing original Alice action evidence is still reportable
    // (not fatal) — it does not add to `issues` — but the blocked manifest
    // itself causes readiness_status != "ready".
    assert!(
        report.issues.is_empty(),
        "missing original Alice action evidence must not generate blocking issues: {:?}",
        report.issues
    );
    let report_json = serde_json::to_value(&report).unwrap();
    let original_evidence = &report_json["original_alice_action_evidence"];
    assert_eq!(original_evidence["status"], "missing");
    assert_eq!(
        original_evidence["summary"],
        "Original Alice action evidence is missing."
    );
    assert_eq!(
        original_evidence["detail"],
        "Original Alice action evidence was not found in the comparison target evidence."
    );
    assert_original_alice_action_evidence_text_is_bounded(original_evidence);

    let baseline = target_evidence_json(&report_json, "baseline");
    assert!(
        baseline["blockers"]
            .as_array()
            .unwrap_or_else(|| panic!("baseline target should expose blockers: {baseline}"))
            .iter()
            .any(|blocker| blocker["code"] == "missing_real_action_evidence"),
        "top-level summary must not remove target-local blockers: {baseline}"
    );
}

#[test]
fn lesson_session_readiness_reports_available_original_alice_action_evidence_without_missing_blocker()
 {
    let root = unique_test_dir("original-alice-action-evidence-structured-available");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    mark_original_alice_action_assertions_passed(&mut manifest);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let report_json = serde_json::to_value(&report).unwrap();
    let original_evidence = &report_json["original_alice_action_evidence"];
    assert_eq!(original_evidence["status"], "available");
    assert_eq!(
        original_evidence["summary"],
        "Original Alice action evidence is available."
    );
    assert_eq!(
        original_evidence["detail"],
        "The readiness report did not find a missing original Alice action evidence blocker."
    );
    assert_original_alice_action_evidence_text_is_bounded(original_evidence);
    for target in report_json["target_evidence"].as_array().unwrap() {
        assert!(
            target["blockers"]
                .as_array()
                .unwrap_or_else(|| panic!("target should expose blockers: {target}"))
                .iter()
                .all(|blocker| blocker["code"] != "missing_real_action_evidence"),
            "available status requires no missing_real_action_evidence blocker: {target}"
        );
    }
}

fn mark_original_alice_action_assertions_passed(manifest: &mut serde_json::Value) {
    let assertions = manifest["targets"]["baseline"]["launch_manifest"]["assertions"]
        .as_object_mut()
        .expect("baseline launch_manifest assertions");
    for assertion_id in [
        "specific_alice_window_detected",
        "activate_alice_window_ui_action",
        "place_object_ui_action",
        "edit_procedure_ui_action",
        "run_world_ui_action",
        "save_project_ui_action",
    ] {
        let assertion = assertions
            .entry(assertion_id)
            .or_insert_with(|| serde_json::json!({}));
        let assertion = assertion
            .as_object_mut()
            .expect("assertion entries should be objects");
        assertion.insert("passed".into(), serde_json::json!(true));
        assertion.insert(
            "detail".into(),
            serde_json::json!("test fixture original Alice action evidence passed"),
        );
    }
}

fn assert_original_alice_action_evidence_text_is_bounded(evidence: &serde_json::Value) {
    let text = serde_json::to_string(evidence).unwrap();
    for prohibited in [
        "Full Alice UI automation succeeded",
        "full UI automation succeeded",
        "grading succeeded",
        "creative assessment succeeded",
        "Save completed",
        "Save completion evidence",
        "lesson completed",
        "First-lesson completion succeeded",
    ] {
        assert!(
            !text.contains(prohibited),
            "original Alice action evidence text must stay bounded; found {prohibited:?} in {text}"
        );
    }
}

fn target_evidence_json<'a>(report: &'a serde_json::Value, role: &str) -> &'a serde_json::Value {
    report["target_evidence"]
        .as_array()
        .unwrap_or_else(|| panic!("report should expose target_evidence[]: {report}"))
        .iter()
        .find(|target| target["role"] == role)
        .unwrap_or_else(|| panic!("missing target_evidence role {role}: {report}"))
}
