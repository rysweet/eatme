use crate::generate_gadugi_adapter_yaml;
use crate::schema::{CrewAsset, EatmeScenarioAsset};
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "instructor-student-save-reopen-export-evidence-handoff";
const REQUIRED_PERSONAS: &[&str] = &[
    "classroom-orchestrator",
    "assessment-curator",
    "reflective-debugger",
    "collaborative-peer-mentor",
];
const REQUIRED_OUTPUTS: &[&str] = &[
    "save_reopen_handoff_card",
    "export_evidence_package_checklist",
    "instructor_review_boundary_note",
];
const REQUIRED_HANDOFF_TEXT: &[&str] = &[
    "starter-project preflight evidence",
    "starter project opens",
    "inspectable setup evidence",
    "save the work with a clear name and location",
    "reopen the saved work",
    "export/share evidence package",
    "hand off the evidence",
    "setup evidence, not as save, reopen, export, or sharing proof",
    "operational evidence quality",
    "human review",
    "not full user interface automation",
    "not automated creative assessment",
    "not learner-world grading",
    "not full Alice coverage",
    "not proof of student learning",
];
const REQUIRED_ADAPTER_TEXT: &[&str] = &[
    "practical Alice starter-project handoff after preflight setup evidence exists",
    "save the work with a clear name and location",
    "reopen the saved work",
    "export/share evidence package",
    "hand off the evidence",
    "setup evidence, not as save, reopen, export, or sharing proof",
    "observable checks",
    "human review",
    "not full user interface automation",
    "not automated creative assessment",
    "not learner-world grading",
    "not full Alice coverage",
    "not proof of student learning",
];

#[test]
fn save_reopen_export_handoff_scenario_fills_the_preflight_to_evidence_gap() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);

    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_required_personas(&scenario);
    assert_contains_all(
        "save/reopen/export handoff resource bridge",
        &resource_text(&scenario),
        &[
            "Starter project open/save/export preflight",
            "Student artifact package share evidence",
            "Use opened starter-project evidence as the setup input for the handoff",
            "share-packet boundaries",
        ],
    );
    assert_contains_all(
        "save/reopen/export handoff scenario",
        &text,
        REQUIRED_HANDOFF_TEXT,
    );
    assert!(
        !normalize_whitespace(&text.to_lowercase()).contains("inspectable action evidence"),
        "scenario must describe preflight as setup evidence, not action evidence"
    );
    assert_contains_all(
        "save/reopen/export handoff outputs",
        &expected_output_text(&scenario),
        REQUIRED_OUTPUTS,
    );
    assert_required_artifacts(&scenario);
}

#[test]
fn generated_adapter_preserves_save_reopen_export_handoff_contract() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the editable save/reopen/export handoff scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated save/reopen/export handoff adapter",
        &generated,
        REQUIRED_ADAPTER_TEXT,
    );
    assert_contains_all(
        "generated save/reopen/export handoff adapter outputs",
        &generated,
        REQUIRED_OUTPUTS,
    );
    assert!(
        generated.contains("persona_asset: assets/personas/alice-user-crew.yaml"),
        "generated adapter must point at the canonical persona asset"
    );
    assert!(
        generated.contains(
            "scenario_asset: assets/scenarios/eatme/instructor-student-save-reopen-export-evidence-handoff.yaml"
        ),
        "generated adapter must point at the canonical eatme scenario asset"
    );
}

#[test]
fn persona_coverage_discovers_handoff_without_broadening_the_persona_scope() {
    let root = repository_root();
    let text = fs::read_to_string(root.join("assets/personas/alice-user-crew.yaml")).unwrap();
    let crew: CrewAsset = serde_yaml::from_str(&text).unwrap();
    let scenario = crew
        .core_scenarios_from_existing_alice_resources
        .iter()
        .find(|scenario| scenario.id == SCENARIO_ID)
        .expect("persona crew must list the save/reopen/export handoff scenario");
    let coverage = crew
        .constituency_coverage
        .iter()
        .find(|coverage| {
            coverage
                .scenario_ids
                .iter()
                .any(|scenario_id| scenario_id == SCENARIO_ID)
        })
        .expect("constituency coverage must make the handoff scenario discoverable");

    assert_eq!(scenario.origin, "existing-alice-resource");
    assert_eq!(scenario.coverage, ["classroom-use", "export-share"]);
    assert_eq!(
        scenario.personas.instructors,
        ["classroom-orchestrator", "assessment-curator"]
    );
    assert_eq!(
        scenario.personas.students,
        ["reflective-debugger", "collaborative-peer-mentor"]
    );
    assert_eq!(coverage.id, "teacher-community-sharing");
    assert_contains_all(
        "persona discoverability for save/reopen/export handoff",
        &coverage.evidence.join("\n"),
        &[
            "artifact references",
            "student explanation",
            "human review boundaries",
        ],
    );
}

