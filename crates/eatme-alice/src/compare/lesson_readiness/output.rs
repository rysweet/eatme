use super::{
    LessonActionAssertionEvidence, LessonSessionNoGoContract,
    progress::{LessonReadinessEvidenceProgress, progress_item_id},
};
use crate::compare::desktop_evidence::{
    DesktopFirstLessonNextActionEvidence, DesktopRunPixelBoundaryEvidence,
    DesktopRunPixelObservationEvidence, FirstLessonEvidenceBoundary, ProjectProofArtifactEvidence,
};
use project_proof_output::{not_yet_shown_detail, progress_item_does_not_prove};
use serde::Serialize;

mod project_proof_output;

#[derive(Clone, Copy)]
struct UnprovenClaim {
    sentence: &'static str,
    non_claim: &'static str,
}

const FULL_ALICE_UI_AUTOMATION: UnprovenClaim = UnprovenClaim {
    sentence: "Full Alice UI automation is not proven.",
    non_claim: "Full Alice UI automation",
};
const GRADING: UnprovenClaim = UnprovenClaim {
    sentence: "Grading is not proven.",
    non_claim: "grading",
};
const CREATIVE_ASSESSMENT: UnprovenClaim = UnprovenClaim {
    sentence: "Creative assessment is not proven.",
    non_claim: "creative assessment",
};
const VISIBLE_RENDERING_CORRECTNESS: UnprovenClaim = UnprovenClaim {
    sentence: "Visible rendering correctness is not proven.",
    non_claim: "visible rendering correctness",
};
const SAVE_COMPLETION: UnprovenClaim = UnprovenClaim {
    sentence: "Save completion is not proven.",
    non_claim: "Save completion",
};
const FIRST_LESSON_COMPLETION: UnprovenClaim = UnprovenClaim {
    sentence: "First-lesson completion is not proven.",
    non_claim: "first-lesson completion",
};

const UNPROVEN_CLAIMS: &[UnprovenClaim] = &[
    FULL_ALICE_UI_AUTOMATION,
    GRADING,
    CREATIVE_ASSESSMENT,
    VISIBLE_RENDERING_CORRECTNESS,
    SAVE_COMPLETION,
    FIRST_LESSON_COMPLETION,
];

const LEGACY_LIMITATIONS: &[&str] = &[
    "does not prove full Alice UI automation",
    "does not automate complete instructor assignment creation",
    "does not automate complete student lesson consumption",
    "does not perform creative assessment",
    "does not grade student worlds",
    "does not prove visible rendering correctness",
    "does not prove first-lesson completion",
    "does not prove broad Alice compatibility beyond the selected scenario",
];

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionReadinessEnvelope {
    pub scenario_id: Option<String>,
    pub role: String,
    pub status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
}

