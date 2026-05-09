use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const RECOVERY_DOC: &str = "docs/pr-publish-recovery.md";
const LOCAL_HOOK_ARTIFACT_DOC: &str = "docs/local-hook-artifacts.md";
const REQUIRED_BASELINE: &str = "e14a14283480bf3bd926309b092b6a8d69713d21";
const PR_BRANCH: &str = "wave6-persona-gap-fill-1778302300";
const ARTIFACT_GLOB: &str = concat!(
    "/home/",
    "azureuser",
    "/.copilot/session-state/0d7fa6b6-9ef5-4278-a6c8-5672d3328455/files/wave7-pr174-*"
);
const ALLOWED_METADATA_SCOPE: &[&str] = &[
    "pyproject.toml",
    "mkdocs.yml",
    "package-facing metadata documentation",
];

static REPOSITORY_ROOT: OnceLock<PathBuf> = OnceLock::new();
static RECOVERY_DOC_TEXT: OnceLock<String> = OnceLock::new();
static RECOVERY_EVIDENCE_TEXT: OnceLock<String> = OnceLock::new();
static NORMALIZED_RECOVERY_DOC_TEXT: OnceLock<String> = OnceLock::new();
static NORMALIZED_RECOVERY_EVIDENCE_TEXT: OnceLock<String> = OnceLock::new();

#[test]
fn evidence_collector_contract_requires_exact_baseline_artifacts_and_config_detection() {
    assert_normalized_contains_all(
        "Evidence Collector contract",
        normalized_recovery_evidence_text(),
        &[
            "## Evidence Collector",
            "Gather local git state",
            "fetch PR #174",
            "verify required head SHA",
            REQUIRED_BASELINE,
            ARTIFACT_GLOB,
            LOCAL_HOOK_ARTIFACT_DOC,
            "confirm repo validation surfaces",
            "detect pre-commit config presence",
            "test ! -f .pre-commit-config.yaml",
        ],
    );
}

#[test]
fn pr_state_inspector_contract_requires_same_head_readiness_evidence() {
    assert_normalized_contains_all(
        "PR State Inspector contract",
        normalized_recovery_doc(),
        &[
            "## PR State Inspector",
            "open/draft status",
            "base branch",
            "head SHA",
            "mergeability",
            "review decision",
            "status checks",
            "changed files",
            "commits",
            "Green checks are evidence, not sufficient merge-ready proof",
            "headRefOid",
            "mergeStateStatus",
        ],
    );
}

#[test]
fn github_service_adapter_contract_bounds_external_calls_and_retries() {
    assert_normalized_contains_all(
        "GitHub Service Adapter contract",
        normalized_recovery_doc(),
        &[
            "## GitHub Service Adapter",
            "authenticated `gh` CLI as the only GitHub API client",
            "gh auth status --hostname github.com",
            "gh pr view 174",
            "Authentication failure",
            "API errors",
            "Rate limiting",
            "Network interruption",
            "retry read-only GitHub metadata reads at most once",
            "never retry `git push` blindly",
            "preserve the intended PR text unchanged outside the repository",
            "record the exact `gh` failure",
            "blocked report",
        ],
    );
}

#[test]
fn artifact_reconciler_contract_bounds_intent_to_focused_metadata_only() {
    assert_normalized_contains_all(
        "Artifact Reconciler contract",
        normalized_recovery_doc(),
        &[
            "## Artifact Reconciler",
            "Compare preserved artifact intent against current PR contents",
            "focused metadata",
            "already present",
            "out of scope",
            "ambiguous",
            "blocked evidence gap",
            "Do not execute artifact contents",
        ],
    );
    assert_normalized_contains_all(
        "allowed focused metadata scope",
        normalized_recovery_doc(),
        ALLOWED_METADATA_SCOPE,
    );
}

#[test]
fn validator_contract_uses_repository_gates_without_timeout_wrappers() {
    assert_normalized_contains_all(
        "Validator contract",
        normalized_recovery_doc(),
        &[
            "## Validator",
            "Run existing repository-appropriate Cargo/MkDocs/asset validation",
            "NODE_OPTIONS=--max-old-space-size=32768",
            "cargo run -q -p eatme-cli -- assets validate --json",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "mkdocs build --strict",
            "Do not use timeout wrappers",
        ],
    );
}

