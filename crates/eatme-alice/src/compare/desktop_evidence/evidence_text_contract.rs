#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EvidenceTextContext {
    Claim,
    Limitation,
}

pub(crate) fn validate_claim_text(field: &str, value: &str) -> Result<(), String> {
    validate_evidence_text(field, value, EvidenceTextContext::Claim)
}

pub(crate) fn validate_limitation_text(field: &str, value: &str) -> Result<(), String> {
    validate_evidence_text(field, value, EvidenceTextContext::Limitation)
}

pub(crate) fn next_action_contract_issue(json: &serde_json::Value) -> Option<String> {
    let mut issues = Vec::new();
    validate_optional_array(json, "candidate_actions", &mut issues);
    validate_optional_array(json, "doesNotClaim", &mut issues);
    validate_optional_array(json, "does_not_claim", &mut issues);
    validate_next_evidence_array(json, &mut issues);
    validate_evidence_boundaries_section(json, &mut issues);
    validate_next_action_text(json, &mut issues);

    (!issues.is_empty()).then(|| issues.join("; "))
}

pub(crate) fn limitation_array<'a>(
    json: &serde_json::Value,
    snake_key: &'a str,
    camel_key: &'a str,
    required: bool,
) -> Result<Vec<(&'a str, String)>, String> {
    if json.get(snake_key).is_none() && json.get(camel_key).is_none() {
        return if required {
            Err(format!("{snake_key} must be a non-empty array of strings"))
        } else {
            Ok(Vec::new())
        };
    }

    let mut merged = Vec::new();
    for key in [snake_key, camel_key] {
        let Some(value) = json.get(key) else {
            continue;
        };
        let Some(array) = value.as_array() else {
            return Err(format!("{key} must be a non-empty array of strings"));
        };
        let items = array
            .iter()
            .map(|item| item.as_str().map(str::trim).filter(|item| !item.is_empty()))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| format!("{key} must contain only non-empty strings"))?;
        if items.is_empty() {
            return Err(format!("{key} must be a non-empty array of strings"));
        }
        merged.extend(items.into_iter().map(|item| (key, item.to_string())));
    }

    Ok(merged)
}

fn validate_evidence_text(
    field: &str,
    value: &str,
    context: EvidenceTextContext,
) -> Result<(), String> {
    let normalized = normalize(value);
    if normalized.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if contains_filler_text(&normalized) {
        return Err(format!("{field} contains filler wording"));
    }
    if context == EvidenceTextContext::Claim
        && let Some(claim) = unsupported_affirmative_claim(&normalized)
    {
        return Err(format!("{field} contains unsupported claim: {claim}"));
    }
    Ok(())
}

fn validate_optional_array(json: &serde_json::Value, key: &str, issues: &mut Vec<String>) {
    if json.get(key).is_none() {
        return;
    }
    if string_array_contract(json, key).is_err() {
        issues.push(format!("{key} must be a non-empty array of strings"));
    }
}

fn validate_next_evidence_array(json: &serde_json::Value, issues: &mut Vec<String>) {
    let has_snake = json.get("requires_next_evidence").is_some();
    let has_camel = json.get("requiresNextEvidence").is_some();
    if !has_snake && !has_camel {
        return;
    }
    let mut valid_items = Vec::new();
    for key in ["requires_next_evidence", "requiresNextEvidence"] {
        match string_array_contract(json, key) {
            Ok(items) => valid_items.extend(items),
            Err(()) if json.get(key).is_some() => {
                issues.push(format!("{key} must be a non-empty array of strings"))
            }
            Err(()) => {}
        }
    }
    if valid_items.is_empty() && issues.is_empty() {
        issues.push("requiresNextEvidence must be a non-empty array of strings".into());
    }
}

fn validate_evidence_boundaries_section(json: &serde_json::Value, issues: &mut Vec<String>) {
    let Some(value) = json
        .get("evidence_boundaries")
        .or_else(|| json.get("evidenceBoundaries"))
    else {
        return;
    };
    match value.as_array() {
        Some(boundaries) if !boundaries.is_empty() => {}
        Some(_) => issues.push("evidence_boundaries must be a non-empty array".into()),
        None => issues.push("evidence_boundaries must be an array".into()),
    }
}

