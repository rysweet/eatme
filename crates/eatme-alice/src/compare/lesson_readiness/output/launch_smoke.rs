use super::{LessonSessionReadinessEnvelope, ReadinessOutput, normalized_readiness_status};

pub(in crate::compare::lesson_readiness) fn build_launch_smoke_readiness_output(
    scenario_id: Option<&str>,
    readiness_status: &str,
    role_statuses: &[(&str, &str)],
) -> ReadinessOutput {
    let scenario = scenario_id.unwrap_or("real-alice-launch-smoke");
    let status = normalized_readiness_status(readiness_status).to_string();
    let blocked_reason = None;
    let required_evidence = launch_smoke_required_evidence();
    let human_summary = launch_smoke_human_summary(scenario, &status);
    let role_readiness =
        launch_smoke_role_readiness(scenario_id, scenario, role_statuses, &required_evidence);
    let lesson_session_readiness = launch_smoke_session_readiness(
        scenario_id,
        &status,
        &human_summary,
        &required_evidence,
        &role_readiness,
    );

    ReadinessOutput {
        status,
        blocked_reason,
        human_summary,
        required_evidence,
        no_go_contracts: Vec::new(),
        lesson_session_readiness,
        role_readiness,
    }
}

fn launch_smoke_human_summary(scenario: &str, status: &str) -> String {
    match status {
        "ready" => format!(
            "{scenario} launch-smoke readiness is ready from existing target launch-smoke manifest evidence only."
        ),
        _ => format!(
            "{scenario} launch-smoke readiness is not ready because required launch-smoke manifest evidence is missing, failed, malformed, or incomplete."
        ),
    }
}

fn launch_smoke_role_readiness(
    scenario_id: Option<&str>,
    scenario: &str,
    role_statuses: &[(&str, &str)],
    required_evidence: &[String],
) -> Vec<LessonSessionReadinessEnvelope> {
    role_statuses
        .iter()
        .map(|(role, role_status)| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: (*role).into(),
            status: (*role_status).into(),
            blocked_reason: None,
            human_summary: launch_smoke_role_summary(scenario, role, role_status),
            required_evidence: required_evidence.to_vec(),
            no_go_contracts: Vec::new(),
        })
        .collect()
}

fn launch_smoke_session_readiness(
    scenario_id: Option<&str>,
    status: &str,
    human_summary: &str,
    required_evidence: &[String],
    role_readiness: &[LessonSessionReadinessEnvelope],
) -> LessonSessionReadinessEnvelope {
    role_readiness
        .iter()
        .find(|readiness| readiness.role == "modernized")
        .cloned()
        .unwrap_or_else(|| LessonSessionReadinessEnvelope {
            scenario_id: scenario_id.map(str::to_string),
            role: "launch-smoke".into(),
            status: status.into(),
            blocked_reason: None,
            human_summary: human_summary.into(),
            required_evidence: required_evidence.to_vec(),
            no_go_contracts: Vec::new(),
        })
}

fn launch_smoke_required_evidence() -> Vec<String> {
    [
        "comparison manifest with baseline and modernized targets for real-alice-launch-smoke",
        "embedded launch-smoke manifest for each target",
        "each target status is passed with no launch failure category",
        "required launch-smoke assertions passed for each target",
        "launch-smoke artifact metadata for window list, screenshot, and log",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn launch_smoke_role_summary(scenario: &str, role: &str, status: &str) -> String {
    if status == "ready" {
        format!("{scenario} {role} target has bounded launch-smoke manifest evidence only.")
    } else {
        format!(
            "{scenario} {role} target launch-smoke manifest evidence is missing, failed, malformed, or incomplete."
        )
    }
}
