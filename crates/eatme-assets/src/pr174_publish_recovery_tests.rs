use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";
const FALLBACK_LOG: &str = "default-workflow-attempt.log";

#[test]
fn recovery_instructions_start_from_pr_174_head_ref_not_a_stale_branch() {
    let doc = normalized_readiness_doc();

    assert_contains_all(
        "PR 174 head recovery instructions",
        &doc,
        &[
            "git fetch origin pull/174/head:",
            "PR #174's actual head",
            "git rev-parse HEAD",
            "starting HEAD",
        ],
    );
}

#[test]
fn publish_evidence_template_is_bounded_to_asset_scope_and_exact_head() {
    let doc = normalized_readiness_doc();

    assert_contains_all(
        "bounded PR publish evidence template",
        &doc,
        &[
            "gh pr comment 174 --body-file",
            "Persona/scenario gap-fill readiness refreshed for HEAD",
            "Asset-scoped evidence:",
            "cargo run -q -p eatme-cli -- assets validate --json",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "Asset changes are limited to canonical persona assets, canonical EatMe scenario assets, and generator-produced Gadugi adapters under `assets/scenarios/gadugi/*.yaml`.",
            "Files modified are listed in the final handoff from `git diff --name-only <merge-base>...HEAD`",
            "Scope note:",
            "does not claim Alice UI automation, grading correctness, creative assessment quality, completed lessons, or full lesson-flow coverage",
        ],
    );
}

#[test]
fn publishing_failure_instructions_preserve_exact_ready_to_publish_text() {
    let doc = normalized_readiness_doc();

    assert_contains_all(
        "GitHub publishing failure fallback",
        &doc,
        &[
            "If GitHub publishing fails",
            "rate limiting",
            "preserve the intended PR text unchanged outside the repository",
            "record the exact `gh` failure",
        ],
    );
}

#[test]
fn fallback_log_is_not_tracked_as_pr_174_recovery_content() {
    let root = repository_root();
    let output = Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "--error-unmatch", FALLBACK_LOG])
        .output()
        .unwrap_or_else(|error| panic!("failed to inspect tracked files with git: {error}"));

    assert!(
        !output.status.success(),
        "{FALLBACK_LOG} must not be included in the final PR 174 branch; move any fallback note to a session artifact outside the repository"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalized_readiness_doc() -> String {
    normalize_whitespace(
        &fs::read_to_string(repository_root().join(READINESS_DOC))
            .unwrap_or_else(|error| panic!("failed to read {READINESS_DOC}: {error}")),
    )
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .filter(|needle| !text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required contract text: {missing:?}"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
