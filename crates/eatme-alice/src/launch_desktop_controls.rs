use crate::launch_ui_actions::UiActionProbe;
use eatme_core::{CommandRunner, CommandSpec};
use std::time::Duration;

pub(crate) fn probe_desktop_save_shortcut(
    runner: &impl CommandRunner,
    display: &str,
    activation_probe: Option<&UiActionProbe>,
) -> UiActionProbe {
    let Some(activation_probe) = activation_probe else {
        return blocked_probe(
            "blocked: Alice window activation is required before desktop save shortcut dispatch",
        );
    };
    if activation_probe.status != "passed" {
        return blocked_probe(
            "blocked: Alice window activation did not pass before desktop save shortcut dispatch",
        );
    }
    let Some(window_id) = activation_probe.window_id.as_deref() else {
        return blocked_probe("blocked: Alice window activation did not record a window id");
    };

    let output = runner.run(
        &CommandSpec::new("xdotool")
            .args([
                "key".to_string(),
                "--window".to_string(),
                window_id.to_string(),
                "--clearmodifiers".to_string(),
                "ctrl+s".to_string(),
            ])
            .env("DISPLAY", display)
            .timeout(Duration::from_secs(5))
            .retries(2, Duration::from_millis(100)),
    );

    match output {
        Ok(output) if output.exit_status == Some(0) => UiActionProbe {
            id: "dispatch-save-project-shortcut".into(),
            status: "passed".into(),
            detail: format!(
                "xdotool dispatched Ctrl+S to Alice window {window_id}; this proves desktop input dispatch only, not saved project content"
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Ok(output) => UiActionProbe {
            id: "dispatch-save-project-shortcut".into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not dispatch Ctrl+S to Alice window {window_id}; exit_status={:?}",
                output.exit_status
            ),
            window_id: Some(window_id.into()),
            command: Some(output.command),
            exit_status: output.exit_status,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Err(error) => UiActionProbe {
            id: "dispatch-save-project-shortcut".into(),
            status: "failed".into(),
            detail: format!(
                "xdotool could not dispatch Ctrl+S to Alice window {window_id}: {error:#}"
            ),
            window_id: Some(window_id.into()),
            command: Some(format!(
                "xdotool key --window {window_id} --clearmodifiers ctrl+s"
            )),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

fn blocked_probe(detail: &str) -> UiActionProbe {
    UiActionProbe {
        id: "dispatch-save-project-shortcut".into(),
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
