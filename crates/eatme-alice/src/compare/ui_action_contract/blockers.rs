use super::{action_ids, has_passed_action_probe, string_field};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(in crate::compare) struct UiActionEvidenceBlocker {
    pub code: String,
    pub action: String,
    pub reason: String,
    pub message: String,
}

impl UiActionEvidenceBlocker {
    pub(super) fn issue(&self) -> String {
        format!(
            "code={} action={} reason={} message={}",
            self.code, self.action, self.reason, self.message
        )
    }
}

pub(in crate::compare) fn ui_action_evidence_blockers(
    role: &str,
    contract: &serde_json::Value,
) -> Vec<UiActionEvidenceBlocker> {
    if !is_original_alice_role(role) {
        return Vec::new();
    }

    let mut blockers = Vec::new();
    let required_actions = action_ids(contract);
    for (action_id, required) in [
        (
            "verify-specific-alice-window",
            required_actions
                .iter()
                .any(|action| action == "verify-specific-alice-window"),
        ),
        (
            "activate-specific-alice-window",
            required_actions
                .iter()
                .any(|action| action == "activate-specific-alice-window")
                || preflight_bool(contract, "specific_alice_window_detected") == Some(true),
        ),
        (
            "dispatch-save-project-shortcut",
            has_passed_action_probe(contract, "activate-specific-alice-window"),
        ),
    ] {
        if required && let Some(reason) = real_action_evidence_blocker_reason(contract, action_id) {
            blockers.push(UiActionEvidenceBlocker {
                code: "missing_real_action_evidence".into(),
                action: format!("{}.{}", role_action_prefix(role), action_id),
                reason,
                message: format!(
                    "{} automation scenarios need real executable evidence for {action_id} before comparison readiness can use that action.",
                    role_label(role)
                ),
            });
        }
    }
    blockers
}

fn real_action_evidence_blocker_reason(
    contract: &serde_json::Value,
    action_id: &str,
) -> Option<String> {
    let Some(probe) = action_probe(contract, action_id) else {
        return Some(if action_id == "verify-specific-alice-window" {
            "required-action-probe-missing".into()
        } else {
            format!("{action_id}-missing")
        });
    };
    if string_field(probe, "status") != Some("passed") {
        return Some(format!(
            "{action_id}-{}",
            string_field(probe, "status").unwrap_or("status-missing")
        ));
    }
    if !real_action_probe_has_executable_evidence(probe) {
        return Some(format!("{action_id}-incomplete"));
    }
    None
}

fn action_probe<'a>(
    contract: &'a serde_json::Value,
    probe_id: &str,
) -> Option<&'a serde_json::Value> {
    contract
        .get("executed_action_probes")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .find(|probe| string_field(probe, "id") == Some(probe_id))
}

fn real_action_probe_has_executable_evidence(probe: &serde_json::Value) -> bool {
    string_field(probe, "detail").is_some_and(|detail| !detail.trim().is_empty())
        && string_field(probe, "command").is_some_and(|command| !command.trim().is_empty())
        && probe
            .get("exit_status")
            .and_then(serde_json::Value::as_i64)
            .is_some()
}

fn preflight_bool(contract: &serde_json::Value, field: &str) -> Option<bool> {
    contract
        .get("preflight_evidence")
        .and_then(|preflight| preflight.get(field))
        .and_then(serde_json::Value::as_bool)
}

fn role_action_prefix(role: &str) -> String {
    role.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn role_label(role: &str) -> String {
    if role == "original Alice" {
        "Original Alice".into()
    } else {
        role.into()
    }
}

fn is_original_alice_role(role: &str) -> bool {
    role == "baseline" || role == "original Alice"
}
