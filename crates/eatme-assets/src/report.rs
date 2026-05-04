use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AssetValidationReport {
    pub schema_version: String,
    pub asset_path: String,
    pub passed: bool,
    pub instructor_count: usize,
    pub student_count: usize,
    pub core_scenario_count: usize,
    pub creative_scenario_count: usize,
    pub scenario_asset_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioAssetValidationReport {
    pub schema_version: String,
    pub asset_path: String,
    pub asset_kind: String,
    pub id: String,
    pub passed: bool,
    pub step_count: usize,
    pub assertion_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
