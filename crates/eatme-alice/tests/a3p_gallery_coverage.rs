//! Integration tests scanning real `.a3p` starter project archives.
//!
//! Gated behind `EATME_REAL_ALICE=1` — requires a packaged Alice checkout
//! with the `starter-projects/` directory populated. Tests use aggregate
//! assertions ("at least one project contains X") to accommodate variety
//! across starter projects.
//!
//! All tests share a single [`GALLERY_CACHE`] that extracts every ZIP
//! exactly once, avoiding redundant I/O when many pattern tests run.

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

// ===================================================================
// Gallery discovery
// ===================================================================

#[test]
fn gallery_is_not_empty() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(
        !GALLERY_CACHE.is_empty(),
        "expected at least one .a3p in {}",
        starter_projects_dir().display()
    );
}

#[test]
fn every_a3p_contains_xml() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(!GALLERY_CACHE.is_empty(), "gallery must not be empty");
    for (path, xml) in GALLERY_CACHE.iter() {
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
    assert!(
        GALLERY_CACHE
            .iter()
            .any(|(_, xml)| SCENE_ENTITY_PATTERN.is_match(xml)),
        "expected at least one .a3p to contain scene entity XML (SScene/SModel/SGround)"
    );
}

#[test]
fn resource_declarations_present() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(
        GALLERY_CACHE
            .iter()
            .any(|(_, xml)| RESOURCE_DECL_PATTERN.is_match(xml)),
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
    assert!(
        GALLERY_CACHE
            .iter()
            .any(|(_, xml)| JOINT_PATTERN.is_match(xml)),
        "expected at least one .a3p to contain joint/skeleton XML patterns"
    );
}

#[test]
fn bounding_box_patterns_are_absent_in_current_real_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(
        GALLERY_CACHE
            .iter()
            .all(|(_, xml)| !BOUNDING_BOX_PATTERN.is_match(xml)),
        "current Alice starter-project gallery unexpectedly contains bounding box XML patterns"
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
    assert!(
        GALLERY_CACHE
            .iter()
            .any(|(_, xml)| CAMERA_PATTERN.is_match(xml)),
        "expected at least one .a3p to contain camera XML patterns"
    );
}

#[test]
fn audio_references_are_absent_in_current_real_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(
        GALLERY_CACHE
            .iter()
            .all(|(_, xml)| !AUDIO_PATTERN.is_match(xml)),
        "current Alice starter-project gallery unexpectedly contains audio resource references"
    );
}

#[test]
fn billboard_elements_are_absent_in_current_real_gallery() {
    if skip_unless_real_alice() {
        return;
    }
    assert!(
        GALLERY_CACHE
            .iter()
            .all(|(_, xml)| !BILLBOARD_PATTERN.is_match(xml)),
        "current Alice starter-project gallery unexpectedly contains billboard/text overlay XML patterns"
    );
}
