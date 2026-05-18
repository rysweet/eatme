//! Desktop save-reopen integration test exercising the full silver thread
//! against real Alice.
//!
//! Gated behind `EATME_REAL_ALICE=1` — requires an actual Alice installation,
//! Xvfb, and the full desktop toolchain. Exercises:
//!   launch → edit → save → reopen → verify edit persistence.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled};

const REOPEN_DISPLAY: &str = ":98";
const REOPEN_SELECTOR: &str = "scene.eatmeFirstLessonStep";
const REOPEN_HOOK: &str = "tools/eatme-reopen-project";
const SCENARIO_ID: &str = "first-lessons-real-ui-actions";

// ── Xvfb lifecycle guard ──────────────────────────────────────────────────

struct XvfbGuard(Child);

impl XvfbGuard {
    fn start(display: &str, log_dir: &Path) -> Self {
        fs::create_dir_all(log_dir).expect("create log dir for reopen Xvfb");
        let child = Command::new("Xvfb")
            .args([display, "-screen", "0", "1280x1024x24", "-ac"])
            .stdout(fs::File::create(log_dir.join("xvfb-reopen-stdout.log")).unwrap())
            .stderr(fs::File::create(log_dir.join("xvfb-reopen-stderr.log")).unwrap())
            .spawn()
            .expect("start Xvfb for reopen phase");
        std::thread::sleep(std::time::Duration::from_secs(2));
        Self(child)
    }
}

impl Drop for XvfbGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

// ── JSON probe helpers ────────────────────────────────────────────────────

fn find_probe<'a>(
    contract: &'a serde_json::Value,
    action_id: &str,
) -> Option<&'a serde_json::Value> {
    contract["candidate_affordance_probes"]
        .as_array()?
        .iter()
        .find(|p| p["action_id"].as_str() == Some(action_id))
}

/// Mirrors `UiActionEditProcedureProbe::proves_edit()` via JSON.
fn proves_edit(probe: &serde_json::Value) -> bool {
    let status = probe["status"].as_str().unwrap_or("");
    let has_artifact = !probe["edited_project_artifact"].is_null();
    let no_errors = probe["validation_errors"]
        .as_array()
        .is_none_or(|a| a.is_empty());
    let verified = probe["edit_procedure_verified"].as_bool().unwrap_or(false);
    ((status == "passed" || status == "proved") && has_artifact && no_errors) || verified
}

/// Mirrors `UiActionSaveProjectProbe::proves_save()` via JSON.
fn proves_save(probe: &serde_json::Value) -> bool {
    probe["status"].as_str() == Some("passed")
        && !probe["saved_project_artifact"].is_null()
        && !probe["save_artifact"].is_null()
        && probe["validation_errors"]
            .as_array()
            .is_none_or(|a| a.is_empty())
}

/// Extracts the saved project path from the save probe, resolving against run_dir.
fn extract_saved_project(probe: &serde_json::Value, run_dir: &Path) -> PathBuf {
    let raw = probe["saved_project_artifact"]["path"]
        .as_str()
        .expect("saved_project_artifact.path must be a string");
    let p = Path::new(raw);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        run_dir.join(p)
    }
}

/// Validates the raw reopen-hook JSON output, mirroring the validation that
/// `probe_project_reopen_hook` performs before setting probe status to "passed".
fn proves_reopen(result: &serde_json::Value, evidence_dir: &Path) -> bool {
    let s = |key: &str| result[key].as_str().unwrap_or("");
    let state = s("reopened_state_artifact");
    s("status") == "reopened"
        && s("state_verification") == "passed"
        && !s("source_saved_project_artifact").is_empty()
        && !s("reopened_project_artifact").is_empty()
        && !s("reopen_artifact").is_empty()
        && !state.is_empty()
        && evidence_dir.join(state).is_file()
}

// ── Reopen hook invocation ────────────────────────────────────────────────

