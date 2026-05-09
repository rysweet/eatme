use super::super::{
    output::LessonTargetEvidence,
    progress::{
        LessonReadinessEvidenceProgress, LessonReadinessEvidenceProgressItem, progress_item,
    },
};

pub(super) fn launch_smoke_evidence_progress(
    required_evidence: &[String],
    target_evidence: &[LessonTargetEvidence],
    issues: &[String],
) -> LessonReadinessEvidenceProgress {
    let baseline = target_evidence
        .iter()
        .find(|target| target.role == "baseline");
    let modernized = target_evidence
        .iter()
        .find(|target| target.role == "modernized");
    let targets = [baseline, modernized];
    let items = launch_smoke_progress_items(required_evidence, &targets, issues);
    let counts = LaunchSmokeProgressCounts::from_items(&items);

    LessonReadinessEvidenceProgress {
        total_required: items.len(),
        summary: counts.summary(items.len()),
        present: counts.present,
        missing: counts.missing,
        invalid: counts.invalid,
        not_observed: counts.not_observed,
        blocked: counts.blocked,
        next_actionable_blocker: launch_smoke_next_blocker(issues),
        next_missing_real_desktop_proof: None,
        items,
    }
}

struct LaunchSmokeProgressCounts {
    present: usize,
    missing: usize,
    invalid: usize,
    not_observed: usize,
    blocked: usize,
}

impl LaunchSmokeProgressCounts {
    fn from_items(items: &[LessonReadinessEvidenceProgressItem]) -> Self {
        Self {
            present: count_launch_smoke_state(items, "present"),
            missing: count_launch_smoke_state(items, "missing"),
            invalid: count_launch_smoke_state(items, "invalid"),
            not_observed: count_launch_smoke_state(items, "not_observed"),
            blocked: count_launch_smoke_state(items, "blocked"),
        }
    }

    fn summary(&self, total_required: usize) -> String {
        format!(
            "{} of {total_required} required launch-smoke evidence items are present; {} missing, {} invalid, {} not observed, {} blocked.",
            self.present, self.missing, self.invalid, self.not_observed, self.blocked
        )
    }
}

fn launch_smoke_progress_items(
    required_evidence: &[String],
    targets: &[Option<&LessonTargetEvidence>; 2],
    issues: &[String],
) -> Vec<LessonReadinessEvidenceProgressItem> {
    vec![
        target_entries_progress_item(required_evidence, targets),
        target_manifests_progress_item(required_evidence, targets),
        progress_item(
            &required_evidence[2],
            launch_smoke_target_status_state(targets, issues),
            "target status and failure-category metadata for both targets",
        ),
        target_assertions_progress_item(required_evidence, targets),
        target_artifacts_progress_item(required_evidence, targets, issues),
    ]
}

fn target_entries_progress_item(
    required_evidence: &[String],
    targets: &[Option<&LessonTargetEvidence>; 2],
) -> LessonReadinessEvidenceProgressItem {
    progress_item(
        &required_evidence[0],
        if all_targets_present(targets) {
            "present"
        } else {
            "missing"
        },
        "baseline and modernized target entries for launch-smoke readiness",
    )
}

fn target_manifests_progress_item(
    required_evidence: &[String],
    targets: &[Option<&LessonTargetEvidence>; 2],
) -> LessonReadinessEvidenceProgressItem {
    progress_item(
        &required_evidence[1],
        target_manifest_state(targets),
        "embedded launch-smoke manifest metadata for both targets",
    )
}

fn target_assertions_progress_item(
    required_evidence: &[String],
    targets: &[Option<&LessonTargetEvidence>; 2],
) -> LessonReadinessEvidenceProgressItem {
    progress_item(
        &required_evidence[3],
        target_assertions_state(targets),
        "required launch-smoke assertions for both targets",
    )
}

fn target_artifacts_progress_item(
    required_evidence: &[String],
    targets: &[Option<&LessonTargetEvidence>; 2],
    issues: &[String],
) -> LessonReadinessEvidenceProgressItem {
    progress_item(
        &required_evidence[4],
        launch_smoke_artifact_metadata_state(targets, issues),
        "window-list, screenshot, and log artifact metadata only",
    )
}

fn target_manifest_state(targets: &[Option<&LessonTargetEvidence>; 2]) -> &'static str {
    if all_targets_present(targets)
        && targets
            .iter()
            .flatten()
            .all(|target| target.launch_manifest_present)
    {
        "present"
    } else {
        "missing"
    }
}

fn target_assertions_state(targets: &[Option<&LessonTargetEvidence>; 2]) -> &'static str {
    if all_targets_present(targets)
        && targets
            .iter()
            .flatten()
            .all(|target| target.missing_assertions.is_empty())
    {
        "present"
    } else {
        "invalid"
    }
}

fn launch_smoke_artifact_metadata_state(
    targets: &[Option<&LessonTargetEvidence>; 2],
    issues: &[String],
) -> &'static str {
    if issues
        .iter()
        .any(|issue| issue.contains("metadata must be present"))
    {
        "missing"
    } else if all_targets_present(targets) {
        "present"
    } else {
        "missing"
    }
}

fn all_targets_present(targets: &[Option<&LessonTargetEvidence>; 2]) -> bool {
    targets.iter().all(Option::is_some)
}

fn launch_smoke_target_status_state(
    targets: &[Option<&LessonTargetEvidence>; 2],
    issues: &[String],
) -> &'static str {
    if targets.iter().any(|target| target.is_none()) {
        return "missing";
    }
    if target_status_issue_present(issues) {
        return "invalid";
    }
    if all_targets_passed(targets) {
        "present"
    } else {
        "invalid"
    }
}

fn target_status_issue_present(issues: &[String]) -> bool {
    issues.iter().any(|issue| {
        issue.contains("target status must be passed")
            || issue.contains("target failure_category must be null")
            || issue.contains("launch_manifest failure_category must be null")
    })
}

fn all_targets_passed(targets: &[Option<&LessonTargetEvidence>; 2]) -> bool {
    targets.iter().flatten().all(|target| {
        target.target_status.as_deref() == Some("passed")
            && target.failure_category.is_none()
            && target.launch_manifest_present
    })
}

fn count_launch_smoke_state(items: &[LessonReadinessEvidenceProgressItem], state: &str) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn launch_smoke_next_blocker(issues: &[String]) -> Option<String> {
    issues
        .first()
        .map(|issue| format!("next launch-smoke readiness evidence gap: {issue}"))
}
