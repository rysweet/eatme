use super::DesktopRunPixelObservationEvidence;

pub(super) fn next_actionable_pixel_observation_blocker(
    evidence: &DesktopRunPixelObservationEvidence,
) -> Option<String> {
    if evidence.status != "blocked" {
        return None;
    }

    let mut details = Vec::new();
    if let Some(action) = explicit_next_action(evidence) {
        details.push(format!("fix next: {action}"));
    } else if let Some(fixes) = blocker_fix_hints(evidence.blocker.as_ref()) {
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

pub(super) fn project_proof_artifact_blocker_detail(
    label: &str,
    blocker: Option<&serde_json::Value>,
) -> Option<String> {
    let mut details = Vec::new();
    if let Some(reason) = blocker_reason(blocker) {
        details.push(reason);
    }
    if let Some(codes) = blocker_code_summary(blocker) {
        details.push(format!("codes: {codes}"));
    }

    (!details.is_empty()).then(|| format!("{label} is blocked: {}", details.join("; ")))
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

fn explicit_next_action(evidence: &DesktopRunPixelObservationEvidence) -> Option<String> {
    plain_next_action(evidence.next_action.as_ref()).or_else(|| {
        let blocker = evidence.blocker.as_ref()?;
        plain_next_action(
            blocker
                .get("next_action")
                .or_else(|| blocker.get("nextAction")),
        )
    })
}

fn plain_next_action(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str().filter(|text| !text.trim().is_empty()) {
        return Some(text.trim().to_string());
    }
    let object = value.as_object()?;
    for key in ["fix", "action", "summary", "description", "detail"] {
        if let Some(text) = object
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            return Some(text.to_string());
        }
    }
    let steps = object
        .get("steps")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|step| !step.is_empty())
        .collect::<Vec<_>>();
    (!steps.is_empty()).then(|| steps.join("; "))
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