#[test]
fn unsupported_policy_blocks_grading_and_completion_overclaims() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario = read_eatme_scenario(&path);

    assert_contains_all(
        "save/reopen/export handoff unsupported policy",
        &unsupported_boundary_text(&scenario),
        &[
            "fail visibly and report the missing evidence",
            "Do not replace instructor judgment",
            "Do not claim full user interface automation",
            "automated creative assessment",
            "learner-world grading",
            "whether the evidence proves student achievement",
            "redirects to operational evidence quality",
            "certified work",
            "mastery",
            "proof of student learning",
        ],
    );
    assert_not_contains_lowercase(
        "scenario",
        &text,
        &[
            "student completed",
            "students completed",
            "student mastered",
            "students mastered",
            "certified student",
            "certified learner",
            "fully finished",
            "proves learning outcomes",
        ],
    );
}

#[test]
fn scenario_wording_requests_handoff_evidence_without_claiming_actions_were_completed() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let scenario = read_eatme_scenario(&path);
    let agentic_flow = scenario
        .agentic_flow
        .as_ref()
        .expect("save/reopen/export handoff scenario must define agentic_flow");
    let bounded_action_text = normalize_whitespace(
        [
            scenario.purpose.as_str(),
            agentic_flow.instructor_goal.as_str(),
        ]
        .join("\n")
        .to_lowercase()
        .as_str(),
    );

    assert_contains_all(
        "save/reopen/export bounded action language",
        &bounded_action_text,
        &[
            "asks the student",
            "observable",
            "evidence collection and human review",
            "evidence references",
        ],
    );
    assert_not_contains_lowercase(
        "scenario bounded action wording",
        &bounded_action_text,
        &[
            "the flow confirms",
            "checks that the saved project reopens",
            "checks that the saved work reopens",
            "prepares an exported evidence package",
            "prepares export/share evidence",
            "records how the evidence is handed",
        ],
    );
}

fn assert_required_personas(scenario: &EatmeScenarioAsset) {
    let personas = scenario
        .personas
        .as_ref()
        .expect("save/reopen/export handoff scenario must define personas");

    for persona in REQUIRED_PERSONAS {
        assert!(
            personas.instructors.iter().any(|p| p == persona)
                || personas.students.iter().any(|p| p == persona),
            "scenario must include persona {persona}"
        );
    }
}

fn resource_text(scenario: &EatmeScenarioAsset) -> String {
    scenario
        .resource_basis
        .iter()
        .map(|resource| format!("{}\n{}\n{}", resource.name, resource.url, resource.use_note))
        .collect::<Vec<_>>()
        .join("\n")
}

fn expected_output_text(scenario: &EatmeScenarioAsset) -> String {
    scenario
        .agentic_flow
        .as_ref()
        .expect("save/reopen/export handoff scenario must define agentic_flow")
        .expected_outputs
        .join("\n")
}

fn assert_required_artifacts(scenario: &EatmeScenarioAsset) {
    for artifact in REQUIRED_OUTPUTS {
        assert!(
            scenario.artifacts.contains_key(*artifact),
            "scenario must define artifacts.{artifact}"
        );
    }
}

fn unsupported_boundary_text(scenario: &EatmeScenarioAsset) -> String {
    let acceptance_boundary = scenario
        .acceptance_criteria
        .iter()
        .map(|criterion| {
            format!(
                "{}\n{}\n{}",
                criterion.given, criterion.when, criterion.then
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        scenario.purpose.as_str(),
        scenario.agentic_test_prompt.as_str(),
        acceptance_boundary.as_str(),
        scenario.unsupported_policy.as_str(),
        &scenario.avoid.join("\n"),
    ]
    .join("\n")
}

fn assert_not_contains_lowercase(label: &str, text: &str, blocked: &[&str]) {
    let normalized_text = normalize_whitespace(&text.to_lowercase());

    for blocked in blocked {
        assert!(
            !normalized_text.contains(blocked),
            "{label} must not include unsupported claim {blocked:?}"
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, scenario_collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(scenario_collection)
        .join(format!("{SCENARIO_ID}.yaml"))
}

fn read_eatme_scenario(path: &Path) -> EatmeScenarioAsset {
    let content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize_whitespace(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize_whitespace(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required contract language: {missing:?}"
    );
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    normalized
}