fn run_reopen_hook(
    alice_home: &Path,
    saved_project: &Path,
    evidence_dir: &Path,
    display: &str,
) -> serde_json::Value {
    fs::create_dir_all(evidence_dir).expect("create reopen evidence dir");
    let hook = alice_home.join(REOPEN_HOOK);
    assert!(hook.is_file(), "reopen hook missing at {}", hook.display());

    let out = Command::new(&hook)
        .args([
            "--saved-project",
            &saved_project.display().to_string(),
            "--reopen-selector",
            REOPEN_SELECTOR,
            "--evidence-dir",
            &evidence_dir.display().to_string(),
            "--json",
        ])
        .current_dir(alice_home)
        .env("DISPLAY", display)
        .output()
        .unwrap_or_else(|e| panic!("reopen hook failed to launch: {e}"));

    assert!(
        out.status.success(),
        "reopen hook exited with {}: stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "reopen hook JSON parse failed: {e}\nstdout: {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

// ── Test ──────────────────────────────────────────────────────────────────

#[test]
fn save_reopen_desktop_real_exercises_full_silver_thread() {
    if !real_alice_enabled() {
        eprintln!("skipping save-reopen desktop test (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let alice = alice_home();
    let runs_dir = env::current_dir()
        .unwrap()
        .join("target/test-work/save-reopen-desktop-real");
    let run_id = format!(
        "sr-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // ── Phase 1: Launch → Edit → Save via run_launch_smoke ──────────────
    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: alice.clone(),
        run_id: run_id.clone(),
        runs_dir: runs_dir.clone(),
        timeout_seconds: 120,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new(SCENARIO_ID),
    })
    .expect("run_launch_smoke should succeed");

    let run_dir = runs_dir.join(SCENARIO_ID).join(&run_id);

    // ── Phase 2: Verify edit and save via ui-action-contract.json ────────
    let contract_path = run_dir.join("ui-action-contract.json");
    assert!(
        contract_path.is_file(),
        "ui-action-contract.json must exist at {}",
        contract_path.display()
    );
    let contract: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&contract_path).expect("read ui-action-contract.json"),
    )
    .expect("parse ui-action-contract.json");

    let edit_probe = find_probe(&contract, "edit-procedure-or-code-block")
        .expect("contract must contain edit-procedure probe");
    assert!(
        proves_edit(edit_probe),
        "edit_procedure_probe.proves_edit() must return true:\n{}",
        serde_json::to_string_pretty(edit_probe).unwrap()
    );

    let save_probe =
        find_probe(&contract, "save-project").expect("contract must contain save-project probe");
    assert!(
        proves_save(save_probe),
        "save_project_probe.proves_save() must return true:\n{}",
        serde_json::to_string_pretty(save_probe).unwrap()
    );

    let saved_project = extract_saved_project(save_probe, &run_dir);
    assert!(
        saved_project.is_file(),
        "saved project artifact must exist at {}",
        saved_project.display()
    );
    assert!(
        saved_project.metadata().unwrap().len() > 0,
        "saved_project_artifact must be non-empty"
    );

    // ── Phase 3: Reopen in a fresh session ──────────────────────────────
    let reopen_evidence = run_dir.join("project-reopen");
    let _xvfb = XvfbGuard::start(REOPEN_DISPLAY, &run_dir);

    let reopen_result = run_reopen_hook(&alice, &saved_project, &reopen_evidence, REOPEN_DISPLAY);

    // ── Phase 4: Verify reopen and edit persistence ─────────────────────
    assert!(
        proves_reopen(&reopen_result, &reopen_evidence),
        "reopen_project_probe.proves_reopen() must return true:\n{}",
        serde_json::to_string_pretty(&reopen_result).unwrap()
    );

    let state_name = reopen_result["reopened_state_artifact"]
        .as_str()
        .expect("reopened_state_artifact must be present");
    let state_path = reopen_evidence.join(state_name);
    assert!(
        state_path.metadata().unwrap().len() > 0,
        "reopened_state_artifact must be non-empty (edit persisted)"
    );

    // ── Phase 5: Manifest JSON round-trip ───────────────────────────────
    let manifest_path = run_dir.join("manifest.json");
    assert!(manifest_path.is_file(), "manifest.json must exist");
    let manifest_json = fs::read_to_string(&manifest_path).expect("read manifest.json");
    let round_tripped: eatme_core::LaunchSmokeManifest =
        serde_json::from_str(&manifest_json).expect("manifest should round-trip through serde");
    assert_eq!(round_tripped.scenario_id, SCENARIO_ID);
    assert_eq!(round_tripped.run_id, run_id);
    assert_eq!(
        round_tripped.assertions.len(),
        manifest.assertions.len(),
        "round-tripped manifest should preserve all assertions"
    );
}
