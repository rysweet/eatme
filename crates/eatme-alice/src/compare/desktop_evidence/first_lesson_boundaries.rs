use serde::Serialize;
use std::fs;
use std::path::Path;

use super::first_lesson_next_action::ProjectProofArtifactInfo;
use super::resolve_run_dir_artifact_path_under_root;

const DEFAULT_BOUNDARY_SOURCE: &str = "automation_scenario";
const CREATIVE_ASSESSMENT_BOUNDARY_LIMIT: &str = "The report can surface available evidence and suggest next steps for the learner's creative work in this scenario, but it does not grade creativity, judge quality, or mark the lesson complete.";
const MISSING_CREATIVE_ASSESSMENT_BOUNDARY_DETAIL: &str = "Creative assessment scenario evidence is missing. The report can surface available evidence and suggest next steps for the learner's creative work in this scenario, but it does not grade creativity, judge quality, or mark the lesson complete.";

#[derive(Clone, Debug, Serialize)]
pub struct FirstLessonEvidenceBoundary {
    pub id: String,
    pub label: String,
    pub status: String,
    pub source: String,
    pub metadata_state: String,
    pub detail: String,
    pub claim: String,
    pub does_not_prove: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ProjectProofArtifactInfo>,
}

impl FirstLessonEvidenceBoundary {
    pub(crate) fn issue_when_invalid(&self) -> Option<String> {
        (self.status == "invalid").then(|| self.detail.clone())
    }
}

pub fn first_lesson_evidence_boundaries(
    json: &serde_json::Value,
    canonical_evidence_root: &Path,
    run_dir: &Path,
) -> Vec<FirstLessonEvidenceBoundary> {
    boundary_specs()
        .iter()
        .map(|spec| {
            json.get("evidence_boundaries")
                .or_else(|| json.get("evidenceBoundaries"))
                .and_then(serde_json::Value::as_array)
                .and_then(|boundaries| boundary_value(boundaries, spec.id))
                .map(|value| normalize_boundary(spec, value, canonical_evidence_root, run_dir))
                .unwrap_or_else(|| missing_boundary(spec))
        })
        .collect()
}

pub(crate) fn missing_boundaries() -> Vec<FirstLessonEvidenceBoundary> {
    boundary_specs().iter().map(missing_boundary).collect()
}

#[derive(Clone, Copy)]
struct BoundarySpec {
    id: &'static str,
    label: &'static str,
    missing_detail: &'static str,
    metadata_noun: &'static str,
    evidence_noun: &'static str,
    present_claim: &'static str,
    does_not_prove: &'static [&'static str],
}

fn boundary_specs() -> &'static [BoundarySpec] {
    &[
        BoundarySpec {
            id: "select_project",
            label: "Select Project scenario evidence",
            missing_detail: "Select Project scenario evidence is missing.",
            metadata_noun: "Select Project scenario metadata",
            evidence_noun: "Select Project scenario evidence",
            present_claim: "The Select Project boundary has auditable scenario evidence.",
            does_not_prove: &["full Alice UI automation", "first-lesson completion"],
        },
        BoundarySpec {
            id: "procedure_edit",
            label: "Procedure/edit scenario evidence",
            missing_detail: "Procedure/edit scenario evidence is missing.",
            metadata_noun: "Procedure/edit scenario metadata",
            evidence_noun: "procedure/edit scenario evidence",
            present_claim: "The procedure/edit boundary has auditable scenario evidence.",
            does_not_prove: &["code correctness", "grading", "first-lesson completion"],
        },
        BoundarySpec {
            id: "save_project",
            label: "Save scenario evidence",
            missing_detail: "Save boundary evidence is missing.",
            metadata_noun: "Save scenario metadata",
            evidence_noun: "Save boundary evidence",
            present_claim: "Save action evidence is present for this scenario boundary.",
            does_not_prove: &[
                "desktop Save completion",
                "grading",
                "creative assessment",
                "first-lesson completion",
            ],
        },
        BoundarySpec {
            id: "visible_rendering",
            label: "Visible rendering scenario evidence",
            missing_detail: "Visible rendering scenario evidence is missing.",
            metadata_noun: "Visible rendering scenario metadata",
            evidence_noun: "visible rendering scenario evidence",
            present_claim: "Visible rendering was observed for this scenario boundary.",
            does_not_prove: &[
                "visible rendering correctness",
                "creative assessment",
                "first-lesson completion",
            ],
        },
        BoundarySpec {
            id: "grading",
            label: "Grading scenario evidence",
            missing_detail: "Grading scenario evidence is missing.",
            metadata_noun: "Grading scenario metadata",
            evidence_noun: "grading scenario evidence",
            present_claim: "The grading boundary has auditable scenario evidence.",
            does_not_prove: &["creative assessment", "first-lesson completion"],
        },
        BoundarySpec {
            id: "creative_assessment",
            label: "Creative assessment scenario evidence",
            missing_detail: MISSING_CREATIVE_ASSESSMENT_BOUNDARY_DETAIL,
            metadata_noun: "Creative assessment scenario metadata",
            evidence_noun: "creative assessment scenario evidence",
            present_claim: CREATIVE_ASSESSMENT_BOUNDARY_LIMIT,
            does_not_prove: &["instructor judgment", "first-lesson completion"],
        },
        BoundarySpec {
            id: "first_lesson_completion",
            label: "First-lesson completion scenario evidence",
            missing_detail: "First-lesson completion scenario evidence is missing.",
            metadata_noun: "First-lesson completion scenario metadata",
            evidence_noun: "first-lesson completion scenario evidence",
            present_claim: "The first-lesson completion boundary has auditable scenario evidence.",
            does_not_prove: &["full Alice UI automation", "creative quality"],
        },
    ]
}

