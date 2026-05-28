use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeMap;

#[allow(dead_code)] // YAML fields needed for deny_unknown_fields but not read by code
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CrewAsset {
    #[serde(default)]
    pub(crate) version: Option<Value>,
    pub(crate) workstream: String,
    pub(crate) title: String,
    pub(crate) purpose: String,
    #[serde(default)]
    pub(crate) philosophy: Option<Value>,
    #[serde(default)]
    pub(crate) source_basis: Option<Value>,
    #[serde(default)]
    pub(crate) asset_shapes: Option<Value>,
    #[serde(default)]
    pub(crate) personality_assets: Option<Value>,
    #[serde(default)]
    pub(crate) constituency_coverage: Vec<ConstituencyCoverage>,
    pub(crate) personas: PersonaGroups,
    #[serde(default)]
    pub(crate) student_outside_in_flow_assets: StudentOutsideInFlowAssets,
    pub(crate) core_scenarios_from_existing_alice_resources: Vec<Scenario>,
    pub(crate) creative_new_teaching_learning_scenarios: Vec<Scenario>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StudentOutsideInFlowAssets {
    #[serde(default)]
    pub(crate) prompt_cards: Vec<PromptCard>,
    #[serde(default)]
    pub(crate) coverage_map: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PromptCard {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) editable_by: String,
    #[serde(default)]
    pub(crate) purpose: String,
    #[serde(default)]
    pub(crate) prompt_frame: String,
    #[serde(default)]
    pub(crate) scenario_ids: Vec<String>,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConstituencyCoverage {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) editable_by: String,
    #[serde(default)]
    pub(crate) persona_ids: Vec<String>,
    #[serde(default)]
    pub(crate) scenario_ids: Vec<String>,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersonaGroups {
    pub(crate) instructors: Vec<Persona>,
    pub(crate) students: Vec<Persona>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioPersonas {
    pub(crate) instructors: Vec<String>,
    pub(crate) students: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioIntent {
    pub(crate) concepts: Vec<String>,
    pub(crate) habits: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioObservables {
    pub(crate) instructor: Vec<String>,
    pub(crate) student: Vec<String>,
    pub(crate) system_or_artifact: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub(crate) resource_basis: Vec<EatmeScenarioResource>,
    #[serde(default)]
    pub(crate) purpose: String,
    #[serde(default)]
    pub(crate) personas: Option<ScenarioPersonas>,
    #[serde(default)]
    pub(crate) launcher: Option<EatmeScenarioLauncher>,
    #[serde(default)]
    pub(crate) real_alice: Option<EatmeScenarioRealAlice>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) persona_assets: Option<Value>,
    #[serde(default)]
    pub(crate) capabilities: Option<ScenarioCapabilities>,
    #[serde(default)]
    pub(crate) adapter: Option<ScenarioAdapter>,
    #[serde(default)]
    pub(crate) smoke_ready: Option<EatmeScenarioSmokeReady>,
    #[serde(default)]
    pub(crate) agentic_flow: Option<EatmeScenarioAgenticFlow>,
    #[serde(default)]
    pub(crate) agentic_test_prompt: String,
    #[serde(default)]
    pub(crate) acceptance_criteria: Vec<EatmeScenarioAcceptanceCriterion>,
    #[serde(default)]
    pub(crate) acceptance_probes: Vec<String>,
    #[serde(default)]
    pub(crate) rubric: Vec<EatmeScenarioRubricCriterion>,
    #[serde(default)]
    pub(crate) avoid: Vec<String>,
    #[serde(default)]
    pub(crate) steps: Vec<EatmeScenarioStep>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) studio_cycle: Option<Value>,
    #[serde(default)]
    pub(crate) agentic_follow_on: Option<ScenarioAgenticFollowOn>,
    #[serde(default)]
    pub(crate) timeouts: BTreeMap<String, u64>,
    #[serde(default)]
    pub(crate) artifacts: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) unsupported_policy: String,
    #[serde(default)]
    pub(crate) portability: Option<EatmeScenarioPortability>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioResource {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default, rename = "use")]
    pub(crate) use_note: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioCapabilities {
    #[serde(default)]
    pub(crate) required: Vec<String>,
    #[serde(default)]
    pub(crate) optional: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioAdapter {
    #[serde(default)]
    pub(crate) targets: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioAgenticFollowOn {
    #[serde(default)]
    pub(crate) prompt_source: String,
    #[serde(default)]
    pub(crate) personality_assets: Vec<String>,
    #[serde(default)]
    pub(crate) deterministic_gate: String,
    #[serde(default)]
    pub(crate) required_observables: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioLauncher {
    #[serde(default)]
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) scenario: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioRealAlice {
    #[serde(default)]
    pub(crate) gated_by: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioSmokeReady {
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioAgenticFlow {
    #[serde(default)]
    pub(crate) focus: String,
    #[serde(default)]
    pub(crate) instructor_goal: String,
    #[serde(default)]
    pub(crate) prompt_source: String,
    #[serde(default)]
    pub(crate) non_coder_editable: Vec<String>,
    #[serde(default)]
    pub(crate) expected_outputs: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioRubricCriterion {
    #[serde(default)]
    pub(crate) criterion: String,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioAcceptanceCriterion {
    #[serde(default)]
    pub(crate) given: String,
    #[serde(default)]
    pub(crate) when: String,
    #[serde(default)]
    pub(crate) then: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EatmeScenarioStep {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiScenarioAsset {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) config: Option<GadugiConfig>,
    #[serde(default)]
    pub(crate) environment: Option<GadugiEnvironment>,
    #[serde(default)]
    pub(crate) agents: Vec<GadugiScenarioAgent>,
    #[serde(default)]
    pub(crate) steps: Vec<GadugiScenarioStep>,
    #[serde(default)]
    pub(crate) assertions: Vec<GadugiScenarioAssertion>,
    #[serde(default)]
    pub(crate) metadata: GadugiMetadata,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiConfig {
    #[serde(default)]
    pub(crate) timeout: u64,
    #[serde(default)]
    pub(crate) retries: u64,
    #[serde(default)]
    pub(crate) parallel: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiEnvironment {
    #[serde(default)]
    pub(crate) requires: Vec<String>,
    #[serde(default)]
    pub(crate) optional: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiScenarioAgent {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, rename = "type")]
    pub(crate) agent_type: String,
    #[serde(default)]
    pub(crate) config: GadugiScenarioAgentConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiScenarioAgentConfig {
    #[serde(default)]
    pub(crate) shell: String,
    #[serde(default)]
    pub(crate) cwd: String,
    #[serde(default)]
    pub(crate) timeout: u64,
    #[serde(default)]
    pub(crate) capture_output: bool,
    #[serde(default)]
    pub(crate) persona_asset: String,
    #[serde(default)]
    pub(crate) scenario_asset: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiScenarioStep {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) agent: String,
    #[serde(default)]
    pub(crate) action: String,
    #[serde(default)]
    pub(crate) params: BTreeMap<String, Value>,
    #[serde(default)]
    pub(crate) expect: Option<GadugiStepExpect>,
    #[serde(default)]
    pub(crate) timeout: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiStepExpect {
    #[serde(default)]
    pub(crate) exit_code: Option<i64>,
    #[serde(default)]
    pub(crate) stdout_contains: Vec<String>,
    #[serde(default)]
    pub(crate) output_contains: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiScenarioAssertion {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, rename = "type")]
    pub(crate) assertion_type: String,
    #[serde(default)]
    pub(crate) agent: String,
    #[serde(default)]
    pub(crate) params: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GadugiMetadata {
    #[serde(default)]
    pub(crate) source_eatme_asset: String,
    #[serde(default)]
    pub(crate) generated_by: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) priority: String,
    #[serde(default)]
    pub(crate) author: String,
    #[serde(default)]
    pub(crate) test_type: String,
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
