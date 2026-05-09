use super::lesson_session_helpers::{
    unique_test_dir, write_executable_blocked_first_lesson_manifest,
};
use super::*;
use std::path::Path;

#[test]
fn lesson_session_readiness_preserves_blocked_project_proof_detail_in_not_yet_shown() {
    let root = unique_test_dir("blocked-project-proof-detail-readiness-check");
    let manifest_path = write_executable_blocked_first_lesson_manifest(&root, false);
    write_blocked_desktop_next_action_evidence(&root);

    let report = check_lesson_session_readiness(&manifest_path).unwrap();
    let save = report
        .not_yet_shown
        .iter()
        .find(|item| item.id == "save_project_proof_artifact")
        .unwrap_or_else(|| {
            panic!(
                "missing Save Project proof item: {:?}",
                report.not_yet_shown
            )
        });

    assert_eq!(save.state, "blocked");
    assert!(
        save.detail
            .contains("Save dialog owner does not expose a stable proof-artifact handoff yet."),
        "blocked proof detail must preserve the actionable blocker, not a generic fallback: {save:?}"
    );
    assert!(
        save.does_not_prove
            .iter()
            .any(|claim| claim == "Save completion"),
        "Save proof gaps must carry bounded non-claims: {save:?}"
    );
    assert!(
        save.does_not_prove
            .iter()
            .any(|claim| claim == "first-lesson completion"),
        "Save proof gaps must not imply lesson completion: {save:?}"
    );
}

fn write_blocked_desktop_next_action_evidence(root: &Path) {
    let evidence_dir = root
        .join("runs")
        .join("first-lessons-real-ui-actions")
        .join("modernized-first-lesson-run")
        .join("run-window-evidence");
    fs::create_dir_all(&evidence_dir).unwrap();
    fs::write(
        evidence_dir.join("desktop-first-lesson-next-action.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "eatme.alice-desktop-first-lesson-next-action/v1",
            "status": "blocked",
            "reason": "Desktop next-action evidence reached project proof collection but remains blocked.",
            "candidate_actions": ["save-project"],
            "requires_next_evidence": ["desktop Save menu readiness or invocation artifact"],
            "does_not_claim": ["Save completion", "first-lesson completion"],
            "save_project_proof_artifact": {
                "blocker": {
                    "reason": "Save dialog owner does not expose a stable proof-artifact handoff yet.",
                    "codes": ["save_dialog_owner_missing"]
                }
            },
            "select_project_proof_artifact": {
                "status": "missing",
                "reason": "Select Project proof artifact was not declared."
            }
        }))
        .unwrap(),
    )
    .unwrap();
}
