//! Focused web platform setup/readiness scenario tests.

#[path = "support/setup_readiness.rs"]
mod setup_readiness;

use setup_readiness::{
    Step, assert_all, execute, http_client, selected_setup_scenarios, setup_scenarios,
    web_base_url, web_platform_enabled,
};

#[test]
fn setup_readiness_scenarios_exercise_preflight_config_create_and_handoff() {
    for (name, steps) in setup_scenarios() {
        assert!(steps.iter().any(|step| matches!(step, Step::Config)));
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::SetupPreflight { scenario } if scenario == name))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::ProjectNew { .. }))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::EvidenceHandoff { scenario } if scenario == name))
        );
    }
}

#[test]
fn live_setup_readiness_scenarios() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }

    let client = http_client();
    let base = web_base_url();
    let scenarios = selected_setup_scenarios();
    assert!(
        !scenarios.is_empty(),
        "EATME_SETUP_READINESS_SCENARIO must name a known setup readiness scenario"
    );
    for (_name, steps) in scenarios {
        assert_all(execute(&base, &client, &steps));
    }
}
