//! Integration tests scanning real `.a3p` starter project archives.
//!
//! Gated behind `EATME_REAL_ALICE=1` — requires a packaged Alice checkout
//! with the `starter-projects/` directory populated. Tests use aggregate
//! assertions ("at least one project contains X") to accommodate variety
//! across starter projects.

#[allow(dead_code)]
mod a3p_content_support;
use a3p_content_support::*;

fn skip_unless_real_alice() -> bool {
    if !real_alice_enabled() {
        eprintln!("skipping: set EATME_REAL_ALICE=1 to enable gallery tests");
        return true;
    }
    false
}

fn gallery_files() -> Vec<std::path::PathBuf> {
    let dir = starter_projects_dir();
    discover_a3p_files(&dir)
}

// ===================================================================
// Gallery discovery
// ===================================================================

#[test]
fn gallery_is_not_empty() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    assert!(
        !files.is_empty(),
        "expected at least one .a3p in {}",
        starter_projects_dir().display()
    );
}

#[test]
fn every_a3p_contains_xml() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    assert!(!files.is_empty(), "gallery must not be empty");
    for path in &files {
        let xml = extract_all_xml(path);
        assert!(
            !xml.is_empty(),
            "{} contains no XML entries",
            path.display()
        );
    }
}

// ===================================================================
// Scene structure
// ===================================================================

#[test]
fn scene_entity_types_present() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| SCENE_ENTITY_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain scene entity XML (SScene/SModel/SGround)"
    );
}

#[test]
fn resource_declarations_present() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| RESOURCE_DECL_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain resource declarations"
    );
}

// ===================================================================
// Model gallery
// ===================================================================

#[test]
fn joint_hierarchy_in_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| JOINT_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain joint/skeleton XML patterns"
    );
}

#[test]
fn bounding_box_in_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| BOUNDING_BOX_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain bounding box XML patterns"
    );
}

// ===================================================================
// Camera / Audio / Billboard
// ===================================================================

#[test]
fn camera_controls_in_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| CAMERA_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain camera XML patterns"
    );
}

#[test]
fn audio_references_in_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| AUDIO_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain audio resource references"
    );
}

#[test]
fn billboard_elements_in_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    let files = gallery_files();
    let matches: Vec<_> = files
        .iter()
        .filter(|p| BILLBOARD_PATTERN.is_match(&extract_all_xml(p)))
        .collect();
    assert!(
        !matches.is_empty(),
        "expected at least one .a3p to contain billboard/text overlay XML patterns"
    );
}
