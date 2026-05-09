use crate::{generate_gadugi_adapters, validate_assets};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";
const FALLBACK_LOG: &str = "default-workflow-attempt.log";
const PR_NUMBER: &str = "174";
const BRANCH_NAME: &str = "wave6-persona-gap-fill-1778302300";
const REQUIRED_LOCAL_CHECKS: &[&str] = &[
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
];
const REQUIRED_SOURCE_ASSETS: &[&str] = &[
    "assets/personas/alice-user-crew.yaml",
    "assets/scenarios/eatme/*.yaml",
];
const REQUIRED_GENERATED_ARTIFACTS: &[&str] = &["assets/scenarios/gadugi/*.yaml"];
const UNSUPPORTED_REVIEW_CLAIMS: &[&str] = &[
    "Alice UI automation completed a full user journey",
    "A learner completed a lesson",
    "A student world was graded or creatively assessed automatically",
    "Save/reopen/export was completed in a live Alice session",
    "Visual rendering, deployed sharing, or classroom success was verified",
];
const PROHIBITED_STALE_EVIDENCE: &[&str] = &[
    "$(git branch --show-current)",
    "PR #164",
    "885f5e8fd8115815cf2d2d507de5dc68acf5acfa",
    "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
    "ready for handoff",
    "validated exact HEAD",
    "learner-world grading passed",
    "creative assessment passed",
    "lesson completion verified",
];
static REPOSITORY_ROOT: OnceLock<PathBuf> = OnceLock::new();
static READINESS_DOC_TEXT: OnceLock<String> = OnceLock::new();
static FALLBACK_LOG_TEXT: OnceLock<String> = OnceLock::new();
static NORMALIZED_READINESS_DOC_TEXT: OnceLock<String> = OnceLock::new();
static NORMALIZED_FALLBACK_LOG_TEXT: OnceLock<String> = OnceLock::new();

#[test]
fn exact_head_readiness_contract_uses_fresh_handoff_evidence_not_checked_in_sha_values() {
    let normalized_evidence = normalized_readiness_doc();

    assert_normalized_contains_all(
        "exact-head readiness contract",
        normalized_evidence,
        &[
            "# PR 174 persona/scenario gap-fill readiness",
            "Do not treat a checked-in commit SHA as current readiness evidence",
            "final PR handoff note or CI logs after the last commit has been pushed",
            "git status --short",
            "git rev-parse HEAD",
            "gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
            &format!("| PR | `{PR_NUMBER}` |"),
            &format!("| Branch | `{BRANCH_NAME}` |"),
            "| Local HEAD | The final commit SHA from `git rev-parse HEAD`. |",
            "| PR `headRefOid` | The same SHA as Local HEAD. |",
            "| GitHub merge state | `CLEAN`. |",
            "| GitHub mergeability | `MERGEABLE`. |",
        ],
    );
    assert_normalized_not_contains_any(
        "stale exact-head readiness contract",
        normalized_evidence,
        PROHIBITED_STALE_EVIDENCE,
    );
    let sha_literals = committed_sha_literals(readiness_doc());
    assert!(
        sha_literals.is_empty(),
        "readiness doc must not check in SHA-shaped readiness evidence: {sha_literals:?}"
    );
}

#[test]
fn review_evidence_contract_is_tied_to_the_same_exact_pr_head() {
    let evidence = normalized_readiness_doc();

    assert_normalized_contains_all(
        "exact-head review evidence contract",
        evidence,
        &[
            "## Review evidence",
            "gh pr view 174 --json headRefOid,reviewDecision,reviews,comments",
            "`headRefOid` must match `git rev-parse HEAD`",
            "`reviewDecision`",
            "`reviews`",
            "`comments`",
            "same exact head",
            "Do not use stale review comments, skipped checks, or local-only validation as review evidence for a moved branch.",
        ],
    );
}

#[test]
fn local_asset_validation_contract_covers_persona_scenarios_and_generated_adapters() {
    let root = repository_root();
    let validation_report = validate_assets(root).unwrap();
    let adapter_report = generate_gadugi_adapters(root, true).unwrap();

    assert!(validation_report.passed, "{:?}", validation_report.errors);
    assert_eq!(validation_report.instructor_count, 11);
    assert_eq!(validation_report.student_count, 13);
    assert_eq!(validation_report.scenario_asset_count, 95);
    assert!(
        adapter_report.passed,
        "generated Gadugi adapters must be fresh: {:?}",
        adapter_report.errors
    );
    assert_eq!(adapter_report.generated_count, 47);
    assert_eq!(adapter_report.checked_count, 47);
}

