use std::fs;

use super::{assert_contains_all, repository_root};

const PR173_EVIDENCE_HEADING: &str = "## PR 173 exact-head readiness evidence";

#[test]
fn pr173_evidence_names_branch_master_sync_and_exact_head_shape() {
    let root = repository_root();
    let docs = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();
    let evidence = pr173_evidence_section(&docs);

    assert_contains_all(
        "PR 173 exact-head readiness evidence",
        evidence,
        &[
            "PR 173",
            "wave6-deployed-sharing-gap-1778302300",
            "origin/master",
            "exact evaluated HEAD SHA",
        ],
    );
    assert!(
        has_40_hex_token(evidence),
        "PR 173 readiness evidence must include one full 40-character evaluated HEAD SHA"
    );
}

#[test]
fn pr173_evidence_lists_required_validation_gates_without_manual_fallback() {
    let root = repository_root();
    let docs = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();
    let evidence = pr173_evidence_section(&docs);

    assert_contains_all(
        "PR 173 validation evidence",
        evidence,
        &[
            "mkdocs build --strict",
            "cargo run -q -p eatme-cli -- assets validate --json",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "TMPDIR=/tmp",
            "./scripts/quality-gates.sh",
        ],
    );
    assert!(
        !evidence.to_lowercase().contains("manual fallback"),
        "PR 173 readiness evidence must not rely on the invalid manual fallback path"
    );
}

#[test]
fn pr173_evidence_keeps_forbidden_claims_explicitly_unproven() {
    let root = repository_root();
    let docs = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();
    let evidence = pr173_evidence_section(&docs);

    assert_contains_all(
        "PR 173 bounded wording evidence",
        evidence,
        &[
            "does not claim hosted sharing",
            "deployed sharing",
            "platform success",
            "full UI automation",
            "grading",
            "creative assessment",
            "Save completion",
            "visible rendering correctness",
            "first-lesson completion",
        ],
    );
    assert_no_success_claims(evidence);
}

#[test]
fn exact_head_detector_rejects_placeholders_short_hashes_and_branch_names() {
    assert!(!has_40_hex_token("exact evaluated HEAD SHA: <pending>"));
    assert!(!has_40_hex_token("exact evaluated HEAD SHA: 4c8118d"));
    assert!(!has_40_hex_token(
        "exact evaluated HEAD SHA: wave6-deployed-sharing-gap-1778302300"
    ));
    assert!(has_40_hex_token(
        "exact evaluated HEAD SHA: 0123456789abcdef0123456789abcdef01234567"
    ));
}

fn pr173_evidence_section(docs: &str) -> &str {
    let start = docs.find(PR173_EVIDENCE_HEADING).unwrap_or_else(|| {
        panic!("docs/sharing-readiness-boundary.md must include `{PR173_EVIDENCE_HEADING}`")
    });
    let after_heading = start + PR173_EVIDENCE_HEADING.len();
    let rest = &docs[after_heading..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &docs[start..after_heading + end]
}

fn has_40_hex_token(text: &str) -> bool {
    text.split(|c: char| !c.is_ascii_hexdigit())
        .any(|token| token.len() == 40 && token.chars().all(|c| c.is_ascii_hexdigit()))
}

fn assert_no_success_claims(evidence: &str) {
    let normalized = evidence.to_lowercase();
    let forbidden = [
        "hosted sharing passed",
        "hosted sharing works",
        "deployed sharing passed",
        "deployed sharing works",
        "platform success passed",
        "full ui automation passed",
        "grading passed",
        "creative assessment passed",
        "save completion passed",
        "visible rendering correctness passed",
        "first-lesson completion passed",
    ];
    let present = forbidden
        .iter()
        .filter(|phrase| normalized.contains(**phrase))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        present.is_empty(),
        "PR 173 evidence must stay bounded to readiness evidence, found success claims: {present:?}"
    );
}
