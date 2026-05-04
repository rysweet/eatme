use crate::deps::check_dependencies;
use crate::discover::{discover_alice, first_non_empty};
use crate::package::{PackageOptions, package_alice};
use anyhow::{Context, Result, bail};
use eatme_core::{
    ArtifactInfo, AssertionResult, CommandRunner, CommandSpec, LaunchSmokeManifest,
    RealCommandRunner, file_size, sha256_file,
};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct LaunchSmokeScenario {
    pub id: String,
    pub run_dir_name: String,
}

impl LaunchSmokeScenario {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            run_dir_name: id.clone(),
            id,
        }
    }

    pub fn real_alice_launch_smoke() -> Self {
        Self::new("real-alice-launch-smoke")
    }

    pub fn accepts_window_evidence(&self) -> bool {
        self.id != "real-alice-launch-smoke"
    }

    pub fn requires_real_ui_actions(&self) -> bool {
        self.id == "first-lessons-real-ui-actions"
    }
}

impl Default for LaunchSmokeScenario {
    fn default() -> Self {
        Self::real_alice_launch_smoke()
    }
}

#[derive(Clone, Debug)]
pub struct LaunchSmokeOptions {
    pub alice_home: PathBuf,
    pub run_id: String,
    pub runs_dir: PathBuf,
    pub timeout_seconds: u64,
    pub json: bool,
    pub no_memory: bool,
    pub offline_package: bool,
    pub scenario: LaunchSmokeScenario,
}

