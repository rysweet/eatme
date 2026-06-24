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
fn fake_toolchain_launch_smoke_uses_scenario_run_directory() {
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
fn launch_smoke_marks_process_started_when_alice_stays_alive() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "process-started-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::default(),
    })
    .unwrap();

    assert!(
        manifest
            .assertions
            .get("process_started")
            .expect("manifest should include process-started assertion")
            .passed
    );
    assert_eq!(manifest.failure_category, None);
}

#[test]
fn launch_smoke_reports_process_exit_when_alice_dies_during_startup_wait() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_quick_exit_java_tool();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "process-exited-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::default(),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("alice_process_exited")
    );
    assert!(
        !manifest
            .assertions
            .get("process_started")
            .expect("manifest should include process-started assertion")
            .passed
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
        scenario: LaunchSmokeScenario::new("building-a-scene-first-world"),
    })
    .unwrap();

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
fn real_ui_action_contract_fails_loudly_when_actions_are_not_automated() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "ui-action-contract-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_automation_unimplemented")
    );
    assert!(
        manifest
            .assertions
            .get("specific_alice_window_detected")
            .expect("window assertion should exist")
            .passed
    );
    assert!(
        !manifest
            .assertions
            .get("place_object_ui_action")
            .expect("object placement assertion should exist")
            .passed
    );
    assert!(manifest.ui_action_contract.is_some());
    assert!(
        fixture
            .root
            .join(
                "runs/first-lessons-real-ui-actions/ui-action-contract-run/ui-action-contract.json"
            )
            .is_file()
    );
    let contract_path = fixture
        .root
        .join("runs/first-lessons-real-ui-actions/ui-action-contract-run/ui-action-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).unwrap()).unwrap();
    let place_object_probe = contract["action_precondition_probes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|probe| probe["action_id"] == "place-object")
        .expect("place-object no-go probe should be machine-readable");
    assert_eq!(place_object_probe["decision"], "no_go");
    assert_eq!(
        place_object_probe["missing_affordance"]["id"],
        "deterministic-alice-object-gallery-placement-affordance"
    );
    assert!(
        place_object_probe["missing_affordance"]["next_implementation"]
            .as_str()
            .unwrap()
            .contains("named gallery selector")
    );
}

#[test]
fn real_ui_action_contract_advances_when_object_placement_hook_proves_placement() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "ui-action-hook-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_remaining_steps_unimplemented")
    );
    assert!(
        manifest
            .assertions
            .get("place_object_ui_action")
            .expect("object placement assertion should exist")
            .passed
    );
    assert!(
        !manifest
            .assertions
            .get("edit_procedure_ui_action")
            .expect("edit procedure assertion should exist")
            .passed
    );
    assert!(
        manifest
            .assertions
            .get("edit_procedure_precondition_no_go_probe")
            .expect("edit procedure no-go assertion should exist")
            .passed
    );
    let contract_path = fixture
        .root
        .join("runs/first-lessons-real-ui-actions/ui-action-hook-run/ui-action-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).unwrap()).unwrap();
    let edit_probe = contract["action_precondition_probes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|probe| probe["action_id"] == "edit-procedure-or-code-block")
        .expect("edit procedure no-go probe should be machine-readable after object placement");
    assert_eq!(edit_probe["decision"], "no_go");
    assert_eq!(
        edit_probe["missing_affordance"]["id"],
        "deterministic-alice-procedure-edit-affordance"
    );
}

#[test]
fn modified_class_portability_writes_bounded_desktop_contract_when_hook_is_missing() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "class-portability-contract-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("modified-class-portability"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("class_portability_desktop_contract_blocked")
    );
    assert!(
        !manifest
            .assertions
            .get("desktop_class_portability_evidence")
            .expect("class portability assertion should exist")
            .passed
    );
    let contract_path = fixture.root.join(
        "runs/modified-class-portability/class-portability-contract-run/portability/desktop-class-portability-contract.json",
    );
    assert!(contract_path.is_file());
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).unwrap()).unwrap();
    assert_eq!(
        contract["schema_version"],
        "eatme.desktop-class-portability-contract/v1"
    );
    assert_eq!(contract["status"], "blocked");
    assert_eq!(
        contract["required_actions"][0]["missing_affordance_id"],
        "deterministic-alice-class-portability-affordance"
    );
    assert!(
        contract["does_not_claim"]
            .as_array()
            .unwrap()
            .iter()
            .any(|claim| claim
                .as_str()
                .unwrap()
                .contains("tools/eatme-class-portability is absent"))
    );
}

#[test]
fn modified_class_portability_passes_when_desktop_hook_returns_artifacts() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_class_portability_hook();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "class-portability-hook-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("modified-class-portability"),
    })
    .unwrap();

    assert_eq!(manifest.failure_category, None);
    assert!(
        manifest
            .assertions
            .get("desktop_class_portability_evidence")
            .expect("class portability assertion should exist")
            .passed
    );
    let contract_path = fixture.root.join(
        "runs/modified-class-portability/class-portability-hook-run/portability/desktop-class-portability-contract.json",
    );
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).unwrap()).unwrap();
    assert_eq!(contract["status"], "passed");
    let probe = &contract["candidate_affordance_probes"][0];
    assert_eq!(probe["status"], "passed");
    assert!(
        probe["exported_class_package"]["size_bytes"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(probe["import_report"]["size_bytes"].as_u64().unwrap() > 0);
    assert!(probe["save_reopen_report"]["size_bytes"].as_u64().unwrap() > 0);
    assert!(
        probe["post_import_behavior"]["size_bytes"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn real_ui_action_contract_finds_window_without_window_manager_client_list() {
    let fixture = TestFixture::new();
    fixture.write_fake_tools();
    fixture.write_window_managerless_alice_tools();
    fixture.write_fake_alice_repo();
    fixture.write_fake_object_placement_hook();
    let _path_override = PathOverride::prepend(&fixture.bin);

    let manifest = run_launch_smoke(&LaunchSmokeOptions {
        alice_home: fixture.alice_home.clone(),
        run_id: "window-managerless-ui-action-run".into(),
        runs_dir: fixture.root.join("runs"),
        timeout_seconds: 1,
        json: true,
        no_memory: true,
        offline_package: true,
        scenario: LaunchSmokeScenario::new("first-lessons-real-ui-actions"),
    })
    .unwrap();

    assert_eq!(
        manifest.failure_category.as_deref(),
        Some("ui_action_remaining_steps_unimplemented")
    );
    assert!(
        manifest
            .assertions
            .get("specific_alice_window_detected")
            .expect("window assertion should exist")
            .passed
    );
    assert!(
        manifest
            .assertions
            .get("activate_alice_window_ui_action")
            .expect("activation assertion should exist")
            .passed
    );
    assert!(
        manifest
            .assertions
            .get("place_object_ui_action")
            .expect("object placement assertion should exist")
            .passed
    );
    let window_list = fs::read_to_string(fixture.root.join(
        "runs/first-lessons-real-ui-actions/window-managerless-ui-action-run/window-list.txt",
    ))
    .unwrap();
    assert!(window_list.contains("xwininfo"));
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
