use crate::launch_ui_actions::UiActionProbe;
use eatme_core::{CommandRunner, CommandSpec};
use std::time::Duration;

pub(crate) fn probe_desktop_save_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    activation_probe: Option<&UiActionProbe>,
) -> UiActionProbe {
    probe_desktop_shortcut(
        runner,
        display,
        activation_probe,
        ShortcutProbe {
            id: "dispatch-save-project-shortcut",
            action_name: "desktop save shortcut dispatch",
            key: "ctrl+s",
            label: "Ctrl+S",
            proves_only: "saved project content",
        },
    )
}

pub(crate) fn probe_desktop_run_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    activation_probe: Option<&UiActionProbe>,
    edit_proven: bool,
) -> UiActionProbe {
    if !edit_proven {
        return blocked_probe(
            "dispatch-run-world-shortcut",
            "blocked: procedure/code-block edit proof is required before desktop run shortcut dispatch",
        );
    }
    probe_desktop_shortcut(
        runner,
        display,
        activation_probe,
        ShortcutProbe {
            id: "dispatch-run-world-shortcut",
            action_name: "desktop run shortcut dispatch",
            key: "ctrl+F5",
            label: "Ctrl+F5",
            proves_only: "world execution",
        },
    )
}

pub(crate) fn probe_run_window_after_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    run_shortcut_probe: &UiActionProbe,
) -> UiActionProbe {
    if run_shortcut_probe.status != "passed" {
        return blocked_probe(
            "observe-run-window-after-shortcut",
            "blocked: desktop Run shortcut dispatch must pass before Run window observation",
        );
    }
    std::thread::sleep(Duration::from_secs(2));
    match capture_window_text(runner, display) {
        Ok((command, text)) if has_run_window_evidence(&text) => UiActionProbe {
            id: "observe-run-window-after-shortcut".into(),
            status: "passed".into(),
            detail: "observed an Alice Run window after Ctrl+F5 dispatch; this proves desktop Run window opening, not world completion".into(),
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
    window_text.lines().any(|line| {
        let normalized = line.to_ascii_lowercase();
        (normalized.contains(" run") || normalized.contains("\"run"))
            && !normalized.contains("firefox")
    })
}

struct ShortcutProbe {
    id: &'static str,
    action_name: &'static str,
    key: &'static str,
    label: &'static str,
    proves_only: &'static str,
}

fn probe_desktop_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    activation_probe: Option<&UiActionProbe>,
    shortcut: ShortcutProbe,
) -> UiActionProbe {
    let Some(activation_probe) = activation_probe else {
        return blocked_probe(
            shortcut.id,
            &format!(
                "blocked: Alice window activation is required before {}",
                shortcut.action_name
            ),
        );
    };
    if activation_probe.status != "passed" {
        return blocked_probe(
            shortcut.id,
            &format!(
                "blocked: Alice window activation did not pass before {}",
                shortcut.action_name
            ),
        );
    }
    let Some(window_id) = activation_probe.window_id.as_deref() else {
        return blocked_probe(
            shortcut.id,
            "blocked: Alice window activation did not record a window id",
        );
    };

    let output = runner.run(
        &CommandSpec::new("xdotool")
            .args([
                "key".to_string(),
                "--window".to_string(),
                window_id.to_string(),
                "--clearmodifiers".to_string(),
                shortcut.key.to_string(),
            ])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );

    match output {
        Ok(output) if output.exit_status == Some(0) => UiActionProbe {
            id: shortcut.id.into(),
            status: "passed".into(),
            detail: format!(
                "xdotool dispatched {} to Alice window {window_id}; this proves desktop input dispatch only, not {}",
                shortcut.label, shortcut.proves_only
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Ok(output) => UiActionProbe {
            id: shortcut.id.into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not dispatch {} to Alice window {window_id}; exit_status={:?}",
                shortcut.label, output.exit_status
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Err(error) => UiActionProbe {
            id: shortcut.id.into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not dispatch {} to Alice window {window_id}: {error:#}",
                shortcut.label
            ),
            window_id: Some(window_id.into()),
            command: Some(format!(
                "xdotool key --window {window_id} --clearmodifiers {}",
                shortcut.key
            )),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
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

#[cfg(test)]
mod tests;
