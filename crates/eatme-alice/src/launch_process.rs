use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub fn start_xvfb(display: &str, run_dir: &Path) -> Result<Child> {
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

pub fn alice_launch_args(alice_home: &Path) -> Result<Vec<String>> {
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
        let jar = std::fs::read_dir(lib_dir)?
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

pub fn start_alice(
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

pub fn wait_for_start(child: &mut Child, seconds: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds.clamp(5, 60));
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return false;
        }
        thread::sleep(Duration::from_millis(500));
    }
    true
}
