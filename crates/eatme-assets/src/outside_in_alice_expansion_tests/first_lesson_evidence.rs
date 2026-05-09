use std::fs;

use super::{
    FIRST_LESSON_REQUIRED_SMOKE_READY_EVIDENCE, FIRST_LESSON_SMOKE_READY_EVIDENCE_COUNT,
    assert_contains_all, assert_not_contains_any, read_eatme_scenario, repository_root,
    scenario_path,
};

#[test]
fn first_lesson_evidence_contracts_stay_explicit_and_honest() {
    let root = repository_root();
    let student_contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "first-lessons-real-ui-actions",
    ))
    .unwrap();
    let instructor_contract = fs::read_to_string(scenario_path(
        &root,
        "eatme",
        "instructor-lesson-materials-remix",
    ))
    .unwrap();
    let launch_contract =
        fs::read_to_string(scenario_path(&root, "eatme", "real-alice-launch-smoke")).unwrap();
    let mut docs = String::new();
    for path in [
        root.join("docs/alice-lesson-smoke.md"),
        root.join("docs/student-missions.md"),
        root.join("docs/instructor-missions.md"),
        root.join("docs/persona-assets.md"),
        root.join("docs/index.md"),
    ] {
        docs.push_str(&fs::read_to_string(path).unwrap());
        docs.push('\n');
    }

    assert_contains_all(
        "first-lessons-real-ui-actions contract",
        &student_contract,
        &[
            "scenario-labeled real Alice launch path",
            "manifest, Alice log, window list, and startup screenshot evidence",
            "Alice window detection",
            "ui-action-contract.json",
            "preflight launch/action readiness evidence only",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not production readiness",
            "not lesson completion",
        ],
    );
    assert_contains_all(
        "instructor-lesson-materials-remix contract",
        &instructor_contract,
        &[
            "lesson-material remix path",
            "scenario-labeled assets",
            "agentic probes",
            "does not grade learner worlds",
            "assess creativity automatically",
            "automated creative grading",
            "learner-world assessment",
        ],
    );
    assert_contains_all(
        "real-alice-launch-smoke contract",
        &launch_contract,
        &[
            "scenario-labeled launch path",
            "manifest/log/window/screenshot evidence",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
        ],
    );
    assert_contains_all(
        "lesson evidence docs",
        &docs,
        &[
            "first-lessons-real-ui-actions",
            "instructor-lesson-materials-remix",
            "real-alice-launch-smoke",
            "preflight launch/action readiness evidence only",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not production readiness",
            "not lesson completion",
            "does not grade learner worlds or assess creativity automatically",
        ],
    );
}

#[test]
fn first_lesson_readiness_purpose_names_preflight_evidence_without_completion_claims() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme", "first-lessons-real-ui-actions");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let smoke_ready = scenario
        .smoke_ready
        .as_ref()
        .expect("first-lessons-real-ui-actions must define smoke_ready evidence");

    assert_eq!(scenario.id, "first-lessons-real-ui-actions");
    assert_eq!(scenario.kind, "alice_real_ui_action_contract");
    assert_eq!(
        smoke_ready.evidence.len(),
        FIRST_LESSON_SMOKE_READY_EVIDENCE_COUNT,
        "first-lessons-real-ui-actions smoke_ready evidence inventory count must stay stable"
    );
    assert_contains_all(
        "first-lessons-real-ui-actions smoke_ready evidence",
        &smoke_ready.evidence.join("\n"),
        FIRST_LESSON_REQUIRED_SMOKE_READY_EVIDENCE,
    );
    assert_contains_all(
        "first-lessons-real-ui-actions purpose",
        &scenario.purpose,
        &[
            "preflight launch/action readiness evidence only",
            "setup, launch support, handoff artifacts, and classroom support preparation",
            "not full UI automation",
            "not creative assessment",
            "not learner-world grading",
            "not production readiness",
            "not lesson completion",
            "not complete end-to-end lesson execution",
            "not broad Alice compatibility",
        ],
    );
    assert_not_contains_any(
        "first-lessons-real-ui-actions purpose",
        &scenario.purpose,
        &[
            "This is launch/action-contract evidence only. It is bounded readiness evidence"
                .to_string(),
        ],
    );
    assert_contains_all(
        "first-lessons-real-ui-actions canonical asset",
        &contract,
        &["purpose: >-", "smoke_ready:", "acceptance_criteria:"],
    );
}
