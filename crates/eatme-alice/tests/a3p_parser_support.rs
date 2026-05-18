//! Unit tests for the `.a3p` ZIP extraction pipeline and content pattern
//! matching. These always run — no environment gate required.
//!
//! Parser robustness tests validate edge cases (empty ZIPs, path traversal,
//! size caps). Content pattern tests build synthetic archives containing
//! specific Alice XML element families and verify the regex patterns match.

#[allow(dead_code)]
mod a3p_content_support;
use a3p_content_support::*;

// ===================================================================
// Parser robustness tests
// ===================================================================

#[test]
fn valid_extraction() {
    let xml = r#"<alice version="3.6"><scene type="SScene"/></alice>"#;
    let zip_bytes = build_synthetic_a3p(vec![("project.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert_eq!(extracted, xml);
}

#[test]
fn empty_zip() {
    let zip_bytes = build_synthetic_a3p(vec![]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(extracted.is_empty(), "empty ZIP should yield no XML");
}

#[test]
fn no_xml_zip() {
    let zip_bytes = build_synthetic_a3p(vec![
        ("texture.png", "fake png data"),
        ("readme.txt", "this is not xml"),
    ]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        extracted.is_empty(),
        "ZIP with no .xml files should yield empty string"
    );
}

#[test]
fn path_traversal_rejection() {
    // The extraction helper skips entries whose names contain ".." or start
    // with "/". We verify that a safe entry is extracted while documenting
    // the guard contract. The zip crate's writer may sanitize names, so we
    // test the guard logic through the extraction of only safe entries.
    let zip_bytes = build_synthetic_a3p(vec![("safe/nested.xml", "<safe/>")]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        extracted.contains("<safe/>"),
        "safe nested entries must be extracted"
    );

    // Verify the guard contract: names with ".." would be skipped.
    // We test this by confirming the extraction function's filtering logic
    // is present — if a name contained ".." it would be dropped.
    let xml = extract_all_xml_bytes(&build_traversal_test_zip());
    assert!(
        !xml.contains("evil"),
        "path-traversal entries should be skipped"
    );
}

#[test]
fn oversized_content_cap() {
    let small_cap = 100;
    let xml_content = format!("<data>{}</data>", "x".repeat(200));
    let zip_bytes = build_synthetic_a3p(vec![("big.xml", &xml_content)]);
    let extracted = extract_all_xml_bytes_with_cap(&zip_bytes, small_cap);
    assert!(
        extracted.len() <= small_cap,
        "extracted {} bytes exceeds cap of {small_cap}",
        extracted.len(),
    );
    assert!(
        !extracted.is_empty(),
        "capped extraction should still produce some content"
    );
}

#[test]
fn nested_directory_handling() {
    let zip_bytes = build_synthetic_a3p(vec![
        ("top.xml", "<top/>"),
        ("sub/nested.xml", "<nested/>"),
        ("sub/deep/leaf.xml", "<leaf/>"),
    ]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        extracted.contains("<top/>"),
        "top-level XML should be extracted"
    );
    assert!(
        extracted.contains("<nested/>"),
        "nested XML should be extracted"
    );
    assert!(
        extracted.contains("<leaf/>"),
        "deeply nested XML should be extracted"
    );
}

#[test]
fn filename_filtering() {
    let zip_bytes = build_synthetic_a3p(vec![
        ("scene.xml", "<scene/>"),
        ("model.json", r#"{"not":"xml"}"#),
        ("texture.png", "binary data"),
        ("config.xml", "<config/>"),
    ]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(extracted.contains("<scene/>"));
    assert!(extracted.contains("<config/>"));
    assert!(
        !extracted.contains("not"),
        "JSON entries should not appear in XML extraction"
    );
    assert!(
        !extracted.contains("binary"),
        "binary entries should not appear in XML extraction"
    );
}

// ===================================================================
// Content pattern matching tests
// ===================================================================

#[test]
fn synthetic_a3p_with_joints_extracts() {
    let xml = concat!(
        r#"<resource><JointedModelResource name="alien">"#,
        r#"<Joint id="ROOT"/><SkeletonVisual/>"#,
        r#"</JointedModelResource></resource>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("model.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        JOINT_PATTERN.is_match(&extracted),
        "joint pattern should match: {extracted}"
    );
}

#[test]
fn synthetic_a3p_with_bounding_box_extracts() {
    let xml = r#"<model><BoundingBox min="-1,-1,-1" max="1,1,1"/></model>"#;
    let zip_bytes = build_synthetic_a3p(vec![("bounds.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        BOUNDING_BOX_PATTERN.is_match(&extracted),
        "bounding box pattern should match"
    );
}

#[test]
fn synthetic_a3p_with_resource_metadata_extracts() {
    let xml = concat!(
        r#"<scene><resourceReference key="bunny" "#,
        r#"type="ModelResourceReference"/></scene>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("resources.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        RESOURCE_DECL_PATTERN.is_match(&extracted),
        "resource declaration pattern should match"
    );
}

#[test]
fn synthetic_a3p_with_camera_extracts() {
    let xml = concat!(
        r#"<scene><CameraMarker>"#,
        r#"<SymmetricPerspectiveCamera fieldOfView="0.5"/>"#,
        r#"</CameraMarker></scene>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("camera.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        CAMERA_PATTERN.is_match(&extracted),
        "camera pattern should match"
    );
}

#[test]
fn synthetic_a3p_with_audio_extracts() {
    let xml = concat!(
        r#"<program><PlayAudio resource="bark.mp3">"#,
        r#"<AudioSource volume="1.0"/>"#,
        r#"</PlayAudio></program>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("audio.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        AUDIO_PATTERN.is_match(&extracted),
        "audio pattern should match"
    );
}

#[test]
fn synthetic_a3p_with_billboard_extracts() {
    let xml = concat!(
        r#"<overlay><Billboard>"#,
        r#"<TextModel><TextString value="Hello World"/></TextModel>"#,
        r#"</Billboard></overlay>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("billboard.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        BILLBOARD_PATTERN.is_match(&extracted),
        "billboard pattern should match"
    );
}

#[test]
fn synthetic_a3p_with_scene_entities_extracts() {
    let xml = concat!(
        r#"<alice><SScene>"#,
        r#"<SModel name="cat"/>"#,
        r#"<SGround texture="grass"/>"#,
        r#"</SScene></alice>"#,
    );
    let zip_bytes = build_synthetic_a3p(vec![("scene.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        SCENE_ENTITY_PATTERN.is_match(&extracted),
        "scene entity pattern should match"
    );
}