#[test]
fn commit_push_handler_contract_allows_pre_commit_escape_only_after_validation() {
    assert_normalized_contains_all(
        "Commit/Push Handler contract",
        normalized_recovery_doc(),
        &[
            "## Commit/Push Handler",
            "Only if required",
            "commit and push artifact-proven focused metadata changes",
            "Do not add .pre-commit-config.yaml",
            "PRE_COMMIT_ALLOW_NO_CONFIG=1",
            "pre-commit is installed",
            "no repo pre-commit config exists",
            "repository gates passed",
            "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
            PR_BRANCH,
        ],
    );
}

#[test]
fn readiness_reporter_contract_requires_literal_no_op_recovery_and_strict_blockers() {
    assert_normalized_contains_all(
        "Readiness Reporter contract",
        normalized_recovery_doc(),
        &[
            "## Readiness Reporter",
            "No-op justification:",
            REQUIRED_BASELINE,
            "Preserved wave7-pr174 artifacts were inspected",
            "Worktree: clean.",
            "Merge-ready blockers/evidence:",
            "Green checks alone were not treated as merge-ready.",
            "Manual merge: not performed.",
            "No recovery commit was made and PR 174 was not manually merged.",
        ],
    );
}

#[test]
fn recovery_workflow_orders_artifact_accounting_before_no_op_commit_or_blocked_report() {
    assert_contains_in_order(
        "recovery workflow order",
        recovery_doc(),
        &[
            "## Recovery components",
            "### 1. Verify the local scope",
            "### 2. Fetch and verify the required PR head",
            "### 3. Capture live PR state",
            "### 4. Account for preserved artifacts",
            "### 5. Reconcile artifact intent with the PR head",
            "### 6. Apply a focused recovery change when required",
            "### 7. Validate through repository gates",
            "### 8. Commit and push only when recovery changed files",
            "### 9. Refresh PR state after any push",
        ],
    );
}

#[test]
fn edge_cases_force_blocked_report_or_no_op_not_merge_ready_claims() {
    assert_normalized_contains_all(
        "blocked edge cases",
        normalized_recovery_doc(),
        &[
            "artifact access denied",
            "required artifacts are inaccessible",
            "PR head moved away from the required baseline",
            "artifact intent is ambiguous",
            "out-of-scope changes",
            "dirty worktree",
            "pending checks",
            "failed checks",
            "GitHub authentication failure",
            "GitHub API errors",
            "GitHub rate limiting",
            "network interruption after the single allowed read-only retry",
            "green checks alone are not merge-ready",
        ],
    );
}

fn repository_root() -> &'static Path {
    REPOSITORY_ROOT
        .get_or_init(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .as_path()
}

fn recovery_doc() -> &'static str {
    RECOVERY_DOC_TEXT
        .get_or_init(|| repo_text(RECOVERY_DOC))
        .as_str()
}

fn normalized_recovery_doc() -> &'static str {
    NORMALIZED_RECOVERY_DOC_TEXT
        .get_or_init(|| normalize_whitespace(recovery_doc()))
        .as_str()
}

fn recovery_evidence_text() -> &'static str {
    RECOVERY_EVIDENCE_TEXT
        .get_or_init(|| format!("{}\n{}", recovery_doc(), repo_text(LOCAL_HOOK_ARTIFACT_DOC)))
        .as_str()
}

fn normalized_recovery_evidence_text() -> &'static str {
    NORMALIZED_RECOVERY_EVIDENCE_TEXT
        .get_or_init(|| normalize_whitespace(recovery_evidence_text()))
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

fn assert_contains_in_order(label: &str, text: &str, needles: &[&str]) {
    let mut cursor = 0;
    let mut missing_or_unordered = Vec::new();
    for needle in needles {
        match text[cursor..].find(needle) {
            Some(offset) => cursor += offset + needle.len(),
            None => missing_or_unordered.push(*needle),
        }
    }
    assert!(
        missing_or_unordered.is_empty(),
        "{label} is missing required ordered text: {missing_or_unordered:?}"
    );
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
