use crate::schema::{CrewAsset, EatmeScenarioAsset, Scenario};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn full_student_journey_threads_project_scene_code_loop_event_run_and_save() {
    let crew = load_crew();
    let journey = [
        find_scenario(&crew, "building-a-scene-first-world"),
        find_scenario(&crew, "code-editor-first-run"),
        find_scenario(&crew, "creature-choreography-loop-lab"),
        find_scenario(&crew, "events-collision-proximity-game"),
    ];
    let starter_project = load_scenario_asset("starter-project-open-save-export-preflight");
    let combined = normalize(&format!(
        "{}\n{}",
        journey
            .iter()
            .map(|scenario| scenario_text(scenario))
            .collect::<Vec<_>>()
            .join("\n"),
        asset_text(&starter_project)
    ));

    for needle in [
        "starter project",
        "virtual world",
        "procedure",
        "loop",
        "event",
        "run",
        "save",
    ] {
        assert!(
            combined.contains(needle),
            "full student journey should include {needle:?}: {combined}"
        );
    }
    assert!(
        starter_project
            .steps
            .iter()
            .any(|step| step.id == "record-starter-world-change")
    );
    assert!(
        starter_project
            .steps
            .iter()
            .any(|step| step.id == "launch-smoke")
    );
    assert!(
        starter_project
            .steps
            .iter()
            .any(|step| step.id == "record-run-observe-readiness-gaps")
    );
}

#[test]
fn instructor_review_assets_cover_rubric_grading_and_feedback_boundaries() {
    let crew = load_crew();
    let reflection_review = find_scenario(&crew, "student-reflection-artifact-review");
    let gallery_walk = load_scenario_asset("classroom-gallery-walk-and-rubric");
    let outcomes_rubric = load_scenario_asset("instructor-student-outcomes-rubric");
    let combined = normalize(&format!(
        "{}\n{}\n{}",
        scenario_text(reflection_review),
        asset_text(&gallery_walk),
        asset_text(&outcomes_rubric)
    ));

    for needle in [
        "feedback",
        "revision",
        "human review",
        "visible alice project behavior",
        "student-owned next revision",
    ] {
        assert!(
            combined.contains(needle),
            "instructor review coverage should include {needle:?}: {combined}"
        );
    }
    assert_eq!(
        gallery_walk
            .agentic_flow
            .as_ref()
            .unwrap()
            .expected_outputs
            .len(),
        5
    );
    assert!(
        outcomes_rubric
            .rubric
            .iter()
            .any(|criterion| criterion.criterion == "Concept evidence")
    );
}

#[test]
fn class_roster_maps_five_students_to_five_distinct_projects_and_shared_grading() {
    let crew = load_crew();
    let roster = [
        ("curious-novice", "building-a-scene-first-world"),
        ("creative-storyteller", "creature-choreography-loop-lab"),
        ("playful-tinkerer", "loops-and-conditionals-mini-challenge"),
        ("systems-puzzle-solver", "game-score-timer-win-lose-loop"),
        ("game-narrative-designer", "mythic-choice-event-tree"),
    ];
    let mut submissions = BTreeMap::new();
    for (student_id, scenario_id) in roster {
        let scenario = find_scenario(&crew, scenario_id);
        assert!(
            scenario
                .personas
                .students
                .iter()
                .any(|student| student == student_id),
            "{scenario_id} should support {student_id}"
        );
        submissions.insert(student_id, scenario_id);
    }

    assert_eq!(submissions.len(), 5);
    assert_eq!(
        submissions
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );

    let gallery_walk = load_scenario_asset("classroom-gallery-walk-and-rubric");
    assert!(gallery_walk.acceptance_probes.iter().any(|probe| {
        normalize(probe).contains("stories, games, simulations, media scenes, or starter projects")
    }));
}

