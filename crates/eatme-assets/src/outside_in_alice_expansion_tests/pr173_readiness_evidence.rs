use super::{
    PR173_EVALUATED_LOCAL_HEAD_SHA, PR173_EVALUATED_WORKTREE_STATE,
    PR173_HISTORICAL_VALIDATION_SHA, PR173_PUBLISHED_HEAD_SHA, PR173_RECOVERY_VALIDATION_COMMANDS,
    SHARING_SUCCESS_CLAIM_PATTERNS, assert_contains_all, sharing_readiness_boundary_doc,
};

const PR173_EVIDENCE_HEADING: &str = "## PR 173 readiness evidence state separation";

#[test]
fn pr173_evidence_names_branch_master_sync_and_separated_state_shape() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

    assert_contains_all(
        "PR 173 readiness evidence state separation",
        evidence,
        &[
            "PR 173",
            "wave6-deployed-sharing-gap-1778302300",
            "origin/master",
            "published PR head SHA",
            "evaluated local HEAD SHA",
            "evaluated worktree state",
        ],
    );
    assert!(
        single_sha_for_row(evidence, "published PR head SHA").is_some(),
        "PR 173 readiness evidence must include one full 40-character published PR head SHA"
    );
    assert!(
        single_sha_for_row(evidence, "evaluated local HEAD SHA").is_some(),
        "PR 173 readiness evidence must include one full 40-character evaluated local HEAD SHA"
    );
}

#[test]
fn pr173_evidence_separates_published_pr_head_from_evaluated_local_state() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());
    let expected_pr_head =
        format!("Fetched PR `#173` head resolved to `{PR173_PUBLISHED_HEAD_SHA}`");

    assert_eq!(
        single_sha_for_row(evidence, "published PR head SHA"),
        Some(PR173_PUBLISHED_HEAD_SHA),
        "PR 173 readiness evidence must pin the published PR head to GitHub metadata"
    );
    assert_eq!(
        single_sha_for_row(evidence, "evaluated local HEAD SHA"),
        Some(PR173_EVALUATED_LOCAL_HEAD_SHA),
        "PR 173 readiness evidence must name the local checkout evaluated during recovery"
    );
    assert_ne!(
        PR173_PUBLISHED_HEAD_SHA, PR173_EVALUATED_LOCAL_HEAD_SHA,
        "this recovery evidence must not collapse the published PR head and evaluated local HEAD"
    );
    assert_contains_all(
        "PR 173 published-head and local-state identity",
        evidence,
        &[
            "`#173`",
            "`wave6-deployed-sharing-gap-1778302300`",
            &expected_pr_head,
            PR173_EVALUATED_WORKTREE_STATE,
            "not the evaluated checkout",
        ],
    );
    assert!(
        !expect_table_row(evidence, "published PR head SHA")
            .contains(PR173_HISTORICAL_VALIDATION_SHA),
        "the older documented SHA must never appear in the published PR head row"
    );
    assert!(
        !expect_table_row(evidence, "evaluated local HEAD SHA")
            .contains(PR173_HISTORICAL_VALIDATION_SHA),
        "the older documented SHA must never appear in the evaluated local HEAD row"
    );
}

#[test]
fn pr173_evidence_does_not_label_published_pr_head_as_exact_evaluated_head() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

    assert!(
        table_row(evidence, "exact evaluated HEAD SHA").is_none(),
        "PR 173 evidence must not include an exact evaluated HEAD row when the evaluated state is a dirty local checkout"
    );
    assert!(
        !expect_table_row(evidence, "PR head verification").contains("exact evaluated HEAD"),
        "PR head verification is a GitHub metadata check, not proof of the evaluated local state"
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
fn pr173_recovery_validation_commands_are_explicitly_not_rerun_for_current_head() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());
    let missing = PR173_RECOVERY_VALIDATION_COMMANDS
        .iter()
        .filter(|command| {
            !validation_status(evidence, command)
                .is_some_and(|status| status == "**not run in this recovery step**")
        })
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "PR 173 recovery evidence must mark each non-rerun validation command exactly as not run in this recovery step: {missing:?}"
    );
}

