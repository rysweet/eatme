use super::*;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn desktop_save_shortcut_dispatches_ctrl_s_to_activated_window() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "xdotool key --window 0x001 --clearmodifiers ctrl+s".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });

    let probe = probe_desktop_save_shortcut(&runner, ":99", Some(&activation_probe("passed")));

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "dispatch-save-project-shortcut");
    assert_eq!(probe.window_id.as_deref(), Some("0x001"));
    assert!(probe.detail.contains("input dispatch only"));
    assert_eq!(
        runner.commands(),
        vec!["xdotool key --window 0x001 --clearmodifiers ctrl+s"]
    );
}

#[test]
fn desktop_save_shortcut_blocks_without_activation() {
    let runner = FakeCommandRunner::default();

    let probe = probe_desktop_save_shortcut(&runner, ":99", None);

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
}

#[test]
fn desktop_save_shortcut_fails_when_xdotool_fails() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "xdotool key --window 0x001 --clearmodifiers ctrl+s".into(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "window not found".into(),
    });

    let probe = probe_desktop_save_shortcut(&runner, ":99", Some(&activation_probe("passed")));

    assert_eq!(probe.status, "failed");
    assert!(probe.detail.contains("could not dispatch Ctrl+S"));
}

#[test]
fn desktop_run_shortcut_dispatches_documented_accelerator_after_edit_proof() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "xdotool windowfocus --sync 0x001 key --clearmodifiers ctrl+F5".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });

    let probe = probe_desktop_run_shortcut(&runner, ":99", Some(&activation_probe("passed")), true);

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "dispatch-run-world-shortcut");
    assert_eq!(probe.window_id.as_deref(), Some("0x001"));
    assert!(probe.detail.contains("input dispatch only"));
    assert_eq!(
        runner.commands(),
        vec!["xdotool windowfocus --sync 0x001 key --clearmodifiers ctrl+F5"]
    );
}

#[test]
fn desktop_run_shortcut_error_reports_refocus_command() {
    let runner = FailingCommandRunner;

    let probe = probe_desktop_run_shortcut(&runner, ":99", Some(&activation_probe("passed")), true);

    assert_eq!(probe.status, "failed");
    assert_eq!(
        probe.command.as_deref(),
        Some("xdotool windowfocus --sync 0x001 key --clearmodifiers ctrl+F5")
    );
}

#[test]
fn desktop_run_shortcut_blocks_before_edit_proof() {
    let runner = FakeCommandRunner::default();

    let probe =
        probe_desktop_run_shortcut(&runner, ":99", Some(&activation_probe("passed")), false);

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
}

struct FailingCommandRunner;

impl CommandRunner for FailingCommandRunner {
    fn run(&self, _spec: &CommandSpec) -> anyhow::Result<CommandOutput> {
        anyhow::bail!("xdotool failed to start")
    }
}

#[test]
fn run_window_observation_passes_when_run_window_is_listed() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "wmctrl -lx".into(),
        exit_status: Some(0),
        stdout: "0x002 host org.alice.stageide.EntryPoint Run...\n".into(),
        stderr: String::new(),
    });

    let run_dir = unique_test_dir("listed-run-window");
    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "observe-run-window-after-shortcut");
    assert!(probe.detail.contains("Run window"));
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_blocks_before_shortcut_dispatch() {
    let runner = FakeCommandRunner::default();
    let run_dir = unique_test_dir("blocked-run-window");

    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("blocked"));

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn desktop_run_toolbar_clicks_visible_button_after_shortcut_misses_window() {
    let runner = FakeCommandRunner::default();
    runner.push_output(fixed_geometry_output());
    runner.push_output(CommandOutput {
        command: "xdotool mousemove --window 0x001 344 47 click 1".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });

    let probe = probe_desktop_run_toolbar_button(
        &runner,
        ":99",
        Some(&activation_probe("passed")),
        &run_shortcut_probe("failed"),
    );

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "dispatch-run-toolbar-button");
    assert_eq!(
        runner.commands(),
        vec![
            "xdotool getwindowgeometry --shell 0x001",
            "xdotool mousemove --window 0x001 344 47 click 1"
        ]
    );
}

#[test]
fn desktop_run_toolbar_blocks_when_geometry_is_not_fixed_launch_size() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "xdotool getwindowgeometry --shell 0x001".into(),
        exit_status: Some(0),
        stdout: "WINDOW=0x001\nX=0\nY=0\nWIDTH=800\nHEIGHT=600\nSCREEN=0\n".into(),
        stderr: String::new(),
    });

    let probe = probe_desktop_run_toolbar_button(
        &runner,
        ":99",
        Some(&activation_probe("passed")),
        &run_shortcut_probe("failed"),
    );

    assert_eq!(probe.status, "blocked");
    assert!(probe.detail.contains("geometry must be 1000x740"));
    assert_eq!(
        runner.commands(),
        vec!["xdotool getwindowgeometry --shell 0x001"]
    );
}

#[test]
fn desktop_run_toolbar_blocks_when_shortcut_already_opened_run_window() {
    let runner = FakeCommandRunner::default();

    let probe = probe_desktop_run_toolbar_button(
        &runner,
        ":99",
        Some(&activation_probe("passed")),
        &UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "passed".into(),
            detail: "opened".into(),
            window_id: Some("0x001".into()),
            command: None,
            exit_status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        },
    );

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
}

