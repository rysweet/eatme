use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";
const FALLBACK_LOG: &str = "default-workflow-attempt.log";
const PR_NUMBER: &str = "174";
const BRANCH_NAME: &str = "wave6-persona-gap-fill-1778302300";
const CANONICAL_ASSETS: &[&str] = &[
    "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
    "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
];
const GENERATED_ADAPTERS: &[&str] = &[
    "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml",
    "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
];
const REQUIRED_COMMANDS: &[&str] = &[
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "cargo test -q -p eatme-assets starter_project_preflight_boundary",
    "cargo test -q -p eatme-assets gadugi",
    "cargo test -q -p eatme-assets outside_in_alice_expansion_tests",
    "gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
    "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh",
];
const PROHIBITED_CLAIMS: &[&str] = &[
    "full Save completion",
    "full UI automation",
    "grading",
    "creative assessment",
    "visible rendering correctness",
    "deployed sharing or platform success",
    "first-lesson completion",
];
const PROHIBITED_PLACEHOLDERS: &[&str] = &[
    "[PLANNED",
    "Implementation Pending",
    "$(git branch --show-current)",
    "$(git rev-parse HEAD)",
    "Replace command substitutions",
    "PR #164",
    "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
];
const PROHIBITED_EVIDENCE_PHRASES: &[&str] = &[
    "validated exact HEAD",
    "default-workflow evidence passed",
    "ready for handoff",
];

#[test]
fn readiness_doc_names_the_exact_pr_branch_assets_and_commands() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);

    assert_contains_all(
        "default-workflow readiness inputs",
        &evidence,
        &[
            "# Default-workflow PR readiness",
            &format!("PR: {PR_NUMBER}"),
            &format!("Branch: {BRANCH_NAME}"),
            "Exact HEAD command: git rev-parse HEAD",
            "Merge source: origin/master",
            "External service command: gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
        ],
    );
    assert_contains_all("canonical EatMe assets", &evidence, CANONICAL_ASSETS);
    assert_contains_all("generated Gadugi adapters", &evidence, GENERATED_ADAPTERS);
    assert_contains_all("required validation commands", &evidence, REQUIRED_COMMANDS);
    assert_not_contains_any(
        "default-workflow readiness inputs",
        &evidence,
        PROHIBITED_PLACEHOLDERS,
    );
    assert_not_contains_any(
        "default-workflow readiness overclaims",
        &evidence,
        PROHIBITED_EVIDENCE_PHRASES,
    );
}

#[test]
fn readiness_doc_requires_external_github_service_state() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);

    assert_contains_all(
        "default-workflow external service gate",
        &evidence,
        &[
            "## External service gate",
            "Local validation is not enough to call the PR ready",
            "gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
            "`headRefOid` | The same commit returned by `git rev-parse HEAD`.",
            "`mergeStateStatus` | `CLEAN`.",
            "`mergeable` | `MERGEABLE`.",
            "`statusCheckRollup` | Required checks completed successfully for `headRefOid`.",
            "If GitHub reports a different `headRefOid`, `DIRTY`, `CONFLICTING`, pending",
            "block readiness even when local commands pass",
        ],
    );
}

#[test]
fn readiness_doc_uses_the_git_repository_root_for_linked_worktrees() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);
    let local_home_path = ["/home/", "azureuser"].concat();
    let recovery_tmp_path = ["/tmp/", "wave7-recovery-"].concat();

    assert_contains_all(
        "default-workflow linked-worktree no-op guard",
        &evidence,
        &[
            "git rev-parse --show-toplevel",
            "git status --short",
            "linked worktree",
            "repository root",
            "not the session directory",
        ],
    );
    assert_not_contains_any(
        "default-workflow linked-worktree no-op guard",
        &evidence,
        &[
            local_home_path.as_str(),
            recovery_tmp_path.as_str(),
            "non-git path",
        ],
    );
}

#[test]
fn readiness_doc_keeps_claims_and_manual_fallbacks_bounded() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);
    let fallback_log = read_repo_file(&root, FALLBACK_LOG);

    assert_contains_all(
        "default-workflow claim boundary",
        &evidence,
        PROHIBITED_CLAIMS,
    );
    assert_contains_all(
        "manual fallback log boundary",
        &fallback_log,
        &[
            "not PR readiness evidence",
            "manual fallback",
            "must not be used to claim exact-HEAD readiness",
        ],
    );
    assert_not_contains_any(
        "manual fallback log boundary",
        &fallback_log,
        PROHIBITED_EVIDENCE_PHRASES,
    );
}

fn repository_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(manifest_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to locate repository root with git: {error}"));
    assert!(
        output.status.success(),
        "git rev-parse --show-toplevel failed from {}: {}",
        manifest_dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("git repository root must be UTF-8")
            .trim(),
    )
}

fn read_repo_file(root: &Path, relative_path: &str) -> String {
    fs::read_to_string(root.join(relative_path)).unwrap_or_else(|error| {
        panic!(
            "failed to read repository file {}: {error}",
            root.join(relative_path).display()
        )
    })
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .filter(|needle| !text.contains(**needle))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required text: {missing:?}"
    );
}

fn assert_not_contains_any(label: &str, text: &str, needles: &[&str]) {
    let found = needles
        .iter()
        .filter(|needle| text.contains(**needle))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        found.is_empty(),
        "{label} contains forbidden text: {found:?}"
    );
}
