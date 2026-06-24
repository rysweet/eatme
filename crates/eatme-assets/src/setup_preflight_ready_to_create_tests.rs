use crate::generate_instructor_agentic_acceptance_output;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn setup_preflight_ready_to_create_covers_curious_novice_and_collaborative_peer_mentor() {
    let root = repository_root();
    let path = scenario_path(&root, "setup-preflight-ready-to-create");
    let contract = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);
    let personas = scenario.personas.as_ref().expect("must define personas");
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    for instructor in &["classroom-orchestrator", "studio-facilitator"] {
        assert!(
            personas.instructors.iter().any(|p| p == instructor),
            "must include instructor persona {instructor}"
        );
    }
    for student in &["curious-novice", "collaborative-peer-mentor"] {
        assert!(
            personas.students.iter().any(|p| p == student),
            "must include student persona {student}"
        );
    }
    assert_contains_all(
        "setup-preflight-ready-to-create contract",
        &contract,
        &[
            "setup readiness checklist",
            "student self-check card",
            "fallback path guide",
            "Chromebook",
            "Java",
            "OpenGL",
            "install permission",
            "no-install",
            "pairing",
            "environment problem",
            "handoff note",
            "not full user interface automation",
            "not automated creative assessment",
            "not learner-world grading",
            "not complete Alice coverage",
        ],
    );
}

#[test]
fn setup_preflight_ready_to_create_generates_runnable_instructor_acceptance_output() {
    let root = repository_root();
    let path = scenario_path(&root, "setup-preflight-ready-to-create");

    let report = generate_instructor_agentic_acceptance_output(&path).unwrap();

    assert!(report.passed, "{:?}", report.errors);
    for output in [
        "setup_readiness_checklist",
        "student_self_check_card",
        "fallback_path_guide",
    ] {
        assert!(
            report.output_evidence.contains_key(output),
            "missing output evidence {output}"
        );
    }
    let evidence = report
        .output_evidence
        .values()
        .map(|output| output.evidence.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_contains_all(
        "setup-preflight-ready-to-create generated acceptance output",
        &evidence,
        &[
            "Windows 64-bit",
            "macOS",
            "Linux .deb",
            "Java 21",
            "OpenGL-capable graphics drivers",
            "install permission",
            "Chromebook",
            "Right now in Alice I can",
            "My device blocker is",
            "environment problem, not a student mistake",
            "Pairing path",
            "Instructor-led demo path",
            "Printed design-planning path",
            "collect every reported blocker",
        ],
    );
    assert!(
        report.probe_results.iter().all(|probe| probe.passed),
        "{:?}",
        report.probe_results
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, id: &str) -> PathBuf {
    root.join("assets/scenarios/eatme")
        .join(format!("{id}.yaml"))
}

fn read_eatme_scenario(path: &Path) -> EatmeScenarioAsset {
    let content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let missing: Vec<_> = needles
        .iter()
        .filter(|n| {
            let nw = n.split_whitespace().collect::<Vec<_>>().join(" ");
            !normalized.contains(&nw)
        })
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{label} is missing required evidence language: {missing:?}"
    );
}