pub fn run_launch_smoke(options: &LaunchSmokeOptions) -> Result<LaunchSmokeManifest> {
    validate_scenario_name(&options.scenario.id)?;
    validate_scenario_name(&options.scenario.run_dir_name)?;

    let runner = RealCommandRunner;
    let deps = check_dependencies(&runner)?;
    let eatme_commit = git_commit(Path::new("."), &runner).unwrap_or_else(|_| "unknown".into());
    let run_dir = options
        .runs_dir
        .join(&options.scenario.run_dir_name)
        .join(&options.run_id);
    prepare_run_dir(&run_dir)?;

    let mut assertions = BTreeMap::new();
    assertions.insert(
        "dependencies_available".into(),
        bool_assert(
            deps.all_required_available,
            "required desktop tools detected",
        ),
    );
    let mut failure_category = if deps.all_required_available {
        None
    } else {
        Some("missing_dependency".to_string())
    };
    if !deps.all_required_available {
        return write_preflight_blocked_manifest(
            options,
            &run_dir,
            deps.tools,
            eatme_commit,
            "missing_dependency",
            "preflight blocked: one or more required desktop dependencies are unavailable",
            assertions,
        );
    }

    let discovery = discover_alice(&options.alice_home, &runner)?;
    let package = package_alice(
        PackageOptions {
            alice_home: &options.alice_home,
            offline: options.offline_package,
        },
        &runner,
    )?;

    let display = choose_display();
    let mut xvfb = start_xvfb(&display, &run_dir)?;
    let display_responsive = wait_for_display(&runner, &display, Duration::from_secs(5));
    assertions.insert(
        "display_responsive".into(),
        bool_assert(
            display_responsive,
            format!("{display} responds to xdpyinfo"),
        ),
    );
    if !display_responsive {
        failure_category = Some("display_unresponsive".into());
    }

    let log_path = run_dir.join("alice.log");
    let launch_args = alice_launch_args(&options.alice_home)?;
    let mut alice = start_alice(
        &options.alice_home,
        &display,
        &run_dir,
        &log_path,
        &launch_args,
    )?;
    let process_started = wait_for_start(&mut alice, options.timeout_seconds.min(60));
    assertions.insert(
        "process_started".into(),
        bool_assert(
            process_started,
            "Alice process stayed alive through startup wait",
        ),
    );
    if !process_started {
        failure_category = Some("alice_process_exited".into());
    }

    let window_list = capture_window_list(&runner, &display, &run_dir).unwrap_or_default();
    let window_evidence_ok = !window_list.trim().is_empty();
    let specific_alice_window_ok = specific_alice_window_detected(&window_list);
    let window_list_artifact = artifact_info(&run_dir.join("window-list.txt")).ok();
    let screenshot = capture_screenshot(&runner, &display, &run_dir).ok();
    let screenshot_ok = screenshot
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let smoke_ready_visual_evidence =
        screenshot_ok || (options.scenario.accepts_window_evidence() && window_evidence_ok);
    assertions.insert(
        "startup_screenshot".into(),
        bool_assert(
            smoke_ready_visual_evidence,
            if options.scenario.accepts_window_evidence() {
                "startup screenshot exists or window evidence was captured"
            } else {
                "startup screenshot exists and is non-empty"
            },
        ),
    );
    if options.scenario.accepts_window_evidence() {
        assertions.insert(
            "startup_window_or_screenshot".into(),
            bool_assert(
                smoke_ready_visual_evidence,
                "startup screenshot or window evidence exists",
            ),
        );
    }
    if !smoke_ready_visual_evidence && failure_category.is_none() {
        failure_category = Some("screenshot_missing".into());
    }
    if options.scenario.requires_real_ui_actions() {
        assertions.insert(
            "specific_alice_window_detected".into(),
            bool_assert(
                specific_alice_window_ok,
                "wmctrl window list contains an Alice Stage IDE window",
            ),
        );
        if !specific_alice_window_ok && failure_category.is_none() {
            failure_category = Some("alice_window_not_detected".into());
        }
    }

    let fatal_log_scan = scan_fatal_logs(&log_path);
    assertions.insert(
        "no_fatal_logs".into(),
        bool_assert(
            fatal_log_scan.is_empty(),
            format!("{} fatal log lines found", fatal_log_scan.len()),
        ),
    );
    if !fatal_log_scan.is_empty() && failure_category.is_none() {
        failure_category = Some("fatal_log".into());
    }

    let log = artifact_info(&log_path).ok();
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let artifact = write_ui_action_contract(
            &run_dir,
            specific_alice_window_ok,
            smoke_ready_visual_evidence,
            log.as_ref()
                .map(|artifact| artifact.size_bytes > 0)
                .unwrap_or(false),
        )?;
        assertions.insert(
            "place_object_ui_action".into(),
            AssertionResult::fail(
                "blocked: no supported Alice desktop automation can add/place an object yet",
            ),
        );
        assertions.insert(
            "edit_procedure_ui_action".into(),
            AssertionResult::fail(
                "blocked: no supported Alice desktop automation can edit a procedure or code block yet",
            ),
        );
        assertions.insert(
            "run_world_ui_action".into(),
            AssertionResult::fail(
                "blocked: no supported Alice desktop automation can run the world yet",
            ),
        );
        assertions.insert(
            "save_project_ui_action".into(),
            AssertionResult::fail(
                "blocked: no supported Alice desktop automation can save the project yet",
            ),
        );
        assertions.insert(
            "ui_action_artifact_captured".into(),
            bool_assert(
                artifact.size_bytes > 0,
                "ui action contract artifact exists and is non-empty",
            ),
        );
        if failure_category.is_none() {
            failure_category = Some("ui_action_automation_unimplemented".into());
        }
        Some(artifact)
    } else {
        None
    };
    let launch_command = format!("java {}", launch_args.join(" "));
    let manifest = LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: discovery.alice_home,
        alice_git_commit: discovery.git_commit,
        eatme_git_commit: eatme_commit,
        java_version: discovery.java_version,
        maven_version: discovery.maven_version,
        dependency_checks: deps.tools,
        build_command: package.command,
        build_exit_status: package.exit_status,
        launch_command,
        display: display.clone(),
        xvfb_pid: Some(xvfb.id()),
        alice_pid: Some(alice.id()),
        timeout_seconds: options.timeout_seconds,
        screenshot,
        window_list: window_list_artifact,
        ui_action_contract,
        log,
        fatal_log_scan,
        assertions,
        failure_category,
    };

    write_manifest(&run_dir, &manifest)?;
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    Ok(manifest)
}

