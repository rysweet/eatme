use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
struct TsPortMetrics {
    source_file_count: usize,
    source_line_count: usize,
    test_file_count: usize,
}

#[test]
fn ts_port_build_is_healthy_when_repo_is_present() {
    let root = ts_port_root();
    if !root.join("package.json").exists() {
        eprintln!(
            "skipping alice-web-prototype build health check; missing repo at {}",
            root.display()
        );
        return;
    }

    let metrics = collect_metrics(&root);
    eprintln!(
        "alice-web-prototype metrics: source_files={}, source_lines={}, test_files={}",
        metrics.source_file_count, metrics.source_line_count, metrics.test_file_count
    );

    let test_output = Command::new("npm")
        .arg("test")
        .current_dir(&root)
        .output()
        .expect("run alice-web-prototype npm test");
    assert!(
        test_output.status.success(),
        "alice-web-prototype npm test failed (source_files={}, source_lines={}, test_files={})\nstdout:\n{}\nstderr:\n{}",
        metrics.source_file_count,
        metrics.source_line_count,
        metrics.test_file_count,
        String::from_utf8_lossy(&test_output.stdout),
        String::from_utf8_lossy(&test_output.stderr)
    );

    let build_output = Command::new("npm")
        .args(["run", "build"])
        .current_dir(&root)
        .output()
        .expect("run alice-web-prototype npm run build");
    assert!(
        build_output.status.success(),
        "alice-web-prototype npm run build failed (source_files={}, source_lines={}, test_files={})\nstdout:\n{}\nstderr:\n{}",
        metrics.source_file_count,
        metrics.source_line_count,
        metrics.test_file_count,
        String::from_utf8_lossy(&build_output.stdout),
        String::from_utf8_lossy(&build_output.stderr)
    );

    assert!(
        metrics.source_file_count > 0,
        "expected TS source files under src/"
    );
    assert!(
        metrics.source_line_count > 0,
        "expected TS source lines under src/ and test/"
    );
    assert!(
        metrics.test_file_count > 0,
        "expected TS test files under test/"
    );
}

fn ts_port_root() -> PathBuf {
    if let Ok(root) = env::var("ALICE_WEB_PROTOTYPE_ROOT") {
        return PathBuf::from(root);
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("src/alice-web-prototype");
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../../alice-web-prototype")
}

fn collect_metrics(root: &Path) -> TsPortMetrics {
    TsPortMetrics {
        source_file_count: count_ts_files(&root.join("src")),
        source_line_count: count_lines(&root.join("src")) + count_lines(&root.join("test")),
        test_file_count: count_test_files(&root.join("test")),
    }
}

fn count_ts_files(dir: &Path) -> usize {
    walk(dir)
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ts"))
        .count()
}

fn count_test_files(dir: &Path) -> usize {
    walk(dir)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".test.ts"))
                .unwrap_or(false)
        })
        .count()
}

fn count_lines(dir: &Path) -> usize {
    walk(dir)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ts") | Some("tsx")
            )
        })
        .map(|path| {
            fs::read_to_string(path)
                .map(|contents| contents.lines().count())
                .unwrap_or(0)
        })
        .sum()
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}
