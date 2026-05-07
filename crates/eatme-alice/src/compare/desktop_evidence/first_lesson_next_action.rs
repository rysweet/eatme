use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const DESKTOP_FIRST_LESSON_NEXT_ACTION: &str =
    "run-window-evidence/desktop-first-lesson-next-action.json";
const MISSING_FIRST_LESSON_NEXT_ACTION_EVIDENCE: &str = "missing desktop first-lesson next-action evidence; expected run-window-evidence/desktop-first-lesson-next-action.json under the comparison evidence root";

#[derive(Clone, Debug, Serialize)]
pub struct DesktopFirstLessonNextActionEvidence {
    pub status: String,
    pub artifact: Option<String>,
    pub detail: String,
    pub candidate_actions: Vec<String>,
    pub blocker: Option<serde_json::Value>,
    pub requires_next_evidence: Vec<String>,
}

impl DesktopFirstLessonNextActionEvidence {
    pub(crate) fn issue_when_invalid(&self) -> Option<String> {
        (self.status == "invalid").then(|| self.detail.clone())
    }

    pub fn next_actionable_blocker(&self) -> Option<String> {
        if self.status != "blocked" {
            return None;
        }

        let mut parts = Vec::new();
        if !self.requires_next_evidence.is_empty() {
            parts.push(format!(
                "fix next: {}",
                self.requires_next_evidence.join("; ")
            ));
        }
        if !self.candidate_actions.is_empty() {
            parts.push(format!(
                "candidate actions: {}",
                self.candidate_actions.join(", ")
            ));
        }
        if !self.detail.is_empty() {
            parts.push(self.detail.clone());
        }
        if let Some(codes) = self.blocker.as_ref().and_then(blocker_codes) {
            parts.push(format!("codes: {codes}"));
        }

        Some(if parts.is_empty() {
            "desktop first-lesson next action is blocked".into()
        } else {
            format!(
                "desktop first-lesson next action is blocked: {}",
                parts.join("; ")
            )
        })
    }
}

pub(crate) fn check_first_lesson_next_action_evidence(
    evidence_root: &Path,
    ui_action_contract_path: &Path,
) -> DesktopFirstLessonNextActionEvidence {
    let Some(run_dir) = ui_action_contract_path.parent() else {
        return missing_first_lesson_next_action();
    };
    let candidate = run_dir.join(DESKTOP_FIRST_LESSON_NEXT_ACTION);
    let Ok(root) = evidence_root.canonicalize() else {
        return missing_first_lesson_next_action();
    };
    let Ok(artifact) = candidate.canonicalize() else {
        return missing_first_lesson_next_action();
    };
    if !artifact.starts_with(root) {
        return missing_first_lesson_next_action();
    }
    let Ok(text) = fs::read_to_string(&artifact) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            "desktop first-lesson next-action evidence exists but is not readable",
        );
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            "desktop first-lesson next-action evidence exists but is not valid JSON",
        );
    };
    if json
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.alice-desktop-first-lesson-next-action/v1")
    {
        return invalid_first_lesson_next_action(
            Some(artifact),
            "desktop first-lesson next-action evidence has the wrong schema_version",
        );
    }
    let Some(status) = json.get("status").and_then(serde_json::Value::as_str) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            "desktop first-lesson next-action evidence is missing status field",
        );
    };

    DesktopFirstLessonNextActionEvidence {
        status: status.into(),
        artifact: Some(artifact.display().to_string()),
        detail: first_lesson_next_action_detail(&json),
        candidate_actions: string_array(&json, "candidate_actions"),
        blocker: json.get("blocker").cloned(),
        requires_next_evidence: requires_next_evidence(&json),
    }
}

fn missing_first_lesson_next_action() -> DesktopFirstLessonNextActionEvidence {
    DesktopFirstLessonNextActionEvidence {
        status: "missing".into(),
        artifact: None,
        detail: MISSING_FIRST_LESSON_NEXT_ACTION_EVIDENCE.into(),
        candidate_actions: Vec::new(),
        blocker: None,
        requires_next_evidence: Vec::new(),
    }
}

fn invalid_first_lesson_next_action(
    artifact: Option<PathBuf>,
    detail: &str,
) -> DesktopFirstLessonNextActionEvidence {
    DesktopFirstLessonNextActionEvidence {
        status: "invalid".into(),
        artifact: artifact.map(|path| path.display().to_string()),
        detail: detail.into(),
        candidate_actions: Vec::new(),
        blocker: None,
        requires_next_evidence: Vec::new(),
    }
}

fn first_lesson_next_action_detail(json: &serde_json::Value) -> String {
    json.get("blocker")
        .and_then(|blocker| blocker.get("reason"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| json.get("reason").and_then(serde_json::Value::as_str))
        .unwrap_or("desktop first-lesson next-action evidence was read")
        .to_string()
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

fn requires_next_evidence(json: &serde_json::Value) -> Vec<String> {
    let mut items = string_array(json, "requiresNextEvidence");
    for item in string_array(json, "requires_next_evidence") {
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

fn blocker_codes(json: &serde_json::Value) -> Option<String> {
    let codes = json
        .get("codes")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    (!codes.is_empty()).then(|| codes.join(", "))
}
