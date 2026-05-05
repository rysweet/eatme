use super::{ComparisonDiff, ComparisonTargetRun};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct ComparisonScorecard {
    pub execution_mode: String,
    pub functionality_result: String,
    pub functionality_detail: String,
    pub timing_result: String,
    pub timing_detail: String,
    pub baseline_duration_ms: Option<u128>,
    pub modernized_duration_ms: Option<u128>,
    pub modernized_minus_baseline_ms: Option<i128>,
    pub faster_target: Option<String>,
}

pub(super) fn build_scorecard(
    execute_requested: bool,
    targets: &BTreeMap<String, ComparisonTargetRun>,
    diff: &ComparisonDiff,
) -> ComparisonScorecard {
    let baseline = targets.get("baseline").expect("baseline target run exists");
    let modernized = targets
        .get("modernized")
        .expect("modernized target run exists");

    if !execute_requested {
        return ComparisonScorecard {
            execution_mode: "manifest_only".into(),
            functionality_result: "not_measured".into(),
            functionality_detail:
                "execution was not requested; target metadata and run intent were recorded".into(),
            timing_result: "not_measured".into(),
            timing_detail: "execution was not requested; target durations are manifest bookkeeping"
                .into(),
            baseline_duration_ms: None,
            modernized_duration_ms: None,
            modernized_minus_baseline_ms: None,
            faster_target: None,
        };
    }

    if baseline.status != "passed" || modernized.status != "passed" {
        return ComparisonScorecard {
            execution_mode: "execute_requested".into(),
            functionality_result: "incomplete".into(),
            functionality_detail: format!(
                "baseline target status is {}; modernized target status is {}",
                baseline.status, modernized.status
            ),
            timing_result: "incomplete".into(),
            timing_detail: "speed comparison requires both targets to pass launch smoke".into(),
            baseline_duration_ms: None,
            modernized_duration_ms: None,
            modernized_minus_baseline_ms: None,
            faster_target: None,
        };
    }

    let functionality_changed =
        diff.status_changed || diff.failure_category_changed || !diff.assertion_diffs.is_empty();
    let (functionality_result, functionality_detail) = if functionality_changed {
        (
            "different",
            "target statuses, failure categories, or assertions differ",
        )
    } else {
        ("matched", "target statuses and assertions match")
    };

    let delta = signed_duration_delta(modernized.duration_ms, baseline.duration_ms);
    let (timing_result, timing_detail, faster_target) = if delta < 0 {
        (
            "modernized_faster",
            "modernized target completed faster than the baseline target",
            Some("modernized".into()),
        )
    } else if delta > 0 {
        (
            "baseline_faster",
            "baseline target completed faster than the modernized target",
            Some("baseline".into()),
        )
    } else {
        ("matched", "target durations match", None)
    };

    ComparisonScorecard {
        execution_mode: "execute_requested".into(),
        functionality_result: functionality_result.into(),
        functionality_detail: functionality_detail.into(),
        timing_result: timing_result.into(),
        timing_detail: timing_detail.into(),
        baseline_duration_ms: Some(baseline.duration_ms),
        modernized_duration_ms: Some(modernized.duration_ms),
        modernized_minus_baseline_ms: Some(delta),
        faster_target,
    }
}

fn signed_duration_delta(modernized_duration_ms: u128, baseline_duration_ms: u128) -> i128 {
    if modernized_duration_ms >= baseline_duration_ms {
        modernized_duration_ms
            .saturating_sub(baseline_duration_ms)
            .min(i128::MAX as u128) as i128
    } else {
        -(baseline_duration_ms
            .saturating_sub(modernized_duration_ms)
            .min(i128::MAX as u128) as i128)
    }
}
