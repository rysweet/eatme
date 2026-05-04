use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::fs;

mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

#[test]
fn fake_toolchain_launch_smoke_writes_passing_manifest() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

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
    assert!(
        manifest
            .assertions
            .get("real_alice_execution_evidence")
            .expect("manifest should include real Alice evidence contract")
            .passed
    );
    assert!(manifest.window_list.is_some());
}

#[test]
fn fake_toolchain_launch_smoke_uses_scenario_run_lane() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "lesson-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(manifest.scenario_id, "code-editor-first-run");
    assert!(
        fixture
            .root
            .join("runs/code-editor-first-run/lesson-run/manifest.json")
            .is_file()
    );
}

#[test]
fn lesson_smoke_is_ready_when_window_evidence_exists_without_screenshot() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_screenshot_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "window-evidence-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(manifest.scenario_id, "code-editor-first-run");
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

#[test]
fn lesson_smoke_rejects_unrelated_window_without_screenshot() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_screenshot_tools();
    fixture.write_unrelated_window_tool();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "unrelated-window-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("screenshot_missing")
    );
    assert!(manifest.screenshot_error.is_some());
    let smoke_ready = manifest
        .assertions
        .get("startup_window_or_screenshot")
        .expect("manifest should assert startup window-or-screenshot evidence");
    assert!(
        !smoke_ready.passed,
        "unrelated windows must not satisfy Alice launch evidence: {:?}",
        smoke_ready
    );
}

#[test]
fn missing_desktop_dependency_writes_blocking_manifest() {
    let fixture = TestFixture::new();
    fixture.write_missing_xvfb_probe();
    let _path_override = PathOverride::replace(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "missing-xvfb-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("missing_dependency")
    );
    let real_evidence = manifest
        .assertions
        .get("real_alice_execution_evidence")
        .expect("blocked manifest should include the evidence contract");
    assert!(!real_evidence.passed);
    assert!(real_evidence.detail.contains("preflight blocked"));
    assert!(
        fixture
            .root
            .join("runs/code-editor-first-run/missing-xvfb-run/manifest.json")
            .is_file()
    );
    assert!(
        fs::read_to_string(
            fixture
                .root
                .join("runs/code-editor-first-run/missing-xvfb-run/alice.log")
        )
        .unwrap()
        .contains("preflight blocked")
    );
}

#[test]
fn package_failure_writes_blocking_manifest_without_real_evidence() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_package_tool();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "package-failed-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("alice_package_failed")
    );
    let real_evidence = manifest
        .assertions
        .get("real_alice_execution_evidence")
        .expect("blocked manifest should include the evidence contract");
    assert!(!real_evidence.passed);
    assert!(real_evidence.detail.contains("Alice package failed"));
    assert!(manifest.alice_pid.is_none());
    assert!(manifest.screenshot.is_none());
}

#[test]
fn launch_smoke_fails_when_alice_log_is_missing() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_missing_log_java_tool();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "missing-log-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("code-editor-first-run"),
    })
    .unwrap();

    assert_eq!(manifest.failure_category.as_deref(), Some("log_unreadable"));
    assert!(manifest.log.is_none());
    assert!(
        manifest
            .log_error
            .as_deref()
            .is_some_and(|error| error.contains("reading Alice log")),
        "missing log read error should be preserved in manifest: {:?}",
        manifest.log_error
    );
    let no_fatal_logs = manifest
        .assertions
        .get("no_fatal_logs")
        .expect("manifest should assert fatal-log scan");
    assert!(
        !no_fatal_logs.passed,
        "missing logs must not be treated as no fatal logs"
    );
}
