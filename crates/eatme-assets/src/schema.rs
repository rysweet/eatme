use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CrewAsset {
    pub(crate) workstream: String,
    pub(crate) title: String,
    pub(crate) purpose: String,
    pub(crate) personas: PersonaGroups,
    pub(crate) core_scenarios_from_existing_alice_resources: Vec<Scenario>,
    pub(crate) creative_new_teaching_learning_scenarios: Vec<Scenario>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PersonaGroups {
    pub(crate) instructors: Vec<Persona>,
    pub(crate) students: Vec<Persona>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Persona {
    pub(crate) id: String,
    pub(crate) role: String,
    pub(crate) archetype: String,
    pub(crate) goals: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) educational_intent: Vec<String>,
    pub(crate) observable_behaviors: Vec<String>,
    pub(crate) anti_behaviors: Vec<String>,
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Scenario {
    pub(crate) id: String,
    pub(crate) origin: String,
    pub(crate) coverage: Vec<String>,
    pub(crate) user_story: String,
    pub(crate) personas: ScenarioPersonas,
    pub(crate) educational_intent: ScenarioIntent,
    pub(crate) constraints: Vec<String>,
    pub(crate) observable_behaviors: ScenarioObservables,
    pub(crate) agentic_test_prompt: String,
    pub(crate) acceptance_probes: Vec<String>,
    pub(crate) avoid: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ScenarioPersonas {
    pub(crate) instructors: Vec<String>,
    pub(crate) students: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ScenarioIntent {
    pub(crate) concepts: Vec<String>,
    pub(crate) habits: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ScenarioObservables {
    pub(crate) instructor: Vec<String>,
    pub(crate) student: Vec<String>,
    pub(crate) system_or_artifact: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct EatmeScenarioAsset {
    #[serde(default)]
    pub(crate) schema_version: String,
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) owner: String,
    #[serde(default)]
    pub(crate) purpose: String,
    #[serde(default)]
    pub(crate) launcher: Option<EatmeScenarioLauncher>,
    #[serde(default)]
    pub(crate) real_alice: Option<EatmeScenarioRealAlice>,
    #[serde(default)]
    pub(crate) smoke_ready: Option<EatmeScenarioSmokeReady>,
    #[serde(default)]
    pub(crate) acceptance_criteria: Vec<EatmeScenarioAcceptanceCriterion>,
    #[serde(default)]
    pub(crate) steps: Vec<EatmeScenarioStep>,
    #[serde(default)]
    pub(crate) timeouts: BTreeMap<String, u64>,
    #[serde(default)]
    pub(crate) artifacts: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) unsupported_policy: String,
    #[serde(default)]
    pub(crate) portability: Option<EatmeScenarioPortability>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioLauncher {
    #[serde(default)]
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) scenario: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioRealAlice {
    #[serde(default)]
    pub(crate) gated_by: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioSmokeReady {
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioAcceptanceCriterion {
    #[serde(default)]
    pub(crate) given: String,
    #[serde(default)]
    pub(crate) when: String,
    #[serde(default)]
    pub(crate) then: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioStep {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct EatmeScenarioPortability {
    #[serde(default)]
    pub(crate) source_project: String,
    #[serde(default)]
    pub(crate) destination_project: String,
    #[serde(default)]
    pub(crate) modified_class: String,
    #[serde(default)]
    pub(crate) share_channel: String,
    #[serde(default)]
    pub(crate) evidence_after_import: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GadugiScenarioAsset {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) steps: Vec<GadugiScenarioStep>,
    #[serde(default)]
    pub(crate) assertions: Vec<GadugiScenarioAssertion>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GadugiScenarioStep {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) agent: String,
    #[serde(default)]
    pub(crate) action: String,
    #[serde(default)]
    pub(crate) params: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) expect: GadugiScenarioExpect,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GadugiScenarioExpect {
    #[serde(default)]
    pub(crate) stdout_contains: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GadugiScenarioAssertion {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, rename = "type")]
    pub(crate) assertion_type: String,
}
