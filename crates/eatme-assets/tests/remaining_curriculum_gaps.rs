use std::fs;
use std::path::{Path, PathBuf};

use eatme_assets::validate_scenario_asset;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_scenario(id: &str) -> (PathBuf, String) {
    let path = repository_root().join(format!("assets/scenarios/eatme/{id}.yaml"));
    let yaml = fs::read_to_string(&path).expect("scenario should exist");
    (path, yaml)
}

#[test]
fn hour_of_code_curriculum_contract_requires_first_scene_animation_and_reflection() {
    let (path, yaml) = read_scenario("hour-of-code-studio-kickoff");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("first scene exists")
            && yaml.contains("first animation runs")
            && yaml.contains("cause/effect reflection"),
        "hour-of-code contract should keep the first scene, first animation, and reflection evidence explicit"
    );
    assert!(
        yaml.contains("Alice.org Hour of Code lesson")
            && yaml.contains("Alice.org Building a Scene lesson"),
        "hour-of-code contract should stay grounded in Alice.org lesson sources"
    );
}

#[test]
fn score_timer_curriculum_contract_requires_arithmetic_end_state_and_playtest_evidence() {
    let (path, yaml) = read_scenario("game-score-timer-win-lose-loop");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("collision_or_proximity_trigger_changes_score")
            && yaml.contains("timer_or_countdown_changes_over_time")
            && yaml.contains("visible_win_or_lose_end_state"),
        "score/timer contract should require scoring, timer changes, and visible win/lose evidence"
    );
    assert!(
        yaml.contains("starting values")
            && yaml.contains("update rules")
            && yaml.contains("both win and lose playtest evidence"),
        "score/timer reflection should ask learners to explain the arithmetic and end-state logic"
    );
}

#[test]
fn class_portability_curriculum_contract_requires_export_import_and_behavior_persistence() {
    let (path, yaml) = read_scenario("modified-class-portability");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("export evidence")
            && yaml.contains("import evidence")
            && yaml.contains("behavior persistence after import")
            && yaml.contains("different Alice project")
            && yaml.contains("browser UI journey")
            && yaml.contains("e2e/class-behavior-package.spec.ts"),
        "class portability contract should require export, import, and post-import behavior proof"
    );
    assert!(
        yaml.contains("before-export") && yaml.contains("after-import"),
        "class portability prompts should keep the before/after comparison explicit"
    );
}

#[test]
fn vr_camera_curriculum_contract_records_real_vr_gate_and_desktop_fallback() {
    let (path, yaml) = read_scenario("vr-camera-locomotion-journey");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("EATME_REAL_VR=1")
            && yaml.contains("VR_HEADSET_AVAILABLE=1")
            && yaml.contains("real_vr_available=false"),
        "vr camera contract should record both the real VR gate and the explicit fallback path"
    );
    assert!(
        yaml.contains("camera marker")
            && yaml.contains("locomotion-comfort")
            && yaml.contains("follow-on guidance only"),
        "vr camera contract should keep viewpoint, locomotion, and comfort evidence as follow-on guidance"
    );
    assert!(
        yaml.contains("guidance only")
            && yaml.contains("does not validate learner viewpoint")
            && !yaml.contains("expected_evidence="),
        "vr camera guidance must not masquerade as validated learner evidence"
    );
}

#[test]
fn vr_camera_lookingglass_wording_is_bounded_to_api_evidence() {
    let root = repository_root();
    let matrix =
        fs::read_to_string(root.join("assets/parity/rabbithole-lookingglass-journey-matrix.yaml"))
            .expect("matrix should exist");
    let coverage = fs::read_to_string(root.join("docs/eatme/alice-howto-coverage.md"))
        .expect("coverage doc should exist");
    let combined = format!("{matrix}\n{coverage}");
    let overstated_browser_claim = format!("{}{}", "browser camera movement ", "journey");
    let overstated_vr_claim = format!("{}{}", "VR-style camera movement ", "journey");

    assert!(
        combined.contains("bounded browser camera comfort API evidence"),
        "LookingGlass VR camera wording should be bounded to API evidence"
    );
    assert!(
        !combined.contains(&overstated_browser_claim) && !combined.contains(&overstated_vr_claim),
        "LookingGlass/browser wording must not imply real camera movement steps were exercised"
    );
}

