use serde::Serialize;
use std::fs;
use std::path::Path;

use super::{blocker, resolve_run_dir_artifact_path_under_root};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProofArtifactState {
    Present,
    Missing,
    Blocked,
    Invalid,
}

impl ProofArtifactState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Blocked => "blocked",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectProofArtifactEvidence {
    pub status: ProofArtifactState,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ProjectProofArtifactInfo>,
}

impl ProjectProofArtifactEvidence {
    pub(crate) fn missing(label: &str) -> Self {
        Self::missing_with_detail(format!(
            "{label} is missing; artifact availability was not declared."
        ))
    }

    fn declared_missing(label: &str, declaration: &serde_json::Value) -> Self {
        let detail = string_field(declaration, "reason")
            .or_else(|| string_field(declaration, "detail"))
            .map(|detail| format!("{label} is missing: {detail}"))
            .unwrap_or_else(|| {
                format!("{label} is missing; artifact availability was declared missing.")
            });
        Self::missing_with_detail(detail)
    }

    fn missing_with_detail(detail: impl Into<String>) -> Self {
        Self {
            status: ProofArtifactState::Missing,
            detail: detail.into(),
            artifact: None,
        }
    }

    fn invalid(detail: impl Into<String>) -> Self {
        Self {
            status: ProofArtifactState::Invalid,
            detail: detail.into(),
            artifact: None,
        }
    }

    pub(crate) fn state(&self) -> &'static str {
        self.status.as_str()
    }

    pub(crate) fn issue_when_invalid(&self) -> Option<String> {
        (self.status == ProofArtifactState::Invalid).then(|| self.detail.clone())
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
}

pub(crate) fn project_proof_artifact(
    json: &serde_json::Value,
    canonical_evidence_root: &Path,
    run_dir: &Path,
    snake_key: &str,
    camel_key: &str,
    label: &str,
) -> ProjectProofArtifactEvidence {
    let Some(declaration) = json.get(snake_key).or_else(|| json.get(camel_key)) else {
        return ProjectProofArtifactEvidence::missing(label);
    };

    let declared_status = declaration
        .get("status")
        .and_then(serde_json::Value::as_str)
        .map(str::trim);
    if declaration.get("status").is_some() && declared_status.is_none() {
        return ProjectProofArtifactEvidence::invalid(format!(
            "{label} status must be a non-empty string"
        ));
    }

    let blocker = declaration.get("blocker").cloned();
    match declared_status {
        Some("blocked") => return blocked_project_proof_artifact(label, blocker.as_ref()),
        Some("missing") => {}
        Some("present") | None => {}
        Some("") => {
            return ProjectProofArtifactEvidence::invalid(format!(
                "{label} status must be a non-empty string"
            ));
        }
        Some(status) => {
            return ProjectProofArtifactEvidence::invalid(format!(
                "{label} status {status:?} is unsupported; expected present, missing, or blocked"
            ));
        }
    }
    if blocker.is_some() {
        return blocked_project_proof_artifact(label, blocker.as_ref());
    }
    if declared_status == Some("missing") {
        return ProjectProofArtifactEvidence::declared_missing(label, declaration);
    }

    let artifact_value = declaration.get("artifact").unwrap_or(declaration);
    let Some(path) =
        string_field(artifact_value, "path").or_else(|| string_field(artifact_value, "file"))
    else {
        return ProjectProofArtifactEvidence::missing(label);
    };
    let Ok(resolved_path) =
        resolve_run_dir_artifact_path_under_root(canonical_evidence_root, run_dir, &path)
    else {
        return ProjectProofArtifactEvidence::missing_with_detail(format!(
            "{label} is missing; declared artifact could not be read as a file."
        ));
    };
    let Ok(metadata) = fs::metadata(&resolved_path) else {
        return ProjectProofArtifactEvidence::missing_with_detail(format!(
            "{label} is missing; declared artifact could not be read as a file."
        ));
    };
    if !metadata.is_file() || fs::File::open(&resolved_path).is_err() {
        return ProjectProofArtifactEvidence::missing_with_detail(format!(
            "{label} is missing; declared artifact could not be read as a file."
        ));
    }
    let reported_path =
        reportable_artifact_path(canonical_evidence_root, &path, Some(&resolved_path));
    let declared_size = artifact_value
        .get("size_bytes")
        .or_else(|| artifact_value.get("sizeBytes"))
        .and_then(serde_json::Value::as_u64);
    let size_bytes = declared_size.or(Some(metadata.len()));
    let artifact = ProjectProofArtifactInfo {
        path: reported_path,
        size_bytes,
        sha256: string_field(artifact_value, "sha256"),
    };

    ProjectProofArtifactEvidence {
        status: ProofArtifactState::Present,
        detail: present_artifact_detail(label, &artifact),
        artifact: Some(artifact),
    }
}

fn blocked_project_proof_artifact(
    label: &str,
    blocker: Option<&serde_json::Value>,
) -> ProjectProofArtifactEvidence {
    ProjectProofArtifactEvidence {
        status: ProofArtifactState::Blocked,
        detail: blocker::project_proof_artifact_blocker_detail(label, blocker)
            .unwrap_or_else(|| format!("{label} is blocked")),
        artifact: None,
    }
}

fn reportable_artifact_path(
    canonical_evidence_root: &Path,
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

    resolved_path?
        .strip_prefix(canonical_evidence_root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn present_artifact_detail(label: &str, artifact: &ProjectProofArtifactInfo) -> String {
    let parts = [
        artifact.path.clone(),
        artifact.size_bytes.map(|size| format!("{size} bytes")),
        artifact.sha256.as_ref().map(|sha| format!("sha256: {sha}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if parts.is_empty() {
        return format!("{label} is present as a readable artifact");
    }

    format!(
        "{label} is present as a readable artifact: {}",
        parts.join("; ")
    )
}

fn string_field(json: &serde_json::Value, key: &str) -> Option<String> {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
