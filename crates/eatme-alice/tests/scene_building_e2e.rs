// Scene-building E2E tests: validates the student-facing contract
// of the Getting Started / Building a Scene grading pipeline.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use eatme_assets::{SceneBuildingGradingInput, StepStatus, grade_scene_building};
use eatme_core::ast::SceneLayout;
use zip::write::SimpleFileOptions;

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_scene;

fn rich_scene_xml() -> &'static str {
    r##"
    <root>
        <element type="UserMethod" name="myFirstMethod" />
        <node type="SceneObject" name="ground" kind="ground" />
        <node type="SceneObject" name="sky" kind="sky" />
        <node type="SceneObject" name="bunny" kind="Biped" position="1,0,-2" size="1.25" color="#ffaa00" opacity="0.80" />
        <node type="SceneObject" name="tree" kind="Prop" position="-3,0,4" size="2.50" color="#00aa44" opacity="1.00" />
        <node type="Camera" position="0,6,12" />
    </root>
    "##
}

fn minimal_scene_xml() -> &'static str {
    r#"
    <root>
        <element type="UserMethod" name="myFirstMethod" />
        <node type="SceneObject" name="ground" kind="ground" />
        <node type="SceneObject" name="chair" kind="Prop" position="0,0,0" />
        <node type="Camera" position="0,4,8" />
    </root>
    "#
}

fn write_test_a3p(name: &str, xml: &str) -> PathBuf {
    let work_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-work/scene-building-e2e");
    std::fs::create_dir_all(&work_dir).expect("create scene-building test work dir");

    let path = work_dir.join(format!(
        "{}-{}-{}.a3p",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let cursor = Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(cursor);
    let options = SimpleFileOptions::default();

    zip_writer
        .start_file("programType.xml", options)
        .expect("start xml entry");
    zip_writer
        .write_all(xml.as_bytes())
        .expect("write scene xml");

    let bytes = zip_writer.finish().expect("finish scene zip").into_inner();
    std::fs::write(&path, bytes).expect("write scene a3p");
    path
}

fn all_ready_input(scene: Option<SceneLayout>) -> SceneBuildingGradingInput {
    SceneBuildingGradingInput {
        assets_valid: true,
        asset_reason: "ok".into(),
        deps_available: true,
        deps_reason: "ok".into(),
        student_scene: scene,
    }
}

#[test]
fn scene_parser_extracts_objects_properties_and_camera() {
    let path = write_test_a3p("rich-scene", rich_scene_xml());
    let scene = parse_a3p_scene(&path).expect("rich scene should parse");
    let _ = std::fs::remove_file(&path);

    assert!(scene.ground_present);
    assert!(scene.sky_present);
    assert_eq!(scene.objects.len(), 2);

    let bunny = scene
        .objects
        .iter()
        .find(|object| object.name == "bunny")
        .unwrap();
    assert_eq!(bunny.kind, "Biped");
    assert_eq!(bunny.position.as_ref().unwrap().x, 1.0);
    assert_eq!(bunny.size, Some(1.25));
    assert_eq!(bunny.color.as_deref(), Some("#ffaa00"));
    assert_eq!(bunny.opacity, Some(0.8));

    let camera = scene.camera.expect("camera should be present");
    assert_eq!(camera.position.y, 6.0);
    assert_eq!(camera.position.z, 12.0);
}

#[test]
fn scene_building_grading_distinguishes_minimal_and_rich_scenes() {
    let minimal_path = write_test_a3p("minimal-scene", minimal_scene_xml());
    let rich_path = write_test_a3p("rich-scene", rich_scene_xml());

    let minimal_scene = parse_a3p_scene(&minimal_path).expect("minimal scene should parse");
    let rich_scene = parse_a3p_scene(&rich_path).expect("rich scene should parse");

    let _ = std::fs::remove_file(&minimal_path);
    let _ = std::fs::remove_file(&rich_path);

    let minimal_report = grade_scene_building(all_ready_input(Some(minimal_scene)));
    let rich_report = grade_scene_building(all_ready_input(Some(rich_scene)));

    let minimal_ready = minimal_report
        .steps
        .iter()
        .filter(|step| step.status == StepStatus::Ready)
        .count();
    let rich_ready = rich_report
        .steps
        .iter()
        .filter(|step| step.status == StepStatus::Ready)
        .count();

    assert!(!minimal_report.passed);
    assert!(rich_report.passed);
    assert!(rich_ready > minimal_ready);

    let minimal_objects = minimal_report
        .steps
        .iter()
        .find(|step| step.name == "place-scene-objects")
        .unwrap();
    assert_eq!(minimal_objects.status, StepStatus::Blocked);

    let rich_properties = rich_report
        .steps
        .iter()
        .find(|step| step.name == "set-object-properties")
        .unwrap();
    assert_eq!(rich_properties.status, StepStatus::Ready);
}

#[test]
fn scene_building_grading_blocked_without_scene() {
    let report = grade_scene_building(all_ready_input(None));
    assert!(!report.passed);

    for name in [
        "add-ground",
        "add-sky",
        "place-scene-objects",
        "position-camera",
        "set-object-properties",
        "save-project",
    ] {
        let step = report.steps.iter().find(|step| step.name == name).unwrap();
        assert_eq!(step.status, StepStatus::Blocked, "{name} should be blocked");
    }
}

#[test]
fn scene_building_schema_version_and_lesson() {
    let path = write_test_a3p("rich-scene", rich_scene_xml());
    let scene = parse_a3p_scene(&path).expect("rich scene should parse");
    let _ = std::fs::remove_file(&path);

    let report = grade_scene_building(all_ready_input(Some(scene)));
    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "building-a-scene-first-world");
}

#[test]
fn scene_layout_survives_json_round_trip() {
    let path = write_test_a3p("rich-scene", rich_scene_xml());
    let scene = parse_a3p_scene(&path).expect("rich scene should parse");
    let _ = std::fs::remove_file(&path);

    let json = serde_json::to_string_pretty(&scene).unwrap();
    let restored: SceneLayout = serde_json::from_str(&json).unwrap();
    assert_eq!(scene, restored);
}
