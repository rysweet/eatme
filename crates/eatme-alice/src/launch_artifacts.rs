use anyhow::Result;
use eatme_core::{ArtifactInfo, file_size, sha256_file};
use std::path::Path;

pub fn artifact_info(path: &Path) -> Result<ArtifactInfo> {
    Ok(ArtifactInfo {
        path: path.display().to_string(),
        size_bytes: file_size(path)?,
        sha256: sha256_file(path)?,
    })
}
