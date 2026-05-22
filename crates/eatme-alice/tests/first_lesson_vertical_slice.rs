//! First-lesson vertical-slice tests: step-by-step go/no_go evidence,
//! structured JSON artifacts, and contract advancement as affordances prove.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::env;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture, alice_home, lock_env, real_alice_enabled};

// ─── Step-evidence helper ────────────────────────────────────────────

struct StepExpectation {
    action_id: &'static str,
    expected_decision: Decision,
}

enum Decision {
    Go,
    NoGo { missing_affordance_id: &'static str },
    Blocked,
}

/// Validates per-step evidence in the ui-action-contract JSON. `Go` checks
/// `executed_action_probes` for `status == "passed"`. `NoGo` checks
/// `action_precondition_probes` for `decision == "no_go"` with the expected
/// `missing_affordance.id`. `Blocked` asserts no precondition probe exists.
fn assert_step_evidence(contract: &serde_json::Value, steps: &[StepExpectation]) {
    let executed = contract["executed_action_probes"]
        .as_array()
        .expect("contract needs executed_action_probes");
    let preconditions = contract["action_precondition_probes"]
        .as_array()
        .expect("contract needs action_precondition_probes");

    for step in steps {
        match &step.expected_decision {
            Decision::Go => {
                let probe = executed
                    .iter()
                    .find(|p| p["id"] == step.action_id)
                    .unwrap_or_else(|| panic!("missing go probe for '{}'", step.action_id));
                assert_eq!(probe["status"], "passed", "{} status", step.action_id);
            }
            Decision::NoGo {
                missing_affordance_id,
            } => {
                let probe = preconditions
                    .iter()
                    .find(|p| p["action_id"] == step.action_id)
                    .unwrap_or_else(|| panic!("missing no_go probe for '{}'", step.action_id));
                assert_eq!(probe["decision"], "no_go", "{} decision", step.action_id);
                assert_eq!(
                    probe["missing_affordance"]["id"], *missing_affordance_id,
                    "{} affordance",
                    step.action_id
                );
            }
            Decision::Blocked => {
                let found = preconditions
                    .iter()
                    .any(|p| p["action_id"] == step.action_id);
                assert!(!found, "'{}' should be blocked (no probe)", step.action_id);
            }
        }
    }
}

// ─── Test 1: Fake toolchain – step-by-step evidence reporting ────────

#[test]
fn fake_toolchain_vertical_slice_reports_step_by_step_evidence() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "vertical-slice-evidence-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .expect("run_launch_smoke should succeed with fake tools");

    // --- Manifest-level assertions ---
    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_automation_unimplemented"),
    );
    assert_eq!(manifest.scenario_id, "first-lessons-real-ui-actions");
    assert!(manifest.ui_action_contract.is_some());

    assert_manifest_assertion(&manifest, "specific_alice_window_detected", true);
    assert_manifest_assertion(&manifest, "activate_alice_window_ui_action", true);
    assert_manifest_assertion(&manifest, "place_object_ui_action", false);
    assert_manifest_assertion(&manifest, "edit_procedure_ui_action", false);

    // --- Contract JSON step-by-step validation ---
    let contract_path = fixture.root.join(
        "runs/first-lessons-real-ui-actions/vertical-slice-evidence-run/ui-action-contract.json",
    );
    assert!(contract_path.is_file(), "contract JSON missing");

    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap())
            .expect("ui-action-contract.json should be valid JSON");

    assert_eq!(
        contract["schema_version"], "eatme.ui-action-contract/v1",
        "contract should use schema v1"
    );

    // Validate sequential step decisions
    assert_step_evidence(
        &contract,
        &[
            StepExpectation {
                action_id: "verify-specific-alice-window",
                expected_decision: Decision::Go,
            },
            StepExpectation {
                action_id: "activate-specific-alice-window",
                expected_decision: Decision::Go,
            },
            StepExpectation {
                action_id: "place-object",
                expected_decision: Decision::NoGo {
                    missing_affordance_id: "deterministic-alice-object-gallery-placement-affordance",
                },
            },
            // edit-procedure, run-world, save-project are blocked behind
            // place-object — they should NOT have precondition probes
            StepExpectation {
                action_id: "edit-procedure-or-code-block",
                expected_decision: Decision::Blocked,
            },
            StepExpectation {
                action_id: "run-world",
                expected_decision: Decision::Blocked,
            },
            StepExpectation {
                action_id: "save-project",
                expected_decision: Decision::Blocked,
            },
        ],
    );

    // Validate that the contract includes required_actions list
    let required_actions = contract["required_actions"].as_array().unwrap();
    assert!(required_actions.len() >= 4, "need ≥4 required_actions");

    assert_eq!(
        contract["preflight_evidence"]["specific_alice_window_detected"],
        true
    );
    assert_eq!(contract["preflight_evidence"]["log_captured"], true);
    assert_eq!(contract["status"], "blocked");
    assert!(
        contract["blocking_reason"]
            .as_str()
            .unwrap()
            .contains("object placement")
    );
}

