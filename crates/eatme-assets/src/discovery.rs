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
            if path.file_name().and_then(|n| n.to_str()) == Some("step-blocks") {
                continue;
            }
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
