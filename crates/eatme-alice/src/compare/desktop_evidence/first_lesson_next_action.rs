use serde::{Serialize, Serializer};
use std::fs;
use std::path::{Path, PathBuf};

use super::{blocker, resolve_run_dir_artifact_path};

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
    pub save_project_proof_artifact: ProjectProofArtifactEvidence,
    pub select_project_proof_artifact: ProjectProofArtifactEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofArtifactState {
    Present,
    Missing,
    Blocked,
}

impl ProofArtifactState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Blocked => "blocked",
        }
    }
}

impl Serialize for ProofArtifactState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectProofArtifactEvidence {
    pub status: ProofArtifactState,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ProjectProofArtifactInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker: Option<serde_json::Value>,
}

impl ProjectProofArtifactEvidence {
    pub(crate) fn missing(label: &str) -> Self {
        Self {
            status: ProofArtifactState::Missing,
            detail: format!("{label} is missing; artifact availability was not declared."),
            artifact: None,
            blocker: None,
        }
    }

    pub(crate) fn state(&self) -> &'static str {
        self.status.as_str()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectProofArtifactInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
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
        save_project_proof_artifact: project_proof_artifact(
            &json,
            evidence_root,
            run_dir,
            "save_project_proof_artifact",
            "saveProjectProofArtifact",
            "Save Project proof artifact",
        ),
        select_project_proof_artifact: project_proof_artifact(
            &json,
            evidence_root,
            run_dir,
            "select_project_proof_artifact",
            "selectProjectProofArtifact",
            "Select Project proof artifact",
        ),
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
        save_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            "Save Project proof artifact",
        ),
        select_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            "Select Project proof artifact",
        ),
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
        save_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            "Save Project proof artifact",
        ),
        select_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            "Select Project proof artifact",
        ),
    }
}

fn project_proof_artifact(
    json: &serde_json::Value,
    evidence_root: &Path,
    run_dir: &Path,
    snake_key: &str,
    camel_key: &str,
    label: &str,
) -> ProjectProofArtifactEvidence {
    let Some(declaration) = json.get(snake_key).or_else(|| json.get(camel_key)) else {
        return ProjectProofArtifactEvidence::missing(label);
    };

    let blocker = declaration.get("blocker").cloned();
    if declaration
        .get("status")
        .and_then(serde_json::Value::as_str)
        == Some("blocked")
        || blocker.is_some()
    {
        return ProjectProofArtifactEvidence {
            status: ProofArtifactState::Blocked,
            detail: blocker::project_proof_artifact_blocker_detail(label, blocker.as_ref())
                .unwrap_or_else(|| format!("{label} is blocked")),
            artifact: None,
            blocker,
        };
    }

    let artifact_value = declaration.get("artifact").unwrap_or(declaration);
    let path =
        string_field(artifact_value, "path").or_else(|| string_field(artifact_value, "file"));
    let resolved_path = path
        .as_deref()
        .and_then(|path| resolve_run_dir_artifact_path(evidence_root, run_dir, path).ok());
    let reported_path = path
        .as_deref()
        .and_then(|path| reportable_artifact_path(evidence_root, path, resolved_path.as_deref()));
    let declared_size = artifact_value
        .get("size_bytes")
        .or_else(|| artifact_value.get("sizeBytes"))
        .and_then(serde_json::Value::as_u64);
    let observed_size = resolved_path
        .as_ref()
        .and_then(|path| fs::metadata(path).ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len());
    let artifact = ProjectProofArtifactInfo {
        path: reported_path,
        size_bytes: declared_size.or(observed_size),
        sha256: string_field(artifact_value, "sha256"),
        metadata: artifact_value.get("metadata").cloned(),
    };

    if resolved_path.is_some() || artifact_has_metadata(&artifact) {
        return ProjectProofArtifactEvidence {
            status: ProofArtifactState::Present,
            detail: present_artifact_detail(label, &artifact),
            artifact: Some(artifact),
            blocker: None,
        };
    }

    ProjectProofArtifactEvidence::missing(label)
}

fn reportable_artifact_path(
    evidence_root: &Path,
    artifact_path: &str,
    resolved_path: Option<&Path>,
) -> Option<String> {
    let path = Path::new(artifact_path);
    if path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Some(path.to_string_lossy().replace('\\', "/"));
    }

    let resolved_path = resolved_path?;
    let root = evidence_root.canonicalize().ok()?;
    resolved_path
        .strip_prefix(root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn artifact_has_metadata(artifact: &ProjectProofArtifactInfo) -> bool {
    artifact.path.is_some()
        || artifact.size_bytes.is_some()
        || artifact.sha256.is_some()
        || artifact.metadata.is_some()
}

fn present_artifact_detail(label: &str, artifact: &ProjectProofArtifactInfo) -> String {
    let mut parts = Vec::new();
    if let Some(path) = &artifact.path {
        parts.push(path.clone());
    }
    if let Some(size_bytes) = artifact.size_bytes {
        parts.push(format!("{size_bytes} bytes"));
    }
    if let Some(sha256) = &artifact.sha256 {
        parts.push(format!("sha256: {sha256}"));
    }
    if let Some(metadata) = metadata_summary(artifact.metadata.as_ref()) {
        parts.push(format!("metadata: {metadata}"));
    }

    if parts.is_empty() {
        format!("{label} is present as artifact availability only")
    } else {
        format!(
            "{label} is present as artifact availability only: {}",
            parts.join("; ")
        )
    }
}

fn metadata_summary(metadata: Option<&serde_json::Value>) -> Option<String> {
    let object = metadata?.as_object()?;
    let mut pairs = object
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .map(|value| format!("{key}={value}"))
                .or_else(|| value.as_bool().map(|value| format!("{key}={value}")))
                .or_else(|| value.as_i64().map(|value| format!("{key}={value}")))
        })
        .collect::<Vec<_>>();
    pairs.sort();
    (!pairs.is_empty()).then(|| pairs.join(", "))
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

fn string_field(json: &serde_json::Value, key: &str) -> Option<String> {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
