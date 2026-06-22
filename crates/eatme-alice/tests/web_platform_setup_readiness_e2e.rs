//! Focused web platform setup/readiness scenario tests.

use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::time::Duration;

fn web_platform_enabled() -> bool {
    env::var("EATME_WEB_PLATFORM")
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn web_base_url() -> String {
    env::var("ALICE_WEB_URL").unwrap_or_else(|_| "http://localhost:3099".into())
}

fn http_client() -> ureq::Agent {
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
enum Step {
    Health,
    Config,
    SetupPreflight { scenario: String },
    ProjectTemplates,
    ProjectNew { project_name: String },
    Launch,
    EvidenceHandoff { scenario: String },
}

#[derive(Debug)]
struct StepResult {
    name: String,
    ok: bool,
    msg: String,
}

fn setup_readiness_steps(scenario: &'static str, project_name: &'static str) -> Vec<Step> {
    vec![
        Step::Health,
        Step::Config,
        Step::SetupPreflight {
            scenario: scenario.into(),
        },
        Step::ProjectTemplates,
        Step::ProjectNew {
            project_name: project_name.into(),
        },
        Step::Launch,
        Step::EvidenceHandoff {
            scenario: scenario.into(),
        },
    ]
}

fn setup_scenarios() -> Vec<(&'static str, Vec<Step>)> {
    vec![
        (
            "setup-preflight-ready-to-create",
            setup_readiness_steps(
                "setup-preflight-ready-to-create",
                "Setup Preflight Ready to Create",
            ),
        ),
        (
            "setup-support-lab-readiness",
            setup_readiness_steps("setup-support-lab-readiness", "Setup Support Lab Readiness"),
        ),
        (
            "instructor-classroom-setup-readiness",
            setup_readiness_steps(
                "instructor-classroom-setup-readiness",
                "Instructor Classroom Setup Readiness",
            ),
        ),
        (
            "instructor-student-launch-evidence-handoff",
            setup_readiness_steps(
                "instructor-student-launch-evidence-handoff",
                "Student Launch Evidence Handoff",
            ),
        ),
    ]
}

fn execute(base: &str, client: &ureq::Agent, steps: &[Step]) -> Vec<StepResult> {
    steps
        .iter()
        .map(|step| match step {
            Step::Health => match client.get(&format!("{base}/api/health")).call() {
                Ok(resp) => match resp.into_json::<HealthResponse>() {
                    Ok(health) => StepResult {
                        name: "health".into(),
                        ok: matches!(health.status.as_str(), "ok" | "running"),
                        msg: health.status,
                    },
                    Err(error) => failed("health", error),
                },
                Err(error) => failed("health", error),
            },
            Step::Config => match client.get(&format!("{base}/api/config")).call() {
                Ok(resp) => match resp.into_json::<ConfigResponse>() {
                    Ok(config) => StepResult {
                        name: "config".into(),
                        ok: config.runtime == "alice-web" && config.platform == "lookingglass",
                        msg: format!("runtime={} platform={}", config.runtime, config.platform),
                    },
                    Err(error) => failed("config", error),
                },
                Err(error) => failed("config", error),
            },
            Step::SetupPreflight { scenario } => run_preflight(base, client, scenario),
            Step::ProjectTemplates => {
                match client.get(&format!("{base}/api/project/templates")).call() {
                    Ok(resp) => match resp.into_json::<TemplatesResponse>() {
                        Ok(response) => StepResult {
                            name: "project-templates".into(),
                            ok: !response.templates.is_empty(),
                            msg: format!("templates={}", response.templates.len()),
                        },
                        Err(error) => failed("project-templates", error),
                    },
                    Err(error) => failed("project-templates", error),
                }
            }
            Step::ProjectNew { project_name } => {
                match client
                    .post(&format!("{base}/api/project/new"))
                    .send_json(ureq::json!({ "projectName": project_name }))
                {
                    Ok(resp) => match resp.into_json::<NewProjectResponse>() {
                        Ok(response) => StepResult {
                            name: format!("project-new({project_name})"),
                            ok: response.status == "created"
                                && response.project_name == *project_name,
                            msg: format!(
                                "status={} project={}",
                                response.status, response.project_name
                            ),
                        },
                        Err(error) => failed(format!("project-new({project_name})"), error),
                    },
                    Err(error) => failed(format!("project-new({project_name})"), error),
                }
            }
            Step::Launch => match client
                .post(&format!("{base}/api/launch"))
                .send_json(ureq::json!({}))
            {
                Ok(resp) => match resp.into_json::<LaunchResponse>() {
                    Ok(response) => StepResult {
                        name: "launch".into(),
                        ok: matches!(response.status.as_str(), "ok" | "launched"),
                        msg: response.status,
                    },
                    Err(error) => failed("launch", error),
                },
                Err(error) => failed("launch", error),
            },
            Step::EvidenceHandoff { scenario } => run_handoff(base, client, scenario),
        })
        .collect()
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

fn run_handoff(base: &str, client: &ureq::Agent, scenario: &str) -> StepResult {
    match client
        .post(&format!("{base}/api/setup/evidence-handoff"))
        .send_json(ureq::json!({ "scenario": scenario }))
    {
        Ok(resp) => match resp.into_json::<EvidenceHandoffResponse>() {
            Ok(response) => {
                let next_actions = response
                    .handoff
                    .get("studentNextActions")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items.iter().any(|item| {
                            item.as_str()
                                .map(|text| text.contains("visible result"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);
                StepResult {
                    name: format!("evidence-handoff({scenario})"),
                    ok: response.status == "handoff-created"
                        && response.platform == "lookingglass"
                        && response.scenario == scenario
                        && !response.evidence_artifact.is_empty()
                        && next_actions,
                    msg: format!(
                        "status={} artifact={}",
                        response.status, response.evidence_artifact
                    ),
                }
            }
            Err(error) => failed(format!("evidence-handoff({scenario})"), error),
        },
        Err(error) => failed(format!("evidence-handoff({scenario})"), error),
    }
}

fn failed(name: impl Into<String>, error: impl std::fmt::Display) -> StepResult {
    StepResult {
        name: name.into(),
        ok: false,
        msg: error.to_string(),
    }
}

fn assert_all(results: Vec<StepResult>) {
    let failures = results
        .iter()
        .filter(|result| !result.ok)
        .map(|result| format!("{}: {}", result.name, result.msg))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}

#[test]
fn setup_readiness_scenarios_exercise_preflight_config_create_and_handoff() {
    for (name, steps) in setup_scenarios() {
        assert!(steps.iter().any(|step| matches!(step, Step::Config)));
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::SetupPreflight { scenario } if scenario == name))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::ProjectNew { .. }))
        );
        assert!(
            steps
                .iter()
                .any(|step| matches!(step, Step::EvidenceHandoff { scenario } if scenario == name))
        );
    }
}

#[test]
fn live_setup_readiness_scenarios() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }

    let client = http_client();
    let base = web_base_url();
    for (_name, steps) in setup_scenarios() {
        assert_all(execute(&base, &client, &steps));
    }
}
