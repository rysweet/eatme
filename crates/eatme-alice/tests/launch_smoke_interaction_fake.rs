//! Post-focus screenshot interaction tests (fake toolchain).
//!
//! These tests verify the blocked-cascade and assertion-recording behavior
//! for the post-focus screenshot captured after Alice window activation.
//! They run against fake binaries — no real Alice, Xvfb, or desktop required.

use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
use std::fs;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{PathOverride, TestFixture};

#[test]
fn first_lessons_manifest_includes_post_focus_screenshot_captured_assertion() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let post_focus = manifest
        .assertions
        .get("post_focus_screenshot_captured")
        .expect(
            "first-lessons-real-ui-actions manifest must include \
             post_focus_screenshot_captured assertion",
        );
    assert!(
        post_focus.passed,
        "post_focus_screenshot_captured should pass when activation succeeds: {}",
        post_focus.detail
    );
}

#[test]
fn post_focus_screenshot_artifact_present_in_manifest() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-artifact-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let artifact = manifest
        .post_focus_screenshot
        .as_ref()
        .expect("manifest should include post_focus_screenshot artifact after activation");
    assert!(
        artifact.size_bytes > 0,
        "post_focus_screenshot artifact should be non-empty"
    );
    assert!(
        artifact.path.contains("post_focus"),
        "post_focus_screenshot path should reference post_focus: {}",
        artifact.path
    );
}

#[test]
fn post_focus_screenshot_file_exists_on_disk() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let _manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-disk-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let run_dir = fixture
        .root
        .join("runs/first-lessons-real-ui-actions/post-focus-disk-run");
    let post_focus_path = run_dir.join("screenshots/post_focus.png");
    assert!(
        post_focus_path.is_file(),
        "post_focus.png should exist at {}",
        post_focus_path.display()
    );
    let content = fs::read(&post_focus_path).unwrap();
    assert!(
        !content.is_empty(),
        "post_focus.png should be non-empty on disk"
    );
}

#[test]
fn post_focus_screenshot_blocked_when_activation_blocked() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_screenshot_tools();
    fixture.write_unrelated_window_tool();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-blocked-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let activation = manifest
        .assertions
        .get("activate_alice_window_ui_action")
        .expect("activation assertion should exist");
    assert!(
        !activation.passed,
        "activation should be blocked with unrelated window"
    );

    let post_focus = manifest
        .assertions
        .get("post_focus_screenshot_captured")
        .expect("post_focus_screenshot_captured should exist even when blocked");
    assert!(
        !post_focus.passed,
        "post_focus_screenshot_captured should fail when activation is blocked: {}",
        post_focus.detail
    );
    assert!(
        post_focus.detail.contains("blocked"),
        "detail should explain blocking: {}",
        post_focus.detail
    );

    assert!(
        manifest.post_focus_screenshot.is_none(),
        "post_focus_screenshot artifact should be None when activation is blocked"
    );
}

#[test]
fn post_focus_screenshot_blocked_when_screenshot_tools_fail() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_failing_screenshot_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-scrot-fail-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let activation = manifest
        .assertions
        .get("activate_alice_window_ui_action")
        .expect("activation assertion should exist");
    assert!(
        activation.passed,
        "activation should still pass with Alice window present"
    );

    let post_focus = manifest
        .assertions
        .get("post_focus_screenshot_captured")
        .expect("post_focus_screenshot_captured should exist when capture fails");
    assert!(
        !post_focus.passed,
        "post_focus_screenshot_captured should fail when scrot/import fail: {}",
        post_focus.detail
    );

    assert!(
        manifest.post_focus_screenshot.is_none(),
        "post_focus_screenshot artifact should be None when capture fails"
    );
    assert!(
        manifest.post_focus_screenshot_error.is_some(),
        "post_focus_screenshot_error should record the capture failure"
    );
}

#[test]
fn default_scenario_omits_post_focus_screenshot_assertion() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "default-scenario-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::default(),
    })
    .unwrap();

    assert!(
        !manifest
            .assertions
            .contains_key("post_focus_screenshot_captured"),
        "default scenario should not include post_focus_screenshot_captured assertion"
    );
    assert!(
        manifest.post_focus_screenshot.is_none(),
        "default scenario should not capture post-focus screenshot"
    );
}

#[test]
fn post_focus_screenshot_manifest_json_includes_new_fields() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let _manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "post-focus-json-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    let manifest_path = fixture
        .root
        .join("runs/first-lessons-real-ui-actions/post-focus-json-run/manifest.json");
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap())
            .expect("manifest.json should be valid JSON");

    assert!(
        json.get("post_focus_screenshot").is_some(),
        "manifest.json should contain post_focus_screenshot field"
    );

    let round_tripped: eatme_core::LaunchSmokeManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap())
            .expect("manifest.json should round-trip through LaunchSmokeManifest");
    assert!(
        round_tripped.post_focus_screenshot.is_some(),
        "round-tripped manifest should preserve post_focus_screenshot artifact"
    );
}
