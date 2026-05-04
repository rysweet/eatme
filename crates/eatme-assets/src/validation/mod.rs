mod crew;
mod portability;
mod scenario;

pub use crew::validate_persona_crew;
pub use scenario::validate_scenario_asset;

use std::path::Path;

pub(crate) fn validate_id(id: &str, kind: &str, errors: &mut Vec<String>) {
    if id.is_empty()
        || id.starts_with('-')
        || id.ends_with('-')
        || !id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        errors.push(format!("{kind} id {id:?} must be kebab-case"));
    }
}

pub(crate) fn contextualize_scenario_errors(
    path: &Path,
    scenario_id: &str,
    errors: Vec<String>,
) -> Vec<String> {
    if errors.is_empty() {
        return errors;
    }
    let path_display = path.display().to_string();
    let context = if scenario_id.trim().is_empty() {
        path_display.clone()
    } else {
        format!("{path_display} ({scenario_id})")
    };
    errors
        .into_iter()
        .map(|error| {
            if (!scenario_id.trim().is_empty() && error.contains(scenario_id))
                || error.contains(&path_display)
            {
                error
            } else {
                format!("{context}: {error}")
            }
        })
        .collect()
}

pub(crate) fn require_nonempty(value: &str, field: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
    }
}

pub(crate) fn require_list(values: &[String], field: &str, errors: &mut Vec<String>) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        errors.push(format!("{field} must contain non-empty values"));
    }
}
