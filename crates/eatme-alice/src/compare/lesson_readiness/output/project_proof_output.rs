use super::{FIRST_LESSON_COMPLETION, SAVE_COMPLETION, user_facing_evidence_label};

pub(super) fn not_yet_shown_detail(evidence: &str, state: &str, progress_detail: &str) -> String {
    if matches!(
        evidence,
        "Save Project proof artifact" | "Select Project proof artifact"
    ) && !progress_detail.trim().is_empty()
    {
        return progress_detail.to_string();
    }

    let label = user_facing_evidence_label(evidence);
    match state {
        "blocked" => format!("{label} is blocked until the next evidence is shown."),
        "invalid" => format!("{label} was shown but cannot be used yet."),
        "not_observed" => format!("{label} is not yet observed."),
        _ => format!("{label} is not yet shown."),
    }
}

pub(super) fn progress_item_does_not_prove(evidence: &str) -> Vec<String> {
    match evidence {
        "Save Project proof artifact" => vec![
            SAVE_COMPLETION.non_claim.into(),
            FIRST_LESSON_COMPLETION.non_claim.into(),
        ],
        "Select Project proof artifact" => vec![FIRST_LESSON_COMPLETION.non_claim.into()],
        _ => Vec::new(),
    }
}