#[test]
fn run_window_observation_passes_when_sentinel_is_written() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "scrot /tmp/run-window.png".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });
    let run_dir = unique_test_dir("sentinel-run-window");
    let sentinel = run_dir.join(RUN_WINDOW_CREATED_SENTINEL);
    std::fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    std::fs::create_dir_all(run_dir.join("screenshots")).unwrap();
    std::fs::write(
        run_dir.join("screenshots/run-window-after-dispatch.png"),
        "png",
    )
    .unwrap();
    std::fs::write(
        &sentinel,
        r#"{"schema_version":"eatme.alice-run-window-created/v1","status":"created"}"#,
    )
    .unwrap();

    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "passed");
    assert!(probe.detail.contains("Run-window-created sentinel"));
    assert!(probe.detail.contains("screenshot artifact"));
    assert!(probe.stdout.contains("eatme.alice-run-window-created/v1"));
    assert_eq!(
        runner.commands(),
        vec![format!(
            "scrot {}",
            run_dir
                .join("screenshots/run-window-after-dispatch.png")
                .display()
        )]
    );
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_keeps_sentinel_passed_when_screenshot_fails() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "scrot /tmp/run-window.png".into(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "no display".into(),
    });
    runner.push_output(CommandOutput {
        command: "import -window root /tmp/run-window.png".into(),
        exit_status: Some(1),
        stdout: String::new(),
        stderr: "no display".into(),
    });
    let run_dir = unique_test_dir("sentinel-screenshot-fail");
    let sentinel = run_dir.join(RUN_WINDOW_CREATED_SENTINEL);
    std::fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    std::fs::write(
        &sentinel,
        r#"{"schema_version":"eatme.alice-run-window-created/v1","status":"created"}"#,
    )
    .unwrap();

    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "passed");
    assert!(probe.detail.contains("screenshot capture failed"));
    assert!(probe.stdout.contains("eatme.alice-run-window-created/v1"));
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_ignores_malformed_sentinel() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "wmctrl -lx".into(),
        exit_status: Some(0),
        stdout: "0x001 host org.alice.stageide.EntryPoint Alice 3\n".into(),
        stderr: String::new(),
    });
    let run_dir = unique_test_dir("malformed-sentinel-run-window");
    let sentinel = run_dir.join(RUN_WINDOW_CREATED_SENTINEL);
    std::fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    std::fs::write(&sentinel, "not json").unwrap();

    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "failed");
    assert!(probe.detail.contains("no Alice Run window"));
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_after_toolbar_passes_with_sentinel_and_screenshot() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "scrot /tmp/run-window.png".into(),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    });
    let run_dir = unique_test_dir("toolbar-sentinel-run-window");
    let sentinel = run_dir.join(RUN_WINDOW_CREATED_SENTINEL);
    std::fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    std::fs::create_dir_all(run_dir.join("screenshots")).unwrap();
    std::fs::write(
        run_dir.join("screenshots/run-window-after-dispatch.png"),
        "png",
    )
    .unwrap();
    std::fs::write(
        &sentinel,
        r#"{"schema_version":"eatme.alice-run-window-created/v1","status":"created"}"#,
    )
    .unwrap();

    let probe = probe_run_window_after_toolbar_button(
        &runner,
        ":99",
        &run_dir,
        &run_shortcut_probe("passed"),
    );

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "observe-run-window-after-toolbar-button");
    assert!(probe.detail.contains("Run toolbar click"));
    assert!(probe.detail.contains("screenshot artifact"));
    assert!(probe.stdout.contains("eatme.alice-run-window-created/v1"));
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_fails_without_run_window() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "wmctrl -lx".into(),
        exit_status: Some(0),
        stdout: "0x001 host org.alice.stageide.EntryPoint Alice 3\n".into(),
        stderr: String::new(),
    });

    let run_dir = unique_test_dir("missing-run-window");
    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "failed");
    assert!(probe.detail.contains("no Alice Run window"));
    let _ = std::fs::remove_dir_all(run_dir);
}

#[test]
fn run_window_observation_blocks_when_license_modal_is_visible() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "xwininfo -root -tree".into(),
        exit_status: Some(0),
        stdout: r#"0x60002a "License Agreement (Part 1 of 2): Alice 3"
0x600007 "Alice 3 ""#
            .into(),
        stderr: String::new(),
    });

    let run_dir = unique_test_dir("license-run-window");
    let probe =
        probe_run_window_after_shortcut(&runner, ":99", &run_dir, &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "blocked");
    assert!(probe.detail.contains("license agreement"));
    let _ = std::fs::remove_dir_all(run_dir);
}

fn run_shortcut_probe(status: &str) -> UiActionProbe {
    UiActionProbe {
        id: "dispatch-run-world-shortcut".into(),
        status: status.into(),
        detail: "run shortcut".into(),
        window_id: Some("0x001".into()),
        command: Some("xdotool key --window 0x001 --clearmodifiers ctrl+F5".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn unique_test_dir(prefix: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("eatme-{prefix}-{nonce}"))
}

fn activation_probe(status: &str) -> UiActionProbe {
    UiActionProbe {
        id: "activate-specific-alice-window".into(),
        status: status.into(),
        detail: "activated".into(),
        window_id: Some("0x001".into()),
        command: Some("wmctrl -ia 0x001".into()),
        exit_status: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn fixed_geometry_output() -> CommandOutput {
    CommandOutput {
        command: "xdotool getwindowgeometry --shell 0x001".into(),
        exit_status: Some(0),
        stdout: "WINDOW=0x001\nX=0\nY=0\nWIDTH=1000\nHEIGHT=740\nSCREEN=0\n".into(),
        stderr: String::new(),
    }
}
