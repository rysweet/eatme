use super::*;
use eatme_core::CommandOutput;
use eatme_test_support::FakeCommandRunner;

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
        command: "xdotool key --window 0x001 --clearmodifiers ctrl+F5".into(),
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
        vec!["xdotool key --window 0x001 --clearmodifiers ctrl+F5"]
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

#[test]
fn run_window_observation_passes_when_run_window_is_listed() {
    let runner = FakeCommandRunner::default();
    runner.push_output(CommandOutput {
        command: "wmctrl -lx".into(),
        exit_status: Some(0),
        stdout: "0x002 host org.alice.stageide.EntryPoint Run...\n".into(),
        stderr: String::new(),
    });

    let probe = probe_run_window_after_shortcut(&runner, ":99", &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "passed");
    assert_eq!(probe.id, "observe-run-window-after-shortcut");
    assert!(probe.detail.contains("Run window"));
}

#[test]
fn run_window_observation_blocks_before_shortcut_dispatch() {
    let runner = FakeCommandRunner::default();

    let probe = probe_run_window_after_shortcut(&runner, ":99", &run_shortcut_probe("blocked"));

    assert_eq!(probe.status, "blocked");
    assert!(runner.commands().is_empty());
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

    let probe = probe_run_window_after_shortcut(&runner, ":99", &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "failed");
    assert!(probe.detail.contains("no Alice Run window"));
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

    let probe = probe_run_window_after_shortcut(&runner, ":99", &run_shortcut_probe("passed"));

    assert_eq!(probe.status, "blocked");
    assert!(probe.detail.contains("license agreement"));
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
