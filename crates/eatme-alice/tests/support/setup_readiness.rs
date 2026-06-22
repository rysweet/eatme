use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::time::Duration;

pub fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM")
        .map(|value| value == "1")
        .unwrap_or(false)
}

pub fn web_base_url() -> String {
    env::var("ALICE_WEB_URL").unwrap_or_else(|_| "http://localhost:3099".into())
}

pub fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ConfigResponse {
    runtime: String,
    platform: String,
}

#[derive(Debug, Deserialize)]
struct SetupPreflightResponse {
    status: String,
    platform: String,
    scenario: String,
    #[serde(rename = "unsupportedCapabilities")]
    unsupported_capabilities: Vec<String>,
    #[serde(rename = "classroomReadiness")]
    classroom_readiness: ClassroomReadiness,
}

#[derive(Debug, Deserialize)]
struct ClassroomReadiness {
    #[serde(rename = "readyToCreateProject")]
    ready_to_create_project: bool,
    #[serde(rename = "readyForLabHandoff")]
    ready_for_lab_handoff: bool,
    #[serde(rename = "readyForEvidenceHandoff")]
    ready_for_evidence_handoff: bool,
}

#[derive(Debug, Deserialize)]
struct TemplatesResponse {
    templates: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct NewProjectResponse {
    status: String,
    #[serde(rename = "projectName")]
    project_name: String,
}

#[derive(Debug, Deserialize)]
struct LaunchResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct EvidenceHandoffResponse {
    status: String,
    platform: String,
    scenario: String,
    #[serde(rename = "evidenceArtifact")]
    evidence_artifact: String,
    handoff: Value,
}

#[derive(Debug, Clone)]
pub enum Step {
    Health,
    Config,
    SetupPreflight { scenario: String },
    ProjectTemplates,
    ProjectNew { project_name: String },
    Launch,
    EvidenceHandoff { scenario: String },
}

#[derive(Debug)]
pub struct StepResult {
    name: String,
    ok: bool,
    msg: String,
}

pub fn setup_scenarios() -> Vec<(&'static str, Vec<Step>)> {
    vec![
        scenario(
            "setup-preflight-ready-to-create",
            "Setup Preflight Ready to Create",
        ),
        scenario("setup-support-lab-readiness", "Setup Support Lab Readiness"),
        scenario(
            "instructor-classroom-setup-readiness",
            "Instructor Classroom Setup Readiness",
        ),
        scenario(
            "instructor-student-launch-evidence-handoff",
            "Student Launch Evidence Handoff",
        ),
    ]
}

fn scenario(id: &'static str, project_name: &'static str) -> (&'static str, Vec<Step>) {
    (
        id,
        vec![
            Step::Health,
            Step::Config,
            Step::SetupPreflight {
                scenario: id.into(),
            },
            Step::ProjectTemplates,
            Step::ProjectNew {
                project_name: project_name.into(),
            },
            Step::Launch,
            Step::EvidenceHandoff {
                scenario: id.into(),
            },
        ],
    )
}

pub fn execute(base: &str, client: &ureq::Agent, steps: &[Step]) -> Vec<StepResult> {
    steps
        .iter()
        .map(|step| execute_step(base, client, step))
        .collect()
}

fn execute_step(base: &str, client: &ureq::Agent, step: &Step) -> StepResult {
    match step {
        Step::Health => get_json(
            client,
            &format!("{base}/api/health"),
            "health",
            |health: HealthResponse| {
                (
                    matches!(health.status.as_str(), "ok" | "running"),
                    health.status,
                )
            },
        ),
        Step::Config => get_json(
            client,
            &format!("{base}/api/config"),
            "config",
            |config: ConfigResponse| {
                (
                    config.runtime == "alice-web" && config.platform == "lookingglass",
                    format!("runtime={} platform={}", config.runtime, config.platform),
                )
            },
        ),
        Step::SetupPreflight { scenario } => run_preflight(base, client, scenario),
        Step::ProjectTemplates => get_json(
            client,
            &format!("{base}/api/project/templates"),
            "project-templates",
            |response: TemplatesResponse| {
                (
                    !response.templates.is_empty(),
                    format!("templates={}", response.templates.len()),
                )
            },
        ),
        Step::ProjectNew { project_name } => run_project_new(base, client, project_name),
        Step::Launch => post_json(
            client,
            &format!("{base}/api/launch"),
            ureq::json!({}),
            "launch",
            |response: LaunchResponse| {
                (
                    matches!(response.status.as_str(), "ok" | "launched"),
                    response.status,
                )
            },
        ),
        Step::EvidenceHandoff { scenario } => run_handoff(base, client, scenario),
    }
}

fn run_preflight(base: &str, client: &ureq::Agent, scenario: &str) -> StepResult {
    match client
        .get(&format!("{base}/api/setup/preflight"))
        .query("scenario", scenario)
        .call()
    {
        Ok(resp) => match resp.into_json::<SetupPreflightResponse>() {
            Ok(preflight) => {
                let ready = preflight.classroom_readiness.ready_to_create_project
                    && preflight.classroom_readiness.ready_for_lab_handoff
                    && preflight.classroom_readiness.ready_for_evidence_handoff;
                let names_desktop_boundary = preflight
                    .unsupported_capabilities
                    .iter()
                    .any(|capability| capability.contains("Java desktop Alice launch"));
                StepResult {
                    name: format!("setup-preflight({scenario})"),
                    ok: preflight.status == "ready"
                        && preflight.platform == "lookingglass"
                        && preflight.scenario == scenario
                        && ready
                        && names_desktop_boundary,
                    msg: format!(
                        "status={} platform={} scenario={}",
                        preflight.status, preflight.platform, preflight.scenario
                    ),
                }
            }
            Err(error) => failed(format!("setup-preflight({scenario})"), error),
        },
        Err(error) => failed(format!("setup-preflight({scenario})"), error),
    }
}

fn run_project_new(base: &str, client: &ureq::Agent, project_name: &str) -> StepResult {
    post_json(
        client,
        &format!("{base}/api/project/new"),
        ureq::json!({ "projectName": project_name }),
        format!("project-new({project_name})"),
        |response: NewProjectResponse| {
            (
                response.status == "created" && response.project_name == project_name,
                format!(
                    "status={} project={}",
                    response.status, response.project_name
                ),
            )
        },
    )
}

fn run_handoff(base: &str, client: &ureq::Agent, scenario: &str) -> StepResult {
    post_json(
        client,
        &format!("{base}/api/setup/evidence-handoff"),
        ureq::json!({ "scenario": scenario }),
        format!("evidence-handoff({scenario})"),
        |response: EvidenceHandoffResponse| {
            (
                response.status == "handoff-created"
                    && response.platform == "lookingglass"
                    && response.scenario == scenario
                    && !response.evidence_artifact.is_empty()
                    && has_visible_result_action(&response.handoff)
                    && has_support_handoff_fields(&response.handoff),
                format!(
                    "status={} artifact={}",
                    response.status, response.evidence_artifact
                ),
            )
        },
    )
}

fn get_json<T, F>(
    client: &ureq::Agent,
    url: &str,
    name: impl Into<String>,
    validate: F,
) -> StepResult
where
    T: for<'de> Deserialize<'de>,
    F: FnOnce(T) -> (bool, String),
{
    match client.get(url).call() {
        Ok(resp) => match resp.into_json::<T>() {
            Ok(value) => {
                let (ok, msg) = validate(value);
                StepResult {
                    name: name.into(),
                    ok,
                    msg,
                }
            }
            Err(error) => failed(name, error),
        },
        Err(error) => failed(name, error),
    }
}

fn post_json<T, F>(
    client: &ureq::Agent,
    url: &str,
    body: Value,
    name: impl Into<String>,
    validate: F,
) -> StepResult
where
    T: for<'de> Deserialize<'de>,
    F: FnOnce(T) -> (bool, String),
{
    match client.post(url).send_json(body) {
        Ok(resp) => match resp.into_json::<T>() {
            Ok(value) => {
                let (ok, msg) = validate(value);
                StepResult {
                    name: name.into(),
                    ok,
                    msg,
                }
            }
            Err(error) => failed(name, error),
        },
        Err(error) => failed(name, error),
    }
}

fn has_visible_result_action(handoff: &Value) -> bool {
    array_contains(handoff, "studentNextActions", "visible result")
}

fn has_support_handoff_fields(handoff: &Value) -> bool {
    [
        "blocker category",
        "owner",
        "fallback role",
        "retest signal",
    ]
    .iter()
    .all(|needle| array_contains(handoff, "supportHandoffFields", needle))
}

fn array_contains(value: &Value, field: &str, needle: &str) -> bool {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|item| {
                item.as_str()
                    .map(|text| text.contains(needle))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn failed(name: impl Into<String>, error: impl std::fmt::Display) -> StepResult {
    StepResult {
        name: name.into(),
        ok: false,
        msg: error.to_string(),
    }
}

pub fn assert_all(results: Vec<StepResult>) {
    let failures = results
        .iter()
        .filter(|result| !result.ok)
        .map(|result| format!("{}: {}", result.name, result.msg))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}
