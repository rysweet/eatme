use super::{
    SHARING_SUCCESS_CLAIM_PATTERNS, assert_contains_all, assert_contains_all_across,
    sharing_readiness_boundary_doc,
};

const TEACHER_SHARING_READINESS_IMPACT: &str = "Readiness impact says the teacher gets a classroom/review handoff and remix feedback package, not proof of hosted or deployed sharing.";
const STUDENT_EATME_SCENARIO: &str =
    include_str!("../../../../assets/scenarios/eatme/student-artifact-package-share-evidence.yaml");
const TEACHER_EATME_SCENARIO: &str =
    include_str!("../../../../assets/scenarios/eatme/teacher-community-sharing-loop.yaml");
const STUDENT_GADUGI_ADAPTER: &str = include_str!(
    "../../../../assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml"
);
const TEACHER_GADUGI_ADAPTER: &str =
    include_str!("../../../../assets/scenarios/gadugi/teacher-community-sharing-loop.yaml");

#[test]
fn teacher_community_source_contract_names_readiness_impact_without_deployment_proof() {
    assert_contains_all(
        "teacher-community-sharing-loop source boundary",
        TEACHER_EATME_SCENARIO,
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
    assert_contains_all(
        "teacher-community-sharing-loop Gadugi adapter",
        TEACHER_GADUGI_ADAPTER,
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
    let docs = sharing_readiness_boundary_doc();

    assert_contains_all_across(
        "student sharing traceability",
        &[docs, STUDENT_EATME_SCENARIO, STUDENT_GADUGI_ADAPTER],
        &[
            "student-artifact-package-share-evidence",
            "artifact review packet checklist",
            "student evidence handoff prompt",
            "instructor review boundary note",
            "source_eatme_asset: assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
        ],
    );
    assert_contains_all_across(
        "teacher sharing traceability",
        &[docs, TEACHER_EATME_SCENARIO, TEACHER_GADUGI_ADAPTER],
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
    let surfaces = [
        (
            "docs/sharing-readiness-boundary.md",
            sharing_readiness_boundary_doc(),
        ),
        (
            "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
            STUDENT_EATME_SCENARIO,
        ),
        (
            "assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
            TEACHER_EATME_SCENARIO,
        ),
        (
            "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
            STUDENT_GADUGI_ADAPTER,
        ),
        (
            "assets/scenarios/gadugi/teacher-community-sharing-loop.yaml",
            TEACHER_GADUGI_ADAPTER,
        ),
    ];

    let mut failures = Vec::new();
    for (label, text) in surfaces {
        collect_positive_overclaims(&mut failures, label, text);
    }

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

fn collect_positive_overclaims(failures: &mut Vec<String>, label: &str, text: &str) {
    let normalized = text.to_lowercase();
    for pattern in SHARING_SUCCESS_CLAIM_PATTERNS {
        if normalized.contains(pattern) {
            failures.push(format!("{label} contains `{pattern}`"));
        }
    }
}
