use std::fs;

use super::{assert_contains_all, repository_root, scenario_path};

const TEACHER_SHARING_READINESS_IMPACT: &str = "Readiness impact says the teacher gets a classroom/review handoff and remix feedback package, not proof that deployed sharing works.";

#[test]
fn teacher_community_source_contract_names_readiness_impact_without_deployment_proof() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "teacher-community-sharing-loop",
    ))
    .unwrap();

    assert_contains_all(
        "teacher-community-sharing-loop source boundary",
        &contract,
        &[
            "teacher-community share card",
            "classroom handoff note",
            "remix feedback prompt",
            TEACHER_SHARING_READINESS_IMPACT,
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn generated_teacher_community_adapter_preserves_readiness_impact_boundary() {
    let root = repository_root();
    let adapter = fs::read_to_string(scenario_path(
        &root,
        "gadugi",
        "teacher-community-sharing-loop",
    ))
    .unwrap();

    assert_contains_all(
        "teacher-community-sharing-loop Gadugi adapter",
        &adapter,
        &[
            "Teacher-community share card includes audience",
            "Classroom handoff note tells the next teacher",
            "Remix feedback prompt asks for classroom fit",
            TEACHER_SHARING_READINESS_IMPACT,
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn sharing_readiness_docs_keep_optional_export_and_handoff_completion_plain() {
    let root = repository_root();
    let docs = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();

    assert_contains_all(
        "sharing readiness docs optional handoff boundary",
        &docs,
        &[
            "The Alice world, screenshot, classroom artifact, or exported file if one is already available.",
            "The handoff loop is complete when the next revision is clear.",
            "`NODE_OPTIONS` is optional local runtime tuning",
            "No deployment or platform configuration is required for sharing readiness.",
        ],
    );
}

#[test]
fn sharing_readiness_docs_do_not_require_save_or_first_lesson_completion() {
    let root = repository_root();
    let docs = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();
    let normalized_docs = docs.to_lowercase();
    let forbidden = [
        "save completion",
        "first-lesson completion",
        "first lesson completion",
        "completed first lesson",
    ];
    let present = forbidden
        .iter()
        .filter(|phrase| normalized_docs.contains(*phrase))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        present.is_empty(),
        "sharing readiness docs must not depend on Save or first-lesson completion claims: {present:?}"
    );
}
