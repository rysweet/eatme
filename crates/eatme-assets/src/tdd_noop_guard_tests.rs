use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn tdd_noop_guard_script_is_repository_owned_and_executable() {
    let script = guard_script_path();

    assert!(
        script.is_file(),
        "{} must exist as the repository-owned TDD no-op guard",
        script.display()
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(&script).unwrap().permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "{} must be executable by the workflow",
            script.display()
        );
    }
}

#[test]
fn tdd_noop_guard_resolves_git_linked_worktree_from_nested_cwd() {
    let fixture = GitFixture::new("linked-clean");
    let linked = fixture.linked_worktree("pr-run-observe-gap");
    let nested = linked.path.join("docs/readiness");
    fs::create_dir_all(&nested).unwrap();

    let output = run_guard(&nested);
    let text = output_text(&output);

    assert!(
        output.status.success(),
        "clean linked worktree should report a clean no-op:\n{text}"
    );
    assert_contains(
        &text,
        &format!("repository_root={}", linked.path.display()),
        "guard must use git rev-parse --show-toplevel from the active linked worktree",
    );
    assert_contains(
        &text,
        "status=clean-noop",
        "guard must name the clean no-op state",
    );
    assert!(
        !text.contains(&format!("repository_root={}", fixture.main.display())),
        "guard must not inspect the original repository when invoked from a linked worktree:\n{text}"
    );
}

#[test]
fn tdd_noop_guard_reports_dirty_linked_worktree_instead_of_clean_noop() {
    let fixture = GitFixture::new("linked-dirty");
    let linked = fixture.linked_worktree("pr-run-observe-dirty");
    fs::write(linked.path.join("run-observe-readiness-gaps.txt"), "gap").unwrap();

    let output = run_guard(&linked.path);
    let text = output_text(&output);

    assert!(
        !output.status.success(),
        "dirty linked worktree must not be reported as a clean no-op:\n{text}"
    );
    assert_contains(
        &text,
        &format!("repository_root={}", linked.path.display()),
        "dirty report must still name the active linked worktree root",
    );
    assert_contains(
        &text,
        "status=dirty",
        "dirty worktree must be surfaced explicitly",
    );
    assert_contains(
        &text,
        "run-observe-readiness-gaps.txt",
        "dirty report must include changed paths",
    );
    assert!(
        !text.contains("status=clean-noop"),
        "dirty worktree must not emit a clean no-op status:\n{text}"
    );
}

#[test]
fn tdd_noop_guard_fails_closed_outside_git_worktree() {
    let root = unique_test_dir("non-git");
    let outside_git = root.join("scratch");
    fs::create_dir_all(&outside_git).unwrap();

    let output = run_guard(&outside_git);
    let text = output_text(&output);

    assert!(
        !output.status.success(),
        "non-git directory must fail closed:\n{text}"
    );
    assert_contains(
        &text,
        "status=not-a-git-worktree",
        "guard must name the non-git failure state",
    );
    assert_contains(
        &text,
        "git rev-parse --show-toplevel failed",
        "guard must expose the Git root resolution failure",
    );
    assert!(
        !text.contains("status=clean-noop"),
        "non-git directory must never be treated as clean:\n{text}"
    );

    let _ = fs::remove_dir_all(root);
}

struct GitFixture {
    root: PathBuf,
    main: PathBuf,
}

struct LinkedWorktree {
    path: PathBuf,
}

impl GitFixture {
    fn new(prefix: &str) -> Self {
        let root = unique_test_dir(prefix);
        let main = root.join("main");
        fs::create_dir_all(&main).unwrap();
        git(&main, &["init"]);
        git(
            &main,
            &["config", "user.email", "eatme-tests@example.invalid"],
        );
        git(&main, &["config", "user.name", "Eatme Tests"]);
        fs::write(main.join("README.md"), "eatme test fixture\n").unwrap();
        git(&main, &["add", "README.md"]);
        git(&main, &["commit", "-m", "initial fixture"]);
        Self { root, main }
    }

    fn linked_worktree(&self, branch: &str) -> LinkedWorktree {
        let path = self.root.join(branch);
        git(
            &self.main,
            &[
                "worktree",
                "add",
                "-b",
                branch,
                path.to_str().expect("test worktree path must be UTF-8"),
            ],
        );
        LinkedWorktree { path }
    }
}

impl Drop for GitFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_guard(cwd: &Path) -> Output {
    let script = guard_script_path();
    Command::new(&script)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "failed to execute {} from {}: {error}",
                script.display(),
                cwd.display()
            )
        })
}

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"));
    assert!(
        output.status.success(),
        "git {args:?} failed in {}:\n{}",
        cwd.display(),
        output_text(&output)
    );
}

fn guard_script_path() -> PathBuf {
    repository_root().join("scripts/tdd-noop-guard.sh")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .join("eatme-assets-tdd-noop-guard-tests")
        .join(format!("{prefix}-{}-{nonce}", std::process::id()))
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_contains(text: &str, needle: &str, context: &str) {
    assert!(
        text.contains(needle),
        "{context}; missing {needle:?} in:\n{text}"
    );
}
