//! Shared infrastructure for the import/export workflow integration tests.
//!
//! Contains Xvfb management, hook runner, evidence validation helpers,
//! typed deserialization structs for hook JSON contracts, and the export
//! probe constructor.

use serde::Deserialize;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Inline typed deserialization structs — define the hook contracts
// ---------------------------------------------------------------------------

/// JSON contract for `tools/eatme-reopen-project --json` output.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ProjectReopenHookResult {
    pub schema_version: String,
    pub status: String,
    pub source_saved_project_artifact: String,
    pub reopen_selector: String,
    pub reopened_project_artifact: String,
    pub reopen_artifact: String,
    pub reopened_state_artifact: String,
    pub state_verification: String,
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

/// Export probe struct matching the save/reopen probe pattern.
///
/// Constructed inline after running the export hook. `proves_export()` returns
/// `true` only when the hook succeeded, the JSON contract is valid, and the
/// exported `build.xml` exists as a non-empty file on disk.
#[allow(dead_code)]
pub struct UiActionExportProjectProbe {
    pub id: String,
    pub action_id: String,
    pub status: String,
    pub detail: String,
    pub export_format: String,
    pub candidate_hook_path: String,
    pub command: Option<String>,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub source_saved_project_artifact: String,
    pub exported_build_file: Option<PathBuf>,
    pub export_artifact: Option<PathBuf>,
    pub validation_errors: Vec<String>,
}

impl UiActionExportProjectProbe {
    pub fn proves_export(&self) -> bool {
        self.status == "passed"
            && !self.source_saved_project_artifact.is_empty()
            && self.exported_build_file.is_some()
            && self.export_artifact.is_some()
            && self.validation_errors.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Xvfb display management with Drop-based cleanup
// ---------------------------------------------------------------------------

pub struct XvfbGuard {
    child: Child,
    pub display: String,
    lock_path: PathBuf,
}

impl Drop for XvfbGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.lock_path);
    }
}

