use crate::scenario::LaunchSmokeScenario;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct LessonSessionComparisonContract {
    pub schema_version: String,
    pub scenario_id: String,
    pub session_kind: String,
    pub automation_status: String,
    pub actor_roles: Vec<String>,
    pub required_session_steps: Vec<String>,
    pub executable_evidence: Vec<String>,
    pub boundaries: Vec<String>,
}

pub(super) fn lesson_session_contract(
    scenario: &LaunchSmokeScenario,
) -> LessonSessionComparisonContract {
    if scenario.requires_real_ui_actions() {
        return LessonSessionComparisonContract {
            schema_version: "eatme.alice-lesson-session-contract/v1".into(),
            scenario_id: scenario.id.clone(),
            session_kind: "first_lesson_action_contract".into(),
            automation_status: "action_contract_blocked_until_ui_automation".into(),
            actor_roles: vec![
                "instructor prepares the Alice classroom task".into(),
                "student opens, changes, runs, saves, and reflects on an Alice project".into(),
            ],
            required_session_steps: vec![
                "instructor selects an Alice lesson objective and starter project".into(),
                "student opens the configured starter project in Alice".into(),
                "student places or modifies an object in the scene".into(),
                "student edits a procedure or code block".into(),
                "student runs the world and observes the visible result".into(),
                "student saves the project and records one next revision".into(),
            ],
            executable_evidence: vec![
                "comparison manifest records both target runs under the same scenario id".into(),
                "target launch manifests record dependency, package, display, window, screenshot, log, and assertion evidence".into(),
                "ui-action-contract.json names the required actions that are not automated yet".into(),
            ],
            boundaries: shared_boundaries(),
        };
    }

    let (session_kind, automation_status) = if scenario.id == "real-alice-launch-smoke" {
        ("launch_readiness", "launch_smoke_only")
    } else {
        (
            "lesson_labeled_launch_smoke",
            "lesson_labeled_launch_without_action_automation",
        )
    };

    LessonSessionComparisonContract {
        schema_version: "eatme.alice-lesson-session-contract/v1".into(),
        scenario_id: scenario.id.clone(),
        session_kind: session_kind.into(),
        automation_status: automation_status.into(),
        actor_roles: vec![
            "instructor uses target readiness evidence before class".into(),
            "student work remains outside automated assessment for this scenario".into(),
        ],
        required_session_steps: vec![
            "resolve and prepare both Alice targets".into(),
            "package each target when execution is requested".into(),
            "launch each target under an isolated virtual display".into(),
            "capture manifest, window, screenshot, log, assertion, and timing evidence".into(),
        ],
        executable_evidence: vec![
            "comparison manifest records target metadata, status, scorecard, timing, and differences".into(),
            "target launch manifests are attached when execution is requested and reaches launch smoke".into(),
        ],
        boundaries: shared_boundaries(),
    }
}

fn shared_boundaries() -> Vec<String> {
    vec![
        "does not automate complete instructor assignment creation".into(),
        "does not automate complete student lesson consumption".into(),
        "does not perform creative assessment".into(),
        "does not grade student worlds".into(),
        "does not prove broad Alice compatibility beyond the selected scenario".into(),
    ]
}
