#[allow(dead_code)]
mod a3p_content_support;
#[allow(dead_code)]
mod a3p_parser_support;

use a3p_content_support::{
    RESOURCE_DECL_PATTERN, build_synthetic_a3p, extract_all_xml, extract_all_xml_bytes,
};
use a3p_parser_support::{parse_a3p_program, parse_a3p_scene};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn write_archive(name: &str, entries: Vec<(&str, &str)>) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/test-work/project-io-resource-management")
        .join(format!("{name}-{nonce}"));
    fs::create_dir_all(&root).expect("create project io test root");
    let path = root.join(format!("{name}.a3p"));
    fs::write(&path, build_synthetic_a3p(entries)).expect("write synthetic .a3p");
    path
}

#[test]
fn new_project_creation_blank_scene_adds_scene_and_character() {
    let xml = r##"
        <root>
            <node type="SceneObject" name="ground" kind="ground" />
            <node type="SceneObject" name="sky" kind="sky" />
            <node type="SceneObject" name="hero" kind="Biped" position="0,0,0" size="1.0" color="#ffaa00" opacity="1.0" />
            <node type="Camera" position="0,6,12" />
            <node type="UserMethod" name="sceneActivated">
                <child type="MethodInvocation" method="addCharacter" />
            </node>
        </root>
    "##;
    let path = write_archive("new-project", vec![("project.xml", xml)]);

    let scene = parse_a3p_scene(&path).expect("scene layout should parse");
    let program = parse_a3p_program(&path).expect("program should parse");

    assert!(scene.ground_present, "blank projects should include ground");
    assert!(scene.sky_present, "blank projects should include sky");
    assert_eq!(
        scene.objects.len(),
        1,
        "character should be added to the scene"
    );
    assert_eq!(scene.objects[0].name, "hero");
    assert_eq!(scene.objects[0].kind, "Biped");
    assert_eq!(program.procedures[0].name, "sceneActivated");
}

#[test]
fn project_save_load_round_trip_preserves_entities_and_methods() {
    let xml = r##"
        <root>
            <node type="SceneObject" name="ground" kind="ground" />
            <node type="SceneObject" name="sky" kind="sky" />
            <node type="SceneObject" name="rabbit" kind="Biped" position="1,0,-2" size="1.25" color="#ffaa00" opacity="0.8" />
            <node type="SceneObject" name="tree" kind="Prop" position="-3,0,4" size="2.5" color="#00aa44" opacity="1.0" />
            <node type="Camera" position="0,6,12" />
            <node type="UserMethod" name="hop">
                <child type="MethodInvocation" method="move" />
            </node>
        </root>
    "##;
    let original_path = write_archive("save-load-original", vec![("project.xml", xml)]);
    let copied_path = write_archive(
        "save-load-copy",
        vec![("project.xml", &extract_all_xml(&original_path))],
    );

    let original_scene = parse_a3p_scene(&original_path).expect("original scene should parse");
    let copied_scene = parse_a3p_scene(&copied_path).expect("copied scene should parse");
    let original_program =
        parse_a3p_program(&original_path).expect("original program should parse");
    let copied_program = parse_a3p_program(&copied_path).expect("copied program should parse");

    assert_eq!(original_scene.objects, copied_scene.objects);
    assert_eq!(original_program.procedures, copied_program.procedures);
}

#[test]
fn resource_import_keeps_external_model_reference_with_project_scene() {
    let project_xml = r##"
        <root>
            <resource type="ModelResource" source="gallery/space-ship.a3r" />
            <node type="SceneObject" name="ship" kind="Prop" position="2,1,-8" size="3.0" color="#2244ff" opacity="1.0" />
        </root>
    "##;
    let bytes = build_synthetic_a3p(vec![("project.xml", project_xml)]);
    let path = write_archive("resource-import", vec![("project.xml", project_xml)]);

    let xml = extract_all_xml_bytes(&bytes);
    let scene = parse_a3p_scene(&path).expect("scene should parse with imported resource");

    assert!(
        RESOURCE_DECL_PATTERN.is_match(&xml),
        "resource declaration should remain in exported XML"
    );
    assert!(xml.contains("gallery/space-ship.a3r"));
    assert_eq!(scene.objects[0].name, "ship");
}

#[test]
fn project_export_contains_standalone_build_artifacts() {
    let bytes = build_synthetic_a3p(vec![
        (
            "project.xml",
            r#"<root><node type="SceneObject" name="hero" kind="Biped" /></root>"#,
        ),
        (
            "build.xml",
            r#"<project><target name="run"><java jar="dist/standalone.jar" /></target></project>"#,
        ),
        (
            "manifest.xml",
            r#"<manifest><entry key="Main-Class" value="org.alice.generated.Main" /></manifest>"#,
        ),
    ]);

    let xml = extract_all_xml_bytes(&bytes);

    assert!(
        xml.contains("<target name=\"run\">"),
        "standalone export should include run target"
    );
    assert!(
        xml.contains("standalone.jar"),
        "standalone export should reference bundled jar"
    );
    assert!(
        xml.contains("Main-Class"),
        "standalone export should include manifest metadata"
    );
}

#[test]
fn template_projects_preserve_expected_scene_structure() {
    let xml = r##"
        <root>
            <node type="SceneObject" name="ground" kind="ground" />
            <node type="SceneObject" name="sky" kind="sky" />
            <node type="SceneObject" name="hero" kind="Biped" position="0,0,0" size="1.0" color="#ffaa00" opacity="1.0" />
            <node type="SceneObject" name="cameraMarker" kind="Marker" position="0,2,8" size="0.5" color="#ffffff" opacity="0.5" />
            <node type="Camera" position="0,6,12" />
        </root>
    "##;
    let path = write_archive("template-project", vec![("template.xml", xml)]);

    let scene = parse_a3p_scene(&path).expect("template scene should parse");
    let names: Vec<_> = scene
        .objects
        .iter()
        .map(|object| object.name.as_str())
        .collect();

    assert!(scene.ground_present);
    assert!(scene.sky_present);
    assert_eq!(names, vec!["hero", "cameraMarker"]);
    assert!(
        scene.camera.is_some(),
        "template should include a camera pose"
    );
}

#[test]
fn project_metadata_survives_archive_readback() {
    let metadata = r#"
        <metadata>
            <author>Alice Modernization</author>
            <created-at>2026-05-27T00:00:00Z</created-at>
            <version>3.0.0-modernized</version>
        </metadata>
    "#;
    let path = write_archive(
        "project-metadata",
        vec![
            (
                "project.xml",
                r#"<root><node type="SceneObject" name="hero" kind="Biped" /></root>"#,
            ),
            ("metadata.xml", metadata),
        ],
    );

    let xml = extract_all_xml(&path);

    assert!(xml.contains("Alice Modernization"));
    assert!(xml.contains("2026-05-27T00:00:00Z"));
    assert!(xml.contains("3.0.0-modernized"));
}