pub fn start_xvfb_for_workflow(runs_dir: &Path) -> XvfbGuard {
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
                let child = match Command::new("Xvfb")
                    .args([&display, "-screen", "0", "1280x1024x24", "-ac"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = fs::remove_file(&lock_path);
                        panic!("start Xvfb on {display}: {e}");
                    }
                };

                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    if Instant::now() > deadline {
                        let guard = XvfbGuard {
                            child,
                            display: display.clone(),
                            lock_path: lock_path.clone(),
                        };
                        drop(guard);
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

pub fn run_hook_with_timeout(
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
pub fn extract_saved_project_path(
    ui_action_contract_path: &Path,
    run_dir: &Path,
) -> Option<PathBuf> {
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
// Path safety + artifact validation helpers
// ---------------------------------------------------------------------------

/// Rejects absolute paths, parent traversal (`..`), and empty strings.
fn is_safe_relative_path(p: &str) -> bool {
    !p.is_empty() && !Path::new(p).is_absolute() && !p.split(['/', '\\']).any(|seg| seg == "..")
}

/// Validates an evidence artifact: path must be a safe relative path,
/// file must exist and be non-empty.
pub fn validate_evidence_artifact(
    dir: &Path,
    relative: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<PathBuf> {
    if !is_safe_relative_path(relative) {
        errors.push(format!(
            "{label} has unsafe path (absolute or parent traversal): {relative}"
        ));
        return None;
    }
    let path = dir.join(relative);
    match fs::metadata(&path) {
        Ok(m) if m.is_file() && m.len() > 0 => Some(path),
        Ok(m) if !m.is_file() => {
            errors.push(format!(
                "{label} at {} is not a regular file",
                path.display()
            ));
            None
        }
        Ok(_) => {
            errors.push(format!("{label} at {} is empty", path.display()));
            None
        }
        Err(_) => {
            errors.push(format!("{label} not found at {}", path.display()));
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Export probe constructor
// ---------------------------------------------------------------------------

/// Runs the export hook and returns a validated `UiActionExportProjectProbe`.
pub fn probe_export_hook(
    alice_home: &Path,
    saved_project: &Path,
    export_evidence_dir: &Path,
    display: &str,
) -> UiActionExportProjectProbe {
    let export_hook = alice_home.join("tools/eatme-export-project");
    let hook_path_str = export_hook.display().to_string();

    if !export_hook.is_file() {
        return UiActionExportProjectProbe {
            id: "alice-side-project-export-command-hook".into(),
            action_id: "export-project".into(),
            status: "blocked".into(),
            detail: format!(
                "blocked: Alice checkout does not expose tools/eatme-export-project at {}",
                export_hook.display()
            ),
            export_format: "netbeans".into(),
            candidate_hook_path: hook_path_str,
            command: None,
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
            source_saved_project_artifact: String::new(),
            exported_build_file: None,
            export_artifact: None,
            validation_errors: vec![format!(
                "export hook not found at {}",
                export_hook.display()
            )],
        };
    }

    fs::create_dir_all(export_evidence_dir).expect("create export evidence dir");
    let saved_project_str = saved_project.display().to_string();
    let evidence_str = export_evidence_dir.display().to_string();

    let output = run_hook_with_timeout(
        &export_hook,
        &[
            "--saved-project",
            &saved_project_str,
            "--export-format",
            "netbeans",
            "--evidence-dir",
            &evidence_str,
            "--json",
        ],
        alice_home,
        display,
        Duration::from_secs(60),
    );

    let stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code();
    let mut validation_errors = Vec::new();

    if !output.status.success() {
        validation_errors.push(format!("export hook exited with {exit_code:?}"));
    }

    let result: Option<ProjectExportHookResult> = serde_json::from_slice(&output.stdout)
        .map_err(|e| {
            validation_errors.push(format!("export hook stdout is not valid JSON: {e}"));
        })
        .ok();

    let mut source = String::new();
    let mut exported_build_file: Option<PathBuf> = None;
    let mut export_artifact: Option<PathBuf> = None;

    if let Some(ref r) = result {
        if r.schema_version != "eatme.alice-project-export-result/v1" {
            validation_errors.push(format!(
                "schema_version must be eatme.alice-project-export-result/v1, got {:?}",
                r.schema_version
            ));
        }
        if r.status != "exported" {
            validation_errors.push(format!("status must be exported, got {:?}", r.status));
        }
        if r.export_format != "netbeans" {
            validation_errors.push(format!(
                "export_format must be netbeans, got {:?}",
                r.export_format
            ));
        }
        if !r.source_saved_project_artifact.starts_with("project-save/")
            || !is_safe_relative_path(&r.source_saved_project_artifact)
        {
            validation_errors.push(format!(
                "source must reference project-save/ dir, got: {}",
                r.source_saved_project_artifact
            ));
        }
        source = r.source_saved_project_artifact.clone();

        exported_build_file = validate_evidence_artifact(
            export_evidence_dir,
            &r.exported_build_file,
            "exported build.xml",
            &mut validation_errors,
        );
        export_artifact = validate_evidence_artifact(
            export_evidence_dir,
            &r.export_artifact,
            "export_artifact",
            &mut validation_errors,
        );
    }

    let status = if validation_errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let detail = if validation_errors.is_empty() {
        "export hook produced valid NetBeans project with build.xml".into()
    } else {
        format!(
            "export hook did not prove export: {}",
            validation_errors.join("; ")
        )
    };

    UiActionExportProjectProbe {
        id: "alice-side-project-export-command-hook".into(),
        action_id: "export-project".into(),
        status: status.into(),
        detail,
        export_format: "netbeans".into(),
        candidate_hook_path: hook_path_str,
        command: Some(format!(
            "{} --saved-project {} --export-format netbeans --evidence-dir {} --json",
            export_hook.display(),
            saved_project_str,
            evidence_str
        )),
        exit_status: exit_code,
        stdout: stdout_str,
        stderr: stderr_str,
        source_saved_project_artifact: source,
        exported_build_file,
        export_artifact,
        validation_errors,
    }
}
