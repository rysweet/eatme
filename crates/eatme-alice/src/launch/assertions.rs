use eatme_core::AssertionResult;

pub(super) fn visual_evidence_detail(
    screenshot_ok: bool,
    window_evidence_ok: bool,
    screenshot_error: Option<&str>,
    window_list_error: Option<&str>,
) -> String {
    if screenshot_ok {
        return "startup screenshot exists and is non-empty".into();
    }
    if window_evidence_ok {
        return "Alice-specific window identity was captured".into();
    }

    let mut details = vec![
        "startup requires a non-empty screenshot or Alice-specific window identity".to_string(),
    ];
    match screenshot_error {
        Some(error) => details.push(format!("screenshot error: {error}")),
        None => details.push("startup screenshot is missing or empty".into()),
    }
    match window_list_error {
        Some(error) => details.push(format!("window list error: {error}")),
        None => details.push("no Alice-specific window identity found".into()),
    }
    details.join("; ")
}

pub(super) fn fatal_log_detail(fatal_lines: &[String], log_error: Option<&str>) -> String {
    if let Some(error) = log_error {
        return format!("Alice log could not be read: {error}");
    }
    format!("{} fatal log lines found", fatal_lines.len())
}

pub(crate) fn bool_assert(passed: bool, detail: impl Into<String>) -> AssertionResult {
    if passed {
        AssertionResult::pass(detail)
    } else {
        AssertionResult::fail(detail)
    }
}
