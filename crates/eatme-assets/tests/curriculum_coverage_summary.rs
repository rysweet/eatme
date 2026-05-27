use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

struct CoverageExpectation {
    topic: &'static str,
    test_file: &'static str,
    markers: &'static [&'static str],
}

const REQUIRED_TOPICS: &[&str] = &[
    "Hello World",
    "scene building",
    "procedures",
    "functions",
    "parameters",
    "variables",
    "loops",
    "conditionals",
    "events",
    "collision",
    "doInOrder",
    "doTogether",
    "arrays",
    "comments",
    "inheritance",
    "games",
    "narrative",
    "camera",
    "audio",
    "vehicles",
    "joints/IK",
    "drag-drop",
    "debugging",
    "project IO",
    "templates",
    "sharing",
];

const COVERAGE_EXPECTATIONS: &[CoverageExpectation] = &[
    CoverageExpectation {
        topic: "Hello World",
        test_file: "crates/eatme-alice/tests/a3p_content_coverage.rs",
        markers: &["hello world"],
    },
    CoverageExpectation {
        topic: "scene building",
        test_file: "crates/eatme-alice/tests/scene_building_e2e.rs",
        markers: &["scene-building"],
    },
    CoverageExpectation {
        topic: "procedures",
        test_file: "crates/eatme-alice/src/launch_edit_procedure/tests.rs",
        markers: &["edit_procedure", "procedure"],
    },
    CoverageExpectation {
        topic: "functions",
        test_file: "crates/eatme-alice/tests/functions_e2e.rs",
        markers: &["functions e2e", "function"],
    },
    CoverageExpectation {
        topic: "parameters",
        test_file: "crates/eatme-alice/tests/parameters_e2e.rs",
        markers: &["parameters e2e", "parameter"],
    },
    CoverageExpectation {
        topic: "variables",
        test_file: "crates/eatme-alice/tests/variables_e2e.rs",
        markers: &["variables"],
    },
    CoverageExpectation {
        topic: "loops",
        test_file: "crates/eatme-alice/tests/loops_and_conditionals_e2e.rs",
        markers: &["loops", "countloop"],
    },
    CoverageExpectation {
        topic: "conditionals",
        test_file: "crates/eatme-alice/tests/loops_and_conditionals_e2e.rs",
        markers: &["conditionals", "ifelse"],
    },
    CoverageExpectation {
        topic: "events",
        test_file: "crates/eatme-alice/tests/events_and_collision_e2e.rs",
        markers: &["events", "eventlistener"],
    },
    CoverageExpectation {
        topic: "collision",
        test_file: "crates/eatme-alice/tests/events_and_collision_e2e.rs",
        markers: &["collision", "collisionlistener"],
    },
    CoverageExpectation {
        topic: "doInOrder",
        test_file: "crates/eatme-alice/tests/sequencing_e2e.rs",
        markers: &["doinorder"],
    },
    CoverageExpectation {
        topic: "doTogether",
        test_file: "crates/eatme-alice/tests/sequencing_e2e.rs",
        markers: &["dotogether"],
    },
    CoverageExpectation {
        topic: "arrays",
        test_file: "crates/eatme-alice/tests/arrays_arithmetic_e2e.rs",
        markers: &["arrays"],
    },
    CoverageExpectation {
        topic: "comments",
        test_file: "crates/eatme-alice/tests/comments_e2e.rs",
        markers: &["comments", "comment"],
    },
    CoverageExpectation {
        topic: "inheritance",
        test_file: "crates/eatme-alice/tests/inheritance_oop_e2e.rs",
        markers: &["inheritance"],
    },
    CoverageExpectation {
        topic: "games",
        test_file: "crates/eatme-alice/tests/games_narrative_e2e.rs",
        markers: &["game"],
    },
    CoverageExpectation {
        topic: "narrative",
        test_file: "crates/eatme-alice/tests/games_narrative_e2e.rs",
        markers: &["narrative"],
    },
    CoverageExpectation {
        topic: "camera",
        test_file: "crates/eatme-alice/tests/camera_and_viewpoint_e2e.rs",
        markers: &["camera"],
    },
    CoverageExpectation {
        topic: "audio",
        test_file: "crates/eatme-alice/tests/a3p_gallery_coverage.rs",
        markers: &["audio"],
    },
    CoverageExpectation {
        topic: "vehicles",
        test_file: "crates/eatme-alice/tests/joint_vehicle_scenarios_e2e.rs",
        markers: &["vehicle"],
    },
    CoverageExpectation {
        topic: "joints/IK",
        test_file: "crates/eatme-alice/tests/joint_vehicle_scenarios_e2e.rs",
        markers: &["joint", "ik"],
    },
    CoverageExpectation {
        topic: "drag-drop",
        test_file: "crates/eatme-alice/tests/advanced_interaction_contracts.rs",
        markers: &["object_placement", "gallery"],
    },
    CoverageExpectation {
        topic: "debugging",
        test_file: "crates/eatme-alice/tests/advanced_interaction_contracts.rs",
        markers: &["debugging", "debug"],
    },
    CoverageExpectation {
        topic: "project IO",
        test_file: "crates/eatme-alice/tests/project_io_resource_management.rs",
        markers: &["project", "save_load"],
    },
    CoverageExpectation {
        topic: "templates",
        test_file: "crates/eatme-alice/tests/project_io_resource_management.rs",
        markers: &["template"],
    },
    CoverageExpectation {
        topic: "sharing",
        test_file: "crates/eatme-assets/src/sharing_platform_tests.rs",
        markers: &["sharing"],
    },
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalized_content(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .to_lowercase()
}

#[test]
fn curriculum_topics_have_corresponding_tests_and_no_gaps() {
    let required_topics = REQUIRED_TOPICS.iter().copied().collect::<BTreeSet<_>>();
    let covered_topics = COVERAGE_EXPECTATIONS
        .iter()
        .map(|expectation| expectation.topic)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        required_topics, covered_topics,
        "curriculum topic coverage map is incomplete"
    );

    let root = repository_root();
    for expectation in COVERAGE_EXPECTATIONS {
        let path = root.join(expectation.test_file);
        assert!(
            path.is_file(),
            "{} must exist for topic {}",
            path.display(),
            expectation.topic
        );
        let content = normalized_content(&path);
        assert!(
            content.contains("#[test]"),
            "{} must define tests for topic {}",
            expectation.test_file,
            expectation.topic
        );
        for marker in expectation.markers {
            assert!(
                content.contains(&marker.to_lowercase()),
                "{} must mention '{}' for topic {}",
                expectation.test_file,
                marker,
                expectation.topic
            );
        }
    }
}
