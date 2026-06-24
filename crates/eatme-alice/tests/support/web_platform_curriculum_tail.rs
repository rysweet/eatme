use super::*;

#[test]
fn scene_transition_declares_two_scenes_and_switches_between_them() {
    let (_, steps) = scene_transition();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    let declarations: Vec<_> = entrypoint
        .iter()
        .filter(|statement| statement.kind == "sceneDeclaration")
        .flat_map(|statement| statement.args.iter().cloned())
        .collect();
    assert_eq!(
        declarations,
        vec!["introScene".to_string(), "creditsScene".to_string()]
    );
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("setActiveScene")
            && statement.args == vec!["creditsScene".to_string()]
    }));
}

#[test]
fn property_animation_animates_opacity_and_color() {
    let (_, steps) = property_animation();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(entrypoint.iter().any(|statement| {
        statement.kind == "animateProperty"
            && statement.args
                == vec![
                    "overlay.opacity".to_string(),
                    "1.0".to_string(),
                    "0.25".to_string(),
                    "1.5".to_string(),
                ]
    }));
    assert!(entrypoint.iter().any(|statement| {
        statement.kind == "animateProperty"
            && statement.args
                == vec![
                    "overlay.color".to_string(),
                    "Color.WHITE".to_string(),
                    "Color.BLUE".to_string(),
                    "1.5".to_string(),
                ]
    }));
}

#[test]
fn nested_control_flow_layers_together_branching_and_loops() {
    let (_, steps) = nested_control_flow();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert_eq!(entrypoint[0].kind, "doTogether");
    assert_eq!(entrypoint[1].kind, "ifElse");
    assert_eq!(entrypoint[2].kind, "countLoop");
    assert_eq!(entrypoint[3].method.as_deref(), Some("logicHero.say"));
}

#[test]
fn full_curriculum_breadth_covered() {
    let names: Vec<_> = all_scenarios().iter().map(|(n, _)| *n).collect();
    for required in [
        "hello-world",
        "building-a-scene-first-world",
        "procedures",
        "parameters",
        "inheritance-oop",
        "comments",
        "events-collision",
        "loops-conditionals",
        "functions",
        "variables",
        "concurrency",
        "arrays",
        "project-io",
        "game-narrative",
        "say-think",
        "design-process",
        "camera-viewpoint",
        "vr-camera-locomotion-journey",
        "accessibility-rescue-camera-captions",
        "audio",
        "vehicle-parenting",
        "joint-manipulation",
        "scene-transition",
        "property-animation",
        "nested-control-flow",
        "full-student-journey",
        "instructor-grading",
        "classroom-gallery-walk-and-rubric",
        "error-recovery",
    ] {
        assert!(names.contains(&required), "missing: {required}");
    }
}

#[test]
fn every_scenario_has_at_least_three_steps() {
    for (name, steps) in all_scenarios() {
        assert!(steps.len() >= 3, "{name} has only {} steps", steps.len());
    }
}

#[test]
fn full_student_journey_covers_student_build_run_and_save_flow() {
    let (_, steps) = full_student_journey();
    let add_count = steps
        .iter()
        .filter(|step| matches!(step, Step::AddObject { .. }))
        .count();
    assert_eq!(
        add_count, 3,
        "student journey should add three authored objects"
    );
    assert!(steps.iter().any(|step| {
        matches!(
            step,
            Step::RegisterEvent { event_type, handler_name }
                if event_type == "collision" && handler_name == "onStudentCollision"
        )
    }));
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(
        entrypoint
            .iter()
            .any(|statement| statement.kind == "countLoop")
    );
    assert!(
        entrypoint
            .iter()
            .any(|statement| statement.kind == "ifElse")
    );
    assert!(
        steps.iter().any(
            |step| matches!(step, Step::Save { path } if path == FULL_STUDENT_JOURNEY_SAVE_PATH)
        )
    );
}

#[test]
fn building_a_scene_first_world_covers_adjust_run_and_save_flow() {
    let (_, steps) = building_a_scene_first_world();
    assert!(steps.iter().any(|step| matches!(step, Step::AddObject { instance_name, .. } if instance_name == "bunny")));
    assert!(steps.iter().any(|step| matches!(step, Step::TransformObject { object_name, .. } if object_name == "bunny")));
    assert!(steps.iter().any(|step| matches!(step, Step::RunWorld)));
    assert!(steps.iter().any(
        |step| matches!(step, Step::Save { path } if path == BUILDING_A_SCENE_SAVE_PATH)
    ));
}

#[test]
fn instructor_grading_round_trips_saved_project_structure() {
    let (_, steps) = instructor_grading();
    let save_index = steps
        .iter()
        .position(
            |step| matches!(step, Step::Save { path } if path == INSTRUCTOR_GRADING_SAVE_PATH),
        )
        .expect("grading scenario should save work");
    let load_index = steps
        .iter()
        .position(
            |step| matches!(step, Step::Load { path } if path == INSTRUCTOR_GRADING_SAVE_PATH),
        )
        .expect("grading scenario should load saved work");
    assert!(
        save_index < load_index,
        "saved work should be loaded after save"
    );
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(
        entrypoint
            .iter()
            .any(|statement| statement.method.as_deref() == Some("learner.walk"))
    );
    assert!(steps.iter().any(|step| matches!(step, Step::RunWorld)));
}

#[test]
fn classroom_gallery_walk_records_gallery_rubric_evidence() {
    let (_, steps) = classroom_gallery_walk_and_rubric();
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::GalleryWalkRubricEvidence)),
        "gallery walk should prove web review/rubric evidence"
    );
    assert_eq!(
        steps
            .iter()
            .filter(|step| matches!(step, Step::AddObject { .. }))
            .count(),
        2,
        "gallery review should have visible project items to review"
    );
}

#[test]
fn live_classroom_gallery_walk_exercises_rubric_api() {
    let (name, steps) = classroom_gallery_walk_and_rubric();
    assert_live_scenario(name, steps);
}

#[test]
fn error_recovery_expects_failures_and_then_recovers() {
    let (_, steps) = error_recovery();
    let error_steps: Vec<_> = steps
        .iter()
        .filter(|step| matches!(step, Step::ExpectError { .. }))
        .collect();
    assert_eq!(
        error_steps.len(),
        2,
        "recovery scenario should intentionally exercise two failure modes"
    );
    assert!(steps.iter().any(|step| matches!(step, Step::AddObject { instance_name, .. } if instance_name == "resilientHero")));
    assert!(steps.iter().any(|step| matches!(step, Step::RunWorld)));
}

#[test]
fn live_hello_world() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &hello_world().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_building_a_scene_first_world() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &building_a_scene_first_world().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_procedures() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &procedures().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_full_student_journey() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &full_student_journey().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_instructor_grading() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &instructor_grading().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_error_recovery() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    for r in execute(&b, &c, &error_recovery().1) {
        assert!(r.ok, "{}: {}", r.name, r.msg);
    }
}

#[test]
fn live_all_curriculum() {
    if !web_platform_enabled() {
        eprintln!("skip (set EATME_WEB_PLATFORM=1)");
        return;
    }
    let c = http_client();
    let b = web_base_url();
    let mut fails = Vec::new();
    for (name, steps) in all_scenarios() {
        for r in execute(&b, &c, &steps) {
            if !r.ok {
                fails.push(format!("{name}/{}: {}", r.name, r.msg));
            }
        }
    }
    assert!(fails.is_empty(), "failures:\n{}", fails.join("\n"));
}
