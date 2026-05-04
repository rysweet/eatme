use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn archives_existing_run_dir_instead_of_deleting_it() {
    let root = unique_test_dir("archive-existing-run");
    let run_dir = root.join("runs/real-alice-launch-smoke/reused-run");
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(run_dir.join("manifest.json"), "old evidence").unwrap();

    prepare_run_dir(&run_dir).unwrap();

    assert!(run_dir.join("screenshots").is_dir());
    let archived_manifest = fs::read_dir(run_dir.parent().unwrap())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("reused-run.previous-"))
                .unwrap_or(false)
                && path.join("manifest.json").is_file()
        });
    assert!(
        archived_manifest.is_some(),
        "existing evidence should be archived next to the new run"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_non_kebab_case_scenario_names() {
    assert!(validate_scenario_name("../bad").is_err());
    assert!(validate_scenario_name("building-a-scene-first-world").is_ok());
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("eatme-alice-tests")
        .join(format!("{prefix}-{nonce}"))
}
