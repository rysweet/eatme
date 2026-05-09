use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn guard_unit_contract_resolves_active_git_root_without_hard_coded_worktree_paths() {
    let script = guard_script();
    let source = fs::read_to_string(&script)
        .unwrap_or_else(|err| panic!("missing guard script {}: {err}", script.display()));

    assert!(
        source.contains("git rev-parse --show-toplevel"),
        "guard must resolve the active repository root at runtime with git rev-parse --show-toplevel"
    );
    let suppressed_output = [">", "/dev/null"].concat();
    assert!(
        !source.contains(&suppressed_output),
        "guard must surface command failures instead of suppressing them with null-device redirection"
    );
    for forbidden in [
        workspace_root().display().to_string(),
        "wave6-real-alice-smoke-report-1778302300".to_string(),
        "/tmp/wave".to_string(),
    ] {
        assert!(
            !source.contains(&forbidden),
            "guard must not contain stale or absolute linked-worktree path {forbidden:?}"
        );
    }
}

#[test]
fn guard_integration_succeeds_for_clean_git_worktree_from_nested_directory() {
    let repo = clean_git_repo("clean-nested");
    let nested = repo.join("docs/readiness");
    fs::create_dir_all(&nested).unwrap();

    let output = Command::new(guard_script())
        .current_dir(&nested)
        .output()
        .unwrap_or_else(|err| panic!("running guard from {} failed: {err}", nested.display()));

    assert_success(
        &output,
        "guard should succeed for a clean git worktree even when invoked below the repo root",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no task-scoped changes"),
        "successful guard output should make the no-op decision explicit: {stdout}"
    );
    assert!(
        stdout.contains(&repo.display().to_string()),
        "successful guard output should identify the resolved git root for exact-head evidence: {stdout}"
    );
}

#[test]
fn guard_edge_case_fails_on_untracked_root_file_when_invoked_from_nested_directory() {
    let repo = clean_git_repo("dirty-root-file");
    let nested = repo.join("docs/readiness");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        repo.join("root-level-untracked-evidence.md"),
        "uncommitted evidence\n",
    )
    .unwrap();

    let output = Command::new(guard_script())
        .current_dir(&nested)
        .output()
        .unwrap_or_else(|err| panic!("running guard from {} failed: {err}", nested.display()));

    assert_failure(
        &output,
        "guard must treat root-level untracked files as changes, not as a clean no-op",
    );
    let combined = combined_output(&output).to_ascii_lowercase();
    assert!(
        combined.contains("git status --porcelain"),
        "dirty-worktree failure should identify the porcelain status check: {combined}"
    );
    assert!(
        combined.contains("untracked") || combined.contains("changes"),
        "dirty-worktree failure should clearly explain why the guard failed: {combined}"
    );
}

#[test]
fn guard_error_handling_fails_clearly_outside_git_worktree() {
    let non_git = scratch_root("non-git");

    let output = Command::new(guard_script())
        .current_dir(&non_git)
        .output()
        .unwrap_or_else(|err| panic!("running guard from {} failed: {err}", non_git.display()));

    assert_failure(
        &output,
        "guard must fail outside a git worktree instead of reporting a clean no-op",
    );
    let combined = combined_output(&output).to_ascii_lowercase();
    assert!(
        combined.contains("git rev-parse --show-toplevel"),
        "non-git failure should name the root-resolution command: {combined}"
    );
    assert!(
        combined.contains("git worktree") || combined.contains("git repository"),
        "non-git failure should clearly say the current directory is not a git worktree: {combined}"
    );
}

fn clean_git_repo(name: &str) -> PathBuf {
    let repo = scratch_root(name);
    run_git(&repo, ["init", "-q"]);
    run_git(
        &repo,
        ["config", "user.email", "eatme-tests@example.invalid"],
    );
    run_git(&repo, ["config", "user.name", "Eatme Tests"]);
    fs::write(repo.join("README.md"), "# Scratch repo\n").unwrap();
    run_git(&repo, ["add", "README.md"]);
    run_git(&repo, ["commit", "-qm", "initial scratch commit"]);
    repo.canonicalize().unwrap()
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|err| panic!("running git in {} failed: {err}", cwd.display()));
    assert_success(&output, "git command failed while preparing guard fixture");
}

fn guard_script() -> PathBuf {
    workspace_root().join("scripts/default-workflow-noop-guard.sh")
}

fn scratch_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join("eatme-cli-integration-tests/default-workflow-noop-guard")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}; status: {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context}; status: {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
