use super::*;
use eatme_alice::compare::{
    DesktopProofContract, FirstLessonEvidenceBoundary, LessonReadinessEvidenceProgress,
    LessonReadinessEvidenceProgressItem, LessonSessionContractCheck, LessonSessionNoGoContract,
    LessonSessionReadinessEnvelope, LessonSessionReadinessReport,
    OriginalAliceActionEvidenceReport, ReadinessEvidenceItem,
};
use std::collections::BTreeMap;

const EVIDENCE_GAP_MESSAGE: &str = "Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.";

#[test]
fn plain_output_omits_legacy_next_actionable_blocker_line() {
    let report = sequence_report(progress_with_blocker(Some(
        "desktop Run pixel observation is blocked: fix next: run Alice with a non-headless graphics environment",
    )));

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Shown:"));
    assert!(output.contains("Not yet shown:"));
    assert!(output.contains("- Save option/action evidence is not yet shown."));
    assert!(output.contains("Unproven:"));
    assert!(!output.contains("Next blocker:"));
    assert!(!output.contains("Evidence progress:"));
}

#[test]
fn plain_output_includes_next_missing_real_desktop_proof_line() {
    let mut progress = progress_with_blocker(None);
    progress.next_missing_real_desktop_proof = Some(
        "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions.".into(),
    );
    let report = sequence_report(progress);

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Not yet shown:"));
    assert!(output.contains("- Save option/action evidence is not yet shown."));
    assert!(output.contains(
        "next missing real-desktop proof: activate the detected Alice main window \
         (activate-specific-alice-window) before claiming later lesson actions."
    ));
    assert!(output.contains("Unproven:"));
}

#[test]
fn plain_output_uses_canonical_blocked_readiness_status() {
    let mut report = sequence_report(progress_with_blocker(None));
    report.passed = true;
    report.status = "blocked".into();
    report.readiness_status = "blocked_until_ui_automation".into();
    report.evidence_boundaries = vec![present_boundary("save_project")];

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("First-lesson/grading gap report: blocked"));
    assert!(output.contains(
        "Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only."
    ));
    assert!(!output.contains("First-lesson/grading gap report: ready"));
}

#[test]
fn plain_output_surfaces_blocked_save_project_proof_artifact_without_completion_claims() {
    let mut progress = progress_with_blocker(None);
    progress.next_missing_real_desktop_proof = Some(
        "next missing real-desktop proof: blocked Save Project proof artifact in desktop next-action evidence: Save dialog owner does not expose a stable proof-artifact handoff yet.".into(),
    );
    let report = sequence_report(progress);

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Save option/action evidence is not yet shown."));
    assert!(output.contains(
        "blocked Save Project proof artifact in desktop next-action evidence: \
         Save dialog owner does not expose a stable proof-artifact handoff yet."
    ));
    assert!(!output.contains("Save completion evidence"));
    assert!(!output.contains("Save completed"));
    assert!(!output.contains("Save Project succeeded"));
    assert!(!output.contains("lesson completed"));
}

#[test]
fn next_actionable_blocker_line_is_absent_without_blocker_detail() {
    let progress = progress_with_blocker(None);

    assert!(next_actionable_blocker_line(&progress).is_none());
}

#[test]
fn plain_output_falls_back_to_first_blocked_evidence_item() {
    let progress = LessonReadinessEvidenceProgress {
        total_required: 1,
        present: 0,
        missing: 0,
        invalid: 0,
        not_observed: 0,
        blocked: 1,
        summary: "0 of 1 required evidence items are present; 0 missing, 0 invalid, 0 not observed, 1 blocked.".into(),
        next_actionable_blocker: None,
        next_missing_real_desktop_proof: None,
        items: vec![LessonReadinessEvidenceProgressItem {
            evidence: "modernized desktop run execution observation".into(),
            state: "blocked".into(),
            detail: "blocked: no supported Alice desktop automation can run the world yet".into(),
        }],
    };

    assert_eq!(
        next_actionable_blocker_line(&progress).as_deref(),
        Some(
            "Next blocker: modernized desktop run execution observation: blocked: no supported Alice desktop automation can run the world yet"
        )
    );
}

#[test]
fn plain_output_escapes_control_characters_from_report_data() {
    let mut report = sequence_report(progress_with_blocker(Some("blocked evidence")));
    report.not_yet_shown[0].summary =
        "Save option/action evidence is not yet shown.\x1b[31m\nInjected line".into();

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(
        !output.contains('\x1b'),
        "plain output must not contain raw terminal control characters: {output:?}"
    );
    assert!(
        !output.contains("\nInjected"),
        "plain output must not allow evidence text to inject extra lines: {output:?}"
    );
    assert!(output.contains("\\u{1b}"));
    assert!(output.contains("\\nInjected"));
}

#[test]
fn plain_output_surfaces_evidence_gap_without_completion_or_grading_claims() {
    let report = sequence_report(progress_with_blocker(None));

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    let gap_line = output
        .lines()
        .find(|line| line.starts_with("Evidence gap:"))
        .unwrap_or_else(|| panic!("missing evidence gap line in output:\n{output}"));
    assert_eq!(gap_line, EVIDENCE_GAP_MESSAGE);
    assert_no_completion_or_grading_claims(gap_line);
}

