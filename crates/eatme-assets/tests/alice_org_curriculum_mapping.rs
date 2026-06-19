use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

struct CurriculumConcept {
    name: &'static str,
    desktop_files: &'static [&'static str],
    web_markers: &'static [&'static str],
}

const ALICE_ORG_CONCEPTS: &[CurriculumConcept] = &[
    CurriculumConcept {
        name: "scene-building",
        desktop_files: &["crates/eatme-alice/tests/scene_building_e2e.rs"],
        web_markers: &["hello-world", "hello_world_adds_object_and_saves"],
    },
    CurriculumConcept {
        name: "methods/procedures",
        desktop_files: &["crates/eatme-alice/tests/code_editor_first_run_e2e.rs"],
        web_markers: &["fn procedures()", "procedures_edits_and_runs"],
    },
    CurriculumConcept {
        name: "parameters",
        desktop_files: &["crates/eatme-alice/tests/parameters_e2e.rs"],
        web_markers: &[
            "fn parameters()",
            "parameters_creates_parameterized_method_and_call",
        ],
    },
    CurriculumConcept {
        name: "functions",
        desktop_files: &["crates/eatme-alice/tests/functions_e2e.rs"],
        web_markers: &["fn functions()", "explorer.distanceTo(treasure) < 2.0"],
    },
    CurriculumConcept {
        name: "variables",
        desktop_files: &["crates/eatme-alice/tests/variables_e2e.rs"],
        web_markers: &["fn variables()", "variables_declares_and_assigns"],
    },
    CurriculumConcept {
        name: "loops (count/while/for-each)",
        desktop_files: &[
            "crates/eatme-alice/tests/loops_and_conditionals_e2e.rs",
            "crates/eatme-alice/tests/nested_control_flow_e2e.rs",
            "crates/eatme-alice/tests/arrays_arithmetic_e2e.rs",
        ],
        web_markers: &[
            "countLoop",
            "eachInArrayTogether",
            "nested_control_flow_layers_together_branching_and_loops",
        ],
    },
    CurriculumConcept {
        name: "conditionals (if/if-else)",
        desktop_files: &[
            "crates/eatme-alice/tests/loops_and_conditionals_e2e.rs",
            "crates/eatme-alice/tests/nested_control_flow_e2e.rs",
        ],
        web_markers: &["ifElse", "loops_conditionals_has_control_flow"],
    },
    CurriculumConcept {
        name: "events (mouse-click, key-press, collision, proximity, scene-activation)",
        desktop_files: &[
            "crates/eatme-alice/tests/events_and_collision_e2e.rs",
            "crates/eatme-alice/tests/advanced_interaction_contracts.rs",
            "crates/eatme-alice/tests/curriculum_scenario_expansion_e2e.rs",
        ],
        web_markers: &[
            "events-collision",
            "keyPress",
            "collision",
            "full_student_journey_covers_student_build_run_and_save_flow",
        ],
    },
    CurriculumConcept {
        name: "doInOrder",
        desktop_files: &["crates/eatme-alice/tests/sequencing_e2e.rs"],
        web_markers: &[
            "full_student_journey",
            "full_student_journey_covers_student_build_run_and_save_flow",
        ],
    },
    CurriculumConcept {
        name: "doTogether",
        desktop_files: &["crates/eatme-alice/tests/sequencing_e2e.rs"],
        web_markers: &["doTogether", "concurrency_uses_do_together"],
    },
    CurriculumConcept {
        name: "arrays",
        desktop_files: &["crates/eatme-alice/tests/arrays_arithmetic_e2e.rs"],
        web_markers: &["fn arrays()", "arrays_uses_each_in_array"],
    },
    CurriculumConcept {
        name: "comments",
        desktop_files: &["crates/eatme-alice/tests/comments_e2e.rs"],
        web_markers: &["fn comments()", "comments_adds_meaningful_comment_text"],
    },
    CurriculumConcept {
        name: "inheritance/OOP",
        desktop_files: &["crates/eatme-alice/tests/inheritance_oop_e2e.rs"],
        web_markers: &[
            "fn inheritance_oop()",
            "inheritance_oop_declares_custom_biped_type",
        ],
    },
    CurriculumConcept {
        name: "say/think",
        desktop_files: &["crates/eatme-alice/tests/text_and_speech_e2e.rs"],
        web_markers: &["player.say", "narrator.say", "logicHero.say"],
    },
    CurriculumConcept {
        name: "move/turn/roll",
        desktop_files: &[
            "crates/eatme-alice/tests/animation_timing_scenarios_e2e.rs",
            "crates/eatme-alice/tests/joint_vehicle_scenarios_e2e.rs",
        ],
        web_markers: &["hero.walk", "hero.turn", "dancer.rightShoulder.turn"],
    },
    CurriculumConcept {
        name: "camera",
        desktop_files: &["crates/eatme-alice/tests/camera_and_viewpoint_e2e.rs"],
        web_markers: &["fn camera_viewpoint()", "camera_uses_camera_methods"],
    },
    CurriculumConcept {
        name: "audio",
        desktop_files: &["crates/eatme-alice/tests/a3p_gallery_coverage.rs"],
        web_markers: &["fn audio()", "audio_uses_play_audio"],
    },
    CurriculumConcept {
        name: "vehicles",
        desktop_files: &["crates/eatme-alice/tests/joint_vehicle_scenarios_e2e.rs"],
        web_markers: &[
            "fn vehicle_parenting()",
            "vehicle_parenting_attaches_camera_to_character",
        ],
    },
    CurriculumConcept {
        name: "markers",
        desktop_files: &[
            "crates/eatme-alice/tests/camera_and_viewpoint_e2e.rs",
            "crates/eatme-alice/tests/advanced_interaction_contracts.rs",
        ],
        web_markers: &[
            "camera-viewpoint",
            "joint_manipulation_targets_biped_joints",
        ],
    },
    CurriculumConcept {
        name: "custom classes",
        desktop_files: &["crates/eatme-alice/tests/inheritance_oop_e2e.rs"],
        web_markers: &["userTypeDeclaration", "instantiateUserType"],
    },
    CurriculumConcept {
        name: "debugging",
        desktop_files: &["crates/eatme-alice/tests/advanced_interaction_contracts.rs"],
        web_markers: &[
            "error_recovery_expects_failures_and_then_recovers",
            "instructor_grading_round_trips_saved_project_structure",
        ],
    },
    CurriculumConcept {
        name: "project-IO",
        desktop_files: &[
            "crates/eatme-alice/tests/project_io_resource_management.rs",
            "crates/eatme-alice/tests/import_export_workflow_real.rs",
        ],
        web_markers: &[
            "project_io_saves_then_reloads_before_verify",
            "load(",
            "save(",
        ],
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
fn alice_org_curriculum_concepts_have_desktop_and_web_coverage() {
    let unique_names = ALICE_ORG_CONCEPTS
        .iter()
        .map(|concept| concept.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        unique_names.len(),
        ALICE_ORG_CONCEPTS.len(),
        "curriculum concept list should not contain duplicates"
    );

    let expected_names = [
        "scene-building",
        "methods/procedures",
        "parameters",
        "functions",
        "variables",
        "loops (count/while/for-each)",
        "conditionals (if/if-else)",
        "events (mouse-click, key-press, collision, proximity, scene-activation)",
        "doInOrder",
        "doTogether",
        "arrays",
        "comments",
        "inheritance/OOP",
        "say/think",
        "move/turn/roll",
        "camera",
        "audio",
        "vehicles",
        "markers",
        "custom classes",
        "debugging",
        "project-IO",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        expected_names, unique_names,
        "curriculum concept inventory changed unexpectedly"
    );

    let root = repository_root();
    let web_file = root.join("crates/eatme-alice/tests/web_platform_curriculum_e2e.rs");
    assert!(web_file.is_file(), "{} must exist", web_file.display());
    let web_content = normalized_content(&web_file);

    let mut gaps = Vec::new();
    for concept in ALICE_ORG_CONCEPTS {
        let missing_desktop = concept
            .desktop_files
            .iter()
            .filter(|file| {
                let path = root.join(file);
                !path.is_file() || !normalized_content(&path).contains("#[test]")
            })
            .copied()
            .collect::<Vec<_>>();
        let missing_web = concept
            .web_markers
            .iter()
            .filter(|marker| !web_content.contains(&marker.to_lowercase()))
            .copied()
            .collect::<Vec<_>>();

        if !missing_desktop.is_empty() || !missing_web.is_empty() {
            gaps.push(format!(
                "{} => missing desktop {:?}, missing web {:?}",
                concept.name, missing_desktop, missing_web
            ));
        }
    }

    assert!(
        gaps.is_empty(),
        "Alice.org curriculum coverage is incomplete:\n{}",
        gaps.join("\n")
    );
}
