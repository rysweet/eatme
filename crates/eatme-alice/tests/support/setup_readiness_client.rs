use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::time::Duration;

use super::setup_readiness_assertions::{handoff_is_specific, preflight_is_ready};
use super::setup_readiness_models::{
    ConfigResponse, EvidenceHandoffResponse, HealthResponse, LaunchResponse, NewProjectResponse,
    SetupPreflightResponse, TemplatesResponse,
};

pub fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM")
        .map(|value| value == "1")
        .unwrap_or(false)
}

pub fn web_base_url() -> String {
    env::var("ALICE_WEB_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://localhost:3099".into())
}

fn local_api_token() -> String {
    env::var("ALICE_LOCAL_API_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gadugi-local-api-token".into())
}

pub fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
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

pub fn selected_setup_scenarios() -> Vec<(&'static str, Vec<Step>)> {
    let selected = env::var("EATME_SETUP_READINESS_SCENARIO")
        .ok()
        .filter(|value| !value.trim().is_empty());
    match selected.as_deref() {
        Some(id) => setup_scenarios()
            .into_iter()
            .filter(|(scenario, _)| *scenario == id)
            .collect(),
        None => setup_scenarios(),
    }
}

pub fn execute(base: &str, client: &ureq::Agent, steps: &[Step]) -> Vec<StepResult> {
    steps
        .iter()
        .map(|step| execute_step(base, client, step))
        .collect()
}

pub fn assert_all(results: Vec<StepResult>) {
    let failures = results
        .iter()
        .filter(|result| !result.ok)
        .map(|result| format!("{}: {}", result.name, result.msg))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
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

fn execute_step(base: &str, client: &ureq::Agent, step: &Step) -> StepResult {
    match step {
        Step::Health => get_json(
            client,
            &format!("{base}/api/health"),
            "health",
            |h: HealthResponse| (matches!(h.status.as_str(), "ok" | "running"), h.status),
        ),
        Step::Config => get_json(
            client,
            &format!("{base}/api/config"),
            "config",
            |c: ConfigResponse| {
                (
                    c.runtime == "alice-web" && c.platform == "lookingglass",
                    format!("runtime={} platform={}", c.runtime, c.platform),
                )
            },
        ),
        Step::SetupPreflight { scenario } => run_preflight(base, client, scenario),
        Step::ProjectTemplates => get_json(
            client,
            &format!("{base}/api/project/templates"),
            "project-templates",
            |r: TemplatesResponse| {
                (
                    !r.templates.is_empty(),
                    format!("templates={}", r.templates.len()),
                )
            },
        ),
        Step::ProjectNew { project_name } => run_project_new(base, client, project_name),
        Step::Launch => post_json(
            client,
            &format!("{base}/api/launch"),
            ureq::json!({}),
            "launch",
            |r: LaunchResponse| (matches!(r.status.as_str(), "ok" | "launched"), r.status),
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
            Ok(preflight) => StepResult {
                name: format!("setup-preflight({scenario})"),
                ok: preflight_is_ready(&preflight, scenario),
                msg: format!(
                    "status={} platform={} scenario={}",
                    preflight.status, preflight.platform, preflight.scenario
                ),
            },
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
        |r: NewProjectResponse| {
            (
                r.status == "created" && r.project_name == project_name,
                format!("status={} project={}", r.status, r.project_name),
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
        |r: EvidenceHandoffResponse| {
            (
                handoff_is_specific(&r, scenario),
                format!("status={} artifact={}", r.status, r.evidence_artifact),
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
    match client
        .post(url)
        .set("X-Alice-Local-Api-Token", &local_api_token())
        .send_json(body)
    {
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

fn failed(name: impl Into<String>, error: impl std::fmt::Display) -> StepResult {
    StepResult {
        name: name.into(),
        ok: false,
        msg: error.to_string(),
    }
}
