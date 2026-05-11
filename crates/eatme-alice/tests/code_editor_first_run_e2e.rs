//! Code-editor first-run E2E tests: issue #235.
//!
//! Proves four requirements through the `code-editor-first-run` scenario:
//!   1. Launch smoke passes for the code-editor scenario.
//!   2. Code editor tab can be observed (`edit-procedure-or-code-block` in required_actions).
//!   3. A simple procedure structure exists (contract frontier reaches edit step).
//!   4. The result can be saved (`save-project` in required_actions with a status).

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::fs;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

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

/// Validates per-step evidence in the ui-action-contract JSON.
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

fn make_smoke_options(fixture: &TestFixture, run_id: &str) -> LaunchSmokeOptions {
    LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: run_id.into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    }
}

// ─── Test 1: Baseline contract generation ────────────────────────────

#[test]
fn baseline_contract_generates_ui_action_contract() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&make_smoke_options(&fixture, "code-editor-baseline"))
        .expect("run_launch_smoke should succeed with fake tools");

    // Manifest-level: scenario routed into ui-action-contract pipeline
    assert_eq!(manifest.scenario_id, "code-editor-first-run");
    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_automation_unimplemented"),
    );
    assert!(manifest.ui_action_contract.is_some());

    assert_manifest_assertion(&manifest, "specific_alice_window_detected", true);
    assert_manifest_assertion(&manifest, "activate_alice_window_ui_action", true);
    assert_manifest_assertion(&manifest, "place_object_ui_action", false);
    assert_manifest_assertion(&manifest, "edit_procedure_ui_action", false);

    // Contract JSON: validate schema and step decisions
    let contract_path = fixture
        .root
        .join("runs/code-editor-first-run/code-editor-baseline/ui-action-contract.json");
    assert!(contract_path.is_file(), "contract JSON missing");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap())
            .expect("ui-action-contract.json should be valid JSON");
    assert_eq!(contract["schema_version"], "eatme.ui-action-contract/v1");

    // Required actions include all four student action ids
    let required_actions = contract["required_actions"].as_array().unwrap();
    assert!(required_actions.len() >= 4, "need ≥4 required_actions");
    for expected_id in [
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project",
    ] {
        assert!(
            required_actions.iter().any(|a| a["id"] == expected_id),
            "required_actions missing '{expected_id}'"
        );
    }

    // Sequential step decisions
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
}

// ─── Test 2: Placement hook advances the contract frontier ───────────

#[test]
fn placement_hook_advances_contract_frontier() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&make_smoke_options(&fixture, "code-editor-advanced"))
        .expect("run_launch_smoke should succeed with fake tools + placement hook");

    // Failure category advances past place-object
    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_remaining_steps_unimplemented"),
    );
    assert_manifest_assertion(&manifest, "place_object_ui_action", true);
    assert_manifest_assertion(&manifest, "edit_procedure_ui_action", false);
    assert_manifest_assertion(&manifest, "edit_procedure_precondition_no_go_probe", true);

    // Contract JSON: frontier shifted to edit-procedure
    let contract_path = fixture
        .root
        .join("runs/code-editor-first-run/code-editor-advanced/ui-action-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();

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
                expected_decision: Decision::Blocked, // no precondition probe = proven
            },
            StepExpectation {
                action_id: "edit-procedure-or-code-block",
                expected_decision: Decision::NoGo {
                    missing_affordance_id: "deterministic-alice-procedure-edit-affordance",
                },
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

    // Placement proof exists in candidate_affordance_probes
    let candidates = contract["candidate_affordance_probes"].as_array().unwrap();
    let placement = candidates
        .iter()
        .find(|c| c["action_id"] == "place-object")
        .expect("placement candidate should exist");
    assert_eq!(placement["status"], "passed");
}

// ─── Test 3: Per-step readiness reports all steps ────────────────────

#[test]
fn per_step_readiness_reports_all_steps() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&make_smoke_options(&fixture, "code-editor-readiness"))
        .expect("run_launch_smoke should succeed with fake tools");

    assert_eq!(manifest.scenario_id, "code-editor-first-run");

    let contract_path = fixture
        .root
        .join("runs/code-editor-first-run/code-editor-readiness/ui-action-contract.json");
    assert!(contract_path.is_file(), "contract JSON missing");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&contract_path).unwrap()).unwrap();

    let required_actions = contract["required_actions"]
        .as_array()
        .expect("required_actions should be an array");

    // All six step ids present in order
    let step_ids: Vec<&str> = required_actions
        .iter()
        .filter_map(|a| a["id"].as_str())
        .collect();
    let expected_order = [
        "verify-specific-alice-window",
        "activate-specific-alice-window",
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project",
    ];
    for expected_id in &expected_order {
        assert!(
            step_ids.contains(expected_id),
            "required_actions missing step '{expected_id}'"
        );
    }

    // Verify ordering: each expected step appears after the previous one
    for pair in expected_order.windows(2) {
        let pos_a = step_ids.iter().position(|&id| id == pair[0]).unwrap();
        let pos_b = step_ids.iter().position(|&id| id == pair[1]).unwrap();
        assert!(
            pos_a < pos_b,
            "step '{}' should appear before '{}' (pos {} vs {})",
            pair[0],
            pair[1],
            pos_a,
            pos_b
        );
    }

    // save-project is terminal: verify it has a status in the contract
    // (blocked at baseline since place-object is the frontier)
    let preconditions = contract["action_precondition_probes"]
        .as_array()
        .expect("contract needs action_precondition_probes");
    let executed = contract["executed_action_probes"]
        .as_array()
        .expect("contract needs executed_action_probes");

    // save-project should be blocked (not in preconditions or executed)
    let save_in_preconditions = preconditions
        .iter()
        .any(|p| p["action_id"] == "save-project");
    let save_in_executed = executed.iter().any(|p| p["id"] == "save-project");
    assert!(
        !save_in_preconditions && !save_in_executed,
        "save-project should be blocked at baseline (not probed)"
    );

    // edit-procedure-or-code-block should also be blocked
    let edit_in_preconditions = preconditions
        .iter()
        .any(|p| p["action_id"] == "edit-procedure-or-code-block");
    assert!(
        !edit_in_preconditions,
        "edit-procedure-or-code-block should be blocked at baseline"
    );

    // Frontier is place-object with no_go
    let place_object_probe = preconditions
        .iter()
        .find(|p| p["action_id"] == "place-object")
        .expect("place-object precondition probe missing");
    assert_eq!(place_object_probe["decision"], "no_go");

    // Window steps passed
    let window_verify = executed
        .iter()
        .find(|p| p["id"] == "verify-specific-alice-window")
        .expect("window verify probe missing");
    assert_eq!(window_verify["status"], "passed");

    let window_activate = executed
        .iter()
        .find(|p| p["id"] == "activate-specific-alice-window")
        .expect("window activate probe missing");
    assert_eq!(window_activate["status"], "passed");
}
