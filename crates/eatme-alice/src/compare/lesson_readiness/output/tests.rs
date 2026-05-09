use super::*;
use crate::compare::LessonReadinessEvidenceProgressItem;

#[test]
fn not_yet_shown_preserves_blocked_project_proof_artifact_detail() {
    let blocked_detail = "blocked Save Project proof artifact in desktop next-action evidence: \
        Save dialog owner does not expose a stable proof-artifact handoff yet.";
    let progress = LessonReadinessEvidenceProgress {
        total_required: 1,
        present: 0,
        missing: 0,
        invalid: 0,
        not_observed: 0,
        blocked: 1,
        summary: "0 of 1 required evidence items are present; 0 missing, 0 invalid, 0 not observed, 1 blocked.".into(),
        next_actionable_blocker: None,
        next_missing_real_desktop_proof: Some(format!(
            "next missing real-desktop proof: {blocked_detail}"
        )),
        items: vec![LessonReadinessEvidenceProgressItem {
            evidence: "Save Project proof artifact".into(),
            state: "blocked".into(),
            detail: blocked_detail.into(),
        }],
    };

    let items = not_yet_shown(&progress, &[]);
    let save = items
        .iter()
        .find(|item| item.id == "save_project_proof_artifact")
        .unwrap_or_else(|| panic!("missing Save Project proof item: {items:?}"));

    assert_eq!(
        save.summary,
        "Save option/action evidence is not yet shown."
    );
    assert_eq!(save.detail, blocked_detail);
    assert!(
        save.does_not_prove
            .iter()
            .any(|claim| claim == "Save completion")
    );
    assert!(
        save.does_not_prove
            .iter()
            .any(|claim| claim == "first-lesson completion")
    );
}
