use std::fs;
use std::path::{Path, PathBuf};

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

    for path in repository_files(&root) {
        if is_allowed_policy_document(&root, &path) {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };

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

fn repository_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_repository_files(root, &mut files);
    files.sort();
    files
}

fn collect_repository_files(path: &Path, files: &mut Vec<PathBuf>) {
    if is_skipped_path(path) {
        return;
    }

    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

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
        collect_repository_files(&entry.unwrap().path(), files);
    }
}

fn is_skipped_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(".git" | ".claude" | "site" | "target")
        )
    })
}

fn is_allowed_policy_document(root: &Path, path: &Path) -> bool {
    path == root.join("docs/local-hook-artifacts.md") || path == root.join(".gitignore")
}

fn list_files(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }

    repository_files(path)
        .into_iter()
        .map(|file| file.display().to_string())
        .collect()
}