#[test]
fn readiness_doc_keeps_source_of_truth_and_validation_claims_bounded_to_assets() {
    let evidence = normalized_readiness_doc();

    assert_normalized_contains_all("editable source assets", evidence, REQUIRED_SOURCE_ASSETS);
    assert_normalized_contains_all(
        "generated Gadugi review artifacts",
        evidence,
        REQUIRED_GENERATED_ARTIFACTS,
    );
    assert_normalized_contains_all("repository-local checks", evidence, REQUIRED_LOCAL_CHECKS);
    assert_normalized_contains_all(
        "bounded supported claims",
        evidence,
        &[
            "PR 174 fills persona/scenario gaps in editable assets",
            "Persona IDs used by canonical EatMe scenarios resolve through the persona crew",
            "Generated Gadugi adapters are fresh relative to canonical EatMe scenarios",
            "Repository-local asset validation passes for the exact PR head",
            "GitHub metadata names the same exact head and reports the PR mergeable",
        ],
    );
    assert_normalized_contains_all(
        "unsupported review claims",
        evidence,
        UNSUPPORTED_REVIEW_CLAIMS,
    );
}

#[test]
fn readiness_doc_blocks_stale_dirty_failed_pending_or_out_of_scope_evidence() {
    let evidence = normalized_readiness_doc();

    assert_normalized_contains_all(
        "readiness error handling",
        evidence,
        &[
            "Block readiness if `git status --short` reports local changes",
            "GitHub reports a different `headRefOid`",
            "a non-clean merge state",
            "failed checks",
            "pending checks",
            "checks for another commit",
            "A skipped manual Alice smoke check, for example, must not be cited as Alice UI evidence.",
            "Skipped checks are acceptable only when they are outside the persona/scenario asset scope and do not expand the readiness claim.",
        ],
    );
}

#[test]
fn manual_fallback_log_cannot_be_used_as_readiness_or_review_evidence() {
    let fallback_log = normalized_fallback_log_text();

    assert_normalized_contains_all(
        "manual fallback evidence boundary",
        fallback_log,
        &[
            "This file is not PR readiness evidence.",
            "manual fallback log must not be used to claim exact-HEAD readiness",
            "manual fallback log must not be used to claim exact-HEAD review evidence",
        ],
    );
    assert_normalized_not_contains_any(
        "manual fallback stale evidence",
        fallback_log,
        PROHIBITED_STALE_EVIDENCE,
    );
}

fn repository_root() -> &'static Path {
    REPOSITORY_ROOT
        .get_or_init(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .as_path()
}

fn readiness_doc() -> &'static str {
    READINESS_DOC_TEXT
        .get_or_init(|| repo_text(READINESS_DOC))
        .as_str()
}

fn fallback_log_text() -> &'static str {
    FALLBACK_LOG_TEXT
        .get_or_init(|| repo_text(FALLBACK_LOG))
        .as_str()
}

fn normalized_readiness_doc() -> &'static str {
    NORMALIZED_READINESS_DOC_TEXT
        .get_or_init(|| normalize_whitespace(readiness_doc()))
        .as_str()
}

fn normalized_fallback_log_text() -> &'static str {
    NORMALIZED_FALLBACK_LOG_TEXT
        .get_or_init(|| normalize_whitespace(fallback_log_text()))
        .as_str()
}

fn repo_text(relative_path: &str) -> String {
    let path = repository_root().join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read repository file {}: {error}", path.display())
    })
}

fn assert_normalized_contains_all(label: &str, normalized_text: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required text: {missing:?}"
    );
}

fn assert_normalized_not_contains_any(label: &str, normalized_text: &str, needles: &[&str]) {
    let found = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        found.is_empty(),
        "{label} contains forbidden text: {found:?}"
    );
}

fn committed_sha_literals(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_hexdigit())
        .filter(|token| token.len() == 40)
        .map(str::to_owned)
        .collect()
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    normalized
}
