use std::fs;

use super::{
    TARGET_SCENARIOS, assert_contains_all, assert_not_contains_any, forbidden_internal_shorthand,
    read_eatme_scenario, repository_root, scenario_path,
};

#[test]
fn target_scenarios_use_required_personas_and_real_alice_gate_without_ci_auto_run() {
    let root = repository_root();
    let mut failures = Vec::new();

    for target in TARGET_SCENARIOS {
        let eatme_path = scenario_path(&root, "eatme", target.id);
        if !eatme_path.is_file() {
            failures.push(format!("{} is missing", eatme_path.display()));
            continue;
        }

        let scenario = read_eatme_scenario(&eatme_path);
        if scenario.kind != "alice_lesson_smoke" {
            failures.push(format!(
                "{} kind must be alice_lesson_smoke, got {}",
                target.id, scenario.kind
            ));
        }
        if scenario
            .launcher
            .as_ref()
            .map(|launcher| launcher.scenario.as_str())
            != Some(target.id)
        {
            failures.push(format!(
                "{} launcher.scenario must match the scenario id",
                target.id
            ));
        }
        if scenario
            .real_alice
            .as_ref()
            .map(|real_alice| real_alice.gated_by.as_str())
            != Some("EATME_REAL_ALICE=1")
        {
            failures.push(format!(
                "{} must keep real Alice execution behind EATME_REAL_ALICE=1",
                target.id
            ));
        }
        if !scenario
            .steps
            .iter()
            .any(|step| step.command.contains("EATME_REAL_ALICE=1"))
        {
            failures.push(format!(
                "{} must document the explicit manual real-Alice gate in a smoke step",
                target.id
            ));
        }
        if !scenario.steps.iter().any(|step| {
            step.command.contains("alice launch-smoke")
                && step
                    .evidence
                    .iter()
                    .any(|evidence| evidence.contains("real_alice_execution_evidence"))
        }) {
            failures.push(format!(
                "{} launch smoke evidence must inspect manifest assertions.real_alice_execution_evidence",
                target.id
            ));
        }

        let Some(personas) = scenario.personas.as_ref() else {
            failures.push(format!(
                "{} must declare instructor/student personas",
                target.id
            ));
            continue;
        };
        for instructor in target.instructors {
            if !personas
                .instructors
                .iter()
                .any(|actual| actual == instructor)
            {
                failures.push(format!(
                    "{} must include instructor persona {}",
                    target.id, instructor
                ));
            }
        }
        for student in target.students {
            if !personas.students.iter().any(|actual| actual == student) {
                failures.push(format!(
                    "{} must include student persona {}",
                    target.id, student
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "outside-in Alice expansion scenario contracts failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn starter_project_preflight_contract_names_real_action_evidence_without_overclaiming() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "starter-project-open-save-export-preflight",
    ))
    .unwrap();

    assert_contains_all(
        "starter-project-open-save-export-preflight contract",
        &contract,
        &[
            "real Alice action evidence",
            "opened starter project",
            "manifest/log/window/screenshot evidence",
            "inspectable action evidence",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
    assert_not_contains_any(
        "starter-project-open-save-export-preflight contract",
        &contract,
        &forbidden_internal_shorthand(),
    );
}

#[test]
fn teacher_community_sharing_loop_contract_names_handoff_and_honest_boundaries() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "teacher-community-sharing-loop",
    ))
    .unwrap();

    assert_contains_all(
        "teacher-community-sharing-loop contract",
        &contract,
        &[
            "teacher-community share card",
            "classroom handoff note",
            "editable scenario and persona links",
            "attribution",
            "classroom constraints",
            "student evidence",
            "remix feedback prompts",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
            "not a deployed community platform",
        ],
    );
}

#[test]
fn student_artifact_package_share_evidence_contract_names_review_handoff_boundary() {
    let root = repository_root();
    let contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "student-artifact-package-share-evidence",
    ))
    .unwrap();

    assert_contains_all(
        "student-artifact-package-share-evidence contract",
        &contract,
        &[
            "artifact review packet checklist",
            "student evidence handoff prompt",
            "instructor review boundary note",
            "artifact or screenshot reference",
            "visible run result",
            "attribution or classroom context",
            "student-owned next revision",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn student_sharing_readiness_docs_define_instructor_and_student_boundaries() {
    let root = repository_root();
    let contract = fs::read_to_string(root.join("docs/sharing-readiness-boundary.md")).unwrap();

    assert_contains_all(
        "sharing readiness boundary docs",
        &contract,
        &[
            "Student and teacher sharing scenarios define a review handoff, not a deployed sharing feature",
            "Student | The student can hand off a packet",
            "Instructor | The instructor can review the packet",
            "Review boundary | A plain statement that the packet is for instructor or peer review, not proof of deployed sharing",
            "No deployment or platform configuration is required for sharing readiness",
            "These commands validate asset shape and adapter freshness. They do not upload, host, publish, moderate, or prove any deployed sharing service.",
        ],
    );
}

#[test]
fn student_sharing_gadugi_adapter_preserves_review_handoff_boundary() {
    let root = repository_root();
    let adapter = fs::read_to_string(scenario_path(
        &root,
        "gadugi",
        "student-artifact-package-share-evidence",
    ))
    .unwrap();

    assert_contains_all(
        "student-artifact-package-share-evidence Gadugi adapter",
        &adapter,
        &[
            "Artifact review packet checklist includes artifact or screenshot reference",
            "Student evidence handoff prompt asks the student to explain one Alice change",
            "Instructor review boundary note separates environment evidence from student learning evidence",
            "classroom review handoff",
            "not hosted sharing",
            "not deployed sharing",
            "not platform success",
        ],
    );
}

#[test]
fn media_audio_cue_storyboard_covers_media_audio_student_persona() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "media-audio-cue-storyboard");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario
        .personas
        .as_ref()
        .expect("media-audio scenario must define personas");

    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert!(
        personas
            .students
            .iter()
            .any(|persona| persona == "media-audio-creator"),
        "media-audio-cue-storyboard must cover media-audio-creator"
    );
    assert_contains_all(
        "media-audio-cue-storyboard contract",
        &contract,
        &[
            "media cue storyboard",
            "student prediction prompt",
            "accessibility fallback note",
            "visible or audible result",
            "student-owned revision",
            "media-audio-creator",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

#[test]
fn lost_robot_debug_museum_covers_reflective_debugger_and_debug_coach() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "lost-robot-debug-museum");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario.personas.as_ref().expect("must define personas");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert!(personas.instructors.iter().any(|p| p == "debug-coach"));
    assert!(personas.students.iter().any(|p| p == "reflective-debugger"));
    assert!(
        personas
            .students
            .iter()
            .any(|p| p == "collaborative-peer-mentor")
    );
    assert_contains_all(
        "lost-robot-debug-museum contract",
        &contract,
        &[
            "debug mystery brief",
            "student debug journal",
            "peer question checkpoint",
            "hypothesis",
            "minimal change",
            "reflective-debugger",
            "collaborative-peer-mentor",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}
