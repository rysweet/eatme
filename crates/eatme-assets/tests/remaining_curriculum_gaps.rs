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
            && yaml.contains("different Alice project"),
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
            && yaml.contains("locomotion plan")
            && yaml.contains("comfort"),
        "vr camera contract should require viewpoint, locomotion, and comfort evidence"
    );
}
