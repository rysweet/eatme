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
fn game_narrative_tracks_score_and_win_state() {
    let (_, steps) = game_narrative();
    assert!(steps.iter().any(|step| {
        matches!(
            step,
            Step::RegisterEvent { event_type, handler_name }
                if event_type == "keyPress" && handler_name == "onSpacePressed"
        )
    }));

    let handler = edit_statements(&steps, "onSpacePressed");
    let score_declaration = handler
        .iter()
        .find(|statement| statement.kind == "localDeclaration")
        .expect("game narrative should declare a score variable");
    assert_eq!(
        score_declaration.args,
        vec!["score".to_string(), "0".to_string()]
    );

    let score_update = handler
        .iter()
        .find(|statement| statement.kind == "assignment")
        .expect("game narrative should update the score");
    assert_eq!(
        score_update.args,
        vec!["score".to_string(), "score + 1".to_string()]
    );

    let win_check = handler
        .iter()
        .find(|statement| statement.kind == "ifElse")
        .expect("game narrative should define a win condition");
    assert_eq!(win_check.args, vec!["score >= 3".to_string()]);

    assert!(handler.iter().any(|statement| {
        statement.method.as_deref() == Some("player.say")
            && statement.args == vec!["\"You win!\"".to_string()]
    }));
}

#[test]
fn say_think_uses_speech_and_thought_bubbles() {
    let (_, steps) = say_think();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("speaker.say")
            && statement.args == vec!["\"Welcome to the bubble lab\"".to_string()]
    }));
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("speaker.think")
            && statement.args == vec!["\"I should keep this plan quiet\"".to_string()]
    }));
}

#[test]
fn design_process_tracks_plan_build_playtest_and_revision() {
    let (_, steps) = design_process();
    let first_run_index = steps
        .iter()
        .position(|step| matches!(step, Step::RunWorld))
        .expect("design process should run the prototype");
    let revision_index = steps
        .iter()
        .position(|step| {
            matches!(step, Step::EditProcedure { statements, .. } if statements
                .iter()
                .any(|statement| statement
                    .args
                    .iter()
                    .any(|arg| arg.contains("Revision: show win feedback"))))
        })
        .expect("design process should revise after playtest");
    let evidence_index = steps
        .iter()
        .position(|step| matches!(step, Step::DesignProcessEvidence))
        .expect("design process should record evidence through the LookingGlass API");
    let run_count = steps
        .iter()
        .filter(|step| matches!(step, Step::RunWorld))
        .count();

    assert_eq!(
        run_count, 2,
        "revision loop should run before and after revise"
    );
    assert!(
        first_run_index < revision_index && revision_index < evidence_index,
        "evidence follows playtest and revision"
    );

    let payload = design_process_evidence_payload();
    let phases = ["plan", "build", "playtest", "revise", "review"];
    for phase in phases {
        assert!(
            payload.to_string().contains(phase),
            "design-process payload should cover {phase}"
        );
    }
}

#[test]
fn live_design_process_records_playtest_revision_and_review_evidence() {
    let (name, steps) = design_process();
    assert_live_scenario(name, steps);
}

#[test]
fn vehicle_parenting_attaches_camera_to_character() {
    let (_, steps) = vehicle_parenting();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("camera.setVehicle")
            && statement.args == vec!["driver".to_string()]
    }));
    assert!(entrypoint.iter().any(|statement| {
        statement.method.as_deref() == Some("driver.walk")
            && statement.args == vec!["1.0".to_string()]
    }));
}

#[test]
fn joint_manipulation_targets_biped_joints() {
    let (_, steps) = joint_manipulation();
    let entrypoint = edit_statements(&steps, "myFirstMethod");
    let joint_methods: Vec<_> = entrypoint
        .iter()
        .filter_map(|statement| statement.method.as_deref())
        .collect();
    assert!(joint_methods.contains(&"dancer.rightShoulder.turn"));
    assert!(joint_methods.contains(&"dancer.leftKnee.turn"));
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
