use crate::schema::EatmeScenarioAsset;

const FIRST_LESSON_SCENARIO_ID: &str = "first-lessons-real-ui-actions";
const STARTER_PROJECT_PREFLIGHT_SCENARIO_ID: &str = "starter-project-open-save-export-preflight";

pub(super) fn generated_description(source_asset: &str, scenario: &EatmeScenarioAsset) -> String {
    format!(
        "Gadugi-compatible CLI scenario generated from {source_asset}. Alice desktop launch behavior remains owned by eatme; {}.{}",
        generated_evidence_scope(scenario),
        generated_boundary_note(scenario)
    )
}

fn generated_evidence_scope(scenario: &EatmeScenarioAsset) -> &'static str {
    match scenario.id.as_str() {
        STARTER_PROJECT_PREFLIGHT_SCENARIO_ID => {
            "gadugi invokes eatme commands, records bounded starter-world and readiness-gap artifacts, and checks eatme launch-smoke evidence without claiming save/reopen/export coverage"
        }
        FIRST_LESSON_SCENARIO_ID => {
            "gadugi invokes eatme commands and checks first-lesson readiness evidence"
        }
        _ => "gadugi invokes eatme commands and checks manifest-level evidence only",
    }
}

fn generated_boundary_note(scenario: &EatmeScenarioAsset) -> &'static str {
    match scenario.id.as_str() {
        STARTER_PROJECT_PREFLIGHT_SCENARIO_ID => {
            " This automation scenario keeps honest limits: opened starter project with manifest/log/window/screenshot evidence and bounded starter-world and readiness-gap artifacts only; not full UI automation, not creative assessment, not learner-world grading, not complete Alice coverage, not visible rendering correctness proof, not first-lesson completion, and not full Save completion."
        }
        FIRST_LESSON_SCENARIO_ID if scenario_declares_honest_boundaries(scenario) => {
            " This generated runner keeps honest limits: not full UI automation, not creative assessment, and not learner-world grading."
        }
        _ if scenario_declares_honest_boundaries(scenario) => {
            " This adapter preserves the source boundary: not full UI automation, not creative assessment, and not learner-world grading."
        }
        _ => "",
    }
}

fn scenario_declares_honest_boundaries(scenario: &EatmeScenarioAsset) -> bool {
    let boundary_text = [
        scenario.purpose.as_str(),
        scenario.unsupported_policy.as_str(),
    ];
    [
        "not full ui automation",
        "not creative assessment",
        "not learner-world grading",
    ]
    .iter()
    .all(|phrase| {
        boundary_text
            .iter()
            .any(|text| contains_ignore_ascii_case(text, phrase))
    })
}

fn contains_ignore_ascii_case(text: &str, phrase: &str) -> bool {
    text.as_bytes()
        .windows(phrase.len())
        .any(|window| window.eq_ignore_ascii_case(phrase.as_bytes()))
}