#[test]
fn plain_output_lists_no_go_blockers_without_success_claims() {
    let mut report = sequence_report(progress_with_blocker(Some("no-go evidence")));
    report.no_go_contracts = vec![no_go_contract_fixture()];
    report.readiness_report.no_go_contracts = report.no_go_contracts.clone();
    report
        .readiness_report
        .lesson_session_readiness
        .no_go_contracts = report.no_go_contracts.clone();

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(
        output.contains("No-go blockers:"),
        "plain output must make unsupported-action no-go contracts visible:\n{output}"
    );
    assert!(
        output.contains(
            "- project_save: Evidence gap: required evidence is missing or incomplete for project save; cannot report this action as supported."
        ),
        "plain output must render no-go affordance and reason:\n{output}"
    );
    assert!(!output.contains("no_go"));
    assert!(!output.contains("missing_affordance_id"));
    assert_no_completion_or_grading_claims(&output);
}

#[test]
fn json_output_exposes_matching_sequence_and_readiness_evidence_gap() {
    let report = sequence_report(progress_with_blocker(None));

    let report_json = serde_json::to_value(&report).unwrap();

    assert_eq!(
        report_json["evidence_gap_message"].as_str(),
        Some(EVIDENCE_GAP_MESSAGE),
        "sequence report should expose evidence_gap_message: {report_json}"
    );
    assert_eq!(
        report_json["readiness_report"]["evidence_gap_message"].as_str(),
        Some(EVIDENCE_GAP_MESSAGE),
        "embedded readiness report should expose the same evidence_gap_message: {report_json}"
    );
}

#[test]
fn plain_output_surfaces_missing_original_alice_action_evidence_without_overclaiming() {
    let mut report = sequence_report(progress_with_blocker(None));
    let missing_evidence = missing_original_alice_action_evidence();
    report.original_alice_action_evidence = missing_evidence;
    report.readiness_report.original_alice_action_evidence = missing_evidence;

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Original Alice action evidence:"));
    assert!(output.contains("- Original Alice action evidence is missing."));
    assert!(output.contains(
        "- Original Alice action evidence was not found in the comparison target evidence."
    ));
    assert_plain_output_does_not_overclaim(&output);
}

#[test]
fn plain_output_omits_original_alice_action_evidence_section_when_available() {
    let mut report = sequence_report(progress_with_blocker(None));
    let available_evidence = available_original_alice_action_evidence();
    report.original_alice_action_evidence = available_evidence;
    report.readiness_report.original_alice_action_evidence = available_evidence;

    let mut output = Vec::new();
    write_first_lesson_readiness_result(&mut output, false, &report).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("Original Alice action evidence:"));
    assert!(!output.contains("Original Alice action evidence is available."));
}

fn progress_with_blocker(blocker: Option<&str>) -> LessonReadinessEvidenceProgress {
    let missing = usize::from(blocker.is_none());
    let blocked = usize::from(blocker.is_some());
    LessonReadinessEvidenceProgress {
        total_required: 1,
        present: 0,
        missing,
        invalid: 0,
        not_observed: 0,
        blocked,
        summary: format!(
            "0 of 1 required evidence items are present; {missing} missing, 0 invalid, 0 not observed, {blocked} blocked."
        ),
        next_actionable_blocker: blocker.map(str::to_string),
        next_missing_real_desktop_proof: None,
        items: vec![LessonReadinessEvidenceProgressItem {
            evidence: "modernized desktop-run-pixel-observation.json status".into(),
            state: if blocker.is_some() {
                "blocked"
            } else {
                "missing"
            }
            .into(),
            detail: "pixel observation detail".into(),
        }],
    }
}

