use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn repository_has_no_local_hook_artifact_directory() {
    let hook_dir = repository_root().join([".github", "hooks"].iter().collect::<PathBuf>());

    assert!(
        !hook_dir.exists(),
        "{} must not exist in the repository; local hook runtime artifacts belong outside the worktree or must be deleted before review. Found: {:?}",
        hook_dir.display(),
        list_files(&hook_dir)
    );
}

#[test]
fn repository_owned_files_do_not_reference_local_hook_runtime_artifacts() {
    let root = repository_root();
    let blocked_markers = [
        ["amplihack-", "hooks"].concat(),
        [".github/", "hooks"].concat(),
        ["${HOME}/.", "amplihack"].concat(),
        ["/home/", "azureuser"].concat(),
    ];
    let mut violations = Vec::new();

    for path in tracked_repository_files(&root) {
        if is_allowed_policy_document(&root, &path) {
            continue;
        }
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!("failed to read repository file {}: {error}", path.display())
        });
        let contents = String::from_utf8_lossy(&bytes);

        for marker in &blocked_markers {
            if contents.contains(marker) {
                violations.push(format!("{} contains {marker}", path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "local hook runtime references are only allowed in docs/local-hook-artifacts.md:\n{}",
        violations.join("\n")
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tracked_repository_files(root: &Path) -> Vec<PathBuf> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .unwrap_or_else(|error| panic!("failed to list tracked repository files: {error}"));
    assert!(
        output.status.success(),
        "failed to list tracked repository files: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut files = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| root.join(String::from_utf8_lossy(path).as_ref()))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_repository_files(root, &mut files);
    files.sort();
    files
}

fn collect_repository_files(path: &Path, files: &mut Vec<PathBuf>) {
    if is_skipped_path(path) {
        return;
    }

    let metadata = fs::metadata(path)
        .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()));

    if metadata.is_file() {
        files.push(path.to_path_buf());
        return;
    }

    if !metadata.is_dir() {
        return;
    }

    for entry in fs::read_dir(path).unwrap_or_else(|error| {
        panic!(
            "failed to read repository directory {}: {error}",
            path.display()
        )
    }) {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", path.display()));
        collect_repository_files(&entry.path(), files);
    }
}

fn is_skipped_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(".git" | ".claude" | "site" | "target" | "worktrees")
        )
    })
}

fn is_allowed_policy_document(root: &Path, path: &Path) -> bool {
    path == root.join("docs/local-hook-artifacts.md")
}

fn list_files(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }

    files_under(path)
        .into_iter()
        .map(|file| file.display().to_string())
        .collect()
}
