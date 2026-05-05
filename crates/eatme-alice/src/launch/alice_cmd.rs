use anyhow::{Context, Result, anyhow, bail};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

pub(super) fn alice_launch_args(alice_home: &Path, starter_project: &Path) -> Result<Vec<String>> {
    let target_dir = alice_home.join("alice-ide/target");
    let lib_dir = target_dir.join("lib");
    let fxmp = javafx_module_path(&lib_dir)?;
    let classpath = alice_classpath(&target_dir)?;
    let starter = starter_project_arg(alice_home, starter_project)?;

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
        classpath,
        "org.alice.stageide.EntryPoint".into(),
        starter,
        "0".into(),
        "0".into(),
        "1000".into(),
        "740".into(),
    ])
}

pub(super) fn start_alice(
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

fn javafx_module_path(lib_dir: &Path) -> Result<String> {
    let jars = read_jars(lib_dir)?;
    let platform = javafx_platform_classifier();
    let mut selected = Vec::new();
    for prefix in ["javafx-base", "javafx-graphics", "javafx-media"] {
        let jar = select_javafx_jar(&jars, prefix, platform)
            .with_context(|| format!("missing {prefix} jar in {}", lib_dir.display()))?;
        selected.push(relative_lib_jar(&jar)?);
    }
    path_list(&selected).context("building JavaFX module path")
}

fn alice_classpath(target_dir: &Path) -> Result<String> {
    let alice_jar = relative_target_jar(&find_alice_ide_jar(target_dir)?)?;
    path_list(&[alice_jar, relative_lib_wildcard()]).context("building classpath")
}

fn starter_project_arg(alice_home: &Path, starter_project: &Path) -> Result<String> {
    let starter_path = if starter_project.is_absolute() {
        starter_project.to_path_buf()
    } else {
        alice_home.join(starter_project)
    };
    if !starter_path.exists() {
        bail!("starter project {} does not exist", starter_path.display());
    }
    Ok(starter_project.display().to_string())
}

fn find_alice_ide_jar(target_dir: &Path) -> Result<PathBuf> {
    let mut candidates = read_jars(target_dir)?
        .into_iter()
        .filter(|path| {
            file_name(path)
                .map(|name| {
                    name.starts_with("alice-ide-")
                        && name.ends_with(".jar")
                        && !name.contains("-sources")
                        && !name.contains("-javadoc")
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        file_name(left)
            .unwrap_or_default()
            .len()
            .cmp(&file_name(right).unwrap_or_default().len())
            .then_with(|| left.cmp(right))
    });
    candidates
        .into_iter()
        .next()
        .with_context(|| format!("missing alice-ide jar in {}", target_dir.display()))
}

fn select_javafx_jar(jars: &[PathBuf], prefix: &str, platform: &str) -> Option<PathBuf> {
    let mut candidates = jars
        .iter()
        .filter(|path| {
            file_name(path)
                .map(|name| {
                    name.starts_with(prefix)
                        && name.ends_with(".jar")
                        && !name.contains("-sources")
                        && !name.contains("-javadoc")
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort();

    candidates
        .iter()
        .find(|path| {
            file_name(path)
                .map(|name| name.ends_with(&format!("-{platform}.jar")))
                .unwrap_or(false)
        })
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

fn read_jars(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut jars = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry
            .with_context(|| format!("reading entry in {}", dir.display()))?
            .path();
        if is_jar_path(&path) {
            jars.push(path);
        }
    }
    Ok(jars)
}

fn is_jar_path(path: &Path) -> bool {
    file_name(path)
        .map(|name| name.ends_with(".jar"))
        .unwrap_or(false)
}

fn path_list(paths: &[PathBuf]) -> Result<String> {
    env::join_paths(paths)
        .map_err(|err| anyhow!("joining Java paths failed: {err}"))
        .and_then(os_string_into_string)
}

fn os_string_into_string(value: OsString) -> Result<String> {
    value
        .into_string()
        .map_err(|value| anyhow!("Java path contains non-UTF-8 data: {:?}", value))
}

fn javafx_platform_classifier() -> &'static str {
    match env::consts::OS {
        "macos" => "mac",
        "windows" => "win",
        _ => "linux",
    }
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

fn relative_target_jar(path: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from("alice-ide/target").join(
        path.file_name()
            .with_context(|| format!("{} has no file name", path.display()))?,
    ))
}

fn relative_lib_jar(path: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from("alice-ide/target/lib").join(
        path.file_name()
            .with_context(|| format!("{} has no file name", path.display()))?,
    ))
}

fn relative_lib_wildcard() -> PathBuf {
    PathBuf::from("alice-ide/target/lib/*")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_launch_args_from_discovered_artifacts_and_custom_starter() {
        let root = unique_test_dir("alice-cmd");
        let target = root.join("alice-ide/target");
        let lib = target.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(target.join("alice-ide-10.0.0.jar"), "jar").unwrap();
        for prefix in ["javafx-base", "javafx-graphics", "javafx-media"] {
            fs::write(
                lib.join(format!("{prefix}-22-{}.jar", javafx_platform_classifier())),
                "jar",
            )
            .unwrap();
        }
        fs::create_dir_all(root.join("custom")).unwrap();
        fs::write(root.join("custom/lesson.a3p"), "project").unwrap();

        let args = alice_launch_args(&root, Path::new("custom/lesson.a3p")).unwrap();

        assert!(
            args.iter()
                .any(|arg| arg.contains("alice-ide/target/alice-ide-10.0.0.jar"))
        );
        assert!(args.iter().any(|arg| arg == "custom/lesson.a3p"));
        assert!(args.iter().any(|arg| arg.contains(&format!(
            "alice-ide/target/lib/javafx-base-22-{}.jar",
            javafx_platform_classifier()
        ))));
        let _ = fs::remove_dir_all(root);
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::current_dir()
            .unwrap()
            .join("target")
            .join("eatme-alice-tests")
            .join(format!("{prefix}-{nonce}"))
    }
}
