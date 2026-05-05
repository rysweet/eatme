use super::*;
use crate::schema::{
    EatmeScenarioAcceptanceCriterion, EatmeScenarioAgenticFlow, EatmeScenarioResource,
    EatmeScenarioRubricCriterion, EatmeScenarioStep, ScenarioPersonas,
};
use std::collections::BTreeMap;

#[test]
fn live_studio_instructor_flow_rejects_missing_classroom_evidence_contract() {
    let scenario = instructor_agentic_scenario("workshop-facilitator-live-studio");

    let report = validate_eatme_scenario(
        Path::new("assets/scenarios/eatme/workshop-facilitator-live-studio.yaml"),
        &scenario,
        None,
        &[],
    );

    assert!(!report.passed);
    for required in super::instructor_agentic_flow::LIVE_STUDIO_REQUIRED_EVIDENCE {
        assert!(
            report.errors.iter().any(|error| error.contains(required)),
            "live-studio validation must report missing {required:?}; got {:?}",
            report.errors
        );
    }
}

fn instructor_agentic_scenario(id: &str) -> EatmeScenarioAsset {
    EatmeScenarioAsset {
        schema_version: "eatme.scenario/v1".into(),
        id: id.into(),
        title: "Instructor Exercise Builder".into(),
        kind: "instructor_agentic_flow".into(),
        owner: "eatme".into(),
        purpose: "help instructors create an Alice exercise from existing resources".into(),
        resource_basis: vec![EatmeScenarioResource {
            name: "Alice.org Programming in Alice".into(),
            url: "https://www.alice.org/resources/lessons/programming-in-alice/".into(),
            use_note: "Ground exercise concepts in procedures, parameters, and run/revise.".into(),
        }],
        personas: Some(ScenarioPersonas {
            instructors: vec!["exercise-forger".into()],
            students: vec!["curious-novice".into()],
        }),
        agentic_flow: Some(EatmeScenarioAgenticFlow {
            focus: "creating-exercises".into(),
            instructor_goal: "draft a classroom-ready Alice exercise".into(),
            prompt_source: "assets/scenarios/eatme/instructor-exercise-builder.yaml".into(),
            non_coder_editable: vec!["agentic_test_prompt".into(), "rubric".into()],
            expected_outputs: vec!["exercise brief".into(), "student evidence checklist".into()],
        }),
        agentic_test_prompt: "Act as the instructor QA agent and produce an exercise brief.".into(),
        acceptance_criteria: vec![EatmeScenarioAcceptanceCriterion {
            given: "an Alice.org lesson concept".into(),
            when: "the instructor agent drafts materials".into(),
            then: "the output names concept evidence and learner choice".into(),
        }],
        acceptance_probes: vec!["Exercise has concept, starter task, and extension.".into()],
        rubric: vec![EatmeScenarioRubricCriterion {
            criterion: "Concept evidence".into(),
            evidence: vec!["Student links a visible world behavior to a concept.".into()],
        }],
        avoid: vec!["Do not require exact coordinates or private implementation details.".into()],
        steps: vec![
            EatmeScenarioStep {
                id: "validate-assets".into(),
                command: "cargo run -q -p eatme-cli -- assets validate --json".into(),
                evidence: vec!["asset validation passes".into()],
            },
            EatmeScenarioStep {
                id: "agentic-instructor-review".into(),
                command: "agentic review using this YAML prompt and acceptance probes".into(),
                evidence: vec!["review returns maintainable lesson materials".into()],
            },
        ],
        timeouts: BTreeMap::from([("agentic_seconds".into(), 900)]),
        artifacts: BTreeMap::from([("lesson_brief".into(), "agentic://lesson-brief".into())]),
        unsupported_policy: "Fail visibly if the agent cannot read this editable asset.".into(),
        ..EatmeScenarioAsset::default()
    }
}
