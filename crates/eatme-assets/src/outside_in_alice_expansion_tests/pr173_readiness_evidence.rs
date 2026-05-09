use super::{assert_contains_all, sharing_readiness_boundary_doc};

const PR173_EVIDENCE_HEADING: &str = "## PR 173 exact-head readiness evidence";

#[test]
fn pr173_evidence_names_branch_master_sync_and_exact_head_shape() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

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
        exact_evaluated_head_sha(evidence).is_some(),
        "PR 173 readiness evidence must include one full 40-character evaluated HEAD SHA in the exact-head row"
    );
}

#[test]
fn pr173_evidence_lists_required_validation_gates_without_manual_fallback() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

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
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

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
    assert!(exact_evaluated_head_sha("| exact evaluated HEAD SHA | `<pending>` |").is_none());
    assert!(exact_evaluated_head_sha("| exact evaluated HEAD SHA | `4c8118d` |").is_none());
    assert!(
        exact_evaluated_head_sha(
            "| exact evaluated HEAD SHA | `wave6-deployed-sharing-gap-1778302300` |"
        )
        .is_none()
    );
    assert_eq!(
        exact_evaluated_head_sha(
            "| exact evaluated HEAD SHA | `0123456789abcdef0123456789abcdef01234567` |"
        ),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
    assert!(
        exact_evaluated_head_sha(
            "A different row includes `0123456789abcdef0123456789abcdef01234567`."
        )
        .is_none()
    );
    assert!(
        exact_evaluated_head_sha(
            "| exact evaluated HEAD SHA | `0123456789abcdef0123456789abcdef01234567` and `abcdef0123456789abcdef0123456789abcdef01` |"
        )
        .is_none()
    );
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

fn exact_evaluated_head_sha(evidence: &str) -> Option<&str> {
    let row = evidence
        .lines()
        .find(|line| line.contains("| exact evaluated HEAD SHA |"))?;
    let mut sha_tokens = row
        .split(|c: char| !c.is_ascii_hexdigit())
        .filter(|token| token.len() == 40 && token.chars().all(|c| c.is_ascii_hexdigit()));
    let sha = sha_tokens.next()?;
    sha_tokens.next().is_none().then_some(sha)
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