fn sequence_report(
    progress: LessonReadinessEvidenceProgress,
) -> FirstLessonReadinessSequenceReport {
    let envelope = LessonSessionReadinessEnvelope {
        scenario_id: Some("first-lessons-real-ui-actions".into()),
        role: "student".into(),
        status: "blocked".into(),
        blocked_reason: Some("blocked_until_ui_automation".into()),
        human_summary: "blocked".into(),
        required_evidence: Vec::new(),
        no_go_contracts: Vec::new(),
    };
    let readiness_report = LessonSessionReadinessReport {
        schema_version: "eatme.alice-lesson-session-readiness/v1".into(),
        manifest_path: "comparison-manifest.json".into(),
        scenario_id: Some("first-lessons-real-ui-actions".into()),
        passed: false,
        status: "blocked".into(),
        readiness_status: "blocked_until_ui_automation".into(),
        blocked_reason: Some("blocked_until_ui_automation".into()),
        human_summary: "blocked".into(),
        evidence_gap_message: Some(EVIDENCE_GAP_MESSAGE.into()),
        desktop_proof_contract: desktop_proof_contract(),
        shown_evidence: Vec::new(),
        not_yet_shown: not_yet_shown_fixture(),
        desktop_next_action: None,
        original_alice_action_evidence: available_original_alice_action_evidence(),
        unproven_claims: unproven_claims_fixture(),
        evidence_progress: progress.clone(),
        evidence_boundaries: Vec::new(),
        required_evidence: Vec::new(),
        no_go_contracts: Vec::new(),
        lesson_session_readiness: envelope.clone(),
        role_readiness: vec![envelope],
        contract_check: LessonSessionContractCheck {
            schema_version: "eatme.alice-lesson-session-check/v1".into(),
            manifest_path: "comparison-manifest.json".into(),
            scenario_id: Some("first-lessons-real-ui-actions".into()),
            session_kind: Some("first_lesson_action_contract".into()),
            automation_status: Some("blocked".into()),
            passed: false,
            issues: Vec::new(),
        },
        execute_requested: Some(true),
        target_evidence: Vec::new(),
        issues: Vec::new(),
        limitations: Vec::new(),
    };
    FirstLessonReadinessSequenceReport {
        schema_version: "eatme.first-lesson-readiness-sequence/v1".into(),
        scenario_id: "first-lessons-real-ui-actions".into(),
        run_id: "test".into(),
        execute_requested: true,
        comparison_manifest_path: "comparison-manifest.json".into(),
        passed: false,
        status: "blocked".into(),
        readiness_status: "blocked_until_ui_automation".into(),
        blocked_reason: Some("blocked_until_ui_automation".into()),
        human_summary: "blocked".into(),
        evidence_gap_message: Some(EVIDENCE_GAP_MESSAGE.into()),
        desktop_proof_contract: desktop_proof_contract(),
        shown_evidence: Vec::new(),
        not_yet_shown: not_yet_shown_fixture(),
        desktop_next_action: None,
        original_alice_action_evidence: available_original_alice_action_evidence(),
        unproven_claims: unproven_claims_fixture(),
        evidence_progress: progress,
        evidence_boundaries: Vec::new(),
        required_evidence: Vec::new(),
        no_go_contracts: Vec::new(),
        role_readiness: Vec::new(),
        target_statuses: BTreeMap::new(),
        issues: Vec::new(),
        limitations: Vec::new(),
        readiness_report,
    }
}

fn not_yet_shown_fixture() -> Vec<ReadinessEvidenceItem> {
    vec![ReadinessEvidenceItem {
        id: "save_project".into(),
        state: "missing".into(),
        summary: "Save option/action evidence is not yet shown.".into(),
        detail: "Save option/action evidence is not yet shown.".into(),
        does_not_prove: vec!["Save completion".into()],
    }]
}

fn unproven_claims_fixture() -> Vec<String> {
    [
        "Full Alice UI automation is not proven.",
        "Grading is not proven.",
        "Creative assessment is not proven.",
        "Visible rendering correctness is not proven.",
        "Save completion is not proven.",
        "First-lesson completion is not proven.",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn missing_original_alice_action_evidence() -> OriginalAliceActionEvidenceReport {
    OriginalAliceActionEvidenceReport::missing()
}

fn available_original_alice_action_evidence() -> OriginalAliceActionEvidenceReport {
    OriginalAliceActionEvidenceReport::available()
}

fn assert_plain_output_does_not_overclaim(output: &str) {
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
            !output.contains(prohibited),
            "plain output must stay factual and bounded; found {prohibited:?} in {output}"
        );
    }
}

fn present_boundary(id: &str) -> FirstLessonEvidenceBoundary {
    FirstLessonEvidenceBoundary {
        id: id.into(),
        label: "Save Project scenario evidence".into(),
        status: "present".into(),
        source: "test".into(),
        metadata_state: "present".into(),
        detail: "Save option/action evidence is present.".into(),
        claim: "Save option/action evidence was observed.".into(),
        does_not_prove: Vec::new(),
        artifact: None,
    }
}

fn desktop_proof_contract() -> DesktopProofContract {
    DesktopProofContract {
        status: "launched_but_unverified".into(),
        reason_code: "desktop_pixel_observation_blocked".into(),
        detail: "desktop proof is not verified".into(),
        target_role: "modernized".into(),
        artifact: None,
    }
}

fn no_go_contract_fixture() -> LessonSessionNoGoContract {
    LessonSessionNoGoContract {
        target_role: "modernized".into(),
        affordance: "project_save".into(),
        decision: "no_go".into(),
        reason: "Evidence gap: required evidence is missing or incomplete for project save; cannot report this action as supported.".into(),
        missing_affordance_id: Some("deterministic-alice-project-save-affordance".into()),
    }
}

fn assert_no_completion_or_grading_claims(text: &str) {
    for unsupported_claim in [
        "lesson completed",
        "first lesson completed",
        "first-lesson completed",
        "completion confirmed",
        "readiness confirmed",
        "graded",
        "grade:",
        "score:",
        "scored",
        "certified",
        "creative assessment complete",
        "creative assessment passed",
    ] {
        assert!(
            !text.contains(unsupported_claim),
            "evidence gap message must not claim {unsupported_claim:?}: {text}"
        );
    }
}