fn prepare_run_dir(run_dir: &Path) -> Result<()> {
    if run_dir.exists() {
        fs::remove_dir_all(run_dir).with_context(|| format!("removing {}", run_dir.display()))?;
    }
    fs::create_dir_all(run_dir.join("screenshots"))
        .with_context(|| format!("creating {}", run_dir.display()))?;
    fs::create_dir_all(run_dir.join("home"))?;
    fs::create_dir_all(run_dir.join("prefs"))?;
    fs::create_dir_all(run_dir.join("tmp"))?;
    Ok(())
}

fn write_preflight_blocked_manifest(
    options: &LaunchSmokeOptions,
    run_dir: &Path,
    dependency_checks: BTreeMap<String, bool>,
    eatme_commit: String,
    failure_category: &str,
    detail: &str,
    mut assertions: BTreeMap<String, AssertionResult>,
) -> Result<LaunchSmokeManifest> {
    let log_path = run_dir.join("alice.log");
    fs::write(&log_path, format!("{detail}\n"))?;
    if options.scenario.requires_real_ui_actions() {
        assertions.insert(
            "specific_alice_window_detected".into(),
            AssertionResult::fail("preflight blocked before an Alice window could be verified"),
        );
        assertions.insert(
            "place_object_ui_action".into(),
            AssertionResult::fail("preflight blocked before add/place object automation could run"),
        );
        assertions.insert(
            "edit_procedure_ui_action".into(),
            AssertionResult::fail(
                "preflight blocked before procedure/code-block editing could run",
            ),
        );
        assertions.insert(
            "run_world_ui_action".into(),
            AssertionResult::fail("preflight blocked before world execution could run"),
        );
        assertions.insert(
            "save_project_ui_action".into(),
            AssertionResult::fail("preflight blocked before project save could run"),
        );
    }
    let log = artifact_info(&log_path).ok();
    let ui_action_contract = if options.scenario.requires_real_ui_actions() {
        let artifact = write_ui_action_contract(run_dir, false, false, log.is_some())?;
        assertions.insert(
            "ui_action_artifact_captured".into(),
            bool_assert(
                artifact.size_bytes > 0,
                "ui action contract artifact exists and is non-empty",
            ),
        );
        Some(artifact)
    } else {
        None
    };
    let manifest = LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: options.alice_home.display().to_string(),
        alice_git_commit: "unknown".into(),
        eatme_git_commit: eatme_commit,
        java_version: "unknown".into(),
        maven_version: "unknown".into(),
        dependency_checks,
        build_command: String::new(),
        build_exit_status: None,
        launch_command: String::new(),
        display: String::new(),
        xvfb_pid: None,
        alice_pid: None,
        timeout_seconds: options.timeout_seconds,
        screenshot: None,
        window_list: None,
        ui_action_contract,
        log,
        fatal_log_scan: vec![detail.to_string()],
        assertions,
        failure_category: Some(failure_category.to_string()),
    };
    write_manifest(run_dir, &manifest)?;
    Ok(manifest)
}

fn validate_scenario_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('-')
        || name.ends_with('-')
        || !name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("launch smoke scenario {name:?} must be kebab-case");
    }
    Ok(())
}

fn choose_display() -> String {
    for display in 90..130 {
        let socket = format!("/tmp/.X11-unix/X{display}");
        if !Path::new(&socket).exists() {
            return format!(":{display}");
        }
    }
    ":99".into()
}

fn specific_alice_window_detected(window_list: &str) -> bool {
    window_list.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.contains("org.alice.stageide") || lower.contains("alice")
    })
}

fn start_xvfb(display: &str, run_dir: &Path) -> Result<Child> {
    let log = File::create(run_dir.join("xvfb.log"))?;
    Command::new("Xvfb")
        .args([
            display,
            "-screen",
            "0",
            "1280x900x24",
            "+extension",
            "GLX",
            "+render",
            "-noreset",
        ])
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
        .with_context(|| format!("starting Xvfb {display}"))
}