#[test]
fn vr_player_comfort_playtest_declared_files_are_written_and_checked() {
    let (path, yaml) = read_scenario("vr-player-comfort-playtest");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("vr_player_preflight_record")
            && yaml.contains("comfort_playtest_guidance_template")
            && yaml.contains("lookingglass_player_comfort_session_evidence")
            && yaml.contains("runs/vr-player-comfort-playtest/${RUN_ID}/vr-player-preflight.txt")
            && yaml.contains(
                "runs/vr-player-comfort-playtest/${RUN_ID}/comfort-playtest-guidance-template.md"
            ),
        "vr player comfort contract should keep the declared preflight and guidance template files explicit"
    );
    assert!(
        yaml.contains("> \"$run_dir/vr-player-preflight.txt\"")
            && yaml.contains("test -s \"$run_dir/vr-player-preflight.txt\"")
            && yaml.contains("grep -Eq \"real_vr_available=(true|false)\"")
            && yaml.contains("template=\"$run_dir/comfort-playtest-guidance-template.md\"")
            && yaml.contains("> \"$template\"")
            && yaml.contains("test -s \"$template\"")
            && yaml.contains("/api/vr/player-comfort-session")
            && yaml.contains("observed-before-after-revision"),
        "vr player comfort commands should write/check declared files and require submitted player session revision-loop evidence"
    );
    assert!(
        yaml.contains("guidance only")
            && yaml.contains("Comfort check")
            && yaml.contains("Orientation note")
            && yaml.contains("Player cue")
            && yaml.contains("Fallback path")
            && yaml.contains("Revision decision"),
        "vr player comfort guidance template should keep the player feedback prompts explicit without claiming observed evidence"
    );
    let legacy_snake = format!("{}{}{}", "comfort_", "playtest_", "notes");
    let legacy_file = format!("{}{}{}", "comfort-", "playtest-", "notes.md");
    let legacy_evidence_claim = format!("{}{}", "notes ask for ", "comfort");
    assert!(
        !yaml.contains(&legacy_snake)
            && !yaml.contains(&legacy_file)
            && !yaml.contains(&legacy_evidence_claim),
        "vr player comfort guidance/template must not masquerade as validated playtest notes"
    );
}

#[test]
fn arrays_curriculum_contract_requires_visible_collection_order_and_boundary_evidence() {
    let (path, yaml) = read_scenario("arrays-collection-choreography");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("visible list/array behavior")
            && yaml.contains("item order")
            && yaml.contains("boundary tests"),
        "arrays contract should require visible collection behavior, ordering, and boundary coverage"
    );
    assert!(
        yaml.contains("manifest scenario_id equals arrays-collection-choreography")
            && yaml.contains("Alice.org Alice 3 lessons"),
        "arrays contract should stay pinned to both the real smoke manifest and Alice lesson sources"
    );
}

#[test]
fn mythic_choice_curriculum_contract_requires_branching_and_alternate_path_evidence() {
    let (path, yaml) = read_scenario("mythic-choice-event-tree");
    let validation = validate_scenario_asset(&path).expect("scenario should validate");

    assert!(validation.passed, "{:?}", validation.errors);
    assert!(
        yaml.contains("player trigger")
            && yaml.contains("state or condition")
            && yaml.contains("tested alternate paths"),
        "mythic choice contract should require trigger, state, and alternate path proof"
    );
    assert!(
        yaml.contains("interactive narrative or game")
            && yaml.contains("Alice.org Programming in Alice lesson family"),
        "mythic choice contract should stay grounded in the narrative/game lesson family"
    );
}

#[test]
fn round_86_targeted_web_curriculum_contracts_keep_missing_topics_explicit() {
    let targeted_contracts = [
        (
            "alien-linguist-parameter-dialogue",
            ["say/think speech", "dialogue cues", "gadugi-cli"],
        ),
        (
            "lost-robot-debug-museum",
            ["breakpoint", "debug checkpoint", "gadugi-cli"],
        ),
        (
            "audio-camera-and-export-sharecase",
            ["viewpoint", "camera marker", "gadugi-cli"],
        ),
        (
            "time-travel-recipe-sequencing",
            ["doInOrder", "sequential execution", "gadugi-cli"],
        ),
        (
            "modified-class-portability",
            [
                "gadugi-cli",
                "behavior persistence after import",
                "different Alice project",
            ],
        ),
    ];

    for (scenario_id, markers) in targeted_contracts {
        let (path, yaml) = read_scenario(scenario_id);
        let validation = validate_scenario_asset(&path).expect("scenario should validate");
        assert!(validation.passed, "{} {:?}", scenario_id, validation.errors);
        assert!(
            markers.iter().all(|marker| yaml.contains(marker)),
            "{} should keep round 86 topic markers {:?} explicit",
            scenario_id,
            markers
        );
    }
}
