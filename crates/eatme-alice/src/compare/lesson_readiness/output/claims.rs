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
const FULL_WORLD_EXECUTION: UnprovenClaim = UnprovenClaim {
    sentence: "Full world execution is not proven.",
    non_claim: "full world execution",
};
const DEPLOYED_SHARING_PLATFORM_SUCCESS: UnprovenClaim = UnprovenClaim {
    sentence: "Deployed sharing/platform success is not proven.",
    non_claim: "deployed sharing/platform success",
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

const LAUNCH_SMOKE_UNPROVEN_CLAIMS: &[UnprovenClaim] = &[
    FIRST_LESSON_COMPLETION,
    FULL_WORLD_EXECUTION,
    GRADING,
    CREATIVE_ASSESSMENT,
    FULL_ALICE_UI_AUTOMATION,
    VISIBLE_RENDERING_CORRECTNESS,
    SAVE_COMPLETION,
    DEPLOYED_SHARING_PLATFORM_SUCCESS,
];

const LAUNCH_SMOKE_LIMITATIONS: &[&str] = &[
    "bounded to existing launch-smoke manifest metadata",
    "does not add lesson-action detection",
    "does not grade student worlds",
    "does not perform creative assessment",
    "does not prove full UI automation",
    "does not prove visible correctness",
];

pub(in crate::compare::lesson_readiness) fn unproven_claims() -> Vec<String> {
    UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence.to_string())
        .collect()
}

pub(in crate::compare::lesson_readiness) fn limitations() -> Vec<String> {
    UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence)
        .chain(LEGACY_LIMITATIONS.iter().copied())
        .map(str::to_string)
        .collect()
}

pub(in crate::compare::lesson_readiness) fn launch_smoke_unproven_claims() -> Vec<String> {
    LAUNCH_SMOKE_UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence.to_string())
        .collect()
}

pub(in crate::compare::lesson_readiness) fn launch_smoke_limitations() -> Vec<String> {
    LAUNCH_SMOKE_UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.sentence)
        .chain(LAUNCH_SMOKE_LIMITATIONS.iter().copied())
        .map(str::to_string)
        .collect()
}

pub(super) fn save_completion_non_claim() -> &'static str {
    SAVE_COMPLETION.non_claim
}

pub(super) fn first_lesson_completion_non_claim() -> &'static str {
    FIRST_LESSON_COMPLETION.non_claim
}

pub(super) fn visible_rendering_correctness_non_claim() -> &'static str {
    VISIBLE_RENDERING_CORRECTNESS.non_claim
}

pub(super) fn desktop_next_action_non_claims(extra_claims: &[String]) -> Vec<String> {
    let mut claims = UNPROVEN_CLAIMS
        .iter()
        .map(|claim| claim.non_claim.to_string())
        .collect();
    for claim in extra_claims {
        super::push_unique(&mut claims, claim);
    }
    claims
}