fn normalize_boundary(
    spec: &BoundarySpec,
    value: &serde_json::Value,
    canonical_evidence_root: &Path,
    run_dir: &Path,
) -> FirstLessonEvidenceBoundary {
    let source = string_field(value, "source").unwrap_or_else(|| DEFAULT_BOUNDARY_SOURCE.into());
    let declared_status = string_field(value, "status");
    let metadata_state =
        string_field(value, "metadata_state").or_else(|| string_field(value, "metadataState"));
    let artifact_result = boundary_artifact(value, canonical_evidence_root, run_dir);

    if artifact_result.is_err() {
        return invalid_boundary(
            spec,
            "artifact path must stay under the scenario evidence root",
        );
    }

    match declared_status.as_deref() {
        Some("present") => FirstLessonEvidenceBoundary {
            id: spec.id.into(),
            label: spec.label.into(),
            status: "present".into(),
            source,
            metadata_state: metadata_state.unwrap_or_else(|| "observed".into()),
            detail: scenario_focused_detail(spec, value)
                .unwrap_or_else(|| format!("{} is present.", spec.label)),
            claim: scenario_focused_claim(spec, value).unwrap_or_else(|| spec.present_claim.into()),
            does_not_prove: merged_does_not_prove(spec, value),
            artifact: artifact_result.ok().flatten(),
        },
        Some("missing") | Some("blocked") | Some("invalid") => {
            let status = declared_status.unwrap();
            FirstLessonEvidenceBoundary {
                id: spec.id.into(),
                label: spec.label.into(),
                status: status.clone(),
                source,
                metadata_state: metadata_state.unwrap_or_else(|| status.clone()),
                detail: if status == "invalid" {
                    format!("{} is invalid.", spec.label)
                } else {
                    scenario_focused_detail(spec, value)
                        .unwrap_or_else(|| spec.missing_detail.into())
                },
                claim: missing_claim(spec),
                does_not_prove: merged_does_not_prove(spec, value),
                artifact: None,
            }
        }
        Some("declared") | Some("observed") => metadata_only_boundary(
            spec,
            &source,
            declared_status.as_deref().unwrap_or("observed"),
            value,
        ),
        Some(_) | None => invalid_boundary(spec, "unsupported or missing boundary status"),
    }
}

fn metadata_only_boundary(
    spec: &BoundarySpec,
    source: &str,
    metadata_state: &str,
    value: &serde_json::Value,
) -> FirstLessonEvidenceBoundary {
    let verb = match metadata_state {
        "declared" => "declared",
        _ => "observed",
    };
    FirstLessonEvidenceBoundary {
        id: spec.id.into(),
        label: spec.label.into(),
        status: "missing".into(),
        source: source.into(),
        metadata_state: verb.into(),
        detail: format!(
            "{} was {verb}; {} is missing.",
            spec.metadata_noun, spec.evidence_noun
        ),
        claim: missing_claim(spec),
        does_not_prove: merged_does_not_prove(spec, value),
        artifact: None,
    }
}

fn missing_boundary(spec: &BoundarySpec) -> FirstLessonEvidenceBoundary {
    FirstLessonEvidenceBoundary {
        id: spec.id.into(),
        label: spec.label.into(),
        status: "missing".into(),
        source: DEFAULT_BOUNDARY_SOURCE.into(),
        metadata_state: "missing".into(),
        detail: spec.missing_detail.into(),
        claim: missing_claim(spec),
        does_not_prove: spec
            .does_not_prove
            .iter()
            .map(|value| (*value).into())
            .collect(),
        artifact: None,
    }
}

