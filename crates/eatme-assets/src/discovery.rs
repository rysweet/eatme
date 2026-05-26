use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn scenario_asset_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_yaml_paths(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_yaml_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_yaml_paths(&path, paths)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension == "yaml" || extension == "yml")
            .unwrap_or(false)
        {
            paths.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scenario_asset_paths_returns_empty_for_missing_root() {
        let root = unique_test_dir("missing-root");
        let missing = root.join("does-not-exist");

        let paths = scenario_asset_paths(&missing).unwrap();

        assert!(paths.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scenario_asset_paths_collects_recursive_yaml_files_in_sorted_order() {
        let root = unique_test_dir("recursive-yaml");
        let nested = root.join("nested/deeper");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("b.yaml"), "id: b\n").unwrap();
        fs::write(root.join("ignore.txt"), "nope\n").unwrap();
        fs::write(root.join("UPPER.YAML"), "id: upper\n").unwrap();
        fs::write(nested.join("a.yml"), "id: a\n").unwrap();
        fs::write(nested.join("z.json"), "{}\n").unwrap();

        let paths = scenario_asset_paths(&root).unwrap();
        let relative = paths
            .iter()
            .map(|path| {
                path.strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect::<Vec<_>>();

        assert_eq!(relative, vec!["b.yaml", "nested/deeper/a.yml"]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scenario_asset_paths_reports_read_dir_errors_with_root_context() {
        let root = unique_test_dir("file-root");
        fs::create_dir_all(&root).unwrap();
        let file_root = root.join("not-a-directory.yaml");
        fs::write(&file_root, "id: single\n").unwrap();

        let error = scenario_asset_paths(&file_root).unwrap_err();

        assert!(error.to_string().contains(&file_root.display().to_string()));
        let _ = fs::remove_dir_all(root);
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::current_dir()
            .unwrap()
            .join("target/test-artifacts")
            .join(format!("discovery-{name}-{stamp}"))
    }
}
