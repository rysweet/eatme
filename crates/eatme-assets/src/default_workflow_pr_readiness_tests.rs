use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";
const FALLBACK_LOG: &str = "default-workflow-attempt.log";

#[test]
fn readiness_handoff_records_exact_head_validation_not_planned_placeholders() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);
    let branch = git_stdout(&root, &["branch", "--show-current"]);

    assert_contains_all(
        "default-workflow exact-HEAD readiness evidence",
        &evidence,
        &[
            "# Default-workflow PR readiness",
            "PR: 174",
            &format!("Branch: {branch}"),
            "Exact HEAD command: git rev-parse HEAD",
            "Merge source: origin/master",
            "External service command: gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
            "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
            "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
            "cargo run -q -p eatme-cli -- assets validate --json",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            "cargo test -q -p eatme-assets starter_project_preflight_boundary",
            "cargo test -q -p eatme-assets gadugi",
            "cargo test -q -p eatme-assets outside_in_alice_expansion_tests",
            "gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
            "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh",
            "Working tree command: git status --short",
            "Working tree: clean handoff requires no output",
            "does not claim full Save completion",
            "full UI automation",
            "grading",
            "creative assessment",
            "visible rendering correctness",
            "deployed sharing or platform success",
            "first-lesson completion",
        ],
    );
    assert_not_contains_any(
        "default-workflow exact-HEAD readiness evidence",
        &evidence,
        &[
            "[PLANNED",
            "Implementation Pending",
            "$(git branch --show-current)",
            "$(git rev-parse HEAD)",
            "Replace command substitutions",
            "use this page as the handoff checklist",
            "PR #164",
            "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
        ],
    );
}

#[test]
fn readiness_handoff_requires_external_github_service_state() {
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
fn readiness_guard_uses_the_git_repository_root_for_linked_worktrees() {
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
fn manual_fallback_log_is_not_used_as_readiness_evidence() {
    let root = repository_root();
    let evidence = read_repo_file(&root, READINESS_DOC);
    let fallback_log = read_repo_file(&root, FALLBACK_LOG);

    assert_contains_all(
        "default-workflow canonical asset boundary",
        &evidence,
        &[
            "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
            "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml",
            "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
        ],
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
        &[
            "validated exact HEAD",
            "default-workflow evidence passed",
            "ready for handoff",
        ],
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

fn git_stdout(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"));
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git stdout must be UTF-8")
        .trim()
        .to_string()
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
