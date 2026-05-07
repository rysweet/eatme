use super::LessonTargetEvidence;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct LessonReadinessEvidenceProgress {
    pub total_required: usize,
    pub present: usize,
    pub missing: usize,
    pub invalid: usize,
    pub not_observed: usize,
    pub blocked: usize,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_actionable_blocker: Option<String>,
    pub items: Vec<LessonReadinessEvidenceProgressItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LessonReadinessEvidenceProgressItem {
    pub evidence: String,
    pub state: String,
    pub detail: String,
}

pub(super) fn evidence_progress(
    required_evidence: &[String],
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> LessonReadinessEvidenceProgress {
    let mut items = Vec::new();
    let baseline = target_evidence
        .iter()
        .find(|target| target.role == "baseline");
    let modernized = target_evidence
        .iter()
        .find(|target| target.role == "modernized");

    items.push(progress_item(
        &required_evidence[0],
        if baseline.is_some() && modernized.is_some() {
            "present"
        } else {
            "missing"
        },
        "baseline and modernized target entries",
    ));
    items.push(progress_item(
        &required_evidence[1],
        if [baseline, modernized]
            .into_iter()
            .flatten()
            .all(|target| target.launch_manifest_present)
            && baseline.is_some()
            && modernized.is_some()
        {
            "present"
        } else {
            "missing"
        },
        "launch manifest evidence for both targets",
    ));
    items.push(progress_item(
        &required_evidence[2],
        desktop_action_state(modernized, "observe-run-window-after-toolbar-button"),
        "modernized Run-window toolbar observation",
    ));
    items.push(pixel_boundary_progress_item(
        &required_evidence[3],
        modernized,
    ));
    items.push(pixel_observation_progress_item(
        &required_evidence[4],
        modernized,
    ));
    items.push(progress_item(
        &required_evidence[5],
        desktop_action_state(
            modernized,
            "observe-desktop-run-execution-after-toolbar-button",
        ),
        "modernized desktop run execution observation",
    ));
    items.push(progress_item(
        &required_evidence[6],
        if issues.iter().any(|issue| {
            issue.contains("missing visible desktop rendering evidence after Run-frame")
        }) {
            "missing"
        } else {
            "present"
        },
        "screenshot/log/window artifact checks",
    ));
    items.push(progress_item(
        &required_evidence[7],
        ui_action_contract_state(baseline, modernized),
        "readable ui-action-contract.json for both targets",
    ));

    let present = count_state(&items, "present");
    let missing = count_state(&items, "missing");
    let invalid = count_state(&items, "invalid");
    let not_observed = count_state(&items, "not_observed");
    let blocked = count_state(&items, "blocked");
    let total_required = items.len();
    let next_actionable_blocker = next_actionable_blocker(modernized);
    let summary = format!(
        "{present} of {total_required} required evidence items are present; {missing} missing, {invalid} invalid, {not_observed} not observed, {blocked} blocked."
    );

    LessonReadinessEvidenceProgress {
        total_required,
        present,
        missing,
        invalid,
        not_observed,
        blocked,
        summary,
        next_actionable_blocker,
        items,
    }
}

fn progress_item(
    evidence: &str,
    state: &str,
    detail: impl Into<String>,
) -> LessonReadinessEvidenceProgressItem {
    LessonReadinessEvidenceProgressItem {
        evidence: evidence.into(),
        state: state.into(),
        detail: detail.into(),
    }
}

fn count_state(items: &[LessonReadinessEvidenceProgressItem], state: &str) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn next_actionable_blocker(target: Option<&LessonTargetEvidence>) -> Option<String> {
    target.and_then(|target| {
        target
            .desktop_first_lesson_next_action
            .as_ref()
            .and_then(|next_action| next_action.next_actionable_blocker())
            .or_else(|| {
                target
                    .desktop_run_pixel_observation
                    .as_ref()
                    .and_then(|observation| observation.next_actionable_blocker())
            })
    })
}

fn desktop_action_state(target: Option<&LessonTargetEvidence>, action_id: &str) -> &'static str {
    let Some(target) = target else {
        return "missing";
    };
    match target
        .action_assertions
        .iter()
        .find(|action| action.action_id == action_id)
    {
        Some(action) if action.passed => "present",
        Some(_) => "blocked",
        None => "missing",
    }
}

fn pixel_boundary_progress_item(
    evidence: &str,
    target: Option<&LessonTargetEvidence>,
) -> LessonReadinessEvidenceProgressItem {
    let Some(pixel_boundary) = target.and_then(|target| target.desktop_run_pixel_boundary.as_ref())
    else {
        return progress_item(
            evidence,
            "missing",
            "desktop-run-pixel-boundary.json was not read",
        );
    };
    match pixel_boundary.status.as_str() {
        "missing" => progress_item(evidence, "missing", pixel_boundary.detail.clone()),
        "invalid" => progress_item(evidence, "invalid", pixel_boundary.detail.clone()),
        "not_observed" => progress_item(evidence, "not_observed", pixel_boundary.detail.clone()),
        _ => progress_item(evidence, "present", pixel_boundary.detail.clone()),
    }
}

fn pixel_observation_progress_item(
    evidence: &str,
    target: Option<&LessonTargetEvidence>,
) -> LessonReadinessEvidenceProgressItem {
    let Some(pixel_observation) =
        target.and_then(|target| target.desktop_run_pixel_observation.as_ref())
    else {
        return progress_item(
            evidence,
            "missing",
            "desktop-run-pixel-observation.json was not read",
        );
    };
    match pixel_observation.status.as_str() {
        "missing" => progress_item(evidence, "missing", pixel_observation.detail.clone()),
        "invalid" => progress_item(evidence, "invalid", pixel_observation.detail.clone()),
        "not_observed" => progress_item(evidence, "not_observed", pixel_observation.detail.clone()),
        "blocked" => progress_item(evidence, "blocked", pixel_observation.detail.clone()),
        _ => progress_item(evidence, "present", pixel_observation.detail.clone()),
    }
}

fn ui_action_contract_state(
    baseline: Option<&LessonTargetEvidence>,
    modernized: Option<&LessonTargetEvidence>,
) -> &'static str {
    match (baseline, modernized) {
        (Some(baseline), Some(modernized))
            if baseline.ui_action_contract_readable && modernized.ui_action_contract_readable =>
        {
            "present"
        }
        (Some(baseline), Some(modernized))
            if baseline.ui_action_contract_path.is_some()
                && modernized.ui_action_contract_path.is_some() =>
        {
            "invalid"
        }
        _ => "missing",
    }
}
