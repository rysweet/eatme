use std::fs;

use super::{
    EXPECTED_SCENARIO_ASSET_COUNT, SHARING_SUCCESS_CLAIM_PATTERNS, assert_contains_all,
    assert_contains_all_across, read_eatme_scenario, repository_root, scenario_path,
    sharing_readiness_boundary_doc,
};

#[test]
fn expected_asset_count_reflects_qa_scenario_addition() {
    assert_eq!(
        EXPECTED_SCENARIO_ASSET_COUNT, 95,
        "EXPECTED_SCENARIO_ASSET_COUNT must be 95 after adding sharing-readiness-boundary-qa (eatme + gadugi)"
    );
}

#[test]
fn qa_scenario_yaml_exists_and_deserializes_as_instructor_agentic_flow() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    assert!(
        path.is_file(),
        "sharing-readiness-boundary-qa.yaml must exist"
    );
    let scenario = read_eatme_scenario(&path);
    assert_eq!(scenario.id, "sharing-readiness-boundary-qa");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_eq!(scenario.owner, "eatme");
}

#[test]
fn qa_scenario_expected_outputs_name_boundary_report_summary_and_remediation() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let content = fs::read_to_string(&path).unwrap();

    assert_contains_all(
        "sharing-readiness-boundary-qa expected outputs",
        &content,
        &[
            "boundary_violation_report",
            "boundary_preservation_summary",
            "remediation_guidance",
        ],
    );
}

#[test]
fn qa_scenario_contract_names_what_sharing_readiness_is_not() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let content = fs::read_to_string(&path).unwrap();

    assert_contains_all(
        "sharing-readiness-boundary-qa negative-space contract",
        &content,
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
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let content = fs::read_to_string(&path).unwrap();
    let normalized = content.to_lowercase();

    let present: Vec<&&str> = SHARING_SUCCESS_CLAIM_PATTERNS
        .iter()
        .filter(|phrase| normalized.contains(**phrase))
        .collect();

    assert!(
        present.is_empty(),
        "sharing-readiness-boundary-qa must not contain overclaim patterns: {present:?}"
    );
}

#[test]
fn qa_scenario_names_all_five_sharing_evidence_surfaces() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let content = fs::read_to_string(&path).unwrap();

    assert_contains_all(
        "sharing-readiness-boundary-qa evidence surfaces",
        &content,
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
    let root = repository_root();
    let adapter_path = scenario_path(&root, "gadugi", "sharing-readiness-boundary-qa");
    assert!(
        adapter_path.is_file(),
        "sharing-readiness-boundary-qa Gadugi adapter must exist"
    );
    let adapter = fs::read_to_string(&adapter_path).unwrap();

    assert_contains_all(
        "sharing-readiness-boundary-qa Gadugi adapter",
        &adapter,
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
    let root = repository_root();
    let qa_content =
        fs::read_to_string(scenario_path(&root, "eatme", "sharing-readiness-boundary-qa"))
            .unwrap();

    assert_contains_all_across(
        "boundary QA traceability",
        &[docs, &qa_content],
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
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let content = fs::read_to_string(&path).unwrap();

    assert_contains_all(
        "sharing-readiness-boundary-qa acceptance probes",
        &content,
        &["overclaim patterns", "classroom review artifacts", "bounded"],
    );
}

#[test]
fn overclaim_guard_covers_qa_scenario_surface() {
    let root = repository_root();
    let qa_path = scenario_path(&root, "eatme", "sharing-readiness-boundary-qa");
    let qa_content = fs::read_to_string(&qa_path).unwrap();
    let qa_gadugi_path = scenario_path(&root, "gadugi", "sharing-readiness-boundary-qa");
    let qa_gadugi = fs::read_to_string(&qa_gadugi_path).unwrap();

    let surfaces = [
        (
            "assets/scenarios/eatme/sharing-readiness-boundary-qa.yaml",
            qa_content.as_str(),
        ),
        (
            "assets/scenarios/gadugi/sharing-readiness-boundary-qa.yaml",
            qa_gadugi.as_str(),
        ),
    ];

    let mut failures = Vec::new();
    for (label, text) in surfaces {
        let normalized = text.to_lowercase();
        for pattern in SHARING_SUCCESS_CLAIM_PATTERNS {
            if normalized.contains(pattern) {
                failures.push(format!("{label} contains `{pattern}`"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "QA scenario and adapter must stay bounded to classroom review artifacts:\n{}",
        failures.join("\n")
    );
}