fn invalid_boundary(spec: &BoundarySpec, reason: &str) -> FirstLessonEvidenceBoundary {
    FirstLessonEvidenceBoundary {
        id: spec.id.into(),
        label: spec.label.into(),
        status: "invalid".into(),
        source: DEFAULT_BOUNDARY_SOURCE.into(),
        metadata_state: "invalid".into(),
        detail: format!("{} is invalid: {reason}.", spec.label),
        claim: missing_claim(spec),
        does_not_prove: spec
            .does_not_prove
            .iter()
            .map(|value| (*value).into())
            .collect(),
        artifact: None,
    }
}

fn missing_claim(spec: &BoundarySpec) -> String {
    format!(
        "{} is not proven; automation scenarios must collect explicit evidence before this can be reported as present.",
        spec.label
    )
}

fn boundary_artifact(
    value: &serde_json::Value,
    canonical_evidence_root: &Path,
    run_dir: &Path,
) -> Result<Option<ProjectProofArtifactInfo>, ()> {
    let Some(artifact_value) = value.get("artifact") else {
        return Ok(None);
    };
    let Some(path) =
        string_field(artifact_value, "path").or_else(|| string_field(artifact_value, "file"))
    else {
        return Ok(None);
    };
    let resolved =
        resolve_run_dir_artifact_path_under_root(canonical_evidence_root, run_dir, &path)
            .map_err(|_| ())?;
    let reported_path = reportable_artifact_path(canonical_evidence_root, &path, Some(&resolved));
    Ok(Some(ProjectProofArtifactInfo {
        path: reported_path,
        size_bytes: artifact_value
            .get("size_bytes")
            .or_else(|| artifact_value.get("sizeBytes"))
            .and_then(serde_json::Value::as_u64)
            .or_else(|| fs::metadata(&resolved).ok().map(|metadata| metadata.len())),
        sha256: string_field(artifact_value, "sha256"),
    }))
}

fn scenario_focused_detail(spec: &BoundarySpec, value: &serde_json::Value) -> Option<String> {
    string_field(value, "detail").filter(|detail| {
        let save_boundary_wording = is_save_boundary_wording(spec, detail);
        (detail.contains("scenario")
            || detail.contains("automation scenarios")
            || save_boundary_wording)
            && (!contains_implementation_jargon(detail) || save_boundary_wording)
            && !detail.to_ascii_lowercase().contains(" is proven")
            && !unsafe_boundary_wording(spec, detail)
    })
}

fn scenario_focused_claim(spec: &BoundarySpec, value: &serde_json::Value) -> Option<String> {
    string_field(value, "claim").filter(|claim| {
        !contains_implementation_jargon(claim)
            && !claim.to_ascii_lowercase().contains(" is proven")
            && !unsafe_boundary_wording(spec, claim)
    })
}

fn unsafe_boundary_wording(spec: &BoundarySpec, value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    spec.id == "save_project"
        && ["save completion evidence", "bounded save completion"]
            .iter()
            .any(|forbidden| value.contains(forbidden))
}

fn is_save_boundary_wording(spec: &BoundarySpec, value: &str) -> bool {
    spec.id == "save_project"
        && ["Save action evidence", "Save boundary evidence"]
            .iter()
            .any(|required| value.contains(required))
}

fn contains_implementation_jargon(value: &str) -> bool {
    [
        "proof artifact",
        "ui-action-contract",
        "desktop-run-pixel",
        "desktop-first-lesson-next-action",
        "action_id",
        "no_go",
        "RabbitHole",
    ]
    .iter()
    .any(|forbidden| value.contains(forbidden))
}

fn merged_does_not_prove(spec: &BoundarySpec, value: &serde_json::Value) -> Vec<String> {
    let mut claims = spec
        .does_not_prove
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    for claim in string_array(value, "does_not_prove")
        .into_iter()
        .chain(string_array(value, "doesNotProve"))
    {
        if !claims.iter().any(|existing| existing == &claim) {
            claims.push(claim);
        }
    }
    claims
}

fn boundary_value<'a>(
    boundaries: &'a [serde_json::Value],
    id: &str,
) -> Option<&'a serde_json::Value> {
    boundaries
        .iter()
        .find(|boundary| boundary.get("id").and_then(serde_json::Value::as_str) == Some(id))
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
