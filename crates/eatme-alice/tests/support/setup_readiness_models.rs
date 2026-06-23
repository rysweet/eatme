use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigResponse {
    pub runtime: String,
    pub platform: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupPreflightResponse {
    pub status: String,
    pub platform: String,
    pub scenario: String,
    #[serde(rename = "unsupportedCapabilities")]
    pub unsupported_capabilities: Vec<String>,
    #[serde(rename = "doesNotClaim")]
    pub does_not_claim: Vec<String>,
    #[serde(rename = "classroomReadiness")]
    pub classroom_readiness: ClassroomReadiness,
}

#[derive(Debug, Deserialize)]
pub struct ClassroomReadiness {
    #[serde(rename = "readyToCreateProject")]
    pub ready_to_create_project: bool,
    #[serde(rename = "readyForLabHandoff")]
    pub ready_for_lab_handoff: bool,
    #[serde(rename = "readyForEvidenceHandoff")]
    pub ready_for_evidence_handoff: bool,
}

#[derive(Debug, Deserialize)]
pub struct TemplatesResponse {
    pub templates: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct NewProjectResponse {
    pub status: String,
    #[serde(rename = "projectName")]
    pub project_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LaunchResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceHandoffResponse {
    pub status: String,
    pub platform: String,
    pub scenario: String,
    #[serde(rename = "evidenceArtifact")]
    pub evidence_artifact: String,
    pub handoff: Value,
}
