use super::lesson_session_helpers::{
    assert_safe_blocker_text, unique_test_dir, write_executable_blocked_first_lesson_manifest,
};
use crate::compare::check_lesson_session_readiness;

#[test]
fn lesson_session_readiness_records_failed_original_alice_action_evidence_blockers() {
    let root = unique_test_dir("failed-original-alice-action-evidence-blockers");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();

    let baseline = report
        .target_evidence
        .iter()
        .find(|target| target.role == "baseline")
        .unwrap_or_else(|| panic!("missing baseline evidence: {:?}", report.target_evidence));
    for action in [
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project",
    ] {
        let blocker = baseline
            .blockers
            .iter()
            .find(|blocker| {
                blocker.code == "missing_real_action_evidence" && blocker.action == action
            })
            .unwrap_or_else(|| panic!("missing blocker for {action}: {:?}", baseline.blockers));
        assert_eq!(
            blocker.reason,
            "Required original Alice action evidence from automation scenarios did not pass."
        );
        assert_safe_blocker_text(&blocker.reason);
    }
    let modernized = report
        .target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .unwrap_or_else(|| panic!("missing modernized evidence: {:?}", report.target_evidence));
    assert!(
        modernized.blockers.is_empty(),
        "non-original target must not receive original Alice action blockers: {:?}",
        modernized.blockers
    );
}
