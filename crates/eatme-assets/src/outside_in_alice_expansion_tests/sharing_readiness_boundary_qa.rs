use crate::schema::EatmeScenarioAsset;

use super::{
    EXPECTED_SCENARIO_ASSET_COUNT, assert_contains_all, assert_contains_all_across,
    assert_no_success_claims, collect_positive_overclaims, sharing_readiness_boundary_doc,
};

const QA_EATME_SCENARIO: &str =
    include_str!("../../../../assets/scenarios/eatme/sharing-readiness-boundary-qa.yaml");
const QA_GADUGI_ADAPTER: &str =
    include_str!("../../../../assets/scenarios/gadugi/sharing-readiness-boundary-qa.yaml");

#[test]
fn expected_asset_count_reflects_qa_scenario_addition() {
    assert_eq!(
        EXPECTED_SCENARIO_ASSET_COUNT, 95,
        "EXPECTED_SCENARIO_ASSET_COUNT must be 95 after adding sharing-readiness-boundary-qa (eatme + gadugi)"
    );
}

#[test]
fn qa_scenario_yaml_exists_and_deserializes_as_instructor_agentic_flow() {
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(QA_EATME_SCENARIO).unwrap();
    assert_eq!(scenario.id, "sharing-readiness-boundary-qa");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_eq!(scenario.owner, "eatme");
}

#[test]
fn qa_scenario_expected_outputs_name_boundary_report_summary_and_remediation() {
    assert_contains_all(
        "sharing-readiness-boundary-qa expected outputs",
        QA_EATME_SCENARIO,
        &[
            "boundary_violation_report",
            "boundary_preservation_summary",
            "remediation_guidance",
        ],
    );
}

#[test]
fn qa_scenario_contract_names_what_sharing_readiness_is_not() {
    assert_contains_all(
        "sharing-readiness-boundary-qa negative-space contract",
        QA_EATME_SCENARIO,
        &[
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn qa_scenario_does_not_contain_any_overclaim_patterns() {
    assert_no_success_claims("sharing-readiness-boundary-qa", QA_EATME_SCENARIO);
}

#[test]
fn qa_scenario_names_all_five_sharing_evidence_surfaces() {
    assert_contains_all(
        "sharing-readiness-boundary-qa evidence surfaces",
        QA_EATME_SCENARIO,
        &[
            "docs/sharing-readiness-boundary.md",
            "assets/scenarios/eatme/student-artifact-package-share-evidence.yaml",
            "assets/scenarios/eatme/teacher-community-sharing-loop.yaml",
            "assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml",
            "assets/scenarios/gadugi/teacher-community-sharing-loop.yaml",
        ],
    );
}

#[test]
fn qa_gadugi_adapter_exists_and_preserves_boundary_language() {
    assert_contains_all(
        "sharing-readiness-boundary-qa Gadugi adapter",
        QA_GADUGI_ADAPTER,
        &[
            "sharing-readiness-boundary-qa",
            "source_eatme_asset: assets/scenarios/eatme/sharing-readiness-boundary-qa.yaml",
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn qa_scenario_traceable_to_boundary_documentation() {
    let docs = sharing_readiness_boundary_doc();

    assert_contains_all_across(
        "boundary QA traceability",
        &[docs, QA_EATME_SCENARIO],
        &[
            "sharing-readiness-boundary-qa",
            "boundary_violation_report",
            "boundary_preservation_summary",
            "remediation_guidance",
        ],
    );
}

#[test]
fn qa_scenario_acceptance_probes_reject_overclaim_patterns() {
    assert_contains_all(
        "sharing-readiness-boundary-qa acceptance probes",
        QA_EATME_SCENARIO,
        &[
            "overclaim patterns",
            "classroom review artifacts",
            "bounded",
        ],
    );
}

#[test]
fn overclaim_guard_covers_qa_scenario_surface() {
    let mut failures = Vec::new();
    collect_positive_overclaims(
        &mut failures,
        "assets/scenarios/eatme/sharing-readiness-boundary-qa.yaml",
        QA_EATME_SCENARIO,
    );
    collect_positive_overclaims(
        &mut failures,
        "assets/scenarios/gadugi/sharing-readiness-boundary-qa.yaml",
        QA_GADUGI_ADAPTER,
    );

    assert!(
        failures.is_empty(),
        "QA scenario and adapter must stay bounded to classroom review artifacts:\n{}",
        failures.join("\n")
    );
}
