//! Round-trip unit tests: build synthetic `.a3p` → extract → verify content.
//! These always run without environment gating.

#[allow(dead_code)]
mod a3p_content_support;
use a3p_content_support::*;

#[test]
fn round_trip_build_extract() {
    let xml = r#"<alice version="3.6"><scene type="SScene"><SModel name="bunny"/></scene></alice>"#;
    let zip_bytes = build_synthetic_a3p(vec![("project.xml", xml)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert_eq!(
        extracted, xml,
        "round-trip should preserve XML content exactly"
    );
}

#[test]
fn multi_entry_zip() {
    let xml_a = "<sceneA/>";
    let xml_b = "<sceneB/>";
    let zip_bytes = build_synthetic_a3p(vec![("a.xml", xml_a), ("b.xml", xml_b)]);
    let extracted = extract_all_xml_bytes(&zip_bytes);
    assert!(
        extracted.contains(xml_a),
        "first XML entry should be in output"
    );
    assert!(
        extracted.contains(xml_b),
        "second XML entry should be in output"
    );
}

#[test]
fn entry_ordering_stability() {
    let entries = vec![
        ("zebra.xml", "<z/>"),
        ("alpha.xml", "<a/>"),
        ("middle.xml", "<m/>"),
    ];
    let zip_bytes_1 = build_synthetic_a3p(entries.clone());
    let zip_bytes_2 = build_synthetic_a3p(entries);
    let extracted_1 = extract_all_xml_bytes(&zip_bytes_1);
    let extracted_2 = extract_all_xml_bytes(&zip_bytes_2);
    assert_eq!(
        extracted_1, extracted_2,
        "same input entries should produce identical extracted output"
    );
}
