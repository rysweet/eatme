use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub struct PathOverride<'a> {
    _guard: MutexGuard<'a, ()>,
    old_path: Option<OsString>,
}

impl<'a> PathOverride<'a> {
    pub fn prepend(bin: &Path) -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let old_path = env::var_os("PATH");
        let mut entries = vec![bin.to_path_buf()];
        if let Some(path) = &old_path {
            entries.extend(env::split_paths(path));
        }
        let new_path = env::join_paths(entries).unwrap();

        unsafe {
            // SAFETY: PATH is process-global. This guard holds ENV_LOCK for the
            // whole fake-toolchain run and restores the original value on drop.
            env::set_var("PATH", new_path);
        }

        Self {
            _guard: guard,
            old_path,
        }
    }

    pub fn replace(bin: &Path) -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let old_path = env::var_os("PATH");

        unsafe {
            // SAFETY: PATH is process-global. This guard holds ENV_LOCK for the
            // whole fake-toolchain run and restores the original value on drop.
            env::set_var("PATH", bin);
        }

        Self {
            _guard: guard,
            old_path,
        }
    }
}

impl Drop for PathOverride<'_> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: ENV_LOCK is still held until this drop completes.
            if let Some(old_path) = &self.old_path {
                env::set_var("PATH", old_path);
            } else {
                env::remove_var("PATH");
            }
        }
    }
}

pub struct TestFixture {
    pub root: PathBuf,
    pub bin: PathBuf,
    pub alice_home: PathBuf,
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture {
    pub fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::current_dir()
            .unwrap()
            .join("target/test-work/launch-smoke")
            .join(format!("{nonce}"));
        let bin = root.join("bin");
        let alice_home = root.join("alice");
        fs::create_dir_all(&bin).unwrap();
        Self {
            root,
            bin,
            alice_home,
        }
    }

    pub fn write_fake_tools(&self) {
        self.write_tool(
            "git",
            r#"#!/bin/sh
echo abcdef1234567890
"#,
        );
        self.write_tool(
            "mvn",
            r#"#!/bin/sh
if [ "$1" = "-version" ]; then
  echo "Apache Maven 3.9.0"
fi
exit 0
"#,
        );
        self.write_tool(
            "java",
            r#"#!/bin/sh
if [ "$1" = "-version" ]; then
  echo "openjdk version 21" 1>&2
  exit 0
fi
echo "fake Alice startup"
sleep 30
"#,
        );
        self.write_tool(
            "Xvfb",
            r#"#!/bin/sh
sleep 30
"#,
        );
        self.write_tool(
            "xdpyinfo",
            r#"#!/bin/sh
echo "display ready"
"#,
        );
        self.write_tool(
            "wmctrl",
            r#"#!/bin/sh
if [ "$1" = "-ia" ]; then
  exit 0
fi
echo "0x001 Alice org.alice.stageide.EntryPoint"
"#,
        );
        self.write_tool(
            "xwininfo",
            r#"#!/bin/sh
cat <<'OUT'
xwininfo: Window id: 0x21f (the root window) (has no name)

  Root window id: 0x21f (the root window) (has no name)
  Parent window id: 0x0 (none)
     1 child:
     0x001 "Alice 3 ": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")  1000x740+0+0  +0+0
OUT
"#,
        );
        self.write_tool(
            "xdotool",
            r#"#!/bin/sh
if [ "$1" = "windowfocus" ]; then
  exit 0
fi
if [ "$1" = "key" ]; then
  exit 0
fi
exit 1
"#,
        );
        self.write_tool(
            "scrot",
            r#"#!/bin/sh
echo screenshot > "$1"
"#,
        );
        self.write_tool(
            "import",
            r#"#!/bin/sh
echo screenshot > "$3"
"#,
        );
        self.write_tool(
            "glxinfo",
            r#"#!/bin/sh
echo "OpenGL renderer string: llvmpipe"
"#,
        );
    }

    pub fn write_failing_screenshot_tools(&self) {
        self.write_tool(
            "scrot",
            r#"#!/bin/sh
exit 1
"#,
        );
        self.write_tool(
            "import",
            r#"#!/bin/sh
exit 1
"#,
        );
    }

    pub fn write_unrelated_window_tool(&self) {
        self.write_tool(
            "wmctrl",
            r#"#!/bin/sh
echo "0x001 unrelated.firefox.Firefox Firefox"
"#,
        );
    }

    #[allow(dead_code)]
    pub fn write_alice_like_license_window_tool(&self) {
        self.write_tool(
            "wmctrl",
            r#"#!/bin/sh
echo '0x002  0 host sun-awt-X11-XDialogPeer License Agreement (Part 1 of 2): Alice 3'
"#,
        );
    }

    #[allow(dead_code)]
    pub fn write_unsupported_activation_tools(&self) {
        self.write_tool(
            "wmctrl",
            r#"#!/bin/sh
if [ "$1" = "-ia" ]; then
  echo "Your windowmanager claims not to support _NET_ACTIVE_WINDOW" 1>&2
  exit 1
fi
echo "0x001  0 host org.alice.stageide.EntryPoint Alice 3"
"#,
        );
        self.write_tool(
            "xdotool",
            r#"#!/bin/sh
if [ "$1" = "windowfocus" ]; then
  echo "XGetWindowProperty[_NET_ACTIVE_WINDOW] failed" 1>&2
  exit 1
fi
exit 1
"#,
        );
    }

