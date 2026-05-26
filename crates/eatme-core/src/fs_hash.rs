use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn file_size(path: &Path) -> Result<u64> {
    Ok(path
        .metadata()
        .with_context(|| format!("reading metadata for {}", path.display()))?
        .len())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("reading {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_path(prefix: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-work/fs-hash-tests");
        fs::create_dir_all(&root).unwrap();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        root.join(format!("{prefix}-{nonce}.txt"))
    }

    #[test]
    fn reports_file_size_and_sha256_for_written_content() {
        let path = unique_test_path("payload");
        fs::write(&path, b"alice").unwrap();

        assert_eq!(file_size(&path).unwrap(), 5);
        assert_eq!(
            sha256_file(&path).unwrap(),
            "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90"
        );

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn hashes_empty_files_with_sha256_empty_digest() {
        let path = unique_test_path("empty");
        fs::write(&path, b"").unwrap();

        assert_eq!(file_size(&path).unwrap(), 0);
        assert_eq!(
            sha256_file(&path).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn includes_path_context_for_missing_files() {
        let path = unique_test_path("missing");

        assert!(
            file_size(&path)
                .unwrap_err()
                .to_string()
                .contains(&path.display().to_string())
        );
        assert!(
            sha256_file(&path)
                .unwrap_err()
                .to_string()
                .contains(&path.display().to_string())
        );
    }

    #[test]
    fn hashes_large_files_across_multiple_read_chunks() {
        let path = unique_test_path("large");
        let payload = vec![b'x'; 20_000];
        fs::write(&path, &payload).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let expected = format!("{:x}", hasher.finalize());

        assert_eq!(file_size(&path).unwrap(), payload.len() as u64);
        assert_eq!(sha256_file(&path).unwrap(), expected);

        fs::remove_file(path).unwrap();
    }
}
