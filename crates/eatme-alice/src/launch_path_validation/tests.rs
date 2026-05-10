use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn normal_components_accepts_simple_filename() {
    assert_eq!(
        normal_components(Path::new("saved-project.a3p")),
        Some(vec!["saved-project.a3p".to_string()])
    );
}

#[test]
fn normal_components_accepts_nested_relative_path() {
    assert_eq!(
        normal_components(Path::new("project-save/saved-project.a3p")),
        Some(vec![
            "project-save".to_string(),
            "saved-project.a3p".to_string(),
        ])
    );
}

#[test]
fn normal_components_rejects_absolute_path() {
    assert_eq!(normal_components(Path::new("/etc/shadow")), None);
}

#[test]
fn normal_components_rejects_parent_traversal() {
    assert_eq!(normal_components(Path::new("../escape")), None);
}

#[test]
fn normal_components_rejects_current_directory_prefix() {
    assert_eq!(normal_components(Path::new("./something")), None);
}

#[test]
fn normal_components_rejects_embedded_parent_traversal() {
    assert_eq!(normal_components(Path::new("a/../../b")), None);
}

#[test]
fn artifact_info_under_accepts_valid_relative_artifact() {
    let root = unique_test_dir("valid-artifact");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("evidence.json"), r#"{"ok":true}"#).unwrap();

    let info = artifact_info_under(&root, "evidence.json", "test_field", "test root");

    assert!(info.is_ok(), "{:?}", info);
    assert!(info.unwrap().size_bytes > 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_info_under_rejects_absolute_path() {
    let root = unique_test_dir("absolute-path");
    fs::create_dir_all(&root).unwrap();

    let result = artifact_info_under(&root, "/etc/shadow", "test_field", "test root");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("simple relative path"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_info_under_rejects_parent_traversal() {
    let root = unique_test_dir("parent-traversal");
    fs::create_dir_all(&root).unwrap();

    let result = artifact_info_under(&root, "../evidence.json", "test_field", "test root");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("simple relative path"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_info_under_rejects_nonexistent_file() {
    let root = unique_test_dir("nonexistent");
    fs::create_dir_all(&root).unwrap();

    let result = artifact_info_under(&root, "missing.json", "test_field", "test root");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not a readable artifact"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn canonical_artifact_under_accepts_artifact_within_root() {
    let root = unique_test_dir("canonical-within");
    fs::create_dir_all(&root).unwrap();
    let artifact_path = root.join("evidence.json");
    fs::write(&artifact_path, r#"{"ok":true}"#).unwrap();

    let result = canonical_artifact_under(&root, &artifact_path, "test_field", "test root");

    assert!(result.is_ok());
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn canonical_artifact_under_rejects_symlink_escaping_root() {
    let root = unique_test_dir("canonical-escape");
    let outside = root.join("outside");
    let inside = root.join("inside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&inside).unwrap();
    fs::write(outside.join("secret.json"), "secret").unwrap();
    std::os::unix::fs::symlink(outside.join("secret.json"), inside.join("symlinked.json")).unwrap();

    let result = canonical_artifact_under(
        &inside,
        &inside.join("symlinked.json"),
        "test_field",
        "test root",
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must stay under"));
    let _ = fs::remove_dir_all(root);
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-path-validation-tests")
        .join(format!("{prefix}-{nonce}"))
}
