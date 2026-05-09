use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "starter-project-open-save-export-preflight";
const CONTRACT_DOC_PATH: &str = "docs/default-workflow-pr-readiness.md";
const EVIDENCE_DOC_PATH: &str = "docs/starter-project-preflight-evidence.md";

const REQUIRED_SOURCE_BOUNDARIES: &[&str] = &[
    "plain automation scenario for instructors and students",
    "opened starter project",
    "small editable starter-world change",
    "attempt to run or observe",
    "save/reopen/export/readiness gaps",
    "not proof of visible rendering correctness",
    "without claiming full Save completion or full UI automation",
    "without claiming first-lesson completion",
    "not grading",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
];

const REQUIRED_ADAPTER_BOUNDARIES: &[&str] = &[
    "opened starter project",
    "manifest/log/window/screenshot evidence",
    "bounded starter-world and readiness-gap artifacts",
    "without claiming save/reopen/export coverage",
    "not full UI automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
    "not visible rendering correctness proof",
    "not first-lesson completion",
    "not full Save completion",
];

const INTERNAL_OR_OVERBROAD_LANGUAGE: &[&str] = &[
    "action evidence",
    "source boundary",
    "manifest-level evidence only",
    "proves visible rendering correctness",
    "proves save/reopen/export",
    "first lesson is complete",
    "grades learner work",
    "assesses creativity",
];

const REQUIRED_DOCUMENTED_CONTRACT_BOUNDARIES: &[&str] = &[
    "Starter-project evidence boundary",
    "Executable starter-project boundary check",
    "docs/default-workflow-pr-readiness.md",
    "docs/starter-project-preflight-evidence.md",
    "Prohibited phrase",
    "Bounded replacement",
];

const PLANNED_EXTENSION_WORDING: &[&str] = &[
    "planned documentation-overclaim extension",
    "planned extension should",
    "planned documentation-overclaim extension will",
];

const PROHIBITED_READINESS_OVERCLAIMS: &[(&str, &str)] = &[
    ("PR ready", "starter-project preflight evidence recorded"),
    ("merge ready", "starter-project evidence boundary satisfied"),
    (
        "production ready",
        "bounded preflight evidence available for review",
    ),
    (
        "ready for merge",
        "readiness gaps are documented for later gates",
    ),
    (
        "readiness guaranteed",
        "readiness depends on the separate readiness gates",
    ),
    (
        "complete PR readiness",
        "starter-project preflight evidence only",
    ),
];

#[test]
fn source_starter_project_preflight_uses_plain_bounded_user_facing_language() {
    let root = repository_root();
    let path = scenario_path(&root, "eatme");
    let text = fs::read_to_string(&path).unwrap();
    let scenario: EatmeScenarioAsset = serde_yaml::from_str(&text).unwrap();

    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "alice_lesson_smoke");
    assert_contains_all(
        "starter-project preflight source",
        &text,
        REQUIRED_SOURCE_BOUNDARIES,
    );
    assert_contains_none(
        "starter-project preflight source",
        &text,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
}

#[test]
fn generated_starter_project_preflight_adapter_uses_same_plain_boundaries() {
    let root = repository_root();
    let source_path = scenario_path(&root, "eatme");
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generate_gadugi_adapter_yaml(&root, &source_path).unwrap();
    let committed = fs::read_to_string(&committed_path).unwrap();

    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the canonical starter-project scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated starter-project preflight adapter",
        &generated,
        REQUIRED_ADAPTER_BOUNDARIES,
    );
    assert_contains_none(
        "generated starter-project preflight adapter",
        &generated,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
}

#[test]
fn documented_contract_defines_current_executable_doc_overclaim_check() {
    let root = repository_root();
    let text = read_repo_text(&root, CONTRACT_DOC_PATH);

    assert_contains_all(
        "starter-project/preflight readiness source contract",
        &text,
        REQUIRED_DOCUMENTED_CONTRACT_BOUNDARIES,
    );
    assert_contains_none_with_message(
        "starter-project/preflight readiness source contract",
        &text,
        PLANNED_EXTENSION_WORDING,
        &format!(
            "{CONTRACT_DOC_PATH} must describe the scoped documentation overclaim check as current executable behavior, not planned future work"
        ),
    );
}

