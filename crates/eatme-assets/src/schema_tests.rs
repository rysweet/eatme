use super::*;

#[test]
fn rejects_unknown_eatme_scenario_fields() {
    let yaml = r#"
schema_version: eatme.scenario/v1
id: strict-test
title: Strict Test
purpose: Catch bad edits.
unknown_field: should-fail
"#;
    let error = serde_yaml::from_str::<EatmeScenarioAsset>(yaml).unwrap_err();
    assert!(error.to_string().contains("unknown field"), "{error}");
}

#[test]
fn rejects_unknown_nested_scenario_fields() {
    let yaml = r#"
schema_version: eatme.scenario/v1
id: strict-test
title: Strict Test
purpose: Catch bad edits.
resource_basis:
  - name: Resource
    href: https://example.invalid
"#;
    let error = serde_yaml::from_str::<EatmeScenarioAsset>(yaml).unwrap_err();
    assert!(error.to_string().contains("unknown field"), "{error}");
}

#[test]
fn rejects_unknown_persona_fields() {
    let yaml = r#"
workstream: alice.eatme
title: Strict Persona Test
purpose: Catch bad persona edits.
personas:
  instructors:
    - id: concept-cartographer
      role: instructor
      archetype: Concept Cartographer
      goals: [Teach concepts]
      constraints: [Limited time]
      educational_intent: [Transfer]
      observable_behaviors: [Names concepts]
      anti_behaviors: [Over-prescribes]
      evidence: [Reflection]
      nickname: Cartographer
  students: []
core_scenarios_from_existing_alice_resources: []
creative_new_teaching_learning_scenarios: []
"#;
    let error = serde_yaml::from_str::<CrewAsset>(yaml).unwrap_err();
    assert!(error.to_string().contains("unknown field"), "{error}");
}
