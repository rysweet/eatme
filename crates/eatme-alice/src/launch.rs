use crate::deps::check_dependencies;
use crate::discover::{AliceDiscovery, discover_alice, first_non_empty};
use crate::package::{PackageOptions, PackageResult, package_alice};
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
        return write_blocked_manifest(
            options,
            &run_dir,
            deps,
            &eatme_commit,
            None,
            None,
            None,
            None,
            "missing_dependency",
            "preflight blocked: one or more required desktop dependencies are unavailable",
            assertions,
        );
    }

    let discovery = match discover_alice(&options.alice_home, &runner) {
        Ok(discovery) => discovery,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                None,
                None,
                None,
                None,
                "alice_discovery_failed",
                format!("preflight blocked: Alice discovery failed: {error:#}"),
                assertions,
            );
        }
    };
    let package = match package_alice(
        PackageOptions {
            alice_home: &options.alice_home,
            offline: options.offline_package,
        },
        &runner,
    ) {
        Ok(package) => package,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                None,
                None,
                None,
                "alice_package_failed",
                format!("preflight blocked: Alice package failed: {error:#}"),
                assertions,
            );
        }
    };

    let display = choose_display();
    let mut xvfb = match start_xvfb(&display, &run_dir) {
        Ok(xvfb) => xvfb,
        Err(error) => {
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                Some(&package),
                Some(&display),
                None,
                "xvfb_start_failed",
                format!("preflight blocked: Xvfb could not start: {error:#}"),
                assertions,
            );
        }
    };
    let display_responsive = wait_for_display(&runner, &display, Duration::from_secs(5));
    assertions.insert(
        "display_responsive".into(),
        bool_assert(
            display_responsive,
            format!("{display} responds to xdpyinfo"),
        ),
    );
    if !display_responsive {
        shutdown(&mut xvfb);
        return write_blocked_manifest(
            options,
            &run_dir,
            deps,
            &eatme_commit,
            Some(&discovery),
            Some(&package),
            Some(&display),
            Some(xvfb.id()),
            "display_unresponsive",
            format!("preflight blocked: {display} did not respond to xdpyinfo"),
            assertions,
        );
    }

    let log_path = run_dir.join("alice.log");
    let launch_args = alice_launch_args(&options.alice_home)?;
    let mut alice = match start_alice(
        &options.alice_home,
        &display,
        &run_dir,
        &log_path,
        &launch_args,
    ) {
        Ok(alice) => alice,
        Err(error) => {
            shutdown(&mut xvfb);
            return write_blocked_manifest(
                options,
                &run_dir,
                deps,
                &eatme_commit,
                Some(&discovery),
                Some(&package),
                Some(&display),
                Some(xvfb.id()),
                "alice_start_failed",
                format!("preflight blocked: Alice process could not start: {error:#}"),
                assertions,
            );
        }
    };
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

    let window_list = capture_window_list(&runner, &display, &run_dir).ok();
    let window_evidence_ok = window_list
        .as_deref()
        .map(|output| !output.trim().is_empty())
        .unwrap_or(false);
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
    let log_ok = log
        .as_ref()
        .map(|artifact| artifact.size_bytes > 0)
        .unwrap_or(false);
    let real_alice_execution_evidence =
        process_started && display_responsive && smoke_ready_visual_evidence && log_ok;
    assertions.insert(
        "real_alice_execution_evidence".into(),
        bool_assert(
            real_alice_execution_evidence,
            "real Alice process, responsive virtual display, visual evidence, and launch log were captured",
        ),
    );
    if !real_alice_execution_evidence && failure_category.is_none() {
        failure_category = Some("real_alice_evidence_missing".into());
    }
    let launch_command = format!("java {}", launch_args.join(" "));
    let manifest = build_manifest(
        options,
        deps,
        &eatme_commit,
        Some(&discovery),
        Some(&package),
        launch_command,
        display.clone(),
        Some(xvfb.id()),
        Some(alice.id()),
        screenshot,
        window_list_artifact,
        log,
        fatal_log_scan,
        assertions,
        failure_category,
    );

    write_manifest(&run_dir, &manifest)?;
    shutdown(&mut alice);
    shutdown(&mut xvfb);
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
fn write_blocked_manifest(
    options: &LaunchSmokeOptions,
    run_dir: &Path,
    deps: crate::deps::DependencyReport,
    eatme_commit: &str,
    discovery: Option<&AliceDiscovery>,
    package: Option<&PackageResult>,
    display: Option<&str>,
    xvfb_pid: Option<u32>,
    category: &str,
    diagnostic: impl Into<String>,
    mut assertions: BTreeMap<String, AssertionResult>,
) -> Result<LaunchSmokeManifest> {
    let diagnostic = diagnostic.into();
    fs::write(run_dir.join("alice.log"), format!("{diagnostic}\n"))?;
    assertions.insert(
        "real_alice_execution_evidence".into(),
        AssertionResult::fail(diagnostic),
    );
    let log = artifact_info(&run_dir.join("alice.log")).ok();
    let manifest = build_manifest(
        options,
        deps,
        eatme_commit,
        discovery,
        package,
        String::new(),
        display.unwrap_or("").to_string(),
        xvfb_pid,
        None,
        None,
        None,
        log,
        Vec::new(),
        assertions,
        Some(category.to_string()),
    );
    write_manifest(run_dir, &manifest)?;
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
fn build_manifest(
    options: &LaunchSmokeOptions,
    deps: crate::deps::DependencyReport,
    eatme_commit: &str,
    discovery: Option<&AliceDiscovery>,
    package: Option<&PackageResult>,
    launch_command: String,
    display: String,
    xvfb_pid: Option<u32>,
    alice_pid: Option<u32>,
    screenshot: Option<ArtifactInfo>,
    window_list: Option<ArtifactInfo>,
    log: Option<ArtifactInfo>,
    fatal_log_scan: Vec<String>,
    assertions: BTreeMap<String, AssertionResult>,
    failure_category: Option<String>,
) -> LaunchSmokeManifest {
    LaunchSmokeManifest {
        schema_version: "eatme.launch-smoke/v1".into(),
        scenario_id: options.scenario.id.clone(),
        run_id: options.run_id.clone(),
        alice_home: discovery
            .map(|value| value.alice_home.clone())
            .unwrap_or_else(|| options.alice_home.display().to_string()),
        alice_git_commit: discovery
            .map(|value| value.git_commit.clone())
            .unwrap_or_else(|| "unknown".into()),
        eatme_git_commit: eatme_commit.to_string(),
        java_version: discovery
            .map(|value| value.java_version.clone())
            .unwrap_or_else(|| "unknown".into()),
        maven_version: discovery
            .map(|value| value.maven_version.clone())
            .unwrap_or_else(|| "unknown".into()),
        dependency_checks: deps.tools,
        build_command: package
            .map(|value| value.command.clone())
            .unwrap_or_default(),
        build_exit_status: package.and_then(|value| value.exit_status),
        launch_command,
        display,
        xvfb_pid,
        alice_pid,
        timeout_seconds: options.timeout_seconds,
        screenshot,
        window_list,
        log,
        fatal_log_scan,
        assertions,
        failure_category,
    }
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