pub(super) struct ReadinessOutput {
    pub status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
    pub lesson_session_readiness: LessonSessionReadinessEnvelope,
    pub role_readiness: Vec<LessonSessionReadinessEnvelope>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadinessEvidenceItem {
    pub id: String,
    pub state: String,
    pub summary: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub does_not_prove: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DesktopNextActionSummary {
    pub status: String,
    pub summary: String,
    pub candidate_actions: Vec<String>,
    pub requires_next_evidence: Vec<String>,
    pub observations: Vec<String>,
    pub does_not_prove: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LessonTargetEvidence {
    pub role: String,
    pub target_id: Option<String>,
    pub target_status: Option<String>,
    pub failure_category: Option<String>,
    pub launch_manifest_present: bool,
    pub ui_action_contract_path: Option<String>,
    pub ui_action_contract_readable: bool,
    pub desktop_run_pixel_boundary: Option<DesktopRunPixelBoundaryEvidence>,
    pub desktop_run_pixel_observation: Option<DesktopRunPixelObservationEvidence>,
    pub desktop_first_lesson_next_action: Option<DesktopFirstLessonNextActionEvidence>,
    pub action_assertions: Vec<LessonActionAssertionEvidence>,
    pub required_actions: Vec<String>,
    pub missing_assertions: Vec<String>,
    pub missing_required_actions: Vec<String>,
    pub blockers: Vec<LessonTargetEvidenceBlocker>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct LessonTargetEvidenceBlocker {
    pub code: &'static str,
    pub action: String,
    pub reason: String,
}

pub(super) fn build_readiness_output(
    scenario_id: Option<&str>,
    readiness_status: &str,
    has_issues: bool,
    no_go_contracts: Vec<LessonSessionNoGoContract>,
    default_scenario_id: &str,
) -> ReadinessOutput {
    let status = normalized_readiness_status(readiness_status).to_string();
    let blocked_reason = (status == "blocked").then(|| readiness_status.to_string());
    let required_evidence = required_evidence();
    let human_summary = human_summary(
        scenario_id,
        &status,
        blocked_reason.as_deref(),
        has_issues,
        default_scenario_id,
    );
    let role_readiness = ["instructor", "student"]
        .into_iter()
        .map(|role| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: role.into(),
            status: status.clone(),
            blocked_reason: blocked_reason.clone(),
            human_summary: human_summary.clone(),
            required_evidence: required_evidence.clone(),
            no_go_contracts: no_go_contracts.clone(),
        })
        .collect::<Vec<_>>();
    let lesson_session_readiness = role_readiness
        .iter()
        .find(|readiness| readiness.role == "student")
        .cloned()
        .unwrap_or_else(|| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: "student".into(),
            status: status.clone(),
            blocked_reason: blocked_reason.clone(),
            human_summary: human_summary.clone(),
            required_evidence: required_evidence.clone(),
            no_go_contracts: no_go_contracts.clone(),
        });

    ReadinessOutput {
        status,
        blocked_reason,
        human_summary,
        required_evidence,
        no_go_contracts,
        lesson_session_readiness,
        role_readiness,
    }
}

fn normalized_readiness_status(readiness_status: &str) -> &'static str {
    match readiness_status {
        "ready" => "ready",
        "blocked_until_ui_automation" => "blocked",
        _ => "not_ready",
    }
}

fn required_evidence() -> Vec<String> {
    [
        "comparison-manifest.json with baseline and modernized targets",
        "launch evidence for each target",
        "modernized Run-window evidence",
        "modernized desktop-run-pixel-boundary.json status",
        "modernized desktop-run-pixel-observation.json status",
        "modernized desktop execution evidence",
        "screenshot, log, and window artifacts",
        "automation scenario action evidence",
        "Save Project proof artifact",
        "Select Project proof artifact",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn human_summary(
    scenario_id: Option<&str>,
    status: &str,
    blocked_reason: Option<&str>,
    has_issues: bool,
    default_scenario_id: &str,
) -> String {
    let scenario = scenario_id.unwrap_or(default_scenario_id);
    match status {
        "ready" => format!(
            "{scenario} has bounded comparison and UI action evidence with no accepted blockers."
        ),
        "blocked" => format!(
            "{scenario} has launch and automation scenario action evidence but is blocked until deterministic desktop UI automation exists ({reason}).",
            reason = blocked_reason.unwrap_or("blocked")
        ),
        "not_ready" if has_issues => format!(
            "{scenario} readiness evidence is not ready because required comparison or UI action evidence is missing, invalid, stale, or inconsistent."
        ),
        _ => format!("{scenario} readiness evidence is not ready."),
    }
}

pub(super) fn shown_evidence(
    progress: &LessonReadinessEvidenceProgress,
    boundaries: &[FirstLessonEvidenceBoundary],
) -> Vec<ReadinessEvidenceItem> {
    let mut items = progress
        .items
        .iter()
        .filter(|item| item.state == "present")
        .map(|item| ReadinessEvidenceItem {
            id: progress_item_id(&item.evidence),
            state: item.state.clone(),
            summary: format!("{} is shown.", user_facing_evidence_label(&item.evidence)),
            detail: item.detail.clone(),
            does_not_prove: Vec::new(),
        })
        .collect::<Vec<_>>();

    for boundary in boundaries
        .iter()
        .filter(|boundary| boundary.status == "present")
    {
        items.push(boundary_readiness_item(boundary, true));
    }

    items
}

pub(super) fn not_yet_shown(
    progress: &LessonReadinessEvidenceProgress,
    boundaries: &[FirstLessonEvidenceBoundary],
) -> Vec<ReadinessEvidenceItem> {
    let mut items = Vec::new();

    for boundary in boundaries
        .iter()
        .filter(|boundary| boundary.status != "present")
    {
        items.push(boundary_readiness_item(boundary, false));
    }

    for item in progress.items.iter().filter(|item| item.state != "present") {
        let id = progress_item_id(&item.evidence);
        if items
            .iter()
            .any(|existing: &ReadinessEvidenceItem| existing.id == id)
        {
            continue;
        }
        items.push(ReadinessEvidenceItem {
            id,
            state: item.state.clone(),
            summary: format!(
                "{} is not yet shown.",
                user_facing_evidence_label(&item.evidence)
            ),
            detail: not_yet_shown_detail(&item.evidence, &item.state, &item.detail),
            does_not_prove: progress_item_does_not_prove(&item.evidence),
        });
    }

    items
}

pub(super) fn unproven_claims() -> Vec<String> {
    UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence.to_string())
        .collect()
}

pub(super) fn limitations() -> Vec<String> {
    UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence)
        .chain(LEGACY_LIMITATIONS.iter().copied())
        .map(str::to_string)
        .collect()
}

pub(super) fn desktop_next_action_summary(
    target_evidence: &[LessonTargetEvidence],
) -> Option<DesktopNextActionSummary> {
    let evidence = target_evidence
        .iter()
        .find(|target| target.role == "modernized")
        .and_then(|target| target.desktop_first_lesson_next_action.as_ref())?;
    if evidence.artifact.is_none() || matches!(evidence.status.as_str(), "missing" | "invalid") {
        return None;
    }

    let mut observations = Vec::new();
    observations.push(format!(
        "Desktop next-action evidence was shown with status {}.",
        plain_status(&evidence.status)
    ));
    add_proof_artifact_observation(
        &mut observations,
        "Save option/action evidence",
        &evidence.save_project_proof_artifact,
    );
    add_proof_artifact_observation(
        &mut observations,
        "Select Project option/action evidence",
        &evidence.select_project_proof_artifact,
    );

    Some(DesktopNextActionSummary {
        status: evidence.status.clone(),
        summary: "Desktop next-action evidence was shown as an observation only.".into(),
        candidate_actions: evidence.candidate_actions.clone(),
        requires_next_evidence: evidence.requires_next_evidence.clone(),
        observations,
        does_not_prove: desktop_next_action_non_claims(evidence),
    })
}

fn boundary_readiness_item(
    boundary: &FirstLessonEvidenceBoundary,
    shown: bool,
) -> ReadinessEvidenceItem {
    let (summary, does_not_prove) = if shown {
        (
            shown_boundary_summary(boundary),
            boundary_does_not_prove(boundary),
        )
    } else {
        (
            not_yet_shown_boundary_summary(boundary),
            boundary_does_not_prove(boundary),
        )
    };
    let detail = if shown {
        boundary.detail.clone()
    } else {
        summary.clone()
    };

    ReadinessEvidenceItem {
        id: boundary.id.clone(),
        state: boundary.status.clone(),
        summary,
        detail,
        does_not_prove,
    }
}

fn shown_boundary_summary(boundary: &FirstLessonEvidenceBoundary) -> String {
    match boundary.id.as_str() {
        "save_project" => {
            "Save option/action evidence is shown as observed option/action only.".into()
        }
        _ => format!("{} is shown.", boundary_subject(boundary)),
    }
}

fn not_yet_shown_boundary_summary(boundary: &FirstLessonEvidenceBoundary) -> String {
    match boundary.id.as_str() {
        "save_project" => "Save option/action evidence is not yet shown.".into(),
        _ => format!("{} is not yet shown.", boundary_subject(boundary)),
    }
}

fn boundary_subject(boundary: &FirstLessonEvidenceBoundary) -> String {
    match boundary.id.as_str() {
        "select_project" => "Select Project".into(),
        "procedure_edit" => "Procedure/edit".into(),
        "visible_rendering" => "Visible rendering".into(),
        "grading" => "Grading".into(),
        "creative_assessment" => "Creative assessment".into(),
        "first_lesson_completion" => "First-lesson completion".into(),
        _ => boundary
            .label
            .trim_end_matches(" scenario evidence")
            .to_string(),
    }
}

fn boundary_does_not_prove(boundary: &FirstLessonEvidenceBoundary) -> Vec<String> {
    let mut claims = boundary.does_not_prove.clone();
    if boundary.id == "save_project" {
        push_unique(&mut claims, SAVE_COMPLETION.non_claim);
        push_unique(&mut claims, FIRST_LESSON_COMPLETION.non_claim);
    }
    if boundary.id == "visible_rendering" {
        push_unique(&mut claims, VISIBLE_RENDERING_CORRECTNESS.non_claim);
    }
    claims
}

fn user_facing_evidence_label(evidence: &str) -> String {
    match evidence {
        "Save Project proof artifact" => "Save option/action evidence".into(),
        "Select Project proof artifact" => "Select Project option/action evidence".into(),
        "comparison-manifest.json with baseline and modernized targets" => {
            "Comparison target setup".into()
        }
        "launch evidence for each target" => "Launch evidence for each target".into(),
        "modernized Run-window evidence" => "Modernized Run-window evidence".into(),
        "modernized desktop-run-pixel-boundary.json status" => {
            "Run-window boundary evidence".into()
        }
        "modernized desktop-run-pixel-observation.json status" => {
            "Run-window observation evidence".into()
        }
        "modernized desktop execution evidence" => "Modernized desktop execution evidence".into(),
        "screenshot, log, and window artifacts" => "Screenshot, log, and window evidence".into(),
        "automation scenario action evidence" => "Automation scenario action evidence".into(),
        _ => evidence.to_string(),
    }
}

fn add_proof_artifact_observation(
    observations: &mut Vec<String>,
    label: &str,
    artifact: &ProjectProofArtifactEvidence,
) {
    observations.push(format!(
        "{label} is {} as an observation only.",
        plain_status(artifact.state())
    ));
}

fn desktop_next_action_non_claims(evidence: &DesktopFirstLessonNextActionEvidence) -> Vec<String> {
    let mut claims = UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.non_claim.to_string())
        .collect();
    for claim in &evidence.does_not_claim {
        push_unique(&mut claims, claim);
    }
    claims
}

fn plain_status(status: &str) -> String {
    status.replace('_', " ")
}

fn push_unique(claims: &mut Vec<String>, claim: &str) {
    if !claims.iter().any(|existing| existing == claim) {
        claims.push(claim.to_string());
    }
}

#[cfg(test)]
mod tests;
