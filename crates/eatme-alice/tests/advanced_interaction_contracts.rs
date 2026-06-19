//! Contract tests for debugging, error-handling, and advanced interaction scenarios.
//!
//! These assertions stay inside the eatme repository and verify that the
//! committed scenario assets and test flows continue to cover the classroom
//! journeys we depend on: visible failures, guided debugging, code editing,
//! expression lessons, type reasoning, and gallery/scene-building flows.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("eatme repo root should resolve")
}

fn read_repo_file(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
}

#[test]
fn runtime_error_contract_fails_loudly_with_structured_output() {
    let contract = read_repo_file("assets/scenarios/gadugi/validation-failure-exit-code.yaml");

    assert!(contract.contains("exit_code: 1"));
    assert!(contract.contains("\"passed\": false"));
    assert!(contract.contains("must be kebab-case"));
}

#[test]
fn debugging_contract_requires_hypothesis_minimal_change_and_rerun_evidence() {
    let scenario = read_repo_file("assets/scenarios/eatme/lost-robot-debug-museum.yaml");

    assert!(scenario.contains("hypothesis"));
    assert!(scenario.contains("minimal change"));
    assert!(scenario.contains("rerun"));
    assert!(scenario.contains("peer question"));
}

#[test]
fn code_editing_contract_tracks_place_edit_run_and_save_steps() {
    let scenario = read_repo_file("assets/scenarios/eatme/code-editor-first-run.yaml");
    let e2e = read_repo_file("crates/eatme-alice/tests/code_editor_first_run_e2e.rs");

    assert!(scenario.contains("code-editor-first-run"));
    assert!(e2e.contains("place-object"));
    assert!(e2e.contains("edit-procedure-or-code-block"));
    assert!(e2e.contains("run-world"));
    assert!(e2e.contains("save-project"));
}

#[test]
fn expression_lessons_cover_arithmetic_and_relational_reasoning() {
    let arithmetic =
        read_repo_file("assets/scenarios/eatme/arithmetic-expressions-math-playground.yaml");
    let relational =
        read_repo_file("assets/scenarios/eatme/relational-expressions-comparison-lab.yaml");

    assert!(arithmetic.contains("addition, subtraction, multiplication, and"));
    assert!(arithmetic.contains("division operators"));
    assert!(relational.contains("comparison operators and boolean logic"));
    assert!(relational.contains("reflective-debugger"));
}

#[test]
fn type_reasoning_lessons_cover_data_types_and_parameters() {
    let data_types = read_repo_file("assets/scenarios/eatme/data-types-alice-catalog.yaml");
    let parameters = read_repo_file("assets/scenarios/eatme/reusable-methods-and-parameters.yaml");

    assert!(data_types.contains("Integer, Double, Boolean, String"));
    assert!(data_types.contains("data types"));
    assert!(parameters.contains("parameters"));
    assert!(parameters.contains("custom procedures"));
}

#[test]
fn gallery_and_scene_building_contracts_cover_browsing_and_object_placement() {
    let gallery = read_repo_file("crates/eatme-alice/tests/starter_project_gallery.rs");
    let building = read_repo_file("assets/scenarios/eatme/building-a-scene-first-world.yaml");

    assert!(gallery.contains("Starter-project gallery integration tests"));
    assert!(gallery.contains("starter projects"));
    assert!(building.contains("Building a Scene First World"));
    assert!(building.contains("scenario_id building-a-scene-first-world"));
}
