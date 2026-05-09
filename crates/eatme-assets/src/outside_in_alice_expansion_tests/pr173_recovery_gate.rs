use super::{
    assert_contains_all, default_workflow_pr_readiness_doc, sharing_readiness_boundary_doc,
};

#[test]
fn preserved_patch_artifact_validation_rejects_untrusted_patch_paths() {
    let docs = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "preserved patch artifact validation contract",
        docs,
        &[
            "treat the preserved patch as untrusted input until inspected",
            "reject absolute paths",
            "reject `..` path traversal",
            "reject secrets and credentials",
            "reject session artifacts and machine-specific files",
            "modify only repository files proven intentional by the readable patch",
        ],
    );
}

#[test]
fn pyproject_package_metadata_change_is_not_inferred_from_matching_version_value() {
    let docs = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "pyproject package metadata recovery contract",
        docs,
        &[
            "pyproject package metadata",
            "`pyproject.toml`",
            "`project.version`",
            "compare the preserved patch hunk with the current branch",
            "do not treat a matching version value as confirmation",
            "reproduce only the metadata change represented by the readable patch",
        ],
    );
}

#[test]
fn commit_path_documents_pre_commit_allow_no_config_only_for_this_repo_convention() {
    let docs = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "pre-commit no-config handling contract",
        docs,
        &[
            "`.pre-commit-config.yaml`",
            "`PRE_COMMIT_ALLOW_NO_CONFIG=1`",
            "only because the repository has no pre-commit config",
            "Cargo and MkDocs quality gates",
            "TMPDIR=/tmp ./scripts/quality-gates.sh",
        ],
    );
}

#[test]
fn no_op_output_requires_literal_no_op_evidence_for_current_pr_head() {
    let docs = default_workflow_pr_readiness_doc();

    assert_contains_all(
        "literal No-op evidence contract",
        docs,
        &[
            "literal `No-op`",
            "current PR head SHA",
            "current check status",
            "mergeability state",
            "confirmation that the preserved patch is already represented",
            "do not use no-op wording when the preserved patch is unreadable",
        ],
    );
}

#[test]
fn sharing_boundary_keeps_unreadable_patch_as_blocked_not_no_op_or_ready() {
    let docs = sharing_readiness_boundary_doc();

    assert_contains_all(
        "unreadable preserved patch sharing boundary",
        docs,
        &[
            "If the patch is unreadable, record `BLOCKED` instead of readiness or no-op.",
            "A matching version number, file value, or check result is not enough by itself.",
            "do not emit `No-op`",
            "do not claim `MERGE_READY`",
            "patch-inspection acceptance criterion is unmet",
        ],
    );
}
