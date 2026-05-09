use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn quality_gate_script_remains_the_authoritative_rust_recovery_entrypoint() {
    let script = read_workspace_file("scripts/quality-gates.sh");
    let workflow = read_workspace_file(".github/workflows/quality-gates.yml");

    for command in [
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-features",
        "cargo llvm-cov --workspace --all-features --fail-under-lines",
    ] {
        assert_contains(&script, command);
        assert_contains(&workflow, command);
    }
    assert_contains(&script, "MODULE_MAX_LINES=\"${MODULE_MAX_LINES:-500}\"");
    assert_contains(&workflow, "MODULE_MAX_LINES=\"${MODULE_MAX_LINES:-500}\"");
}

#[test]
fn readiness_reference_keeps_docs_build_distinct_from_rust_quality_gate() {
    let doc = read_workspace_file("docs/default-workflow-pr-readiness.md");

    assert_contains(
        &doc,
        "`scripts/quality-gates.sh` is the authoritative local validation entrypoint",
    );
    assert_contains(
        &doc,
        "`mkdocs build --strict` is the authoritative documentation check.",
    );
    assert_contains(
        &doc,
        "separate docs-site validation command, not part of `scripts/quality-gates.sh`",
    );
    assert_contains(
        &doc,
        "Do not describe a passing `scripts/quality-gates.sh` run as docs-site evidence.",
    );
}

#[test]
fn pr188_handoff_contract_requires_exact_final_head_evidence_after_validation() {
    let doc = read_workspace_file("docs/default-workflow-pr-readiness.md");

    assert_contains(&doc, "PR #188 uses the same bounded evidence shape.");
    assert_contains(&doc, "exact final commit SHA");
    assert_contains(
        &doc,
        "NODE_OPTIONS=--max-old-space-size=32768\nTMPDIR=/tmp ./scripts/quality-gates.sh",
    );
    assert_contains(&doc, "`mkdocs build --strict` result when docs changed");
    assert_contains(&doc, "If another commit is added");
    assert_contains(&doc, "after the PR body or review comment is updated");
}

#[test]
fn alice_smoke_doc_uses_collection_language_for_first_lesson_boundary_evidence() {
    let doc = read_workspace_file("docs/alice-lesson-smoke.md");

    assert_contains(
        &doc,
        "| Collect bounded first-lesson scenario evidence | `first-lessons-real-ui-actions` |",
    );
    for forbidden in [
        "Prove the student first-lesson scenario has bounded automation scenario evidence",
        "unless the matching evidence boundary is present",
    ] {
        assert_not_contains(&doc, forbidden);
    }
}

#[test]
fn alice_smoke_pr_recovery_non_claims_are_unconditional() {
    let doc = read_workspace_file("docs/alice-lesson-smoke.md");
    let recovery = doc
        .split("For PR recovery,")
        .nth(1)
        .unwrap_or_else(|| panic!("missing PR recovery evidence section in alice smoke doc"));

    for non_claim in [
        "full Alice UI automation",
        "grading",
        "creative assessment",
        "visible rendering correctness",
        "Save completion",
        "first-lesson completion",
    ] {
        assert_contains(recovery, non_claim);
    }
    assert_not_contains(recovery, "unless");
    assert_not_contains(recovery, "when boundary evidence is present");
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "expected text to contain {expected:?}"
    );
}

fn assert_not_contains(text: &str, forbidden: &str) {
    assert!(
        !text.contains(forbidden),
        "text must not contain unsupported PR #188 evidence wording {forbidden:?}"
    );
}
