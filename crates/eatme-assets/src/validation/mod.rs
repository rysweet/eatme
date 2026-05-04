mod crew;
mod persona_discovery;
mod scenario;

pub(crate) use crew::persona_reference_index;
pub use crew::validate_persona_crew;
pub(crate) use persona_discovery::{PersonaDiscovery, discover_scenario_personas};
pub use scenario::validate_scenario_asset;
pub(crate) use scenario::validate_scenario_asset_with_personas;

use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub(crate) struct PersonaReferenceIndex {
    pub(crate) instructors: BTreeSet<String>,
    pub(crate) students: BTreeSet<String>,
    pub(crate) all: BTreeSet<String>,
}

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

pub(crate) fn validate_reference_list(
    scenario_id: &str,
    refs: &[String],
    expected_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    role: &str,
    errors: &mut Vec<String>,
) {
    for id in refs {
        if !expected_ids.contains(id) {
            let suffix = if all_ids.contains(id) {
                " with wrong role"
            } else {
                ""
            };
            errors.push(format!(
                "scenario {scenario_id} references missing {role} persona {id}{suffix}"
            ));
        }
    }
}
