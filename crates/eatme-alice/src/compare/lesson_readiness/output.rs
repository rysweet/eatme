use super::LessonSessionNoGoContract;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionReadinessEnvelope {
    pub scenario_id: Option<String>,
    pub role: String,
    pub status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
}

pub(super) struct ReadinessOutput {
    pub status: String,
    pub blocked_reason: Option<String>,
    pub human_summary: String,
    pub required_evidence: Vec<String>,
    pub no_go_contracts: Vec<LessonSessionNoGoContract>,
    pub lesson_session_readiness: LessonSessionReadinessEnvelope,
    pub role_readiness: Vec<LessonSessionReadinessEnvelope>,
}

pub(super) fn build_readiness_output(
    scenario_id: Option<&str>,
    readiness_status: &str,
    has_issues: bool,
    no_go_contracts: Vec<LessonSessionNoGoContract>,
    default_scenario_id: &str,
) -> ReadinessOutput {
    let status = normalized_readiness_status(readiness_status).to_string();
    let blocked_reason = (status == "blocked").then(|| readiness_status.to_string());
    let required_evidence = required_evidence();
    let human_summary = human_summary(
        scenario_id,
        &status,
        blocked_reason.as_deref(),
        has_issues,
        default_scenario_id,
    );
    let role_readiness = ["instructor", "student"]
        .into_iter()
        .map(|role| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: role.into(),
            status: status.clone(),
            blocked_reason: blocked_reason.clone(),
            human_summary: human_summary.clone(),
            required_evidence: required_evidence.clone(),
            no_go_contracts: no_go_contracts.clone(),
        })
        .collect::<Vec<_>>();
    let lesson_session_readiness = role_readiness
        .iter()
        .find(|readiness| readiness.role == "student")
        .cloned()
        .unwrap_or_else(|| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: "student".into(),
            status: status.clone(),
            blocked_reason: blocked_reason.clone(),
            human_summary: human_summary.clone(),
            required_evidence: required_evidence.clone(),
            no_go_contracts: no_go_contracts.clone(),
        });

    ReadinessOutput {
        status,
        blocked_reason,
        human_summary,
        required_evidence,
        no_go_contracts,
        lesson_session_readiness,
        role_readiness,
    }
}

fn normalized_readiness_status(readiness_status: &str) -> &'static str {
    match readiness_status {
        "ready" => "ready",
        "blocked_until_ui_automation" => "blocked",
        _ => "not_ready",
    }
}

fn required_evidence() -> Vec<String> {
    ["comparison-manifest.json", "ui-action-contract.json"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn human_summary(
    scenario_id: Option<&str>,
    status: &str,
    blocked_reason: Option<&str>,
    has_issues: bool,
    default_scenario_id: &str,
) -> String {
    let scenario = scenario_id.unwrap_or(default_scenario_id);
    match status {
        "ready" => format!(
            "{scenario} has complete comparison and UI action evidence with no accepted blockers."
        ),
        "blocked" => format!(
            "{scenario} has launch/action-contract evidence but is blocked until deterministic desktop UI automation exists ({reason}).",
            reason = blocked_reason.unwrap_or("blocked")
        ),
        "not_ready" if has_issues => format!(
            "{scenario} readiness evidence is not ready because required comparison or UI action evidence is missing, invalid, stale, or inconsistent."
        ),
        _ => format!("{scenario} readiness evidence is not ready."),
    }
}
