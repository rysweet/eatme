use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

const RUN_WINDOW_AFTER_DISPATCH_SCREENSHOT: &str = "screenshots/run-window-after-dispatch.png";
const DESKTOP_RUN_PIXEL_BOUNDARY: &str = "run-window-evidence/desktop-run-pixel-boundary.json";
const MISSING_VISIBLE_DESKTOP_EVIDENCE: &str = "missing visible desktop rendering evidence after Run-frame and VM statement execution; expected screenshots/run-window-after-dispatch.png under the comparison evidence root";
const MISSING_PIXEL_BOUNDARY_EVIDENCE: &str = "missing desktop Run pixel-boundary evidence; expected run-window-evidence/desktop-run-pixel-boundary.json under the comparison evidence root";

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

impl DesktopRunPixelBoundaryEvidence {
    pub(crate) fn issue_when_missing_or_invalid(&self) -> Option<String> {
        match self.status.as_str() {
            "missing" | "invalid" => Some(self.detail.clone()),
            _ => None,
        }
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
    let status = json
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("invalid");
    if status == "invalid" {
        return invalid_pixel_boundary(
            Some(artifact),
            "desktop Run pixel-boundary evidence is missing status",
        );
    }
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
