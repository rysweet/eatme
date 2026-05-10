use super::lesson_session_helpers::{
    assert_safe_blocker_text, unique_test_dir, write_executable_blocked_first_lesson_manifest,
};
use super::*;

#[test]
fn lesson_session_blocked_summary_is_gap_reporting_not_capability_readiness_claim() {
    let root = unique_test_dir("blocked-summary-gap-reporting");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert_eq!(report.status, "blocked");
    assert_eq!(report.readiness_status, "blocked_until_ui_automation");
    let summaries = std::iter::once(&report.human_summary)
        .chain(
            report
                .role_readiness
                .iter()
                .map(|readiness| &readiness.human_summary),
        )
        .collect::<Vec<_>>();
    for summary in summaries {
        assert!(
            summary.contains("gap-reporting evidence"),
            "blocked readiness summary must describe bounded gap reporting: {summary}"
        );
        assert!(
            summary.contains("cannot confirm first-lesson readiness"),
            "blocked readiness summary must avoid success-shaped readiness claims: {summary}"
        );
        assert!(
            summary.contains("evidence gaps remain"),
            "blocked readiness summary must preserve missing/incomplete evidence language: {summary}"
        );
        assert!(
            summary.contains("blocked_until_ui_automation"),
            "blocked readiness summary should keep the machine-readable blocker visible: {summary}"
        );
        assert!(
            !summary.contains("deterministic desktop UI automation exists"),
            "blocked readiness summary must not imply missing automation is the product claim: {summary}"
        );
        assert!(
            !summary.contains("has launch and automation scenario action evidence"),
            "blocked readiness summary must not recast partial evidence as broad action evidence: {summary}"
        );
    }
}

#[test]
fn lesson_session_no_go_reasons_are_evidence_gaps_not_capability_failures() {
    let root = unique_test_dir("no-go-evidence-gap-reasons");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    assert!(
        !report.no_go_contracts.is_empty(),
        "fixture should produce no-go contracts"
    );
    for contract in &report.no_go_contracts {
        assert_eq!(contract.decision, "no_go");
        assert!(
            contract.reason.starts_with("Evidence gap:"),
            "no-go reason must be framed as a reporting gap: {contract:?}"
        );
        assert!(
            contract.reason.contains("required evidence"),
            "no-go reason must name missing required evidence, not just capability absence: {contract:?}"
        );
        let reason = contract.reason.to_ascii_lowercase();
        assert!(
            !reason.contains("missing deterministic desktop affordance"),
            "no-go reason must not present the gap as only an implementation capability failure: {contract:?}"
        );
        assert!(
            !reason.contains("automation can"),
            "no-go reason must not imply a broader automation capability claim: {contract:?}"
        );
        assert_safe_blocker_text(&contract.reason);
    }
}

#[test]
fn lesson_session_no_go_reasons_do_not_echo_raw_contract_evidence() {
    let root = unique_test_dir("no-go-reason-raw-contract-evidence");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let contract_path =
        manifest["targets"]["baseline"]["launch_manifest"]["ui_action_contract"]["path"]
            .as_str()
            .unwrap()
            .to_string();
    let mut contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();
    let unsafe_required_evidence = "/tmp/secret/stdout\nInjected line";
    contract["action_precondition_probes"]
        .as_array_mut()
        .unwrap()[0]["required_evidence"] = serde_json::json!(unsafe_required_evidence);
    let place_object = contract["required_actions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|action| action.get("id").and_then(serde_json::Value::as_str) == Some("place-object"))
        .unwrap();
    place_object["required_evidence"] = serde_json::json!(unsafe_required_evidence);
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let blocker_text = serde_json::to_string(&report.no_go_contracts).unwrap();

    assert!(blocker_text.contains("object placement"));
    assert!(!blocker_text.contains("/tmp/secret"));
    assert!(!blocker_text.contains("stdout"));
    assert!(!blocker_text.contains("Injected line"));
    for contract in &report.no_go_contracts {
        assert_safe_blocker_text(&contract.reason);
    }
}

#[test]
fn lesson_session_readiness_reports_creative_assessment_gap_plainly() {
    let root = unique_test_dir("creative-assessment-gap-readiness-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let report_json = serde_json::to_value(&report).unwrap();
    let creative_boundary = report_json["evidence_boundaries"]
        .as_array()
        .unwrap_or_else(|| panic!("report should expose evidence_boundaries[]: {report_json}"))
        .iter()
        .find(|boundary| boundary["id"] == "creative_assessment")
        .unwrap_or_else(|| panic!("missing creative_assessment boundary: {report_json}"));
    let boundary_text = serde_json::to_string(creative_boundary).unwrap();

    assert!(boundary_text.contains("surface available evidence"));
    assert!(boundary_text.contains("suggest next steps"));
    assert!(boundary_text.contains("learner's creative work in this scenario"));
    assert!(
        boundary_text
            .contains("does not grade creativity, judge quality, or mark the lesson complete")
    );
}
