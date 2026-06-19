use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn walk_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", current.display()))
        {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn count_rust_test_markers(root: &Path) -> usize {
    walk_files(&root.join("crates"), "rs")
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
                .matches("#[test]")
                .count()
        })
        .sum()
}

fn count_listed_workspace_tests(root: &Path) -> usize {
    let output = Command::new("cargo")
        .args(["test", "--workspace", "--quiet", "--", "--list"])
        .current_dir(root)
        .output()
        .expect("run cargo test --workspace -- --list");
    assert!(
        output.status.success(),
        "cargo test --workspace -- --list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.ends_with(": test"))
        .count()
}

fn read_yaml(path: &Path) -> Value {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn string_list_at(value: &Value, path: &[&str]) -> Vec<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment).unwrap_or(&Value::Null);
    }
    current
        .as_sequence()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str().map(str::to_owned)
}

fn web_capable_desktop_scenarios(root: &Path) -> usize {
    walk_files(&root.join("assets/scenarios/eatme"), "yaml")
        .into_iter()
        .filter(|path| {
            let yaml = read_yaml(path);
            string_list_at(&yaml, &["adapter", "targets"])
                .iter()
                .any(|target| target == "gadugi-cli")
        })
        .count()
}

fn generated_web_scenarios(root: &Path) -> usize {
    walk_files(&root.join("assets/scenarios/gadugi"), "yaml")
        .into_iter()
        .filter(|path| {
            let yaml = read_yaml(path);
            string_at(&yaml, &["metadata", "source_eatme_asset"]).is_some()
        })
        .count()
}

#[test]
fn workspace_reports_current_test_and_web_scenario_coverage_metrics() {
    let root = repository_root();
    let rust_test_markers = count_rust_test_markers(&root);
    let listed_workspace_tests = count_listed_workspace_tests(&root);
    let rust_source_files = walk_files(&root.join("crates"), "rs").len();
    let web_capable = web_capable_desktop_scenarios(&root);
    let generated_web = generated_web_scenarios(&root);

    println!(
        "eatme coverage summary: listed_workspace_tests={listed_workspace_tests}, rust_test_markers={rust_test_markers}, rust_source_files={rust_source_files}, web_capable_desktop_scenarios={web_capable}, generated_web_scenarios={generated_web}"
    );

    assert!(
        listed_workspace_tests >= 1_390,
        "expected at least 1390 listed workspace tests, found {listed_workspace_tests}"
    );
    assert!(
        listed_workspace_tests >= rust_test_markers,
        "listed workspace tests ({listed_workspace_tests}) should be at least marker count ({rust_test_markers})"
    );
    assert!(
        web_capable >= 26,
        "expected at least 26 web-capable scenarios, found {web_capable}"
    );
    assert!(
        generated_web >= 26,
        "expected at least 26 generated web scenarios, found {generated_web}"
    );
}
