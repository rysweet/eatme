use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

mod blocker;

const RUN_WINDOW_AFTER_DISPATCH_SCREENSHOT: &str = "screenshots/run-window-after-dispatch.png";
const DESKTOP_RUN_PIXEL_BOUNDARY: &str = "run-window-evidence/desktop-run-pixel-boundary.json";
const DESKTOP_RUN_PIXEL_OBSERVATION: &str =
    "run-window-evidence/desktop-run-pixel-observation.json";
const MISSING_VISIBLE_DESKTOP_EVIDENCE: &str = "missing visible desktop rendering evidence after Run-frame and VM statement execution; expected screenshots/run-window-after-dispatch.png under the comparison evidence root";
const MISSING_PIXEL_BOUNDARY_EVIDENCE: &str = "missing desktop Run pixel-boundary evidence; expected run-window-evidence/desktop-run-pixel-boundary.json under the comparison evidence root";
const MISSING_PIXEL_OBSERVATION_EVIDENCE: &str = "missing desktop Run pixel-observation evidence; expected run-window-evidence/desktop-run-pixel-observation.json under the comparison evidence root";

pub(crate) struct DesktopEvidenceCheck {
    pub(crate) observed: bool,
    pub(crate) artifact: Option<PathBuf>,
    pub(crate) issue: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DesktopRunPixelBoundaryEvidence {
    pub status: String,
    pub artifact: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DesktopRunPixelObservationEvidence {
    pub status: String,
    pub artifact: Option<String>,
    pub detail: String,
    pub screenshot: Option<serde_json::Value>,
    pub sample: Option<serde_json::Value>,
    pub blocker: Option<serde_json::Value>,
    pub component_state: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<serde_json::Value>,
}

impl DesktopRunPixelBoundaryEvidence {
    pub(crate) fn issue_when_missing_or_invalid(&self) -> Option<String> {
        match self.status.as_str() {
            "missing" | "invalid" => Some(self.detail.clone()),
            _ => None,
        }
    }
}

impl DesktopRunPixelObservationEvidence {
    pub(crate) fn issue_when_missing_or_invalid(&self) -> Option<String> {
        match self.status.as_str() {
            "missing" | "invalid" => Some(self.detail.clone()),
            _ => None,
        }
    }

    pub fn next_actionable_blocker(&self) -> Option<String> {
        blocker::next_actionable_pixel_observation_blocker(self)
    }
}

impl DesktopEvidenceCheck {
    pub(crate) fn issue_when_missing(self) -> Option<String> {
        if self.observed {
            debug_assert!(self.artifact.is_some());
            None
        } else {
            self.issue
        }
    }
}

pub(crate) fn check_visible_desktop_evidence(
    evidence_root: &Path,
    ui_action_contract_path: &Path,
) -> DesktopEvidenceCheck {
    let Some(run_dir) = ui_action_contract_path.parent() else {
        return missing();
    };
    let candidate = run_dir.join(RUN_WINDOW_AFTER_DISPATCH_SCREENSHOT);
    let Ok(root) = evidence_root.canonicalize() else {
        return missing();
    };
    let Ok(artifact) = candidate.canonicalize() else {
        return missing();
    };
    if !artifact.starts_with(root) {
        return missing();
    }
    let Ok(metadata) = fs::metadata(&artifact) else {
        return missing();
    };
    if !metadata.is_file() || metadata.len() == 0 || fs::File::open(&artifact).is_err() {
        return missing();
    }
    DesktopEvidenceCheck {
        observed: true,
        artifact: Some(artifact),
        issue: None,
    }
}

pub(crate) fn check_pixel_boundary_evidence(
    evidence_root: &Path,
    ui_action_contract_path: &Path,
) -> DesktopRunPixelBoundaryEvidence {
    let Some(run_dir) = ui_action_contract_path.parent() else {
        return missing_pixel_boundary();
    };
    let candidate = run_dir.join(DESKTOP_RUN_PIXEL_BOUNDARY);
    let Ok(root) = evidence_root.canonicalize() else {
        return missing_pixel_boundary();
    };
    let Ok(artifact) = candidate.canonicalize() else {
        return missing_pixel_boundary();
    };
    if !artifact.starts_with(root) {
        return missing_pixel_boundary();
    }
    let Ok(text) = fs::read_to_string(&artifact) else {
        return invalid_pixel_boundary(
            Some(artifact),
            "desktop Run pixel-boundary evidence exists but is not readable",
        );
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return invalid_pixel_boundary(
            Some(artifact),
            "desktop Run pixel-boundary evidence exists but is not valid JSON",
        );
    };
    if json
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.alice-desktop-run-pixel-boundary/v1")
    {
        return invalid_pixel_boundary(
            Some(artifact),
            "desktop Run pixel-boundary evidence has the wrong schema_version",
        );
    }
    let Some(status) = json.get("status").and_then(serde_json::Value::as_str) else {
        return invalid_pixel_boundary(
            Some(artifact),
            "desktop Run pixel-boundary evidence is missing status field",
        );
    };
    let detail = json
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("desktop Run pixel-boundary evidence was read")
        .to_string();
    DesktopRunPixelBoundaryEvidence {
        status: status.into(),
        artifact: Some(artifact.display().to_string()),
        detail,
    }
}

pub(crate) fn check_pixel_observation_evidence(
    evidence_root: &Path,
    ui_action_contract_path: &Path,
) -> DesktopRunPixelObservationEvidence {
    let Some(run_dir) = ui_action_contract_path.parent() else {
        return missing_pixel_observation();
    };
    let candidate = run_dir.join(DESKTOP_RUN_PIXEL_OBSERVATION);
    let Ok(root) = evidence_root.canonicalize() else {
        return missing_pixel_observation();
    };
    let Ok(artifact) = candidate.canonicalize() else {
        return missing_pixel_observation();
    };
    if !artifact.starts_with(root) {
        return missing_pixel_observation();
    }
    let Ok(text) = fs::read_to_string(&artifact) else {
        return invalid_pixel_observation(
            Some(artifact),
            "desktop Run pixel-observation evidence exists but is not readable",
        );
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return invalid_pixel_observation(
            Some(artifact),
            "desktop Run pixel-observation evidence exists but is not valid JSON",
        );
    };
    if json
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("eatme.alice-desktop-run-pixel-observation/v1")
    {
        return invalid_pixel_observation(
            Some(artifact),
            "desktop Run pixel-observation evidence has the wrong schema_version",
        );
    }
    let Some(status) = json.get("status").and_then(serde_json::Value::as_str) else {
        return invalid_pixel_observation(
            Some(artifact),
            "desktop Run pixel-observation evidence is missing status field",
        );
    };
    let detail = pixel_observation_detail(&json);
    DesktopRunPixelObservationEvidence {
        status: status.into(),
        artifact: Some(artifact.display().to_string()),
        detail,
        screenshot: json.get("screenshot").cloned(),
        sample: json.get("sample").cloned(),
        blocker: json.get("blocker").cloned(),
        component_state: json.get("component_state").cloned(),
        next_action: json
            .get("next_action")
            .or_else(|| json.get("nextAction"))
            .cloned(),
    }
}

fn missing() -> DesktopEvidenceCheck {
    DesktopEvidenceCheck {
        observed: false,
        artifact: None,
        issue: Some(MISSING_VISIBLE_DESKTOP_EVIDENCE.into()),
    }
}

fn missing_pixel_boundary() -> DesktopRunPixelBoundaryEvidence {
    DesktopRunPixelBoundaryEvidence {
        status: "missing".into(),
        artifact: None,
        detail: MISSING_PIXEL_BOUNDARY_EVIDENCE.into(),
    }
}

fn missing_pixel_observation() -> DesktopRunPixelObservationEvidence {
    DesktopRunPixelObservationEvidence {
        status: "missing".into(),
        artifact: None,
        detail: MISSING_PIXEL_OBSERVATION_EVIDENCE.into(),
        screenshot: None,
        sample: None,
        blocker: None,
        component_state: None,
        next_action: None,
    }
}

fn invalid_pixel_boundary(
    artifact: Option<PathBuf>,
    detail: &str,
) -> DesktopRunPixelBoundaryEvidence {
    DesktopRunPixelBoundaryEvidence {
        status: "invalid".into(),
        artifact: artifact.map(|path| path.display().to_string()),
        detail: detail.into(),
    }
}

fn invalid_pixel_observation(
    artifact: Option<PathBuf>,
    detail: &str,
) -> DesktopRunPixelObservationEvidence {
    DesktopRunPixelObservationEvidence {
        status: "invalid".into(),
        artifact: artifact.map(|path| path.display().to_string()),
        detail: detail.into(),
        screenshot: None,
        sample: None,
        blocker: None,
        component_state: None,
        next_action: None,
    }
}

fn pixel_observation_detail(json: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    if let Some(claim) = json.get("claim").and_then(serde_json::Value::as_str) {
        parts.push(claim.to_string());
    } else if let Some(reason) = json
        .get("blocker")
        .and_then(|blocker| blocker.get("reason"))
        .and_then(serde_json::Value::as_str)
    {
        parts.push(reason.to_string());
    } else if let Some(reason) = json.get("reason").and_then(serde_json::Value::as_str) {
        parts.push(reason.to_string());
    } else {
        parts.push("desktop Run pixel-observation evidence was read".into());
    }
    if let Some(file) = json
        .get("screenshot")
        .and_then(|screenshot| screenshot.get("file"))
        .and_then(serde_json::Value::as_str)
    {
        parts.push(format!("screenshot: {file}"));
    }
    if let Some(argb) = json
        .get("sample")
        .and_then(|sample| sample.get("argb"))
        .and_then(serde_json::Value::as_str)
    {
        parts.push(format!("center pixel: {argb}"));
    }
    if let Some(codes) = blocker_codes(json) {
        parts.push(format!("blocker codes: {codes}"));
    }
    parts.join("; ")
}

fn blocker_codes(json: &serde_json::Value) -> Option<String> {
    let codes = json
        .get("blocker")
        .and_then(|blocker| blocker.get("codes"))
        .and_then(serde_json::Value::as_array)?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    (!codes.is_empty()).then(|| codes.join(", "))
}

pub(super) fn resolve_artifact_path(manifest_path: &Path, artifact_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(artifact_path);
    if path.as_os_str().is_empty() {
        bail!("artifact path must not be empty");
    }
    let evidence_root = comparison_evidence_root(manifest_path);

    if path.is_absolute() {
        let artifact = path
            .canonicalize()
            .with_context(|| format!("resolving artifact path {}", path.display()))?;
        let root = evidence_root.canonicalize().with_context(|| {
            format!(
                "resolving comparison evidence root {}",
                evidence_root.display()
            )
        })?;
        if !artifact.starts_with(&root) {
            bail!(
                "absolute artifact path {} must stay under comparison evidence root {}",
                artifact.display(),
                root.display()
            );
        }
        return Ok(artifact);
    }

    reject_unsafe_relative_path(&path)?;
    let root = canonical_evidence_root(&evidence_root)?;
    if let Some(parent) = manifest_path.parent() {
        let candidate = parent.join(&path);
        if candidate.exists() {
            return canonical_artifact_under_root(&candidate, &root);
        }
    }
    let candidate = evidence_root.join(&path);
    if candidate.exists() {
        return canonical_artifact_under_root(&candidate, &root);
    }
    if path.exists() {
        return canonical_artifact_under_root(&path, &root);
    }
    Ok(candidate)
}

pub(super) fn comparison_evidence_root(manifest_path: &Path) -> PathBuf {
    let parent = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    for ancestor in parent.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("comparisons") {
            return ancestor.parent().unwrap_or(parent).to_path_buf();
        }
    }
    parent.to_path_buf()
}

fn reject_unsafe_relative_path(path: &Path) -> Result<()> {
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("relative artifact path must not contain parent, current, or root components");
    }
    Ok(())
}

fn canonical_evidence_root(evidence_root: &Path) -> Result<PathBuf> {
    evidence_root.canonicalize().with_context(|| {
        format!(
            "resolving comparison evidence root {}",
            evidence_root.display()
        )
    })
}

fn canonical_artifact_under_root(candidate: &Path, root: &Path) -> Result<PathBuf> {
    let resolved = candidate
        .canonicalize()
        .with_context(|| format!("resolving artifact path {}", candidate.display()))?;
    if !resolved.starts_with(root) {
        bail!(
            "artifact path {} must stay under comparison evidence root {}",
            resolved.display(),
            root.display()
        );
    }
    Ok(resolved)
}
