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
