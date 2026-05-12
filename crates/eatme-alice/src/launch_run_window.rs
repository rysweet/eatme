use crate::launch_ui_actions::UiActionProbe;
use eatme_core::{CommandRunner, CommandSpec};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(crate) const RUN_WINDOW_CREATED_SENTINEL: &str = "run-window-evidence/run-window-created.json";
const ALICE_LAUNCH_WINDOW_WIDTH: u32 = 1000;
const ALICE_LAUNCH_WINDOW_HEIGHT: u32 = 740;
const RUN_TOOLBAR_BUTTON_X: u32 = 344;
const RUN_TOOLBAR_BUTTON_Y: u32 = 47;

pub(crate) fn probe_run_window_after_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    run_shortcut_probe: &UiActionProbe,
) -> UiActionProbe {
    if run_shortcut_probe.status != "passed" {
        return blocked_probe(
            "observe-run-window-after-shortcut",
            "blocked: desktop Run shortcut dispatch must pass before Run window observation",
        );
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Some(probe) = run_window_created_sentinel_probe(
        "observe-run-window-after-shortcut",
        "observed RabbitHole Run-window-created sentinel after Ctrl+F5 dispatch; this records Alice preparing the desktop Run frame, not world completion",
        run_dir,
        run_shortcut_probe,
    ) {
        return with_run_window_screenshot(runner, display, run_dir, probe);
    }
    match capture_window_text(runner, display) {
        Ok((command, text)) if has_run_window_evidence(&text) => UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "passed".into(),
            detail: "observed a window listing consistent with an Alice Run window after Ctrl+F5 dispatch; this indicates desktop Run window opening, not world completion".into(),
            window_id: run_shortcut_probe.window_id.clone(),
            command: Some(command),
            exit_status: Some(0),
            stdout: text,
            stderr: String::new(),
        },
        Ok((command, text)) if has_license_modal_evidence(&text) => UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "blocked".into(),
            detail: "blocked: Alice license agreement window was still visible after Ctrl+F5 dispatch; Run-window observation requires clearing that modal first".into(),
            window_id: run_shortcut_probe.window_id.clone(),
            command: Some(command),
            exit_status: Some(0),
            stdout: text,
            stderr: String::new(),
        },
        Ok((command, text)) => UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "failed".into(),
            detail: "Ctrl+F5 dispatch succeeded, but no Alice Run window was observed".into(),
            window_id: run_shortcut_probe.window_id.clone(),
            command: Some(command),
            exit_status: Some(0),
            stdout: text,
            stderr: String::new(),
        },
        Err(error) => UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "failed".into(),
            detail: format!("could not inspect windows after Ctrl+F5 dispatch: {error}"),
            window_id: run_shortcut_probe.window_id.clone(),
            command: Some("wmctrl -lx; xwininfo -root -tree".into()),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

