use crate::default_workflow_pr_readiness::{
    CheckConclusion, CheckRollupEvidence, CheckRunEvidence, Decision, FinalizationEvidence,
    ScopeSurface, SupplementalValidation, evaluate_finalization, required_supplemental_validations,
};

const HEAD_SHA: &str = super::HEAD_SHA;

#[test]
fn check_evidence_reader_rejects_required_skipped_checks_only() {
    let required_skipped = CheckRollupEvidence::for_head(
        HEAD_SHA,
        vec![
            check("required-secret-scan", CheckConclusion::Skipped, true),
            check("optional-doc-preview", CheckConclusion::Skipped, false),
        ],
    );

    let result = required_skipped.require_green_current_checks();

    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(message.contains("required-secret-scan"));
    assert!(message.contains("skipped"));
    assert!(!message.contains("optional-doc-preview"));
}

#[test]
fn required_success_and_optional_skipped_checks_are_merge_ready_evidence() {
    let checks = CheckRollupEvidence::for_head(
        HEAD_SHA,
        vec![
            check("required-quality-gate", CheckConclusion::Success, true),
            check("optional-doc-preview", CheckConclusion::Skipped, false),
        ],
    );

    assert!(checks.require_green_current_checks().is_ok());
}

#[test]
fn mergeable_must_be_explicitly_true_or_mergeable_before_no_op_or_merge_ready() {
    for mergeable_value in [r#""UNKNOWN""#, "false", "null"] {
        let evidence = FinalizationEvidence::from_offline_json(
            &offline_evidence_json_with_mergeable_value(mergeable_value),
        )
        .expect("non-mergeable evidence should parse before evaluation");

        let decision = evaluate_finalization(evidence);

        assert_eq!(decision.decision, Decision::NotMergeReady);
        assert!(decision.no_op_justification.is_none());
        assert_contains(&decision.blockers, "mergeable");
        assert_contains(&decision.blockers, "MERGEABLE");
    }
}

#[test]
fn boolean_true_mergeable_evidence_is_accepted_as_mergeable() {
    let evidence = FinalizationEvidence::from_offline_json(
        &offline_evidence_json_with_mergeable_value("true"),
    )
    .expect("boolean true mergeability evidence should parse");

    let decision = evaluate_finalization(evidence);

    assert_eq!(decision.decision, Decision::MergeReady);
    assert!(decision.no_op_justification.is_some());
}

#[test]
fn gadugi_adapter_paths_require_gadugi_freshness_not_asset_validation() {
    let evidence =
        FinalizationEvidence::from_offline_json(&offline_evidence_json_with_changed_files(
            &["assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml"],
            r#""MERGEABLE""#,
        ))
        .expect("offline evidence should parse changed Gadugi adapter paths");

    assert_eq!(
        evidence.scope_changes[0].surface,
        ScopeSurface::GeneratedGadugiAdapter
    );

    let required = required_supplemental_validations(&evidence.scope_changes, &evidence.checks);

    assert!(required.contains(&SupplementalValidation::GadugiFreshness));
    assert!(!required.contains(&SupplementalValidation::AssetValidation));
}

#[test]
fn canonical_scenario_paths_with_gadugi_in_the_name_still_require_asset_validation() {
    let evidence =
        FinalizationEvidence::from_offline_json(&offline_evidence_json_with_changed_files(
            &["assets/scenarios/eatme/gadugi-boundary-regression.yaml"],
            r#""MERGEABLE""#,
        ))
        .expect("offline evidence should parse changed source scenario paths");

    assert_eq!(
        evidence.scope_changes[0].surface,
        ScopeSurface::ScenarioAsset
    );

    let required = required_supplemental_validations(&evidence.scope_changes, &evidence.checks);

    assert!(required.contains(&SupplementalValidation::AssetValidation));
    assert!(required.contains(&SupplementalValidation::GadugiFreshness));
}

fn check(name: &str, conclusion: CheckConclusion, required: bool) -> CheckRunEvidence {
    CheckRunEvidence {
        name: name.into(),
        head_sha: HEAD_SHA.into(),
        conclusion,
        required,
        workflow_name: Some("CI".into()),
        details_url: Some(format!(
            "https://github.com/rysweet/eatme/actions/runs/{name}"
        )),
    }
}

fn offline_evidence_json_with_mergeable_value(mergeable_value: &str) -> String {
    offline_evidence_json_with_changed_files(
        &["docs/default-workflow-pr-readiness.md"],
        mergeable_value,
    )
}

fn offline_evidence_json_with_changed_files(
    changed_files: &[&str],
    mergeable_value: &str,
) -> String {
    let changed_files_json = changed_files
        .iter()
        .map(|path| format!(r#""{path}""#))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
            "repository": "rysweet/eatme",
            "pr_number": 173,
            "head_ref_name": "wave6-deployed-sharing-gap-1778302300",
            "pr_head_sha": "{HEAD_SHA}",
            "state": "OPEN",
            "draft": false,
            "local_branch": "wave6-deployed-sharing-gap-1778302300",
            "local_head_sha": "{HEAD_SHA}",
            "final_pr_head_sha": "{HEAD_SHA}",
            "worktree_clean": true,
            "merge_state_status": "CLEAN",
            "mergeable": {mergeable_value},
            "checks": [
                {{
                    "name": "quality-gates",
                    "head_sha": "{HEAD_SHA}",
                    "conclusion": "SUCCESS",
                    "required": true,
                    "workflow_name": "CI",
                    "details_url": "https://github.com/rysweet/eatme/actions/runs/quality-gates"
                }}
            ],
            "validated_gates": ["mkdocs build --strict"],
            "changed_files": [{changed_files_json}],
            "quality_audit_cycles": [
                {{
                    "seek": "scope and claim accuracy",
                    "validate": "reviewed readiness docs and PR metadata",
                    "fix": "no repository change required"
                }},
                {{
                    "seek": "canonical and generated asset consistency",
                    "validate": "GitHub checks current for head",
                    "fix": "no repository change required"
                }},
                {{
                    "seek": "gate completeness and final readiness",
                    "validate": "final PR head re-check matched",
                    "fix": "no repository change required"
                }}
            ]
        }}"#
    )
}

fn assert_contains(lines: &[String], needle: &str) {
    assert!(
        lines.iter().any(|line| line.contains(needle)),
        "expected {lines:?} to contain {needle}"
    );
}
