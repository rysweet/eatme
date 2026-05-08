use crate::validation::validate_persona_crew_against_scenario_assets;
use std::collections::BTreeSet;

use super::{TARGET_SCENARIOS, repository_root};

#[test]
fn missing_expansion_prompt_card_scenario_assets_fail_loudly() {
    let root = repository_root();
    let scenario_asset_ids = BTreeSet::from(["building-a-scene-first-world".to_string()]);
    let report = validate_persona_crew_against_scenario_assets(
        &root.join("assets/personas/alice-user-crew.yaml"),
        Some(&scenario_asset_ids),
    )
    .unwrap();

    assert!(!report.passed);
    for target in TARGET_SCENARIOS {
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("prompt card")
                    && error.contains(target.id)
                    && error.contains("missing scenario asset")),
            "missing expansion prompt-card scenario asset {} must be reported explicitly; got {:?}",
            target.id,
            report.errors
        );
    }
}