    pub fn write_window_managerless_alice_tools(&self) {
        self.write_tool(
            "wmctrl",
            r#"#!/bin/sh
if [ "$1" = "-lx" ]; then
  echo "Cannot get client list properties." 1>&2
  exit 1
fi
if [ "$1" = "-ia" ]; then
  echo "Your windowmanager claims not to support _NET_ACTIVE_WINDOW" 1>&2
  exit 1
fi
exit 1
"#,
        );
        self.write_tool(
            "xwininfo",
            r#"#!/bin/sh
cat <<'OUT'
xwininfo: Window id: 0x21f (the root window) (has no name)

  Root window id: 0x21f (the root window) (has no name)
  Parent window id: 0x0 (none)
     1 child:
     0x600007 "Alice 3 ": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")  1000x740+0+0  +0+0
OUT
"#,
        );
        self.write_tool(
            "xdotool",
            r#"#!/bin/sh
if [ "$1" = "windowfocus" ] && [ "$2" = "0x600007" ]; then
  exit 0
fi
exit 1
"#,
        );
    }

    pub fn write_missing_xvfb_probe(&self) {
        self.write_tool(
            "sh",
            r#"#!/bin/sh
if [ "$1" = "-c" ]; then
  case "$2" in
    "command -v Xvfb") exit 1 ;;
    "command -v "*) echo "/fake/${2#command -v }"; exit 0 ;;
  esac
fi
exec /bin/sh "$@"
"#,
        );
    }

    pub fn write_failing_package_tool(&self) {
        self.write_tool(
            "mvn",
            r#"#!/bin/sh
if [ "$1" = "-version" ]; then
  echo "Apache Maven 3.9.0"
  exit 0
fi
echo "intentional package failure" 1>&2
exit 42
"#,
        );
    }

    pub fn write_missing_log_java_tool(&self) {
        self.write_tool(
            "java",
            r#"#!/bin/sh
if [ "$1" = "-version" ]; then
  echo "openjdk version 21" 1>&2
  exit 0
fi
log_path="$(readlink /proc/$$/fd/1 2>/dev/null || true)"
if [ -n "$log_path" ]; then
  rm -f "$log_path"
fi
sleep 30
"#,
        );
    }

    fn write_tool(&self, name: &str, script: &str) {
        let path = self.bin.join(name);
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    pub fn write_fake_alice_repo(&self) {
        fs::create_dir_all(&self.alice_home).unwrap();
        fs::write(self.alice_home.join("pom.xml"), "<project/>").unwrap();
        let lib = self.alice_home.join("alice-ide/target/lib");
        fs::create_dir_all(&lib).unwrap();
        for jar in [
            "javafx-base-21-linux.jar",
            "javafx-graphics-21-linux.jar",
            "javafx-media-21-linux.jar",
        ] {
            fs::write(lib.join(jar), "jar").unwrap();
        }
        fs::write(
            self.alice_home
                .join("alice-ide/target/alice-ide-9.1.0-SNAPSHOT.jar"),
            "jar",
        )
        .unwrap();
        let starter = self
            .alice_home
            .join("core/resources/target/distribution/application/starter-projects");
        fs::create_dir_all(&starter).unwrap();
        fs::write(starter.join("africa.a3p"), "project").unwrap();
    }

    pub fn write_fake_object_placement_hook(&self) {
        let tools = self.alice_home.join("tools");
        fs::create_dir_all(&tools).unwrap();
        self.write_tool_at(
            &tools.join("eatme-place-object"),
            r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --evidence-dir) shift; evidence_dir="$1" ;;
  esac
  shift
done
mkdir -p "$evidence_dir"
echo '{"placed":true}' > "$evidence_dir/placement.json"
echo '{"added":["bunny"]}' > "$evidence_dir/scene.diff.json"
printf '%s\n' '{"schema_version":"eatme.alice-object-placement-result/v1","status":"placed","object_identifier":"alice-gallery://animals/bunny","placement_artifact":"placement.json","scene_or_project_diff":"scene.diff.json"}'
"#,
        );
    }

    fn write_tool_at(&self, path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// ---------------------------------------------------------------------------
// Real-Alice environment helpers (shared across integration test files)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[allow(dead_code)]
pub fn real_alice_enabled() -> bool {
    env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn alice_home() -> PathBuf {
    PathBuf::from(env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()))
}

#[allow(dead_code)]
pub fn starter_projects_dir() -> PathBuf {
    let alice_home = alice_home();
    for relative in [
        "starter-projects",
        "core/resources/target/distribution/application/starter-projects",
        "core/resources/src/application/resources/starter-projects",
    ] {
        let candidate = alice_home.join(relative);
        if candidate.is_dir() {
            return candidate;
        }
    }
    alice_home.join("core/resources/target/distribution/application/starter-projects")
}

#[allow(dead_code)]
pub fn starter_project_path(name: &str) -> PathBuf {
    starter_projects_dir().join(format!("{name}.a3p"))
}
