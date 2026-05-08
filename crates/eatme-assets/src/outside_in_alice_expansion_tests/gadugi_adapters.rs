use crate::{generate_gadugi_adapters, validate_assets};

use super::{EXPECTED_SCENARIO_ASSET_COUNT, TARGET_SCENARIOS, repository_root, scenario_path};

#[test]
fn alice_outside_in_expansion_assets_exist_validate_and_have_fresh_gadugi_adapters() {
    let root = repository_root();
    let report = validate_assets(&root).unwrap();
    let gadugi_report = generate_gadugi_adapters(&root, true).unwrap();
    let mut failures = Vec::new();

    if report.scenario_asset_count != EXPECTED_SCENARIO_ASSET_COUNT {
        failures.push(format!(
            "expected {EXPECTED_SCENARIO_ASSET_COUNT} scenario YAML assets after adding outside-in expansion and workshop coverage assets, got {}",
            report.scenario_asset_count
        ));
    }
    if !report.passed {
        failures.push(format!(
            "expanded asset inventory must validate cleanly: {:?}",
            report.errors
        ));
    }
    if !gadugi_report.passed {
        failures.push(format!(
            "expanded Gadugi adapters must be fresh: {:?}",
            gadugi_report.errors
        ));
    }

    for target in TARGET_SCENARIOS {
        let eatme_path = scenario_path(&root, "eatme", target.id);
        let gadugi_path = scenario_path(&root, "gadugi", target.id);

        if !eatme_path.is_file() {
            failures.push(format!(
                "{} must exist as the canonical outside-in Alice scenario",
                eatme_path.display()
            ));
            continue;
        }
        if !gadugi_path.is_file() {
            failures.push(format!(
                "{} must exist as the generated Gadugi adapter",
                gadugi_path.display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "outside-in Alice expansion asset contract failed:\n{}",
        failures.join("\n")
    );
}