pub(crate) fn probe_desktop_run_toolbar_button(
    runner: &impl CommandRunner,
    display: &str,
    activation_probe: Option<&UiActionProbe>,
    run_window_after_shortcut_probe: &UiActionProbe,
) -> UiActionProbe {
    if run_window_after_shortcut_probe.status == "passed" {
        return blocked_probe(
            "dispatch-run-toolbar-button",
            "blocked: Run window already opened after Ctrl+F5; toolbar fallback was not needed",
        );
    }
    let Some(activation_probe) = activation_probe else {
        return blocked_probe(
            "dispatch-run-toolbar-button",
            "blocked: Alice window activation is required before desktop Run toolbar dispatch",
        );
    };
    if activation_probe.status != "passed" {
        return blocked_probe(
            "dispatch-run-toolbar-button",
            "blocked: Alice window activation did not pass before desktop Run toolbar dispatch",
        );
    }
    let Some(window_id) = activation_probe.window_id.as_deref() else {
        return blocked_probe(
            "dispatch-run-toolbar-button",
            "blocked: Alice window activation did not record a window id",
        );
    };
    let geometry_command = match validate_run_toolbar_click_geometry(runner, display, window_id) {
        Ok(command) => command,
        Err(probe) => return *probe,
    };
    let args = vec![
        "mousemove".to_string(),
        "--window".to_string(),
        window_id.to_string(),
        RUN_TOOLBAR_BUTTON_X.to_string(),
        RUN_TOOLBAR_BUTTON_Y.to_string(),
        "click".to_string(),
        "1".to_string(),
    ];
    let command_display = format!("xdotool {}", args.join(" "));
    let output = runner.run(
        &CommandSpec::new("xdotool")
            .args(args)
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );
    match output {
        Ok(output) if output.exit_status == Some(0) => UiActionProbe {
            id: "dispatch-run-toolbar-button".into(),
            status: "passed".into(),
            detail: format!(
                "xdotool clicked the configured Run toolbar coordinate ({RUN_TOOLBAR_BUTTON_X},{RUN_TOOLBAR_BUTTON_Y}) after verifying Alice's {ALICE_LAUNCH_WINDOW_WIDTH}x{ALICE_LAUNCH_WINDOW_HEIGHT} launch geometry; this indicates bounded desktop input dispatch only, not world execution"
            ),
            window_id: Some(window_id.into()),
            command: Some(format!("{geometry_command}; {}", output.command)),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Ok(output) => UiActionProbe {
            id: "dispatch-run-toolbar-button".into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not click the visible Run toolbar button in Alice window {window_id}; exit_status={:?}",
                output.exit_status
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Err(error) => UiActionProbe {
            id: "dispatch-run-toolbar-button".into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not click the visible Run toolbar button in Alice window {window_id}: {error:#}"
            ),
            window_id: Some(window_id.into()),
            command: Some(command_display),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

pub(crate) fn probe_run_toolbar_sequence(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    activation_probe: Option<&UiActionProbe>,
    run_window_after_shortcut_probe: &UiActionProbe,
) -> (UiActionProbe, UiActionProbe) {
    let toolbar_probe = probe_desktop_run_toolbar_button(
        runner,
        display,
        activation_probe,
        run_window_after_shortcut_probe,
    );
    let window_probe =
        probe_run_window_after_toolbar_button(runner, display, run_dir, &toolbar_probe);
    (toolbar_probe, window_probe)
}

pub(crate) fn probe_run_window_after_toolbar_button(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    toolbar_probe: &UiActionProbe,
) -> UiActionProbe {
    if toolbar_probe.status != "passed" {
        return blocked_probe(
            "observe-run-window-after-toolbar-button",
            "blocked: desktop Run toolbar dispatch must pass before Run window observation",
        );
    }
    if let Some(probe) = run_window_created_sentinel_probe(
        "observe-run-window-after-toolbar-button",
        "observed RabbitHole Run-window-created sentinel after Run toolbar click; this records Alice preparing the desktop Run frame, not world completion",
        run_dir,
        toolbar_probe,
    ) {
        return with_run_window_screenshot(runner, display, run_dir, probe);
    }
    crate::launch_run_window_poll::poll_for_run_window(
        runner,
        display,
        toolbar_probe.window_id.as_deref(),
    )
    .into_toolbar_probe(toolbar_probe.window_id.clone())
}

fn run_window_created_sentinel_probe(
    id: &str,
    detail: &str,
    run_dir: &Path,
    action_probe: &UiActionProbe,
) -> Option<UiActionProbe> {
    let path = run_dir.join(RUN_WINDOW_CREATED_SENTINEL);
    let content = fs::read_to_string(&path).ok()?;
    if !sentinel_content_indicates_run_window_created(&content) {
        return None;
    }
    Some(UiActionProbe {
        id: id.into(),
        status: "passed".into(),
        detail: detail.into(),
        window_id: action_probe.window_id.clone(),
        command: Some(format!("read {}", path.display())),
        exit_status: Some(0),
        stdout: content,
        stderr: String::new(),
    })
}

fn with_run_window_screenshot(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    mut probe: UiActionProbe,
) -> UiActionProbe {
    match capture_run_window_screenshot(runner, display, run_dir) {
        Ok(command) => {
            probe.detail.push_str(
                "; captured screenshot artifact screenshots/run-window-after-dispatch.png",
            );
            probe.command = Some(format!(
                "{}; {command}",
                probe.command.clone().unwrap_or_default()
            ));
            probe
        }
        Err(error) => {
            probe.detail.push_str(&format!(
                "; screenshot capture failed without changing sentinel result: {error}"
            ));
            probe
        }
    }
}

fn validate_run_toolbar_click_geometry(
    runner: &impl CommandRunner,
    display: &str,
    window_id: &str,
) -> Result<String, Box<UiActionProbe>> {
    let args = vec![
        "getwindowgeometry".to_string(),
        "--shell".to_string(),
        window_id.to_string(),
    ];
    let command_display = format!("xdotool {}", args.join(" "));
    let output = runner.run(
        &CommandSpec::new("xdotool")
            .args(args)
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );
    match output {
        Ok(output) if output.exit_status == Some(0) => {
            let text = command_text(&output.stdout, &output.stderr);
            if geometry_matches_fixed_launch_window(&text) {
                Ok(output.command)
            } else {
                Err(Box::new(UiActionProbe {
                    id: "dispatch-run-toolbar-button".into(),
                    status: "blocked".into(),
                    detail: format!(
                        "blocked: Alice window geometry must be {ALICE_LAUNCH_WINDOW_WIDTH}x{ALICE_LAUNCH_WINDOW_HEIGHT} before the bounded Run toolbar coordinate click at ({RUN_TOOLBAR_BUTTON_X},{RUN_TOOLBAR_BUTTON_Y}); geometry output did not match"
                    ),
                    window_id: Some(window_id.into()),
                    command: Some(output.command),
                    exit_status: output.exit_status,
                    stdout: output.stdout,
                    stderr: output.stderr,
                }))
            }
        }
        Ok(output) => Err(Box::new(UiActionProbe {
            id: "dispatch-run-toolbar-button".into(),
            status: "blocked".into(),
            detail: format!(
                "blocked: could not verify Alice window geometry before bounded Run toolbar coordinate click; exit_status={:?}",
                output.exit_status
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        })),
        Err(error) => Err(Box::new(UiActionProbe {
            id: "dispatch-run-toolbar-button".into(),
            status: "blocked".into(),
            detail: format!(
                "blocked: could not verify Alice window geometry before bounded Run toolbar coordinate click: {error:#}"
            ),
            window_id: Some(window_id.into()),
            command: Some(command_display),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        })),
    }
}

fn capture_run_window_screenshot(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
) -> Result<String, String> {
    let screenshot_dir = run_dir.join("screenshots");
    fs::create_dir_all(&screenshot_dir).map_err(|error| error.to_string())?;
    let path = screenshot_dir.join("run-window-after-dispatch.png");
    let scrot = runner
        .run(
            &CommandSpec::new("scrot")
                .args([path.display().to_string()])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(10))
                .retries(2, Duration::from_millis(100)),
        )
        .map_err(|error| format!("{error:#}"))?;
    if scrot.exit_status == Some(0) && non_empty_file(&path) {
        return Ok(scrot.command);
    }
    let import = runner
        .run(
            &CommandSpec::new("import")
                .args(["-window".into(), "root".into(), path.display().to_string()])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(10))
                .retries(2, Duration::from_millis(100)),
        )
        .map_err(|error| format!("{error:#}"))?;
    if import.exit_status == Some(0) && non_empty_file(&path) {
        return Ok(import.command);
    }
    Err(format!(
        "scrot={:?}; import={:?}",
        scrot.exit_status, import.exit_status
    ))
}

fn non_empty_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn capture_window_text(
    runner: &impl CommandRunner,
    display: &str,
) -> Result<(String, String), String> {
    let wmctrl = runner
        .run(
            &CommandSpec::new("wmctrl")
                .args(["-lx"])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(5))
                .retries(2, Duration::from_millis(100)),
        )
        .map_err(|error| format!("{error:#}"))?;
    if wmctrl.exit_status == Some(0) && !command_text(&wmctrl.stdout, &wmctrl.stderr).is_empty() {
        return Ok((wmctrl.command, command_text(&wmctrl.stdout, &wmctrl.stderr)));
    }
    let xwininfo = runner
        .run(
            &CommandSpec::new("xwininfo")
                .args(["-root", "-tree"])
                .env("DISPLAY", display)
                .timeout(Duration::from_secs(5))
                .retries(2, Duration::from_millis(100)),
        )
        .map_err(|error| format!("{error:#}"))?;
    if xwininfo.exit_status == Some(0) {
        return Ok((
            xwininfo.command,
            command_text(&xwininfo.stdout, &xwininfo.stderr),
        ));
    }
    Err(format!(
        "wmctrl={:?}; xwininfo={:?}",
        wmctrl.exit_status, xwininfo.exit_status
    ))
}

fn command_text(stdout: &str, stderr: &str) -> String {
    if stdout.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    }
}

fn has_run_window_evidence(window_text: &str) -> bool {
    window_text
        .lines()
        .any(crate::launch_run_window_poll::line_is_alice_run_window)
}

fn sentinel_content_indicates_run_window_created(content: &str) -> bool {
    let normalized = content.to_ascii_lowercase();
    normalized.contains("eatme.alice-run-window-created/v1")
        && normalized.contains("\"status\"")
        && normalized.contains("\"created\"")
}

fn geometry_matches_fixed_launch_window(text: &str) -> bool {
    shell_value(text, "WIDTH") == Some(ALICE_LAUNCH_WINDOW_WIDTH)
        && shell_value(text, "HEIGHT") == Some(ALICE_LAUNCH_WINDOW_HEIGHT)
}

fn shell_value(text: &str, key: &str) -> Option<u32> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(candidate, value)| (candidate == key).then(|| value.trim().parse().ok()))?
}

fn has_license_modal_evidence(window_text: &str) -> bool {
    window_text
        .lines()
        .any(|line| line.to_ascii_lowercase().contains("license agreement"))
}

fn blocked_probe(id: &str, detail: &str) -> UiActionProbe {
    UiActionProbe {
        id: id.into(),
        status: "blocked".into(),
        detail: detail.into(),
        window_id: None,
        command: None,
        exit_status: None,
        stdout: String::new(),
        stderr: String::new(),
    }
}
