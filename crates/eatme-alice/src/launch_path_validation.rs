use crate::launch_artifacts::artifact_info;
use eatme_core::ArtifactInfo;
use std::path::{Component, Path, PathBuf};

pub(crate) fn artifact_info_under(
    root_dir: &Path,
    relative_path: &str,
    field: &str,
    root_label: &str,
) -> std::result::Result<ArtifactInfo, String> {
    let path = Path::new(relative_path);
    if path.is_absolute() || normal_components(path).is_none() {
        return Err(format!(
            "{field} must be a simple relative path under {root_label}"
        ));
    }

    let full_path = root_dir.join(path);
    canonical_artifact_under(root_dir, &full_path, field, root_label)?;
    artifact_info(&full_path).map_err(|error| {
        format!(
            "{field} {} is not a readable artifact: {error:#}",
            full_path.display()
        )
    })
}

pub(crate) fn canonical_artifact_under(
    root_dir: &Path,
    artifact_path: &Path,
    field: &str,
    root_label: &str,
) -> std::result::Result<PathBuf, String> {
    let root = root_dir.canonicalize().map_err(|error| {
        format!(
            "{root_label} {} is not readable: {error:#}",
            root_dir.display()
        )
    })?;
    let artifact = artifact_path.canonicalize().map_err(|error| {
        format!(
            "{field} {} is not a readable artifact: {error:#}",
            artifact_path.display()
        )
    })?;
    if !artifact.starts_with(&root) {
        return Err(format!("{field} must stay under {root_label}"));
    }
    Ok(artifact)
}

pub(crate) fn normal_components(path: &Path) -> Option<Vec<String>> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
