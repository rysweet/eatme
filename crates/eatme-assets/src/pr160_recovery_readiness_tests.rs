use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn integration_pr160_no_op_recovery_requires_exact_head_evidence() {
    let docs = read_doc("docs/pr-160-gap-reporting-readiness.md");
    let recovery_section = section_between(
        &docs,
        "## Recovery no-op guard for requested head",
        "## Validation commands",
    );
    let current_head = current_git_head();

    assert_contains_all(
        "PR #160 exact-head no-op recovery contract",
        recovery_section,
        &[
            "No-op justification:",
            "`<exact-head-sha>` from `git rev-parse HEAD`",
            "- Exact head: <exact-head-sha> matches the PR head.",
            "GitHub Actions",
            "focused PR diff",
            "Runnable QA/scenario evidence",
            "Docs impact",
            "Quality-audit cycles",
            "PR description evidence",
            "Remaining blockers",
        ],
    );
    assert_no_stale_commit_heads(
        "PR #160 exact-head no-op recovery contract",
        recovery_section,
        &current_head,
    );
}

#[test]
fn edge_pr160_validation_commands_use_direct_no_timeout_execution() {
    let docs = read_doc("docs/pr-160-gap-reporting-readiness.md");
    let validation_section = section_after(&docs, "## Validation commands");

    assert_contains_all(
        "PR #160 validation command list",
        validation_section,
        &[
            "TMPDIR=/tmp cargo test -p eatme-alice --all-features",
            "TMPDIR=/tmp cargo test -p eatme-cli --all-features",
            "cargo run -q -p eatme-cli -- assets validate --json",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "mkdocs build --strict",
            "TMPDIR=/tmp ./scripts/quality-gates.sh",
        ],
    );
    assert_no_timeout_wrappers(validation_section);
}

#[test]
fn error_missing_pr160_merge_gate_requires_not_merge_ready_blocker() {
    let docs = read_doc("docs/pr-160-gap-reporting-readiness.md");

    assert_contains_all(
        "PR #160 missing-gate blocker contract",
        &docs,
        &[
            "NOT_MERGE_READY",
            "If any item is missing",
            "blocker",
            "Green checks and workflow completion are required, but they are not sufficient",
            "Do not claim full UI automation",
            "Save completion",
            "first-lesson completion",
        ],
    );
}

fn read_doc(relative_path: &str) -> String {
    fs::read_to_string(repository_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn section_after<'a>(docs: &'a str, heading: &str) -> &'a str {
    docs.split_once(heading)
        .map(|(_, section)| section)
        .unwrap_or_else(|| panic!("missing section heading {heading:?}"))
}

fn section_between<'a>(docs: &'a str, heading: &str, next_heading: &str) -> &'a str {
    section_after(docs, heading)
        .split_once(next_heading)
        .map(|(section, _)| section)
        .unwrap_or_else(|| panic!("missing next section heading {next_heading:?}"))
}

fn assert_contains_all(label: &str, text: &str, expected: &[&str]) {
    let missing = expected
        .iter()
        .filter(|expected| !text.contains(**expected))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "{label} is missing required text:\n{}\n\nDocument excerpt:\n{}",
        missing.join("\n"),
        excerpt(text)
    );
}

fn assert_no_stale_commit_heads(label: &str, text: &str, current_head: &str) {
    let stale_heads = commit_sha_literals(text)
        .into_iter()
        .filter(|sha| sha != current_head)
        .collect::<Vec<_>>();

    assert!(
        stale_heads.is_empty(),
        "{label} contains stale exact-head evidence; only the current git HEAD \
         may be used as a concrete SHA:\n{}\n\nCurrent HEAD: {current_head}",
        stale_heads.join("\n")
    );
}

fn commit_sha_literals(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_ascii_hexdigit())
        .filter(|token| token.len() == 40 && token.chars().all(|ch| ch.is_ascii_hexdigit()))
        .map(str::to_string)
        .collect()
}

fn current_git_head() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repository_root())
        .output()
        .expect("failed to run git rev-parse HEAD");

    assert!(
        output.status.success(),
        "git rev-parse HEAD failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let head = String::from_utf8(output.stdout)
        .expect("git rev-parse HEAD returned non-UTF-8 output")
        .trim()
        .to_string();

    assert!(
        head.len() == 40 && head.chars().all(|ch| ch.is_ascii_hexdigit()),
        "git rev-parse HEAD returned an invalid commit SHA: {head:?}"
    );
    head
}

fn assert_no_timeout_wrappers(command_section: &str) {
    let violations = command_section
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("timeout ")
                || line.starts_with("gtimeout ")
                || line.starts_with("command timeout ")
                || line.contains(" timeout ")
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "PR #160 recovery commands must run directly without timeout wrappers:\n{}",
        violations.join("\n")
    );
}

fn excerpt(text: &str) -> String {
    text.lines().take(80).collect::<Vec<_>>().join("\n")
}