fn validate_next_action_text(json: &serde_json::Value, issues: &mut Vec<String>) {
    for (field, text) in [
        ("reason", string_field(json, "reason")),
        (
            "blocker.reason",
            json.get("blocker")
                .and_then(|blocker| blocker.get("reason"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        ),
    ] {
        if let Some(text) = text
            && let Err(reason) = validate_claim_text(field, &text)
        {
            issues.push(reason);
        }
    }

    for key in ["requires_next_evidence", "requiresNextEvidence"] {
        for item in string_array(json, key) {
            if let Err(reason) = validate_claim_text(key, &item) {
                issues.push(reason);
            }
        }
    }
    for key in ["does_not_claim", "doesNotClaim"] {
        for item in string_array(json, key) {
            if let Err(reason) = validate_limitation_text(key, &item) {
                issues.push(reason);
            }
        }
    }
}

fn string_array_contract(json: &serde_json::Value, key: &str) -> Result<Vec<String>, ()> {
    let Some(array) = json.get(key).and_then(serde_json::Value::as_array) else {
        return Err(());
    };
    let items = array
        .iter()
        .map(|item| item.as_str().map(str::trim).filter(|item| !item.is_empty()))
        .collect::<Option<Vec<_>>>()
        .ok_or(())?;
    if items.is_empty() {
        return Err(());
    }
    Ok(items.into_iter().map(str::to_string).collect())
}

fn string_array(json: &serde_json::Value, key: &str) -> Vec<String> {
    json.get(key)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn string_field(json: &serde_json::Value, key: &str) -> Option<String> {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn contains_filler_text(value: &str) -> bool {
    [
        "todo",
        "fixme",
        "tbd",
        "lorem",
        "ipsum",
        "dummy",
        "placeholder",
        "sample scenario",
        "sample evidence",
        "sample text",
        "example scenario",
        "example evidence",
    ]
    .iter()
    .any(|forbidden| value.contains(forbidden))
}

fn unsupported_affirmative_claim(value: &str) -> Option<String> {
    let mut claims = Vec::new();
    for (claim, patterns) in [
        (
            "first-lesson completion",
            &[
                "proves first-lesson completion",
                "proves first lesson completion",
                "first-lesson completion is proven",
                "first lesson completion is proven",
                "lesson completed",
                "first lesson completed",
                "first-lesson completed",
            ][..],
        ),
        (
            "full Alice UI automation",
            &[
                "full alice ui automation succeeded",
                "full alice ui automation is proven",
                "full alice ui automation complete",
                "full ui automation succeeded",
                "ui automation succeeded",
                "ui automation is complete",
                "automation passed",
            ][..],
        ),
        (
            "grading",
            &[
                "grading is complete",
                "grading complete",
                "grading occurred",
                "grading passed",
                "grade passed",
            ][..],
        ),
        (
            "creative assessment",
            &[
                "creative assessment passed",
                "creative assessment is complete",
                "creative assessment complete",
                "creative quality assessed",
            ][..],
        ),
        (
            "Save completion",
            &[
                "save completion evidence",
                "save completed",
                "desktop save completion is proven",
                "bounded save completion evidence",
                "save project succeeded",
            ][..],
        ),
        (
            "full world execution",
            &[
                "full world execution",
                "world execution succeeded",
                "world execution success",
                "world execution is proven",
                "world execution complete",
            ][..],
        ),
        (
            "deployed sharing",
            &[
                "deployed sharing",
                "sharing deployment succeeded",
                "sharing deployment success",
                "sharing deployment is proven",
                "sharing deployment complete",
            ][..],
        ),
        (
            "platform success",
            &[
                "platform success",
                "platform succeeded",
                "platform is proven",
                "platform passed",
            ][..],
        ),
    ] {
        if patterns.iter().any(|pattern| value.contains(pattern))
            && !contains_limitation_wording(value)
        {
            claims.push(claim);
        }
    }
    (!claims.is_empty()).then(|| claims.join(", "))
}

fn contains_limitation_wording(value: &str) -> bool {
    [
        "does not prove",
        "doesn't prove",
        "not proven",
        "is not proven",
        "not assessed",
        "is not assessed",
        "not complete",
        "is not complete",
        "not claimed",
        "does not claim",
        "doesn't claim",
        "out of scope",
        "unproven",
        "missing",
        "blocked",
        "before reporting",
        "requires next evidence",
    ]
    .iter()
    .any(|allowed| value.contains(allowed))
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}
