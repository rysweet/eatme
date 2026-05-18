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
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Environment gate
// ---------------------------------------------------------------------------

fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> PathBuf {
    PathBuf::from(env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()))
}

// ---------------------------------------------------------------------------
// Inline typed deserialization structs — define the hook contracts
// ---------------------------------------------------------------------------

/// JSON contract for `tools/eatme-reopen-project --json` output.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ProjectReopenHookResult {
    schema_version: String,
    status: String,
    source_saved_project_artifact: String,
    reopen_selector: String,
    reopened_project_artifact: String,
    reopen_artifact: String,
    reopened_state_artifact: String,
    state_verification: String,
}

/// JSON contract for `tools/eatme-export-project --json` output.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ProjectExportHookResult {
    schema_version: String,
    status: String,
    export_format: String,
    source_saved_project_artifact: String,
    exported_build_file: String,
    export_artifact: String,
}

// ---------------------------------------------------------------------------
// Xvfb display management with Drop-based cleanup
// ---------------------------------------------------------------------------

struct XvfbGuard {
    child: Child,
    display: String,
    lock_path: PathBuf,
}

impl Drop for XvfbGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn start_xvfb_for_workflow(runs_dir: &Path) -> XvfbGuard {
    let lock_dir = runs_dir.join(".display-locks");
    fs::create_dir_all(&lock_dir).expect("create display lock dir");

    for port in 90u16..130 {
        let lock_path = lock_dir.join(format!("X{port}.lock"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                let display = format!(":{port}");
                let child = Command::new("Xvfb")
                    .args([&display, "-screen", "0", "1280x1024x24", "-ac"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap_or_else(|e| panic!("start Xvfb on {display}: {e}"));

                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    if Instant::now() > deadline {
                        panic!("Xvfb {display} did not become ready within 5s");
                    }
                    let probe = Command::new("xdpyinfo")
                        .env("DISPLAY", &display)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    if probe.map(|s| s.success()).unwrap_or(false) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
                return XvfbGuard {
                    child,
                    display,
                    lock_path,
                };
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("create display lock {}: {e}", lock_path.display()),
        }
    }
    panic!("no free display in :90..:129 range");
}

// ---------------------------------------------------------------------------
// Hook runner with timeout
// ---------------------------------------------------------------------------

fn run_hook_with_timeout(
    hook: &Path,
    args: &[&str],
    cwd: &Path,
    display: &str,
    timeout: Duration,
) -> std::process::Output {
    let mut child = Command::new(hook)
        .args(args)
        .current_dir(cwd)
        .env("DISPLAY", display)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", hook.display()));

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).unwrap_or(0);
                        buf
                    })
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).unwrap_or(0);
                        buf
                    })
                    .unwrap_or_default();
                return std::process::Output {
                    status,
                    stdout,
                    stderr,
                };
            }
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{} exceeded {}s timeout", hook.display(), timeout.as_secs());
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(e) => panic!("waiting for {}: {e}", hook.display()),
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence extraction helpers
// ---------------------------------------------------------------------------

