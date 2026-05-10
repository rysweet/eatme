//! Real-Alice launch smoke integration test.
//!
//! Gated behind `EATME_REAL_ALICE=1` — requires an actual Alice installation,
//! Xvfb, and the full desktop toolchain. CI sets the env var; local devs skip
//! by default.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::env;
use std::fs;
use std::path::PathBuf;

fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> PathBuf {
    PathBuf::from(
        env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()),
    )
}

#[test]
fn real_alice_launch_smoke_produces_deterministic_evidence() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice smoke test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/launch-smoke-real");
    let run_id = format!(
        "real-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: alice_home(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 90,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::default(),
    })
    .expect("run_launch_smoke should succeed");

    // --- Assertion 1: all 6 manifest assertions pass ---
    let expected_assertions = [
        "dependencies_available",
        "display_responsive",
        "process_started",
        "startup_screenshot",
        "no_fatal_logs",
        "real_alice_execution_evidence",
    ];
    for key in &expected_assertions {
        let result = manifest
            .assertions
            .get(*key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(
            result.passed,
            "assertion {key} failed: {}",
            result.detail,
        );
    }

    // --- Assertion 2: no failure category ---
    assert!(
        manifest.failure_category.is_none(),
        "expected no failure category, got: {:?}",
        manifest.failure_category,
    );

    // --- Assertion 3: screenshot exists and is a valid PNG ---
    let screenshot_artifact = manifest
        .screenshot
        .as_ref()
        .expect("manifest should include a screenshot artifact");
    assert!(
        screenshot_artifact.size_bytes > 0,
        "screenshot should be non-empty",
    );
    let screenshot_path = PathBuf::from(&screenshot_artifact.path);
    assert!(
        screenshot_path.exists(),
        "screenshot file should exist at {}",
        screenshot_path.display(),
    );
    let screenshot_bytes = fs::read(&screenshot_path)
        .unwrap_or_else(|e| panic!("reading screenshot {}: {e}", screenshot_path.display()));
    assert!(
        screenshot_bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "screenshot should have PNG magic bytes, got: {:02x?}",
        &screenshot_bytes[..4.min(screenshot_bytes.len())],
    );

    // --- Assertion 4: manifest.json written to expected run directory ---
    let manifest_path = runs_dir
        .join("real-alice-launch-smoke")
        .join(&run_id)
        .join("manifest.json");
    assert!(
        manifest_path.is_file(),
        "manifest.json should exist at {}",
        manifest_path.display(),
    );

    // --- Assertion 5: manifest round-trips through JSON ---
    let manifest_json = fs::read_to_string(&manifest_path).unwrap();
    let round_tripped: eatme_core::LaunchSmokeManifest =
        serde_json::from_str(&manifest_json).expect("manifest should deserialize from disk");
    assert_eq!(round_tripped.scenario_id, "real-alice-launch-smoke");
    assert_eq!(round_tripped.run_id, run_id);
    assert_eq!(
        round_tripped.assertions.len(),
        manifest.assertions.len(),
        "round-tripped manifest should preserve all assertions",
    );

    // --- Assertion 6: alice.log captured ---
    let log_artifact = manifest
        .log
        .as_ref()
        .expect("manifest should include a log artifact");
    assert!(
        log_artifact.size_bytes > 0,
        "alice.log should be non-empty",
    );
}