#[test]
fn cross_lesson_reference_reuses_lesson_two_procedure_concept_in_later_sequence() {
    let crew = load_crew();
    let lesson_two = find_scenario(&crew, "code-editor-first-run");
    let later_lesson = find_scenario(&crew, "creature-choreography-loop-lab");
    let sequence = find_scenario(&crew, "curriculum-sequence-remix-pack");
    let lesson_two_concepts = normalize(&lesson_two.educational_intent.concepts.join(" "));
    let later_concepts = normalize(&later_lesson.educational_intent.concepts.join(" "));
    let sequence_text = normalize(&scenario_text(sequence));

    assert!(lesson_two_concepts.contains("procedure"));
    assert!(later_concepts.contains("procedure"));
    assert!(sequence_text.contains("prerequisite"));
    assert!(sequence_text.contains("concept progression"));
}

#[test]
fn export_portfolio_collects_completed_lessons_into_shareable_teacher_and_student_handoffs() {
    let completed_lessons = [
        "building-a-scene-first-world",
        "code-editor-first-run",
        "creature-choreography-loop-lab",
        "events-collision-proximity-game",
        "starter-project-open-save-export-preflight",
    ];
    let portfolio = completed_lessons
        .iter()
        .map(|lesson| (*lesson, format!("agentic://portfolio/{lesson}")))
        .collect::<BTreeMap<_, _>>();
    let share_packet = load_scenario_asset("student-artifact-package-share-evidence");
    let teacher_share = load_scenario_asset("teacher-community-sharing-loop");
    let combined = normalize(&format!(
        "{}\n{}",
        asset_text(&share_packet),
        asset_text(&teacher_share)
    ));

    assert_eq!(portfolio.len(), completed_lessons.len());
    for needle in [
        "artifact or screenshot reference",
        "attribution",
        "classroom handoff note",
        "student evidence",
        "next revision",
    ] {
        assert!(
            combined.contains(needle),
            "portfolio export coverage should include {needle:?}: {combined}"
        );
    }
    assert!(
        teacher_share
            .artifacts
            .contains_key("teacher_community_share_card")
    );
    assert!(
        share_packet
            .artifacts
            .contains_key("artifact_share_packet_checklist")
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_crew() -> CrewAsset {
    let path = repository_root().join("assets/personas/alice-user-crew.yaml");
    serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn load_scenario_asset(id: &str) -> EatmeScenarioAsset {
    let path = repository_root()
        .join("assets/scenarios/eatme")
        .join(format!("{id}.yaml"));
    serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn find_scenario<'a>(crew: &'a CrewAsset, id: &str) -> &'a Scenario {
    crew.core_scenarios_from_existing_alice_resources
        .iter()
        .chain(crew.creative_new_teaching_learning_scenarios.iter())
        .find(|scenario| scenario.id == id)
        .unwrap_or_else(|| panic!("missing scenario {id}"))
}

fn scenario_text(scenario: &Scenario) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        scenario.id,
        scenario.user_story,
        scenario.educational_intent.concepts.join(" "),
        scenario.educational_intent.habits.join(" "),
        scenario.observable_behaviors.instructor.join(" "),
        scenario.observable_behaviors.student.join(" "),
        scenario.acceptance_probes.join(" ")
    )
}

fn asset_text(asset: &EatmeScenarioAsset) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {}",
        asset.id,
        asset.purpose,
        asset.acceptance_probes.join(" "),
        asset
            .acceptance_criteria
            .iter()
            .map(|criterion| format!("{} {} {}", criterion.given, criterion.when, criterion.then))
            .collect::<Vec<_>>()
            .join(" "),
        asset
            .rubric
            .iter()
            .map(|criterion| format!("{} {}", criterion.criterion, criterion.evidence.join(" ")))
            .collect::<Vec<_>>()
            .join(" "),
        asset
            .steps
            .iter()
            .map(|step| format!("{} {}", step.id, step.evidence.join(" ")))
            .collect::<Vec<_>>()
            .join(" "),
        asset
            .artifacts
            .iter()
            .map(|(key, value)| format!("{key} {value}"))
            .collect::<Vec<_>>()
            .join(" "),
        asset
            .agentic_flow
            .as_ref()
            .map(|flow| flow.expected_outputs.join(" "))
            .unwrap_or_default(),
        asset.unsupported_policy,
    )
}

fn normalize(text: &str) -> String {
    text.split_whitespace()
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}
