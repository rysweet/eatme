//! Scene-building grading — covers the "Getting Started / Building a Scene" lesson.

use eatme_core::ast::{SceneLayout, SceneObject};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    blocked_by_reason, build_preconditions, cascade_blocked, no_scene_reason,
};

pub struct SceneBuildingGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_scene: Option<SceneLayout>,
}

pub fn grade_scene_building(input: SceneBuildingGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("add-ground", &["launch-smoke"]),
            cascade_blocked("add-sky", &["launch-smoke"]),
            cascade_blocked("place-scene-objects", &["launch-smoke"]),
            cascade_blocked("position-camera", &["launch-smoke"]),
            cascade_blocked("set-object-properties", &["place-scene-objects"]),
            StepGrade {
                name: "save-project".into(),
                status: StepStatus::Blocked,
                reason: blocked_by_reason(
                    "save-project",
                    &[
                        "add-ground",
                        "add-sky",
                        "place-scene-objects",
                        "position-camera",
                        "set-object-properties",
                    ],
                ),
                depends_on: vec![
                    "add-ground".into(),
                    "add-sky".into(),
                    "place-scene-objects".into(),
                    "position-camera".into(),
                    "set-object-properties".into(),
                ],
            },
        ]
    } else {
        evaluate_scene_steps(&input.student_scene)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "building-a-scene-first-world",
        passed,
        steps,
    )
}

fn evaluate_scene_steps(scene: &Option<SceneLayout>) -> Vec<StepGrade> {
    let Some(scene) = scene else {
        return vec![
            missing_scene_step("add-ground", &["launch-smoke"]),
            missing_scene_step("add-sky", &["launch-smoke"]),
            missing_scene_step("place-scene-objects", &["launch-smoke"]),
            missing_scene_step("position-camera", &["launch-smoke"]),
            missing_scene_step("set-object-properties", &["place-scene-objects"]),
            StepGrade {
                name: "save-project".into(),
                status: StepStatus::Blocked,
                reason: no_scene_reason("save-project"),
                depends_on: vec![
                    "add-ground".into(),
                    "add-sky".into(),
                    "place-scene-objects".into(),
                    "position-camera".into(),
                    "set-object-properties".into(),
                ],
            },
        ];
    };

    let has_ground = scene.ground_present;
    let has_sky = scene.sky_present;
    let has_two_objects = scene.objects.len() >= 2;
    let has_camera = scene.camera.is_some();
    let has_properties = has_two_objects && scene.objects.iter().all(has_all_visual_properties);
    let all_met = has_ground && has_sky && has_two_objects && has_camera && has_properties;

    vec![
        ready_or_blocked(
            "add-ground",
            &["launch-smoke"],
            has_ground,
            "Ground found in scene",
            "No ground found in scene",
        ),
        ready_or_blocked(
            "add-sky",
            &["launch-smoke"],
            has_sky,
            "Sky found in scene",
            "No sky found in scene",
        ),
        ready_or_blocked(
            "place-scene-objects",
            &["launch-smoke"],
            has_two_objects,
            "At least two placed objects found in scene",
            "Fewer than two placed objects found in scene",
        ),
        ready_or_blocked(
            "position-camera",
            &["launch-smoke"],
            has_camera,
            "Camera positioning found in scene",
            "No camera positioning found in scene",
        ),
        ready_or_blocked(
            "set-object-properties",
            &["place-scene-objects"],
            has_properties,
            "Placed objects include position, size, color, and opacity",
            "Placed objects are missing position, size, color, or opacity",
        ),
        ready_or_blocked(
            "save-project",
            &[
                "add-ground",
                "add-sky",
                "place-scene-objects",
                "position-camera",
                "set-object-properties",
            ],
            all_met,
            "Scene completeness verified for save/reopen grading",
            "Scene is incomplete for save/reopen grading",
        ),
    ]
}

fn has_all_visual_properties(object: &SceneObject) -> bool {
    object.position.is_some()
        && object.size.is_some()
        && object.color.as_ref().is_some_and(|color| !color.is_empty())
        && object.opacity.is_some()
}

fn ready_or_blocked(
    name: &str,
    deps: &[&str],
    ready: bool,
    success_reason: &str,
    failure_reason: &str,
) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: if ready {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: if ready {
            format!("{success_reason}. Keep the scene saved so you can use it in the next step.")
        } else {
            format!(
                "{failure_reason}. Update the scene in Alice, save the project, and rerun grading."
            )
        },
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

fn missing_scene_step(name: &str, deps: &[&str]) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: no_scene_reason(name),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}
