use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn fake_toolchain_launch_smoke_writes_passing_manifest() {
    let _guard = ENV_LOCK.lock().unwrap();
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();

    let old_path = env::var("PATH").unwrap_or_default();
    unsafe {
        env::set_var("PATH", format!("{}:{old_path}", fixture.bin.display()));
    }

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "fake-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::default(),
    })
    .unwrap();

    unsafe {
        env::set_var("PATH", old_path);
    }

    assert!(manifest.failure_category.is_none());
    assert!(
        manifest
            .assertions
            .values()
            .all(|assertion| assertion.passed)
    );
    assert!(
        fixture
            .root
            .join("runs/real-alice-launch-smoke/fake-run/manifest.json")
            .is_file()
    );
}

#[test]
fn fake_toolchain_launch_smoke_uses_scenario_run_lane() {
    let _guard = ENV_LOCK.lock().unwrap();
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();

    let old_path = env::var("PATH").unwrap_or_default();
    unsafe {
        env::set_var("PATH", format!("{}:{old_path}", fixture.bin.display()));
    }

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "lesson-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("building-a-scene-first-world"),
    })
    .unwrap();

    unsafe {
        env::set_var("PATH", old_path);
    }

    assert_eq!(manifest.scenario_id, "building-a-scene-first-world");
    assert!(
        fixture
            .root
            .join("runs/building-a-scene-first-world/lesson-run/manifest.json")
            .is_file()
    );
}

#[test]
fn lesson_smoke_is_ready_when_window_evidence_exists_without_screenshot() {
    let _guard = ENV_LOCK.lock().unwrap();
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_screenshot_tools();
    fixture.write_fake_alice_repo();

    let old_path = env::var("PATH").unwrap_or_default();
    unsafe {
        env::set_var("PATH", format!("{}:{old_path}", fixture.bin.display()));
    }

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "window-evidence-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("building-a-scene-first-world"),
    })
    .unwrap();

    unsafe {
        env::set_var("PATH", old_path);
    }

    assert_eq!(manifest.scenario_id, "building-a-scene-first-world");
    assert!(
        manifest.failure_category.is_none(),
        "window evidence should satisfy lesson smoke-ready state without a screenshot: {:?}",
        manifest.failure_category
    );
    let smoke_ready = manifest
        .assertions
        .get("startup_window_or_screenshot")
        .expect("manifest should assert startup window-or-screenshot evidence");
    assert!(
        smoke_ready.passed,
        "window evidence should pass startup smoke-ready assertion: {:?}",
        smoke_ready
    );
}

struct TestFixture {
    root: PathBuf,
    bin: PathBuf,
    alice_home: PathBuf,
}

impl TestFixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("eatme-launch-smoke-{nonce}"));
        let bin = root.join("bin");
        let alice_home = root.join("alice");
        fs::create_dir_all(&bin).unwrap();
        Self {
            root,
            bin,
            alice_home,
        }
    }

    fn write_fake_tools(&self) {
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
echo "0x001 Alice org.alice.stageide.EntryPoint"
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

    fn write_failing_screenshot_tools(&self) {
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

    fn write_tool(&self, name: &str, script: &str) {
        let path = self.bin.join(name);
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn write_fake_alice_repo(&self) {
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
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
