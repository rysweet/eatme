use super::{require_list, require_nonempty};
use crate::schema::EatmeScenarioAsset;

pub(crate) fn is_class_portability_scenario(scenario: &EatmeScenarioAsset) -> bool {
    scenario.id == "modified-class-portability" || scenario.kind == "alice_class_portability_smoke"
}

pub(crate) fn validate_class_portability_scenario(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if scenario.kind != "alice_class_portability_smoke" {
        errors.push("kind must be alice_class_portability_smoke".into());
    }
    if scenario.owner != "eatme" {
        errors.push("owner must be eatme".into());
    }
    match &scenario.real_alice {
        Some(real_alice) if real_alice.gated_by == "EATME_REAL_ALICE=1" => {}
        Some(real_alice) => errors.push(format!(
            "real_alice.gated_by must be EATME_REAL_ALICE=1, got {}",
            real_alice.gated_by
        )),
        None => errors.push("real_alice.gated_by must be EATME_REAL_ALICE=1".into()),
    }
    match &scenario.smoke_ready {
        Some(smoke_ready) => require_list(&smoke_ready.evidence, "smoke_ready.evidence", errors),
        None => errors.push("smoke_ready.evidence must be defined".into()),
    }
    if scenario.acceptance_criteria.is_empty() {
        errors.push("acceptance_criteria must contain at least one criterion".into());
    }
    for (index, criterion) in scenario.acceptance_criteria.iter().enumerate() {
        require_nonempty(
            &criterion.given,
            &format!("acceptance_criteria[{index}].given"),
            errors,
        );
        require_nonempty(
            &criterion.when,
            &format!("acceptance_criteria[{index}].when"),
            errors,
        );
        require_nonempty(
            &criterion.then,
            &format!("acceptance_criteria[{index}].then"),
            errors,
        );
    }
    match &scenario.portability {
        Some(portability) => {
            require_nonempty(
                &portability.source_project,
                "portability.source_project",
                errors,
            );
            require_nonempty(
                &portability.destination_project,
                "portability.destination_project",
                errors,
            );
            require_nonempty(
                &portability.modified_class,
                "portability.modified_class",
                errors,
            );
            require_nonempty(
                &portability.share_channel,
                "portability.share_channel",
                errors,
            );
            require_list(
                &portability.evidence_after_import,
                "portability.evidence_after_import",
                errors,
            );
        }
        None => errors.push("portability.evidence_after_import must be defined".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        EatmeScenarioAcceptanceCriterion, EatmeScenarioPortability, EatmeScenarioRealAlice,
        EatmeScenarioSmokeReady,
    };

    fn class_portability_scenario() -> EatmeScenarioAsset {
        EatmeScenarioAsset {
            id: "modified-class-portability".into(),
            kind: "alice_class_portability_smoke".into(),
            owner: "eatme".into(),
            real_alice: Some(EatmeScenarioRealAlice {
                gated_by: "EATME_REAL_ALICE=1".into(),
            }),
            smoke_ready: Some(EatmeScenarioSmokeReady {
                evidence: vec!["manifest assertions".into()],
            }),
            acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
                given: "a modified class is exported from a source project".into(),
                when: "a different Alice project imports and runs it".into(),
                then: "the imported behavior is still visible".into(),
            }],
            ..EatmeScenarioAsset::default()
        }
    }

    #[test]
    fn class_portability_requires_import_persistence_evidence() {
        let scenario = class_portability_scenario();
        let mut errors = Vec::new();
        validate_class_portability_scenario(&scenario, &mut errors);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("portability.evidence_after_import")),
            "expected missing portability evidence error: {errors:?}"
        );

        let scenario = EatmeScenarioAsset {
            portability: Some(EatmeScenarioPortability {
                source_project: "source".into(),
                destination_project: "destination".into(),
                modified_class: "helper-character".into(),
                share_channel: "exported class package".into(),
                evidence_after_import: vec!["post-import run shows the modified behavior".into()],
            }),
            ..scenario
        };
        errors.clear();
        validate_class_portability_scenario(&scenario, &mut errors);

        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn class_portability_detection_accepts_matching_id_or_kind() {
        let scenario = class_portability_scenario();
        assert!(is_class_portability_scenario(&scenario));

        let by_kind = EatmeScenarioAsset {
            id: "another-scenario".into(),
            ..scenario.clone()
        };
        assert!(is_class_portability_scenario(&by_kind));

        let unrelated = EatmeScenarioAsset {
            id: "creative-world".into(),
            kind: "alice_scene_building".into(),
            ..scenario
        };
        assert!(!is_class_portability_scenario(&unrelated));
    }

    #[test]
    fn class_portability_reports_missing_acceptance_and_portability_fields() {
        let scenario = EatmeScenarioAsset {
            kind: "wrong-kind".into(),
            owner: "other".into(),
            real_alice: Some(EatmeScenarioRealAlice {
                gated_by: "REAL_ALICE=1".into(),
            }),
            smoke_ready: Some(EatmeScenarioSmokeReady { evidence: vec![] }),
            acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
                given: "".into(),
                when: "".into(),
                then: "".into(),
            }],
            portability: Some(EatmeScenarioPortability {
                source_project: "".into(),
                destination_project: "".into(),
                modified_class: "".into(),
                share_channel: "".into(),
                evidence_after_import: vec![],
            }),
            ..class_portability_scenario()
        };
        let mut errors = Vec::new();
        validate_class_portability_scenario(&scenario, &mut errors);

        for expected in [
            "kind must be alice_class_portability_smoke",
            "owner must be eatme",
            "real_alice.gated_by must be EATME_REAL_ALICE=1",
            "smoke_ready.evidence must contain non-empty values",
            "acceptance_criteria[0].given must not be empty",
            "acceptance_criteria[0].when must not be empty",
            "acceptance_criteria[0].then must not be empty",
            "portability.source_project must not be empty",
            "portability.destination_project must not be empty",
            "portability.modified_class must not be empty",
            "portability.share_channel must not be empty",
            "portability.evidence_after_import must contain non-empty values",
        ] {
            assert!(
                errors.iter().any(|error| error.contains(expected)),
                "missing expected error {expected:?} in {errors:?}"
            );
        }
    }
}
