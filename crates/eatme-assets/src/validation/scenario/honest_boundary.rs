use crate::schema::EatmeScenarioAsset;

pub(crate) fn validate_launch_smoke_boundary_claims(
    scenario: &EatmeScenarioAsset,
    errors: &mut Vec<String>,
) {
    let overclaimed = scenario_text_fields(scenario).iter().any(|text| {
        has_unqualified_phrase(text, "full UI automation")
            || has_unqualified_phrase(text, "creative assessment")
            || has_unqualified_phrase(text, "learner-world grading")
    });
    if overclaimed {
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
    scenario_text_fields(scenario).iter().any(|text| {
        has_unqualified_phrase(text, "automated creative grading")
            || has_unqualified_phrase(text, "automated creative assessment")
            || has_unqualified_phrase(text, "automated learner-world grading")
            || has_unqualified_phrase(text, "learner-world assessment")
            || has_unqualified_phrase(text, "learner-world grading")
            || has_unqualified_phrase(text, "grades learner worlds")
            || has_unqualified_phrase(text, "grade learner worlds")
            || has_unqualified_phrase(text, "creative assessment is scored")
            || has_unqualified_phrase(text, "assign automated creative grades")
            || has_unqualified_phrase(text, "automatically grades learner worlds")
    })
}

fn has_unqualified_phrase(text: &str, phrase: &str) -> bool {
    let text = normalize_claim_text(text);
    let phrase = normalize_claim_text(phrase);
    if !text.contains(&phrase) {
        return false;
    }

    let broad_honest_markers = [
        "do not claim",
        "without claiming",
        "does not claim",
        "does not grade",
        "do not replace",
        "must not present",
        "must not claim",
        "not present",
        "avoids",
        "avoid claiming",
        "avoid automated",
    ];
    if broad_honest_markers
        .iter()
        .any(|marker| text.contains(marker))
    {
        return false;
    }

    let honest_markers = [
        format!("not {phrase}"),
        format!("not claim {phrase}"),
        format!("does not {phrase}"),
        format!("do not {phrase}"),
        format!("do not claim {phrase}"),
        format!("must not {phrase}"),
        format!("without {phrase}"),
        format!("without claiming {phrase}"),
        format!("avoids {phrase}"),
        format!("avoid {phrase}"),
        format!("reject {phrase}"),
        format!("rejects {phrase}"),
    ];
    !honest_markers
        .iter()
        .any(|marker| text.contains(marker.as_str()))
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
