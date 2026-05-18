const DEFAULT_WORKFLOW_DOC: &str = include_str!("../../../docs/default-workflow-pr-readiness.md");
const SAVE_REOPEN_DOC: &str = include_str!("../../../docs/save-reopen-readiness.md");
const STARTER_PREFLIGHT_DOC: &str =
    include_str!("../../../docs/starter-project-preflight-evidence.md");

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

const NO_OP_RECOVERY_REQUIRED_WORDING: &[&str] = &[
    "No-op justification:",
    "exact PR head",
    "current check status",
    "starter/save-reopen readiness boundary",
];

#[test]
fn readiness_docs_use_plain_public_wording_for_save_reopen_boundaries() {
    let save_reopen = SAVE_REOPEN_DOC;
    let starter_preflight = STARTER_PREFLIGHT_DOC;
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
    let doc = STARTER_PREFLIGHT_DOC;

    assert_contains_all(
        "starter-project preflight doc",
        doc,
        &[
            "opening the bundled starter project",
            "does not prove that any save, reopen, or export workflow has completed successfully",
            "Only the first item is supported by this preflight report",
            "should not inherit completion claims from this document",
        ],
    );
    assert_contains_none("starter-project preflight doc", doc, FORBIDDEN_OVERCLAIMS);
}

#[test]
fn save_reopen_doc_requires_reopen_proof_separate_from_ui_action_contract() {
    let doc = SAVE_REOPEN_DOC;

    assert_contains_all(
        "save/reopen readiness doc",
        doc,
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
    let doc = SAVE_REOPEN_DOC;

    assert_contains_all(
        "save/reopen readiness doc",
        doc,
        &[
            "Report unsupported or missing affordances as `blocked`, not as success",
            "A required earlier proof or deterministic Alice affordance is missing",
            "missing hook",
            "missing precondition",
            "bounded `blocked` result",
        ],
    );
}

#[test]
fn recovery_evidence_requires_files_modified_or_explicit_no_op_with_limitations() {
    let save_reopen = SAVE_REOPEN_DOC;
    let default_workflow = DEFAULT_WORKFLOW_DOC;
    let combined = format!("{save_reopen}\n{default_workflow}");

    assert_contains_all(
        "save/reopen recovery evidence docs",
        &combined,
        &[
            "Files modified or no-op justification",
            "Files modified:",
            "No-op justification:",
            "Validation: name only commands actually run for this finalization",
            "Checks run:",
            "List only commands actually executed for this finalization",
            "No full Alice UI automation claim",
            "No grading validation claim",
            "No creative-assessment validation claim",
            "No broad product-readiness claim",
        ],
    );
}

#[test]
fn no_op_recovery_justification_is_tied_to_current_head_checks_and_starter_save_reopen_scope() {
    let save_reopen = SAVE_REOPEN_DOC;
    let default_workflow = DEFAULT_WORKFLOW_DOC;

    assert_contains_all(
        "save/reopen no-op recovery template",
        save_reopen,
        NO_OP_RECOVERY_REQUIRED_WORDING,
    );
    assert_contains_all(
        "default-workflow no-op recovery template",
        default_workflow,
        NO_OP_RECOVERY_REQUIRED_WORDING,
    );
}

#[test]
fn durable_recovery_docs_keep_pr_specific_evidence_out_of_committed_templates() {
    let save_reopen = SAVE_REOPEN_DOC;
    let default_workflow = DEFAULT_WORKFLOW_DOC;
    let combined = format!("{save_reopen}\n{default_workflow}");

    assert_contains_all(
        "durable recovery template docs",
        &combined,
        &[
            "PR #<number>",
            "<branch-name>",
            "<exact-head-sha>",
            "do not bake point-in-time branch or SHA evidence into this durable document",
        ],
    );
    assert_contains_none(
        "durable recovery template docs",
        &combined,
        &[
            "PR #172",
            "pull/172",
            "46d22db1593e245e5637a09fb2422f7134669a41",
            "wave6-save-reopen-readiness-1778302300",
            "PR #164",
            "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
        ],
    );
}

#[test]
fn save_reopen_docs_describe_artifact_evidence_without_semantic_change_overclaim() {
    let doc = SAVE_REOPEN_DOC;

    assert_contains_all(
        "save/reopen readiness doc",
        doc,
        &[
            "saved artifact from the edited project path",
            "It does not treat those artifacts as semantic project-change proof",
            "Avoid wording that turns artifact evidence into a broader product claim",
        ],
    );
    assert_contains_none(
        "save/reopen readiness doc",
        doc,
        &[
            "changed Alice project artifact",
            "changed `.a3p`",
            "semantic project change is proven",
            "broad product readiness is proven",
        ],
    );
}

#[test]
fn save_reopen_doc_documents_path_validation_defenses() {
    assert_contains_all(
        "save/reopen readiness doc",
        SAVE_REOPEN_DOC,
        &[
            "Absolute path rejection",
            "Parent traversal rejection",
            "Symlink escape rejection",
            "canonical_artifact_under",
            "artifact_info_under",
        ],
    );
}

#[test]
fn save_reopen_doc_documents_hook_api_contracts() {
    assert_contains_all(
        "save/reopen readiness doc hook API",
        SAVE_REOPEN_DOC,
        &[
            "tools/eatme-save-project",
            "tools/eatme-reopen-project",
            "eatme.alice-project-save-result/v1",
            "eatme.alice-project-reopen-result/v1",
            "scene.myFirstMethod",
        ],
    );
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
