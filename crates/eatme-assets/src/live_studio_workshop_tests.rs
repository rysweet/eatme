use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const LIVE_STUDIO_ID: &str = "workshop-facilitator-live-studio";
const REQUIRED_CONTRACT_TEXT: &[&str] = &[
    "setup checklist",
    "timing plan",
    "observation points",
    "intervention cues",
    "checkpoint artifacts",
    "share-out support",
    "instructor-facing acceptance probes",
    "student prompt cards",
    "student-owned Alice action evidence",
    "add or adjust one visible behavior",
    "run it",
    "record the observed result",
    "revise one small choice",
    "help signals",
    "peer feedback",
    "revision behavior",
    "reflection",
    "share-out artifacts",
    "not full Alice user interface automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
];
const REQUIRED_OUTPUTS: &[&str] = &[
    "facilitation_plan",
    "timing_plan",
    "observation_intervention_guide",
    "participant_checkpoint_board",
    "student_prompt_cards",
    "help_signal_board",
    "peer_feedback_notes",
    "revision_reflection_log",
    "share_out_artifacts",
    "real_alice_action_evidence_notes",
    "instructor_acceptance_probe_notes",
];

#[test]
fn canonical_live_studio_scenario_names_instructor_and_student_evidence_contract() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(&text).unwrap();
    let expected_outputs = scenario
        .agentic_flow
        .as_ref()
        .expect("live-studio scenario must define agentic_flow")
        .expected_outputs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    assert_eq!(scenario.id, LIVE_STUDIO_ID);
    assert_eq!(scenario.kind, "instructor_agentic_flow");
    assert_contains_all(
        "live-studio scenario contract",
        &text,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "live-studio expected outputs",
        &expected_outputs.join("\n"),
        REQUIRED_OUTPUTS,
    );
    for artifact in REQUIRED_OUTPUTS {
        assert!(
            scenario.artifacts.contains_key(*artifact),
            "live-studio scenario must define artifacts.{artifact}"
        );
    }
}

#[test]
fn live_studio_generated_adapter_preserves_the_expanded_evidence_contract() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the editable live-studio scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated live-studio adapter contract",
        &generated,
        REQUIRED_CONTRACT_TEXT,
    );
    assert_contains_all(
        "generated live-studio adapter expected outputs",
        &generated,
        REQUIRED_OUTPUTS,
    );
}

#[test]
fn live_studio_touched_assets_and_generator_avoid_rejected_internal_shorthand() {
    let root = repository_root();
    let paths = [
        scenario_path(&root, "eatme"),
        scenario_path(&root, "gadugi"),
        root.join("docs/live-studio-workshop-evidence.md"),
        root.join("crates/eatme-assets/src/gadugi_instructor.rs"),
    ];
    let mut violations = Vec::new();

    for path in paths {
        let text = fs::read_to_string(&path).unwrap();
        violations.extend(rejected_shorthand_violations(&path, &text));
    }

    assert!(
        violations.is_empty(),
        "live-studio durable assets and generator wording must avoid rejected shorthand:\n{}",
        violations.join("\n")
    );
}

#[test]
fn instructor_generator_uses_acceptance_review_wording_for_agentic_flow_adapters() {
    let root = repository_root();
    let generated = generate_gadugi_adapter_yaml(&root, &scenario_path(&root, "eatme")).unwrap();
    let normalized = normalize_whitespace(&generated);

    assert_contains_all(
        "live-studio generated acceptance-review wording",
        &normalized,
        &[
            "instructor acceptance adapter",
            "Run instructor agentic acceptance review",
            "instructor-acceptance-agent",
            "Instructor Agentic Acceptance Review Covers Probes",
        ],
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, scenario_collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(scenario_collection)
        .join(format!("{LIVE_STUDIO_ID}.yaml"))
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
        "{label} is missing required evidence language: {missing:?}"
    );
}

fn rejected_shorthand_violations(path: &Path, text: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        for token in ["QA", "UI"] {
            if contains_word_token(line, token) {
                violations.push(format!(
                    "{}:{} contains rejected shorthand {token:?}: {line}",
                    path.display(),
                    line_index + 1
                ));
            }
        }
    }
    violations
}

fn contains_word_token(line: &str, token: &str) -> bool {
    line.split(|ch| !is_word_char(ch)).any(|word| word == token)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::new();
    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    normalized
}
