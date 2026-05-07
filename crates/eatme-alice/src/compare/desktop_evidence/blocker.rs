use super::DesktopRunPixelObservationEvidence;

pub(super) fn next_actionable_pixel_observation_blocker(
    evidence: &DesktopRunPixelObservationEvidence,
) -> Option<String> {
    if evidence.status != "blocked" {
        return None;
    }

    let mut details = Vec::new();
    if let Some(fixes) = blocker_fix_hints(evidence.blocker.as_ref()) {
        details.push(format!("fix next: {fixes}"));
    }
    if let Some(reason) = blocker_reason(evidence.blocker.as_ref()) {
        details.push(reason);
    } else if !evidence.detail.is_empty() {
        details.push(evidence.detail.clone());
    }
    if let Some(codes) = blocker_code_summary(evidence.blocker.as_ref()) {
        details.push(format!("codes: {codes}"));
    }
    if let Some(component_state) = component_state_summary(evidence.component_state.as_ref()) {
        details.push(format!("component state: {component_state}"));
    }

    let summary = if details.is_empty() {
        "desktop Run pixel observation is blocked".into()
    } else {
        format!(
            "desktop Run pixel observation is blocked: {}",
            details.join("; ")
        )
    };
    Some(summary)
}

fn blocker_reason(blocker: Option<&serde_json::Value>) -> Option<String> {
    blocker?
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .filter(|reason| !reason.is_empty())
        .map(str::to_string)
}

fn blocker_code_summary(blocker: Option<&serde_json::Value>) -> Option<String> {
    let blocker = blocker?;
    if let Some(codes) = blocker.get("codes").and_then(serde_json::Value::as_array) {
        let codes = codes
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter(|code| !code.is_empty())
            .collect::<Vec<_>>();
        if !codes.is_empty() {
            return Some(codes.join(", "));
        }
    }
    blocker
        .get("code")
        .and_then(serde_json::Value::as_str)
        .filter(|code| !code.is_empty())
        .map(str::to_string)
}

fn blocker_fix_hints(blocker: Option<&serde_json::Value>) -> Option<String> {
    let codes = blocker_code_summary(blocker)?
        .split(',')
        .map(str::trim)
        .filter_map(fix_hint_for_code)
        .collect::<Vec<_>>();
    (!codes.is_empty()).then(|| codes.join("; "))
}

fn fix_hint_for_code(code: &str) -> Option<&'static str> {
    match code {
        "java_awt_headless" => Some("run Alice with a non-headless graphics environment"),
        "render_target_not_displayable" => Some("make the Run render target displayable"),
        "render_target_not_showing" => Some("make the Run render target visible on screen"),
        "render_target_has_no_positive_size" => Some("give the Run render target a positive size"),
        "screen_capture_unavailable" | "screen_capture_denied" => {
            Some("allow desktop screen capture")
        }
        _ => None,
    }
}

fn component_state_summary(component_state: Option<&serde_json::Value>) -> Option<String> {
    let component_state = component_state?.as_object()?;
    let preferred = [
        "graphicsEnvironmentHeadless",
        "renderTargetDisplayable",
        "renderTargetShowing",
        "renderTargetWidth",
        "renderTargetHeight",
    ];
    let mut parts = Vec::new();
    for key in preferred {
        if let Some(value) = component_state.get(key) {
            parts.push(format!("{key}={}", plain_json_value(value)));
        }
    }
    let mut remaining = component_state
        .iter()
        .filter(|(key, _)| !preferred.contains(&key.as_str()))
        .collect::<Vec<_>>();
    remaining.sort_by_key(|(key, _)| *key);
    for (key, value) in remaining {
        parts.push(format!("{key}={}", plain_json_value(value)));
    }
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn plain_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}
