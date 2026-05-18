//! Shared test helpers for Alice `.a3p` content coverage tests.
//!
//! Provides ZIP extraction, synthetic archive building, file discovery,
//! and compiled regex patterns for Alice 3 XML elements. Imported by
//! sibling test files via `mod a3p_content_support;`.

use regex::Regex;
use std::env;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Maximum bytes of concatenated XML to extract from a single archive.
pub const MAX_XML_BYTES: usize = 50 * 1024 * 1024;

// ===================================================================
// Environment helpers
// ===================================================================

/// Returns `true` when `EATME_REAL_ALICE` is set to `"1"`.
pub fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Resolves the `starter-projects/` directory from `ALICE_HOME` or the
/// default checkout path (`../alice3-modernization`).
pub fn starter_projects_dir() -> PathBuf {
    let alice_home = env::var("ALICE_HOME").unwrap_or_else(|_| "../alice3-modernization".into());
    PathBuf::from(alice_home)
        .join("core/resources/target/distribution/application/starter-projects")
}

// ===================================================================
// File discovery
// ===================================================================

/// Recursively discovers all `.a3p` files under `dir`, skipping hidden
/// directories and symlinks. Returns sorted paths.
pub fn discover_a3p_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if dir.is_dir() {
        collect_a3p_recursive(dir, &mut results);
    }
    results.sort();
    results
}

fn collect_a3p_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            collect_a3p_recursive(&path, out);
        } else if name.ends_with(".a3p") {
            out.push(path);
        }
    }
}

// ===================================================================
// ZIP XML extraction
// ===================================================================

/// Opens a `.a3p` ZIP file by path and extracts all `.xml` entries into
/// a single concatenated string. Applies path-traversal guard and 50 MB cap.
pub fn extract_all_xml(path: &Path) -> String {
    let file = std::fs::File::open(path)
        .unwrap_or_else(|e| panic!("failed to open {}: {e}", path.display()));
    let mut archive =
        zip::ZipArchive::new(file).unwrap_or_else(|e| panic!("bad ZIP {}: {e}", path.display()));
    extract_xml_entries(&mut archive, MAX_XML_BYTES)
}

/// Same as [`extract_all_xml`] but accepts `&[u8]` via `Cursor`.
/// Used by unit tests that build synthetic archives in memory.
pub fn extract_all_xml_bytes(bytes: &[u8]) -> String {
    extract_all_xml_bytes_with_cap(bytes, MAX_XML_BYTES)
}

/// Extraction with a configurable byte cap — used by unit tests to
/// verify truncation without building 50 MB archives.
pub fn extract_all_xml_bytes_with_cap(bytes: &[u8], max_bytes: usize) -> String {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("failed to read bytes as ZIP");
    extract_xml_entries(&mut archive, max_bytes)
}

fn extract_xml_entries<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    max_bytes: usize,
) -> String {
    let mut all_xml = String::new();
    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.name().to_string();
        // Path-traversal guard: skip entries with ".." or absolute paths
        if name.contains("..") || name.starts_with('/') {
            continue;
        }
        if !name.ends_with(".xml") {
            continue;
        }
        let mut content = String::new();
        if entry.read_to_string(&mut content).is_ok() {
            if all_xml.len() + content.len() > max_bytes {
                let remaining = max_bytes.saturating_sub(all_xml.len());
                all_xml.push_str(&content[..remaining]);
                break;
            }
            all_xml.push_str(&content);
        }
    }
    all_xml
}

// ===================================================================
// Synthetic archive builder
// ===================================================================

/// Builds an in-memory `.a3p` ZIP from `(filename, content)` pairs.
/// Returns raw bytes for passing to [`extract_all_xml_bytes`].
pub fn build_synthetic_a3p(entries: Vec<(&str, &str)>) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    let options = zip::write::SimpleFileOptions::default();
    for (name, content) in entries {
        writer.start_file(name, options).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

// ===================================================================
// Regex pattern constants
// ===================================================================

pub static JOINT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"JointedModel|Joint(?:Id)?|SkeletonVisual").unwrap());

pub static BOUNDING_BOX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"BoundingBox|boundingBox").unwrap());

pub static CAMERA_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"CameraMarker|VantagePoint|SymmetricPerspectiveCamera|fieldOfView").unwrap()
});

pub static AUDIO_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"PlayAudio|AudioSource|\.mp3|\.wav").unwrap());

pub static BILLBOARD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Billboard|TextModel|Marker|TextString").unwrap());

pub static SCENE_ENTITY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"SScene|SModel|SGround").unwrap());

pub static RESOURCE_DECL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"resourceReference|ModelResourceReference").unwrap());
