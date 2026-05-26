use crate::launch_ui_actions::UiActionProbe;

pub(crate) fn ui_action_activation_failure_category(probe: &UiActionProbe) -> &'static str {
    let detail = probe.detail.to_ascii_lowercase();
    let stderr = probe.stderr.to_ascii_lowercase();
    if detail.contains("alice-like window") {
        "alice_like_window_not_main"
    } else if detail.contains("no alice main window") {
        "alice_window_not_detected"
    } else if activation_output_is_unsupported(&detail)
        || activation_output_is_unsupported(&stderr)
        || probe
            .command
            .as_deref()
            .is_some_and(|command| command == "xdotool windowfocus")
    {
        "alice_window_activation_unsupported"
    } else {
        "alice_window_activation_failed"
    }
}

pub(crate) fn activation_failure_detail(
    window_id: &str,
    wmctrl_output: Option<&eatme_core::CommandOutput>,
    xdotool_output: &eatme_core::CommandOutput,
) -> String {
    if activation_output_is_unsupported(&xdotool_output.stderr)
        || activation_output_is_unsupported(&xdotool_output.stdout)
        || wmctrl_output.is_some_and(|output| {
            activation_output_is_unsupported(&output.stderr)
                || activation_output_is_unsupported(&output.stdout)
        })
    {
        format!(
            "unsupported activation method for Alice window {window_id}; wmctrl/xdotool did not provide a usable focus path"
        )
    } else {
        format!(
            "wmctrl could not activate Alice window {window_id}; xdotool windowfocus exit_status={:?}",
            xdotool_output.exit_status
        )
    }
}

fn activation_output_is_unsupported(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    normalized.contains("not to support")
        || normalized.contains("_net_active_window")
        || normalized.contains("not supported")
        || normalized.contains("unsupported")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launch_ui_actions::UiActionProbe;
    use eatme_core::CommandOutput;

    fn probe(detail: &str, stderr: &str, command: Option<&str>) -> UiActionProbe {
        UiActionProbe {
            id: "activate-alice-window".into(),
            status: "failed".into(),
            detail: detail.into(),
            window_id: Some("0x04200001".into()),
            command: command.map(str::to_string),
            exit_status: Some(1),
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    fn output(stdout: &str, stderr: &str, exit_status: Option<i32>) -> CommandOutput {
        CommandOutput {
            command: "xdotool windowfocus 0x04200001".into(),
            exit_status,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    #[test]
    fn failure_category_distinguishes_detection_and_unsupported_paths() {
        assert_eq!(
            ui_action_activation_failure_category(&probe(
                "Detected Alice-like window but not the main frame",
                "",
                None
            )),
            "alice_like_window_not_main"
        );
        assert_eq!(
            ui_action_activation_failure_category(&probe(
                "No Alice main window was detected",
                "",
                None
            )),
            "alice_window_not_detected"
        );
        assert_eq!(
            ui_action_activation_failure_category(&probe(
                "activation failed",
                "NET_ACTIVE_WINDOW not supported",
                Some("xdotool windowfocus")
            )),
            "alice_window_activation_unsupported"
        );
        assert_eq!(
            ui_action_activation_failure_category(&probe(
                "wmctrl failed",
                "",
                Some("wmctrl -ia 0x04200001")
            )),
            "alice_window_activation_failed"
        );
    }

    #[test]
    fn activation_failure_detail_prefers_unsupported_message_when_any_tool_reports_it() {
        let wmctrl = output("", "This WM does not support _NET_ACTIVE_WINDOW", Some(1));
        let xdotool = output("", "window activation unsupported", Some(1));

        let detail = activation_failure_detail("0x04200001", Some(&wmctrl), &xdotool);

        assert!(detail.contains("unsupported activation method"));
        assert!(!detail.contains("exit_status"));
    }

    #[test]
    fn activation_failure_detail_reports_exit_status_for_supported_but_unsuccessful_tools() {
        let xdotool = output("", "focus request failed", Some(2));

        let detail = activation_failure_detail("0x04200001", None, &xdotool);

        assert!(detail.contains("wmctrl could not activate Alice window 0x04200001"));
        assert!(detail.contains("Some(2)"));
    }
}
