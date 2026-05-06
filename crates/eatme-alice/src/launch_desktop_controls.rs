use crate::launch_ui_actions::UiActionProbe;
use eatme_core::{CommandRunner, CommandSpec};
use std::time::Duration;

#[cfg(test)]
pub(crate) use crate::launch_run_window::{
    RUN_WINDOW_CREATED_SENTINEL, probe_desktop_run_toolbar_button, probe_run_window_after_shortcut,
    probe_run_window_after_toolbar_button,
};

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
            refocus_before_key: false,
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
            refocus_before_key: true,
        },
    )
}

struct ShortcutProbe {
    id: &'static str,
    action_name: &'static str,
    key: &'static str,
    label: &'static str,
    proves_only: &'static str,
    refocus_before_key: bool,
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

    let args = shortcut_args(window_id, &shortcut);
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
            command: Some(command_display),
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

fn shortcut_args(window_id: &str, shortcut: &ShortcutProbe) -> Vec<String> {
    if shortcut.refocus_before_key {
        vec![
            "windowfocus".to_string(),
            "--sync".to_string(),
            window_id.to_string(),
            "key".to_string(),
            "--clearmodifiers".to_string(),
            shortcut.key.to_string(),
        ]
    } else {
        vec![
            "key".to_string(),
            "--window".to_string(),
            window_id.to_string(),
            "--clearmodifiers".to_string(),
            shortcut.key.to_string(),
        ]
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