fn alice_launch_args(alice_home: &Path) -> Result<Vec<String>> {
    let fxmp = javafx_module_path(&alice_home.join("alice-ide/target/lib"))?;
    Ok(vec![
        "-ea".into(),
        "-Xmx1024m".into(),
        "-Dorg.alice.ide.rootDirectory=./core/resources/target/distribution".into(),
        "-Dedu.cmu.cs.dennisc.java.util.logging.Logger.Level=WARNING".into(),
        "-Dorg.alice.ide.internalTesting=true".into(),
        "-Dorg.lgna.croquet.Element.isIdCheckDesired=true".into(),
        "-Djogamp.gluegen.UseTempJarCache=false".into(),
        "-Dorg.alice.stageide.isCrashDetectionDesired=false".into(),
        "--add-opens=java.base/java.io=ALL-UNNAMED".into(),
        "--add-opens=java.desktop/sun.awt=ALL-UNNAMED".into(),
        "--add-opens=java.base/java.time=ALL-UNNAMED".into(),
        "--module-path".into(),
        fxmp,
        "--add-modules".into(),
        "javafx.graphics,javafx.media".into(),
        "-cp".into(),
        "alice-ide/target/alice-ide-9.1.0-SNAPSHOT.jar:alice-ide/target/lib/*".into(),
        "org.alice.stageide.EntryPoint".into(),
        "core/resources/target/distribution/application/starter-projects/africa.a3p".into(),
        "0".into(),
        "0".into(),
        "1000".into(),
        "740".into(),
    ])
}

fn javafx_module_path(lib_dir: &Path) -> Result<String> {
    let required = ["javafx-base", "javafx-graphics", "javafx-media"];
    let mut jars = Vec::new();
    for prefix in required {
        let jar = fs::read_dir(lib_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(prefix) && name.ends_with("-linux.jar"))
                    .unwrap_or(false)
            })
            .with_context(|| format!("missing {prefix} linux jar in {}", lib_dir.display()))?;
        jars.push(jar.display().to_string());
    }
    Ok(jars.join(":"))
}

fn start_alice(
    alice_home: &Path,
    display: &str,
    run_dir: &Path,
    log_path: &Path,
    args: &[String],
) -> Result<Child> {
    let log = File::create(log_path)?;
    let mut command = Command::new("java");
    command
        .current_dir(alice_home)
        .env("DISPLAY", display)
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("HOME", run_dir.join("home"))
        .env("TMPDIR", run_dir.join("tmp"))
        .arg(format!("-Duser.home={}", run_dir.join("home").display()))
        .arg(format!(
            "-Djava.util.prefs.userRoot={}",
            run_dir.join("prefs").display()
        ))
        .arg(format!(
            "-Djava.io.tmpdir={}",
            run_dir.join("tmp").display()
        ))
        .args(args)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log));
    command.spawn().context("starting Alice")
}

fn wait_for_start(child: &mut Child, seconds: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds.clamp(5, 60));
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return false;
        }
        thread::sleep(Duration::from_millis(500));
    }
    true
}

fn wait_for_display(runner: &impl CommandRunner, display: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if command_ok(
            runner,
            CommandSpec::new("xdpyinfo")
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(2))
                .retries(2, Duration::from_millis(100)),
        ) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