#[test]
fn scoped_starter_project_preflight_docs_do_not_overclaim_readiness_or_evidence() {
    let root = repository_root();
    let text = read_repo_text(&root, EVIDENCE_DOC_PATH);

    assert_no_readiness_overclaims(EVIDENCE_DOC_PATH, &text);
}

#[test]
fn readiness_overclaim_detector_allows_negative_boundary_statements() {
    let text = "\
starter-project preflight evidence is not pull request readiness, mergeability, \
production suitability, complete lesson execution, user-like Alice UI coverage, \
save/reopen/export completion, grading, creative assessment, visible rendering \
correctness, or complete Alice coverage.";

    assert_no_readiness_overclaims("docs/example.md", text);
}

#[test]
fn readiness_overclaim_detector_reports_actionable_failure_details() {
    let violations = readiness_overclaims_in(
        "docs/example.md",
        "This starter-project preflight evidence is PR ready.",
    );

    assert_eq!(violations.len(), 1);
    let details = format_overclaim_failures(&violations);
    assert!(details.contains("docs/example.md"));
    assert!(details.contains("PR ready"));
    assert!(details.contains(CONTRACT_DOC_PATH));
    assert!(details.contains("starter-project preflight evidence recorded"));
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(collection)
        .join(format!("{SCENARIO_ID}.yaml"))
}

fn read_repo_text(root: &Path, repo_relative_path: &str) -> String {
    fs::read_to_string(root.join(repo_relative_path))
        .unwrap_or_else(|error| panic!("failed to read {repo_relative_path}: {error}"))
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text);
    let missing = needles
        .iter()
        .filter(|needle| !normalized_text.contains(&normalize(needle)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} is missing required bounded language: {missing:?}"
    );
}

fn assert_contains_none(label: &str, text: &str, needles: &[&str]) {
    assert_contains_none_with_message(
        label,
        text,
        needles,
        &format!("{label} contains internal or overbroad language"),
    );
}

fn assert_contains_none_with_message(_label: &str, text: &str, needles: &[&str], message: &str) {
    let normalized_text = normalize(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(present.is_empty(), "{message}: {present:?}");
}

#[derive(Debug, PartialEq, Eq)]
struct ReadinessOverclaim {
    file: &'static str,
    line_number: usize,
    phrase: &'static str,
    bounded_replacement: &'static str,
}

fn assert_no_readiness_overclaims(file: &'static str, text: &str) {
    let violations = readiness_overclaims_in(file, text);
    assert!(
        violations.is_empty(),
        "{}",
        format_overclaim_failures(&violations)
    );
}

fn readiness_overclaims_in(file: &'static str, text: &str) -> Vec<ReadinessOverclaim> {
    text.lines()
        .enumerate()
        .flat_map(|(line_index, line)| {
            let normalized_line = normalize(line).to_lowercase();
            PROHIBITED_READINESS_OVERCLAIMS
                .iter()
                .filter(move |(phrase, _)| normalized_line.contains(&phrase.to_lowercase()))
                .map(move |(phrase, bounded_replacement)| ReadinessOverclaim {
                    file,
                    line_number: line_index + 1,
                    phrase,
                    bounded_replacement,
                })
        })
        .collect()
}

fn format_overclaim_failures(violations: &[ReadinessOverclaim]) -> String {
    violations
        .iter()
        .map(|violation| {
            format!(
                "{} overclaims starter-project/preflight readiness with prohibited phrase `{}` on line {}; source contract: {}; use bounded wording such as `{}`",
                violation.file,
                violation.phrase,
                violation.line_number,
                CONTRACT_DOC_PATH,
                violation.bounded_replacement
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
