use super::LessonTargetEvidence;
use super::project_proof::{save_project_proof_progress_item, select_project_proof_progress_item};
use serde::{Serialize, Serializer, ser::SerializeStruct};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_missing_real_desktop_proof: Option<String>,
    pub items: Vec<LessonReadinessEvidenceProgressItem>,
}

#[derive(Clone, Debug)]
pub struct LessonReadinessEvidenceProgressItem {
    pub evidence: String,
    pub state: String,
    pub detail: String,
}

impl Serialize for LessonReadinessEvidenceProgressItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("LessonReadinessEvidenceProgressItem", 4)?;
        state.serialize_field("id", &progress_item_id(&self.evidence))?;
        state.serialize_field("evidence", &self.evidence)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("detail", &self.detail)?;
        state.end()
    }
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
    items.push(save_project_proof_progress_item(
        &required_evidence[8],
        modernized,
    ));
    items.push(select_project_proof_progress_item(
        &required_evidence[9],
        modernized,
    ));

    let present = count_state(&items, "present");
    let missing = count_state(&items, "missing");
    let invalid = count_state(&items, "invalid");
    let not_observed = count_state(&items, "not_observed");
    let blocked = count_state(&items, "blocked");
    let total_required = items.len();
    let next_actionable_blocker = next_actionable_blocker(modernized);
    let next_missing_real_desktop_proof = next_missing_real_desktop_proof(modernized, &items);
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
        next_missing_real_desktop_proof,
        items,
    }
}

pub(super) fn progress_item(
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

fn progress_item_id(evidence: &str) -> String {
    match evidence {
        "Save Project proof artifact" => "save_project_proof_artifact".into(),
        "Select Project proof artifact" => "select_project_proof_artifact".into(),
        _ => evidence
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .split('_')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("_"),
    }
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

fn next_missing_real_desktop_proof(
    target: Option<&LessonTargetEvidence>,
    items: &[LessonReadinessEvidenceProgressItem],
) -> Option<String> {
    let target = target?;
    for (action_id, message) in [
        (
            "verify-specific-alice-window",
            "next missing real-desktop proof: identify the Alice main window (verify-specific-alice-window) before claiming later lesson actions.",
        ),
        (
            "activate-specific-alice-window",
            "next missing real-desktop proof: activate the detected Alice main window (activate-specific-alice-window) before claiming later lesson actions.",
        ),
        (
            "observe-run-window-after-toolbar-button",
            "next missing real-desktop proof: observe the Alice Run window after toolbar dispatch (observe-run-window-after-toolbar-button).",
        ),
        (
            "observe-desktop-run-execution-after-toolbar-button",
            "next missing real-desktop proof: observe desktop Run execution after toolbar dispatch (observe-desktop-run-execution-after-toolbar-button).",
        ),
    ] {
        if !action_passed(target, action_id) {
            return Some(message.into());
        }
    }

    if item_needs_proof(items, "screenshot, log, and window artifacts") {
        return Some(
            "next missing real-desktop proof: capture screenshots/run-window-after-dispatch.png under the modernized comparison evidence root after Run-frame and VM statement execution.".into(),
        );
    }
    if item_state_is(
        items,
        "modernized desktop-run-pixel-boundary.json status",
        &["missing", "invalid"],
    ) {
        return Some(
            "next missing real-desktop proof: record run-window-evidence/desktop-run-pixel-boundary.json under the modernized comparison evidence root.".into(),
        );
    }
    if item_state_is(
        items,
        "modernized desktop-run-pixel-observation.json status",
        &["missing", "invalid", "not_observed", "blocked"],
    ) {
        return Some(
            "next missing real-desktop proof: record an observed desktop Run pixel sample in run-window-evidence/desktop-run-pixel-observation.json from a non-headless visible desktop.".into(),
        );
    }

    // After the Run-pixel chain is complete, surface the first RabbitHole hook action
    // that has not yet been proven so a plain user knows exactly which hook to wire next.
    for (action_id, hook_path, label) in [
        (
            "place-object",
            "tools/eatme-place-object",
            "object placement",
        ),
        (
            "edit-procedure-or-code-block",
            "tools/eatme-edit-procedure",
            "procedure/code-block editing",
        ),
        ("run-world", "tools/eatme-run-world", "world run"),
        ("save-project", "tools/eatme-save-project", "project save"),
    ] {
        if !action_passed(target, action_id) {
            return Some(format!(
                "next missing real-desktop proof: wire the {label} hook ({action_id}) \
                 at {hook_path} so the harness can collect deterministic evidence; \
                 this does not prove full UI automation."
            ));
        }
    }

    None
}

fn action_passed(target: &LessonTargetEvidence, action_id: &str) -> bool {
    target
        .action_assertions
        .iter()
        .any(|action| action.action_id == action_id && action.passed)
}

fn item_needs_proof(items: &[LessonReadinessEvidenceProgressItem], evidence: &str) -> bool {
    item_state_is(
        items,
        evidence,
        &["missing", "invalid", "not_observed", "blocked"],
    )
}

fn item_state_is(
    items: &[LessonReadinessEvidenceProgressItem],
    evidence: &str,
    states: &[&str],
) -> bool {
    items
        .iter()
        .any(|item| item.evidence == evidence && states.contains(&item.state.as_str()))
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