fn capture_window_list(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<String> {
    let output = runner.run(
        &CommandSpec::new("wmctrl")
            .args(["-lx"])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if output.exit_status != Some(0) {
        bail!(
            "capturing window list failed with {:?}\n{}{}",
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    let combined = first_non_empty(&output.stdout, &output.stderr);
    fs::write(run_dir.join("window-list.txt"), &combined)?;
    Ok(combined)
}

fn capture_screenshot(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("screenshots/startup.png");
    let scrot = CommandSpec::new("scrot")
        .args([path.display().to_string()])
        .env("DISPLAY", display)
        .timeout(Duration::from_secs(10))
        .retries(2, Duration::from_millis(100));
    let output = runner.run(&scrot)?;
    if output.exit_status != Some(0) {
        let fallback = runner.run(
            &CommandSpec::new("import")
                .args(["-window".into(), "root".into(), path.display().to_string()])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(10))
                .retries(2, Duration::from_millis(100)),
        )?;
        if fallback.exit_status != Some(0) {
            bail!(
                "capturing startup screenshot failed: scrot={:?}, import={:?}\nscrot stdout:\n{}scrot stderr:\n{}import stdout:\n{}import stderr:\n{}",
                output.exit_status,
                fallback.exit_status,
                output.stdout,
                output.stderr,
                fallback.stdout,
                fallback.stderr
            );
        }
    }
    artifact_info(&path).with_context(|| format!("capturing screenshot {}", path.display()))
}

fn artifact_info(path: &Path) -> Result<ArtifactInfo> {
    Ok(ArtifactInfo {
        path: path.display().to_string(),
        size_bytes: file_size(path)?,
        sha256: sha256_file(path)?,
    })
}

fn write_ui_action_contract(
    run_dir: &Path,
    specific_alice_window_detected: bool,
    visual_evidence_captured: bool,
    log_captured: bool,
) -> Result<ArtifactInfo> {
    let path = run_dir.join("ui-action-contract.json");
    let json = serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "No supported deterministic Alice desktop automation is wired for object placement, procedure editing, world run, or project save yet.",
        "preflight_evidence": {
            "specific_alice_window_detected": specific_alice_window_detected,
            "visual_evidence_captured": visual_evidence_captured,
            "log_captured": log_captured
        },
        "required_actions": [
            {
                "id": "verify-specific-alice-window",
                "required_evidence": "wmctrl output identifies an Alice Stage IDE window"
            },
            {
                "id": "place-object",
                "required_evidence": "artifact proves an object was added to the scene and placed"
            },
            {
                "id": "edit-procedure-or-code-block",
                "required_evidence": "artifact proves a procedure or code block was edited"
            },
            {
                "id": "run-world",
                "required_evidence": "artifact proves the world run control was invoked"
            },
            {
                "id": "save-project",
                "required_evidence": "saved .a3p project artifact exists and is non-empty"
            }
        ]
    });
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    artifact_info(&path)
}

fn scan_fatal_logs(log_path: &Path) -> Vec<String> {
    let content = fs::read_to_string(log_path).unwrap_or_default();
    let patterns = [
        "Unable to open DISPLAY",
        "No X11 DISPLAY",
        "SEVERE",
        "Exception in thread",
        "HeadlessException",
        "GLException",
    ];
    content
        .lines()
        .filter(|line| patterns.iter().any(|pattern| line.contains(pattern)))
        .map(str::to_string)
        .collect()
}

fn command_ok(runner: &impl CommandRunner, spec: CommandSpec) -> bool {
    runner
        .run(&spec)
        .map(|output| output.exit_status == Some(0))
        .unwrap_or(false)
}

fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}

fn git_commit(path: &Path, runner: &impl CommandRunner) -> Result<String> {
    let output = runner.run(
        &CommandSpec::new("git")
            .args(["rev-parse", "HEAD"])
            .cwd(path)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    )?;
    if output.exit_status != Some(0) {
        bail!(
            "reading git commit in {} failed with {:?}\n{}{}",
            path.display(),
            output.exit_status,
            output.stdout,
            output.stderr
        );
    }
    Ok(output.stdout.trim().to_string())
}

fn write_manifest(run_dir: &Path, manifest: &LaunchSmokeManifest) -> Result<()> {
    let path = run_dir.join("manifest.json");
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json)?;
    Ok(())
}

fn shutdown(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_non_default_display_format() {
        assert!(choose_display().starts_with(':'));
    }

    #[test]
    fn rejects_non_kebab_case_scenario_names() {
        assert!(validate_scenario_name("../bad").is_err());
        assert!(validate_scenario_name("building-a-scene-first-world").is_ok());
    }
}
