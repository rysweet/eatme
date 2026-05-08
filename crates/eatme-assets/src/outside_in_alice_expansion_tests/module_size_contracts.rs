use std::fs;
use std::path::{Path, PathBuf};

use super::repository_root;

const RUST_MODULE_MAX_LINES: usize = 500;
const PARENT_MODULE: &str = "crates/eatme-assets/src/outside_in_alice_expansion_tests.rs";
const CHILD_MODULE_DIR: &str = "crates/eatme-assets/src/outside_in_alice_expansion_tests";

#[test]
fn alice_expansion_test_modules_stay_within_quality_gate_line_limit() {
    let root = repository_root();
    let mut module_paths = vec![root.join(PARENT_MODULE)];
    module_paths.extend(rust_files_in(&root.join(CHILD_MODULE_DIR)));

    let mut failures = Vec::new();

    for path in module_paths {
        let content = fs::read_to_string(&path).unwrap();
        let line_count = source_line_count(&content);

        if line_count > RUST_MODULE_MAX_LINES {
            failures.push(format!(
                "{} has {line_count} lines; maximum is {RUST_MODULE_MAX_LINES}",
                path.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "outside-in Alice expansion test modules must stay within the quality-gate line limit:\n{}",
        failures.join("\n")
    );
}

fn rust_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn source_line_count(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }

    let newline_count = content.bytes().filter(|byte| *byte == b'\n').count();
    if content.ends_with('\n') {
        newline_count
    } else {
        newline_count + 1
    }
}
