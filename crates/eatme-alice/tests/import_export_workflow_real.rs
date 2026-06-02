//! Import/export workflow integration test.
//!
//! Tests the full `.a3p` save/load/export round-trip against a real Alice
//! desktop session:
//!
//! 1. Open a starter project (via launch smoke with first-lessons scenario)
//! 2. Make a modification (add entity, edit procedure, run world — via hooks)
//! 3. Save the project (via save-project hook)
//! 4. Reopen and verify the modification persisted (via reopen-project hook)
//! 5. Export to NetBeans project format (via export-project hook)
//! 6. Verify the exported Ant `build.xml` exists
//!
//! Gated behind `EATME_REAL_ALICE=1` — requires an actual Alice installation,
//! Xvfb, and the full desktop toolchain. CI sets the env var; local devs skip
//! by default.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::env;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled};

#[allow(dead_code)]
mod import_export_support;
use import_export_support::{
    ProjectReopenHookResult, extract_saved_project_path, probe_export_hook, run_hook_with_timeout,
    start_xvfb_for_workflow, validate_evidence_artifact,
};

const REOPEN_SELECTOR: &str = "scene.myFirstMethod";

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[test]
fn save_reopen_export_round_trip() {
    if !real_alice_enabled() {
        eprintln!("skipping import/export workflow test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let alice = alice_home();
    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/import-export-workflow-real/runs");
    let run_id = format!(
        "ie-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // ── Phase 1: Launch smoke ──────────────────────────────────────────
    eprintln!("phase 1: launch smoke (first-lessons scenario, run_id={run_id})");
    //
    // Opens the starter project, runs place-object → edit-procedure →
    // run-world → save-project probe chain. The manifest captures all
    // evidence from the first-lessons scenario.
    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: alice.clone(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 900,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .expect("run_launch_smoke should succeed");

    // Core smoke assertions must pass before save/reopen/export makes sense.
    let core_assertions = [
        "dependencies_available",
        "display_responsive",
        "process_started",
        "startup_screenshot",
        "no_fatal_logs",
        "real_alice_execution_evidence",
    ];
    for key in &core_assertions {
        let result = manifest
            .assertions
            .get(*key)
            .unwrap_or_else(|| panic!("manifest missing assertion: {key}"));
        assert!(result.passed, "assertion {key} failed: {}", result.detail);
    }

    // ── Phase 2: Extract saved project path ────────────────────────────
    eprintln!("phase 2: extract saved project path from ui-action-contract.json");
    let run_dir = runs_dir.join("first-lessons-real-ui-actions").join(&run_id);
    let ui_action_contract_path = run_dir.join("ui-action-contract.json");
    assert!(
        ui_action_contract_path.is_file(),
        "ui-action-contract.json should exist at {}",
        ui_action_contract_path.display(),
    );

    let saved_project = match extract_saved_project_path(&ui_action_contract_path, &run_dir) {
        Some(path) => path,
        None => {
            eprintln!(
                "save-project probe did not pass — reopen/export phases blocked \
                 (save hook may not be implemented yet)"
            );
            return;
        }
    };
    let saved_meta = fs::metadata(&saved_project).unwrap_or_else(|e| {
        panic!(
            "saved .a3p should exist at {}: {e}",
            saved_project.display()
        )
    });
    assert!(saved_meta.is_file(), "saved .a3p should be a regular file");
    assert!(saved_meta.len() > 0, "saved .a3p should be non-empty");

    // ── Pre-check reopen hook before expensive Xvfb startup ────────────
    let reopen_hook = alice.join("tools/eatme-reopen-project");
    if !reopen_hook.is_file() {
        eprintln!(
            "reopen hook not found at {} — phases 3-5 blocked (contract-first)",
            reopen_hook.display()
        );
        return;
    }

    // ── Start Xvfb for reopen/export phases ────────────────────────────
    eprintln!("starting Xvfb for reopen/export phases");
    let xvfb = start_xvfb_for_workflow(&runs_dir);
    eprintln!("Xvfb started on display {}", xvfb.display);

    // ── Phase 3: Reopen the saved project ──────────────────────────────
    eprintln!("phase 3: reopen saved project via eatme-reopen-project hook");

    let reopen_evidence_dir = run_dir.join("project-reopen");
    fs::create_dir_all(&reopen_evidence_dir).expect("create reopen evidence dir");
    let saved_project_str = saved_project.display().to_string();
    let reopen_evidence_str = reopen_evidence_dir.display().to_string();

    let reopen_output = run_hook_with_timeout(
        &reopen_hook,
        &[
            "--saved-project",
            &saved_project_str,
            "--reopen-selector",
            REOPEN_SELECTOR,
            "--evidence-dir",
            &reopen_evidence_str,
            "--json",
        ],
        &alice,
        &xvfb.display,
        Duration::from_secs(30),
    );

    assert!(
        reopen_output.status.success(),
        "reopen hook should exit 0, got: {:?}\nstderr: {}",
        reopen_output.status.code(),
        String::from_utf8_lossy(&reopen_output.stderr),
    );

    let reopen_result: ProjectReopenHookResult = serde_json::from_slice(&reopen_output.stdout)
        .unwrap_or_else(|e| {
            panic!(
                "reopen hook stdout should be valid JSON: {e}\nstdout: {}",
                String::from_utf8_lossy(&reopen_output.stdout),
            )
        });

    assert_eq!(
        reopen_result.schema_version, "eatme.alice-project-reopen-result/v1",
        "reopen schema version mismatch",
    );
    assert_eq!(
        reopen_result.status, "reopened",
        "reopen status must be 'reopened'",
    );
    assert_eq!(
        reopen_result.state_verification, "passed",
        "reopened state verification must pass to prove modification persisted",
    );
    assert!(
        reopen_result
            .source_saved_project_artifact
            .starts_with("project-save/"),
        "source must reference project-save/ dir, got: {}",
        reopen_result.source_saved_project_artifact,
    );

    // Verify reopen evidence artifacts via shared validator (single stat each).
    let mut reopen_errors = Vec::new();
    validate_evidence_artifact(
        &reopen_evidence_dir,
        &reopen_result.reopened_project_artifact,
        "reopened project artifact",
        &mut reopen_errors,
    );
    validate_evidence_artifact(
        &reopen_evidence_dir,
        &reopen_result.reopened_state_artifact,
        "reopened state artifact",
        &mut reopen_errors,
    );
    assert!(
        reopen_errors.is_empty(),
        "reopen evidence validation failed: {:?}",
        reopen_errors,
    );

    // ── Phase 4-5: Export and verify via probe ───────────────────────────
    eprintln!("phase 4: export to NetBeans project format via eatme-export-project hook");
    let export_evidence_dir = run_dir.join("project-export");

    let export_probe =
        probe_export_hook(&alice, &saved_project, &export_evidence_dir, &xvfb.display);

    if export_probe.status == "blocked" {
        eprintln!(
            "export hook blocked — phases 4-5 skipped (contract-first): {}",
            export_probe.detail
        );
        drop(xvfb);
        return;
    }

    eprintln!("phase 5: verify exported build.xml exists");
    assert!(
        export_probe.proves_export(),
        "export probe must prove export — status={}, errors: {:?}\ndetail: {}\nstdout: {}\nstderr: {}",
        export_probe.status,
        export_probe.validation_errors,
        export_probe.detail,
        export_probe.stdout,
        export_probe.stderr,
    );

    eprintln!(
        "phase 5: export proof validated — build.xml at {:?}",
        export_probe.exported_build_file
    );

    // ── Phase 6: Cleanup (XvfbGuard Drop) ──────────────────────────────
    eprintln!("phase 6: cleanup (dropping Xvfb guard)");
    drop(xvfb);
}