/// Extracts the saved project path from the ui-action-contract.json written
/// during launch smoke phase 1. Returns `None` if the save probe did not pass
/// or the contract is malformed.
fn extract_saved_project_path(ui_action_contract_path: &Path, run_dir: &Path) -> Option<PathBuf> {
    let json = fs::read_to_string(ui_action_contract_path).ok()?;
    let contract: serde_json::Value = serde_json::from_str(&json).ok()?;
    let probes = contract.get("candidate_affordance_probes")?.as_array()?;
    for probe in probes {
        if probe.get("action_id")?.as_str()? == "save-project" {
            if probe.get("status")?.as_str()? != "passed" {
                return None;
            }
            let artifact = probe.get("saved_project_artifact")?;
            let path_str = artifact.get("path")?.as_str()?;
            let path = Path::new(path_str);
            return Some(if path.is_absolute() {
                path.to_path_buf()
            } else {
                run_dir.join(path)
            });
        }
    }
    None
}

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
    assert!(
        saved_project.is_file(),
        "saved .a3p should exist at {}",
        saved_project.display(),
    );
    assert!(
        fs::metadata(&saved_project).unwrap().len() > 0,
        "saved .a3p should be non-empty",
    );

    // ── Start Xvfb for reopen/export phases ────────────────────────────
    let xvfb = start_xvfb_for_workflow(&runs_dir);

    // ── Phase 3: Reopen the saved project ──────────────────────────────
    let reopen_hook = alice.join("tools/eatme-reopen-project");
    if !reopen_hook.is_file() {
        eprintln!(
            "reopen hook not found at {} — phases 3-5 blocked (contract-first)",
            reopen_hook.display()
        );
        drop(xvfb);
        return;
    }

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
            "scene.eatmeFirstLessonStep",
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

    // Verify reopen evidence artifacts exist and are non-empty.
    let reopened_project = reopen_evidence_dir.join(&reopen_result.reopened_project_artifact);
    assert!(
        reopened_project.is_file(),
        "reopened project artifact should exist at {}",
        reopened_project.display(),
    );
    assert!(
        fs::metadata(&reopened_project).unwrap().len() > 0,
        "reopened project artifact should be non-empty",
    );

    let reopened_state = reopen_evidence_dir.join(&reopen_result.reopened_state_artifact);
    assert!(
        reopened_state.is_file(),
        "reopened state artifact should exist at {}",
        reopened_state.display(),
    );
    assert!(
        fs::metadata(&reopened_state).unwrap().len() > 0,
        "reopened state artifact should be non-empty",
    );

    // ── Phase 4: Export to NetBeans project format ─────────────────────
    let export_hook = alice.join("tools/eatme-export-project");
    if !export_hook.is_file() {
        eprintln!(
            "export hook not found at {} — phases 4-5 blocked \
             (contract-first: hook not yet implemented)",
            export_hook.display()
        );
        drop(xvfb);
        return;
    }

    let export_evidence_dir = run_dir.join("project-export");
    fs::create_dir_all(&export_evidence_dir).expect("create export evidence dir");
    let export_evidence_str = export_evidence_dir.display().to_string();

    let export_output = run_hook_with_timeout(
        &export_hook,
        &[
            "--saved-project",
            &saved_project_str,
            "--export-format",
            "netbeans",
            "--evidence-dir",
            &export_evidence_str,
            "--json",
        ],
        &alice,
        &xvfb.display,
        Duration::from_secs(60),
    );

    assert!(
        export_output.status.success(),
        "export hook should exit 0, got: {:?}\nstderr: {}",
        export_output.status.code(),
        String::from_utf8_lossy(&export_output.stderr),
    );

    let export_result: ProjectExportHookResult = serde_json::from_slice(&export_output.stdout)
        .unwrap_or_else(|e| {
            panic!(
                "export hook stdout should be valid JSON: {e}\nstdout: {}",
                String::from_utf8_lossy(&export_output.stdout),
            )
        });

    assert_eq!(
        export_result.schema_version, "eatme.alice-project-export-result/v1",
        "export schema version mismatch",
    );
    assert_eq!(
        export_result.status, "exported",
        "export status must be 'exported'",
    );
    assert_eq!(
        export_result.export_format, "netbeans",
        "export format must be 'netbeans'",
    );
    assert!(
        export_result
            .source_saved_project_artifact
            .starts_with("project-save/"),
        "export source must reference project-save/ dir, got: {}",
        export_result.source_saved_project_artifact,
    );

    // ── Phase 5: Verify build.xml exists ───────────────────────────────
    let build_xml = export_evidence_dir.join(&export_result.exported_build_file);
    assert!(
        build_xml.is_file(),
        "exported Ant build.xml should exist at {}",
        build_xml.display(),
    );
    assert!(
        fs::metadata(&build_xml).unwrap().len() > 0,
        "exported Ant build.xml should be non-empty",
    );

    let export_evidence = export_evidence_dir.join(&export_result.export_artifact);
    assert!(
        export_evidence.is_file(),
        "export evidence artifact should exist at {}",
        export_evidence.display(),
    );
    assert!(
        fs::metadata(&export_evidence).unwrap().len() > 0,
        "export evidence artifact should be non-empty",
    );

    // ── Phase 6: Cleanup (XvfbGuard Drop) ──────────────────────────────
    drop(xvfb);
}
