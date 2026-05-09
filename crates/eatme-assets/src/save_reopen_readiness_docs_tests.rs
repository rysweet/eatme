use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_PLAIN_WORDING: &[&str] = &[
    "action evidence",
    "hook-declared",
    "manifest-level evidence only",
    "source boundary",
];

const FORBIDDEN_OVERCLAIMS: &[&str] = &[
    "full ui automation is proven",
    "visible rendering correctness is proven",
    "save completion is proven",
    "deployed sharing worked",
    "platform success is proven",
    "first-lesson completion is proven",
    "lesson is complete",
    "grades learner work",
    "assesses creativity",
];

#[test]
fn readiness_docs_use_plain_public_wording_for_save_reopen_boundaries() {
    let save_reopen = read_doc("save-reopen-readiness.md");
    let starter_preflight = read_doc("starter-project-preflight-evidence.md");
    let combined = format!("{save_reopen}\n{starter_preflight}");

    assert_contains_all(
        "readiness docs",
        &combined,
        &[
            "saved artifact",
            "reopened-state evidence",
            "separate persistence evidence",
            "First-lesson completion is not proven",
        ],
    );
    assert_contains_none("readiness docs", &combined, FORBIDDEN_PLAIN_WORDING);
}

#[test]
fn starter_preflight_doc_does_not_borrow_persistence_or_lesson_completion_claims() {
    let doc = read_doc("starter-project-preflight-evidence.md");

    assert_contains_all(
        "starter-project preflight doc",
        &doc,
        &[
            "opening the bundled starter project",
            "does not prove that any save, reopen, or export workflow has completed successfully",
            "Only the first item is supported by this preflight report",
            "should not inherit completion claims from this document",
        ],
    );
    assert_contains_none("starter-project preflight doc", &doc, FORBIDDEN_OVERCLAIMS);
}

#[test]
fn save_reopen_doc_requires_reopen_proof_separate_from_ui_action_contract() {
    let doc = read_doc("save-reopen-readiness.md");

    assert_contains_all(
        "save/reopen readiness doc",
        &doc,
        &[
            "Do not infer reopen proof from `ui-action-contract.json` unless that file explicitly contains a dedicated `reopen-project` probe",
            "reopen proof requires its own explicit persistence evidence",
            "depends on accepted `save-project` proof from the same run",
            "reports `source_saved_project_artifact` as the same canonical artifact",
        ],
    );
}

#[test]
fn missing_or_unsupported_readiness_evidence_is_blocked_not_success() {
    let doc = read_doc("save-reopen-readiness.md");

    assert_contains_all(
        "save/reopen readiness doc",
        &doc,
        &[
            "Report unsupported or missing affordances as `blocked`, not as success",
            "A required earlier proof or deterministic Alice affordance is missing",
            "missing hook",
            "missing precondition",
            "bounded `blocked` result",
        ],
    );
}

fn read_doc(name: &str) -> String {
    fs::read_to_string(repository_root().join("docs").join(name)).unwrap()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required wording: {missing:?}"
    );
}

fn assert_contains_none(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        present.is_empty(),
        "{label} still contains non-plain or overbroad wording: {present:?}"
    );
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