// ─── Test 2: Real-Alice – per-step evidence with screenshots ─────────

#[test]
fn real_alice_vertical_slice_captures_per_step_evidence() {
    if !real_alice_enabled() {
        eprintln!("skipping real-Alice vertical-slice test (set EATME_REAL_ALICE=1)");
        return;
    }

    let _env_guard = lock_env();

    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/launch-smoke-real");
    let run_id = format!(
        "vertical-slice-{}",
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
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .expect("run_launch_smoke should succeed with real Alice");

    // Screenshot evidence: exists, non-empty, PNG magic bytes
    let screenshot = manifest
        .screenshot
        .as_ref()
        .expect("screenshot artifact missing");
    assert!(screenshot.size_bytes > 0);
    let screenshot_path = PathBuf::from(&screenshot.path);
    assert!(screenshot_path.exists(), "screenshot file missing");
    let bytes = fs::read(&screenshot_path).unwrap();
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']), "not PNG");

    // Per-step manifest assertions exist
    for key in [
        "specific_alice_window_detected",
        "activate_alice_window_ui_action",
        "place_object_ui_action",
        "edit_procedure_ui_action",
    ] {
        assert!(manifest.assertions.contains_key(key), "missing '{key}'");
    }

    // UI-action contract written with valid schema
    let contract_path = runs_dir
        .join("first-lessons-real-ui-actions")
        .join(&run_id)
        .join("ui-action-contract.json");
    assert!(contract_path.is_file(), "contract JSON missing");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();
    assert_eq!(contract["schema_version"], "eatme.ui-action-contract/v1");

    // Every executed probe has id + status
    let executed = contract["executed_action_probes"].as_array().unwrap();
    assert!(!executed.is_empty(), "need ≥1 executed probe");
    for probe in executed {
        assert!(probe["id"].is_string() && probe["status"].is_string());
    }

    // Manifest round-trip
    let manifest_path = runs_dir
        .join("first-lessons-real-ui-actions")
        .join(&run_id)
        .join("manifest.json");
    assert!(manifest_path.is_file());
    let rt: eatme_core::LaunchSmokeManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(rt.scenario_id, "first-lessons-real-ui-actions");
    assert_eq!(rt.run_id, run_id);
}

// ─── Test 3: Fake toolchain – contract advances with placement hook ──

#[test]
fn fake_toolchain_vertical_slice_advances_with_object_placement_hook() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "vertical-slice-hook-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .expect("run_launch_smoke should succeed with fake tools + placement hook");

    // --- Failure category should advance to remaining_steps ---
    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_remaining_steps_unimplemented"),
    );

    assert_manifest_assertion(&manifest, "place_object_ui_action", true);
    assert_manifest_assertion(&manifest, "edit_procedure_ui_action", false);
    assert_manifest_assertion(&manifest, "edit_procedure_precondition_no_go_probe", true);

    // --- Contract JSON validation ---
    let contract_path = fixture
        .root
        .join("runs/first-lessons-real-ui-actions/vertical-slice-hook-run/ui-action-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();

    // Validate the frontier has advanced
    assert_step_evidence(
        &contract,
        &[
            StepExpectation {
                action_id: "verify-specific-alice-window",
                expected_decision: Decision::Go,
            },
            StepExpectation {
                action_id: "activate-specific-alice-window",
                expected_decision: Decision::Go,
            },
            // place-object should NOT have a no_go probe anymore (it passed)
            StepExpectation {
                action_id: "place-object",
                expected_decision: Decision::Blocked, // no precondition probe = proven
            },
            // edit-procedure should now be the no_go frontier
            StepExpectation {
                action_id: "edit-procedure-or-code-block",
                expected_decision: Decision::NoGo {
                    missing_affordance_id: "deterministic-alice-procedure-edit-affordance",
                },
            },
            // run-world and save-project remain blocked behind edit-procedure
            StepExpectation {
                action_id: "run-world",
                expected_decision: Decision::Blocked,
            },
            StepExpectation {
                action_id: "save-project",
                expected_decision: Decision::Blocked,
            },
        ],
    );

    assert!(
        contract["blocking_reason"]
            .as_str()
            .unwrap()
            .contains("procedure")
    );

    // Placement proof in candidate_affordance_probes
    let candidates = contract["candidate_affordance_probes"].as_array().unwrap();
    let placement = candidates
        .iter()
        .find(|c| c["action_id"] == "place-object")
        .expect("placement candidate should exist");
    assert_eq!(placement["status"], "passed");
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn assert_manifest_assertion(
    manifest: &eatme_core::LaunchSmokeManifest,
    key: &str,
    expected_passed: bool,
) {
    let result = manifest
        .assertions
        .get(key)
        .unwrap_or_else(|| panic!("missing manifest assertion '{key}'"));
    assert_eq!(result.passed, expected_passed, "{key}: {}", result.detail);
}
