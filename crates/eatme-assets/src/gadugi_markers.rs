use crate::schema::EatmeScenarioStep;

pub(super) fn manifest_assertion_markers(step: &EatmeScenarioStep) -> Vec<&str> {
    let mut markers = Vec::new();
    for evidence in &step.evidence {
        if let Some(marker) = manifest_assertion_marker(evidence)
            && !markers.contains(&marker)
        {
            markers.push(marker);
        }
    }
    markers
}

fn manifest_assertion_marker(evidence: &str) -> Option<&str> {
    let marker_text = evidence.split_once("manifest assertions include ")?.1;
    marker_text
        .split_whitespace()
        .next()
        .map(|token| {
            token.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && character != '_'
            })
        })
        .filter(|token| !token.is_empty())
}
