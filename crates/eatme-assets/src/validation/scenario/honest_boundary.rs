use crate::schema::EatmeScenarioAsset;

const LAUNCH_SMOKE_BOUNDARY_CLAIMS: &[&str] = &[
    "full UI automation",
    "creative assessment",
    "learner-world grading",
];
pub(crate) const REAL_UI_ACTION_BOUNDARY_PHRASES: &[&str] = &[
    "ui_action_automation_unimplemented",
    "not full UI automation",
    "not creative assessment",
    "not learner-world grading",
];
const INSTRUCTOR_GRADING_CLAIMS: &[&str] = &[
    "automated creative grading",
    "automated creative assessment",
    "automated learner-world grading",
    "learner-world assessment",
    "learner-world grading",
    "grades learner worlds",
    "grade learner worlds",
    "creative assessment is scored",
    "assign automated creative grades",
    "automatically grades learner worlds",
];

pub(crate) fn validate_launch_smoke_boundary_claims(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    if scenario_has_unqualified_claim(scenario, LAUNCH_SMOKE_BOUNDARY_CLAIMS) {
        errors.push(
            "launch smoke evidence must not claim full UI automation, creative assessment, or learner-world grading; describe those as explicit limitations instead"
                .into(),
        );
    }
}

pub(crate) fn scenario_contains_all_boundary_phrases(
    scenario: &EatmeScenarioAsset,
    phrases: &[&str],
) -> bool {
    let text = scenario_text_fields(scenario)
        .join("\n")
        .to_ascii_lowercase();
    phrases
        .iter()
        .all(|phrase| text.contains(&phrase.to_ascii_lowercase()))
}

pub(crate) fn scenario_has_unqualified_automated_grading_claim(
    scenario: &EatmeScenarioAsset,
) -> bool {
    scenario_has_unqualified_claim(scenario, INSTRUCTOR_GRADING_CLAIMS)
}

fn scenario_has_unqualified_claim(scenario: &EatmeScenarioAsset, phrases: &[&str]) -> bool {
    scenario_text_fields(scenario).iter().any(|text| {
        phrases
            .iter()
            .any(|phrase| has_unqualified_phrase(text, phrase))
    })
}

fn has_unqualified_phrase(text: &str, phrase: &str) -> bool {
    let text = normalize_claim_text(text);
    let phrase = normalize_claim_text(phrase);
    let mut search_start = 0;
    while let Some(relative_start) = text[search_start..].find(&phrase) {
        let phrase_start = search_start + relative_start;
        if !has_honest_qualifier(&text, phrase_start) {
            return true;
        }
        search_start = phrase_start + phrase.len();
    }
    false
}

fn has_honest_qualifier(text: &str, phrase_start: usize) -> bool {
    let close_prefix_start = phrase_start.saturating_sub(32);
    let close_prefix = &text[close_prefix_start..phrase_start];
    if [
        "not ",
        "without ",
        "instead of ",
        "rather than ",
        "does not ",
        "do not ",
        "must not ",
    ]
    .iter()
    .any(|qualifier| close_prefix.ends_with(qualifier))
    {
        return true;
    }

    let sentence_start = text[..phrase_start]
        .rfind(|character| matches!(character, '.' | ';' | '!' | '?'))
        .map(|index| index + 1)
        .unwrap_or(0);
    let list_prefix_start = sentence_start.max(phrase_start.saturating_sub(128));
    let list_prefix = &text[list_prefix_start..phrase_start];
    [
        "avoid ",
        "avoids ",
        "does not claim",
        "does not grade",
        "do not claim",
        "do not replace",
        "must not claim",
        "reject ",
        "rejects ",
        "without claiming",
    ]
    .iter()
    .any(|qualifier| list_prefix.contains(qualifier))
}

fn scenario_text_fields(scenario: &EatmeScenarioAsset) -> Vec<String> {
    let mut fields = vec![
        scenario.purpose.clone(),
        scenario.agentic_test_prompt.clone(),
        scenario.unsupported_policy.clone(),
    ];
    if let Some(smoke_ready) = &scenario.smoke_ready {
        fields.extend(smoke_ready.evidence.iter().cloned());
    }
    if let Some(flow) = &scenario.agentic_flow {
        fields.push(flow.focus.clone());
        fields.push(flow.instructor_goal.clone());
    }
    if let Some(follow_on) = &scenario.agentic_follow_on {
        fields.push(follow_on.deterministic_gate.clone());
        fields.extend(follow_on.required_observables.iter().cloned());
    }
    for criterion in &scenario.acceptance_criteria {
        fields.push(criterion.given.clone());
        fields.push(criterion.when.clone());
        fields.push(criterion.then.clone());
    }
    fields.extend(scenario.acceptance_probes.iter().cloned());
    for item in &scenario.rubric {
        fields.push(item.criterion.clone());
        fields.extend(item.evidence.iter().cloned());
    }
    fields.extend(scenario.avoid.iter().map(|item| format!("avoid {item}")));
    for step in &scenario.steps {
        fields.push(step.command.clone());
        fields.extend(step.evidence.iter().cloned());
    }
    fields
}

fn normalize_claim_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