#[test]
fn pr173_historical_validation_sha_is_context_not_current_head_proof() {
    let evidence = pr173_evidence_section(sharing_readiness_boundary_doc());

    assert_contains_all(
        "PR 173 historical validation separation",
        evidence,
        &[
            PR173_HISTORICAL_VALIDATION_SHA,
            "historical context only",
            "is not current-head proof",
            PR173_PUBLISHED_HEAD_SHA,
        ],
    );
    assert!(
        historical_sha_mentions_are_context(evidence),
        "every mention of the historical validation SHA must explicitly keep it out of current-head proof"
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
    assert!(
        single_sha_for_row(
            "| evaluated local HEAD SHA | `<pending>` |",
            "evaluated local HEAD SHA"
        )
        .is_none()
    );
    assert!(
        single_sha_for_row(
            "| evaluated local HEAD SHA | `4c8118d` |",
            "evaluated local HEAD SHA"
        )
        .is_none()
    );
    assert!(
        single_sha_for_row(
            "| evaluated local HEAD SHA | `wave6-deployed-sharing-gap-1778302300` |",
            "evaluated local HEAD SHA"
        )
        .is_none()
    );
    assert_eq!(
        single_sha_for_row(
            "| evaluated local HEAD SHA | `0123456789abcdef0123456789abcdef01234567` |",
            "evaluated local HEAD SHA"
        ),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
    assert!(
        single_sha_for_row(
            "A different row includes `0123456789abcdef0123456789abcdef01234567`.",
            "evaluated local HEAD SHA"
        )
        .is_none()
    );
    assert!(
        single_sha_for_row(
            "| evaluated local HEAD SHA | `0123456789abcdef0123456789abcdef01234567` and `abcdef0123456789abcdef0123456789abcdef01` |",
            "evaluated local HEAD SHA"
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
    let end = match rest.find("\n## ") {
        Some(next_heading) => next_heading,
        None => rest.len(),
    };
    &docs[start..after_heading + end]
}

fn single_sha_for_row<'a>(evidence: &'a str, row_label: &str) -> Option<&'a str> {
    let row = table_row(evidence, row_label)?;
    let mut sha_tokens = row
        .split(|c: char| !c.is_ascii_hexdigit())
        .filter(|token| token.len() == 40 && token.chars().all(|c| c.is_ascii_hexdigit()));
    let sha = sha_tokens.next()?;
    sha_tokens.next().is_none().then_some(sha)
}

fn table_row<'a>(evidence: &'a str, row_label: &str) -> Option<&'a str> {
    let expected_cell = format!("| {row_label} |");
    evidence.lines().find(|line| line.contains(&expected_cell))
}

fn expect_table_row<'a>(evidence: &'a str, row_label: &str) -> &'a str {
    table_row(evidence, row_label)
        .unwrap_or_else(|| panic!("PR 173 evidence must include a `{row_label}` table row"))
}

fn validation_status<'a>(evidence: &'a str, command: &str) -> Option<&'a str> {
    evidence
        .lines()
        .find(|line| line.contains(command))
        .and_then(markdown_table_status)
}

fn markdown_table_status(line: &str) -> Option<&str> {
    let mut cells = line.trim().trim_matches('|').split('|').map(str::trim);
    let _command = cells.next()?;
    let status = cells.next()?;
    cells.next().is_none().then_some(status)
}

fn historical_sha_mentions_are_context(evidence: &str) -> bool {
    evidence
        .split('.')
        .map(str::trim)
        .filter(|sentence| sentence.contains(PR173_HISTORICAL_VALIDATION_SHA))
        .all(|sentence| {
            sentence.contains("historical context only")
                || sentence.contains("not current-head proof")
        })
}

fn assert_no_success_claims(evidence: &str) {
    let normalized = evidence.to_lowercase();
    let present = SHARING_SUCCESS_CLAIM_PATTERNS
        .iter()
        .filter(|phrase| normalized.contains(**phrase))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        present.is_empty(),
        "PR 173 evidence must stay bounded to readiness evidence, found success claims: {present:?}"
    );
}
