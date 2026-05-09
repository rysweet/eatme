use std::fs;

use super::{
    SHARING_OVERCLAIM_PATTERNS, assert_contains_all, repository_root, scenario_path,
    sharing_readiness_boundary_doc,
};

const TEACHER_SHARING_READINESS_IMPACT: &str = "Readiness impact says the teacher gets a classroom/review handoff and remix feedback package, not proof of hosted or deployed sharing.";

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
fn sharing_recovery_evidence_stays_traceable_to_canonical_scenarios_and_adapters() {
    let root = repository_root();
    let student_source = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "student-artifact-package-share-evidence",
    ))
    .unwrap();
    let teacher_source = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "teacher-community-sharing-loop",
    ))
    .unwrap();
    let student_adapter = fs::read_to_string(scenario_path(
        &root,
        "gadugi",
        "student-artifact-package-share-evidence",
    ))
    .unwrap();
    let teacher_adapter = fs::read_to_string(scenario_path(
        &root,
        "gadugi",
        "teacher-community-sharing-loop",
    ))
    .unwrap();
    let docs = sharing_readiness_boundary_doc();

    assert_contains_all(
        "student sharing traceability",
        &format!("{docs}\n{student_source}\n{student_adapter}"),
        &[
            "student-artifact-package-share-evidence",
            "artifact review packet checklist",
            "student evidence handoff prompt",
            "instructor review boundary note",
            "source_eatme_asset: assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
        ],
    );
    assert_contains_all(
        "teacher sharing traceability",
        &format!("{docs}\n{teacher_source}\n{teacher_adapter}"),
        &[
            "teacher-community-sharing-loop",
            "teacher-community share card",
            "classroom handoff note",
            "remix feedback prompt",
            "source_eatme_asset: assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
        ],
    );
}

#[test]
fn sharing_evidence_surfaces_do_not_turn_boundary_terms_into_success_claims() {
    let root = repository_root();
    let surfaces = [
        (
            "docs/sharing-readiness-boundary.md",
            sharing_readiness_boundary_doc().to_owned(),
        ),
        (
            "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
            fs::read_to_string(scenario_path(
                &root,
                "eatme",
                "student-artifact-package-share-evidence",
            ))
            .unwrap(),
        ),
        (
            "assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
            fs::read_to_string(scenario_path(
                &root,
                "eatme",
                "teacher-community-sharing-loop",
            ))
            .unwrap(),
        ),
        (
            "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
            fs::read_to_string(scenario_path(
                &root,
                "gadugi",
                "student-artifact-package-share-evidence",
            ))
            .unwrap(),
        ),
        (
            "assets/scenarios/gadugi/teacher-community-sharing-loop.yaml",
            fs::read_to_string(scenario_path(
                &root,
                "gadugi",
                "teacher-community-sharing-loop",
            ))
            .unwrap(),
        ),
    ];

    let failures = surfaces
        .iter()
        .flat_map(|(label, text)| positive_overclaims(label, text))
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "sharing evidence surfaces must stay bounded to classroom review artifacts:\n{}",
        failures.join("\n")
    );
}

#[test]
fn sharing_readiness_docs_keep_optional_export_and_handoff_completion_plain() {
    let docs = sharing_readiness_boundary_doc();

    assert_contains_all(
        "sharing readiness docs optional handoff boundary",
        docs,
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
    let docs = sharing_readiness_boundary_doc();
    let normalized_docs = docs.to_lowercase();
    let forbidden = [
        "requires save completion",
        "depends on save completion",
        "save completion passed",
        "requires first-lesson completion",
        "depends on first-lesson completion",
        "first-lesson completion passed",
        "requires first lesson completion",
        "depends on first lesson completion",
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

fn positive_overclaims(label: &str, text: &str) -> Vec<String> {
    let normalized = text.to_lowercase();
    SHARING_OVERCLAIM_PATTERNS
        .iter()
        .filter(|pattern| normalized.contains(&pattern.to_lowercase()))
        .map(|pattern| format!("{label} contains `{pattern}`"))
        .collect()
}
