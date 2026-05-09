use serde::{Serialize, Serializer};
use std::fs;
use std::path::{Path, PathBuf};

use super::evidence_text_contract::next_action_contract_issue;
use super::first_lesson_boundaries::{
    FirstLessonEvidenceBoundary, first_lesson_evidence_boundaries, missing_boundaries,
};
use super::{blocker, resolve_run_dir_artifact_path_under_root};

pub(crate) const DESKTOP_FIRST_LESSON_NEXT_ACTION: &str =
    "run-window-evidence/desktop-first-lesson-next-action.json";
pub(crate) const DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL: &str = "desktop next-action evidence";
const MISSING_FIRST_LESSON_NEXT_ACTION_EVIDENCE: &str = "missing desktop next-action evidence; expected desktop next-action evidence under the comparison evidence root";
const SAVE_PROJECT_PROOF_LABEL: &str = "Save Project proof artifact";
const SELECT_PROJECT_PROOF_LABEL: &str = "Select Project proof artifact";

#[derive(Clone, Debug, Serialize)]
pub struct DesktopFirstLessonNextActionEvidence {
    pub status: String,
    pub artifact: Option<String>,
    pub detail: String,
    pub candidate_actions: Vec<String>,
    pub blocker: Option<serde_json::Value>,
    pub requires_next_evidence: Vec<String>,
    pub does_not_claim: Vec<String>,
    pub save_project_proof_artifact: ProjectProofArtifactEvidence,
    pub select_project_proof_artifact: ProjectProofArtifactEvidence,
    pub evidence_boundaries: Vec<FirstLessonEvidenceBoundary>,
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

    pub(crate) fn boundary_issues(&self) -> Vec<String> {
        self.evidence_boundaries
            .iter()
            .filter_map(FirstLessonEvidenceBoundary::issue_when_invalid)
            .collect()
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
    if !artifact.starts_with(&root) {
        return missing_first_lesson_next_action();
    }
    let Ok(text) = fs::read_to_string(&artifact) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} exists but is not readable"),
        );
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} exists but is not valid JSON"),
        );
    };
    if json
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.alice-desktop-first-lesson-next-action/v1")
    {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} has the wrong schema_version"),
        );
    }
    let Some(status) = json.get("status").and_then(serde_json::Value::as_str) else {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} is missing status field"),
        );
    };
    if status.trim().is_empty() {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} status must not be empty"),
        );
    }
    if let Some(reason) = next_action_contract_issue(&json) {
        return invalid_first_lesson_next_action(
            Some(artifact),
            format!("{DESKTOP_FIRST_LESSON_NEXT_ACTION_LABEL} is invalid: {reason}"),
        );
    }

    DesktopFirstLessonNextActionEvidence {
        status: status.into(),
        artifact: Some(artifact.display().to_string()),
        detail: first_lesson_next_action_detail(&json),
        candidate_actions: string_array(&json, "candidate_actions"),
        blocker: json.get("blocker").cloned(),
        requires_next_evidence: requires_next_evidence(&json),
        does_not_claim: does_not_claim(&json),
        save_project_proof_artifact: project_proof_artifact(
            &json,
            &root,
            run_dir,
            "save_project_proof_artifact",
            "saveProjectProofArtifact",
            SAVE_PROJECT_PROOF_LABEL,
        ),
        select_project_proof_artifact: project_proof_artifact(
            &json,
            &root,
            run_dir,
            "select_project_proof_artifact",
            "selectProjectProofArtifact",
            SELECT_PROJECT_PROOF_LABEL,
        ),
        evidence_boundaries: first_lesson_evidence_boundaries(&json, &root, run_dir),
    }
}

fn missing_first_lesson_next_action() -> DesktopFirstLessonNextActionEvidence {
    first_lesson_next_action_with_empty_proof_artifacts(
        "missing",
        None,
        MISSING_FIRST_LESSON_NEXT_ACTION_EVIDENCE,
    )
}

fn invalid_first_lesson_next_action(
    artifact: Option<PathBuf>,
    detail: impl Into<String>,
) -> DesktopFirstLessonNextActionEvidence {
    first_lesson_next_action_with_empty_proof_artifacts("invalid", artifact, detail)
}

fn first_lesson_next_action_with_empty_proof_artifacts(
    status: &str,
    artifact: Option<PathBuf>,
    detail: impl Into<String>,
) -> DesktopFirstLessonNextActionEvidence {
    DesktopFirstLessonNextActionEvidence {
        status: status.into(),
        artifact: artifact.map(|path| path.display().to_string()),
        detail: detail.into(),
        candidate_actions: Vec::new(),
        blocker: None,
        requires_next_evidence: Vec::new(),
        does_not_claim: Vec::new(),
        save_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            SAVE_PROJECT_PROOF_LABEL,
        ),
        select_project_proof_artifact: ProjectProofArtifactEvidence::missing(
            SELECT_PROJECT_PROOF_LABEL,
        ),
        evidence_boundaries: missing_boundaries(),
    }
}

fn project_proof_artifact(
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
        };
    }
    if declaration
        .get("status")
        .and_then(serde_json::Value::as_str)
        == Some("missing")
    {
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

    let resolved_path = resolved_path?;
    resolved_path
        .strip_prefix(canonical_evidence_root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
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

    if parts.is_empty() {
        format!("{label} is present as a readable artifact")
    } else {
        format!(
            "{label} is present as a readable artifact: {}",
            parts.join("; ")
        )
    }
}

fn first_lesson_next_action_detail(json: &serde_json::Value) -> String {
    json.get("blocker")
        .and_then(|blocker| blocker.get("reason"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| json.get("reason").and_then(serde_json::Value::as_str))
        .unwrap_or("Desktop next-action evidence was read.")
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
    let mut items = string_array(json, "requiresNextEvidence")
        .into_iter()
        .map(user_facing_next_evidence)
        .collect::<Vec<_>>();
    for item in string_array(json, "requires_next_evidence")
        .into_iter()
        .map(user_facing_next_evidence)
    {
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

fn user_facing_next_evidence(value: String) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("desktop save menu readiness or invocation artifact") {
        return "desktop Save menu readiness or invocation artifact".into();
    }
    if lower.contains("code editor/procedure action readiness or invocation artifact") {
        return "code editor/procedure action readiness or invocation artifact".into();
    }
    if lower.contains("save completion evidence") {
        return "Collect explicit Save finish-state evidence before reporting Save completion."
            .into();
    }
    value
}

fn does_not_claim(json: &serde_json::Value) -> Vec<String> {
    let mut items = string_array(json, "doesNotClaim");
    for item in string_array(json, "does_not_claim") {
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
