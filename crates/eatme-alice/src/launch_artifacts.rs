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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn artifact_info_reports_path_size_and_hash() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/artifact-info-tests")
            .join(format!("{nonce}"));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("artifact.txt");
        fs::write(&path, "artifact").unwrap();

        let info = artifact_info(&path).unwrap();

        assert_eq!(info.path, path.display().to_string());
        assert_eq!(info.size_bytes, 8);
        assert!(!info.sha256.is_empty());

        let _ = fs::remove_dir_all(root);
    }
}
