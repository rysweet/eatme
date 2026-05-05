use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub(super) struct GeneratedGadugiAdapter {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) version: String,
    pub(super) config: GeneratedConfig,
    pub(super) environment: GeneratedEnvironment,
    pub(super) agents: Vec<GeneratedAgent>,
    pub(super) steps: Vec<GeneratedStep>,
    pub(super) assertions: Vec<GeneratedAssertion>,
    pub(super) metadata: GeneratedMetadata,
}

#[derive(Serialize)]
pub(super) struct GeneratedConfig {
    pub(super) timeout: u64,
    pub(super) retries: u64,
    pub(super) parallel: bool,
}

#[derive(Serialize)]
pub(super) struct GeneratedEnvironment {
    pub(super) requires: Vec<String>,
    pub(super) optional: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct GeneratedAgent {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) agent_type: String,
    pub(super) config: GeneratedAgentConfig,
}

#[derive(Serialize)]
pub(super) struct GeneratedAgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) cwd: Option<String>,
    pub(super) timeout: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) capture_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) persona_asset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) scenario_asset: Option<String>,
}

#[derive(Serialize)]
pub(super) struct GeneratedStep {
    pub(super) name: String,
    pub(super) agent: String,
    pub(super) action: String,
    pub(super) params: BTreeMap<String, String>,
    pub(super) expect: GeneratedExpect,
    pub(super) timeout: u64,
}

#[derive(Serialize)]
pub(super) struct GeneratedExpect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) exit_code: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) stdout_contains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) output_contains: Option<Vec<String>>,
}

#[derive(Serialize)]
pub(super) struct GeneratedAssertion {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) assertion_type: String,
    pub(super) agent: String,
    pub(super) params: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub(super) struct GeneratedMetadata {
    pub(super) source_eatme_asset: String,
    pub(super) generated_by: String,
    pub(super) tags: Vec<String>,
    pub(super) priority: String,
    pub(super) author: String,
    pub(super) test_type: String,
}
