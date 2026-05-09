use crate::generate_gadugi_adapter_yaml;
use crate::overclaim_test_helpers::{
    CONTRACT_DOC_PATH, EVIDENCE_DOC_PATH, OverclaimRule, assert_contains_none_with_message,
    assert_no_doc_overclaims, assert_rules_match_contract, doc_overclaims_in,
    format_overclaim_failures, overclaim_rules_from_contract, read_contract_overclaim_rules,
    read_repo_text,
};
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const SCENARIO_ID: &str = "starter-project-open-save-export-preflight";
const SOURCE_SCENARIO_PATH: &str =
    "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml";
const GENERATED_ADAPTER_PATH: &str =
    "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml";

static SOURCE_TEXT: OnceLock<String> = OnceLock::new();
static SOURCE_SCENARIO: OnceLock<EatmeScenarioAsset> = OnceLock::new();
static GENERATED_ADAPTER: OnceLock<String> = OnceLock::new();

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

const REQUIRED_DOCUMENTED_OVERCLAIM_RULES: &[(&str, &str)] = &[
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
    (
        "proves visible rendering correctness",
        "screenshot or window evidence is observation evidence only",
    ),
    (
        "proves save/reopen/export",
        "save, reopen, and export remain readiness gaps",
    ),
    (
        "first lesson is complete",
        "starter-project preflight evidence only",
    ),
    (
        "grades learner work",
        "records evidence for review; it does not grade",
    ),
    (
        "assesses creativity",
        "names an editable change without assessing creativity",
    ),
];

const REQUIRED_READINESS_REPORT_FIELDS: &[&str] = &[
    "silver_thread=minimal open/save/export path",
    "open_evidence=",
    "starter_world_change_evidence=",
    "starter_program_change_evidence=",
    "save_evidence=",
    "reopen_evidence=",
    "export_evidence=",
    "configuration_state=",
    "claim_boundary=",
];

const REQUIRED_ABSENT_EVIDENCE_STATES: &[&str] = &[
    "starter_world_change_evidence=not observed",
    "starter_program_change_evidence=not observed",
    "save_evidence=missing",
    "reopen_evidence=not observed",
    "export_evidence=unavailable",
    "configuration_state=not configured",
];

const PROHIBITED_READINESS_REPORT_CLAIMS: &[&str] = &[
    "lesson",
    "lesson completion",
    "grading",
    "scoring",
    "rubric",
    "curriculum",
    "curriculum validation",
];

#[test]
fn source_starter_project_preflight_uses_plain_bounded_user_facing_language() {
    let text = source_text();
    let scenario = source_scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.kind, "alice_lesson_smoke");
    assert_contains_all(
        "starter-project preflight source",
        text,
        REQUIRED_SOURCE_BOUNDARIES,
    );
    assert_contains_none(
        "starter-project preflight source",
        text,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
    let root = repository_root();
    assert_no_doc_overclaims(
        SOURCE_SCENARIO_PATH,
        text,
        &read_contract_overclaim_rules(&root),
    );
}

#[test]
fn source_readiness_report_keeps_silver_thread_contract_on_existing_artifact() {
    let command = source_readiness_report_command();
    let report = starter_project_readiness_report_segment(command);
    assert_readiness_report_contract("starter-project readiness report", report);
}

#[test]
fn source_silver_thread_contract_stays_on_existing_readiness_report_artifact() {
    let scenario = source_scenario();
    let report_surface_count = scenario
        .artifacts
        .iter()
        .filter(|(key, value)| {
            key.contains("readiness_report")
                || value.ends_with("starter-project-readiness-report.txt")
        })
        .count();
    assert_eq!(
        report_surface_count, 1,
        "silver-thread evidence must stay on the existing readiness report artifact"
    );
    assert_eq!(
        scenario
            .artifacts
            .get("starter_project_readiness_report")
            .map(String::as_str),
        Some(
            "runs/starter-project-open-save-export-preflight/\
             ${RUN_ID}/starter-project-readiness-report.txt"
        )
    );
}

#[test]
fn generated_starter_project_preflight_adapter_uses_same_plain_boundaries() {
    let root = repository_root();
    let committed_path = scenario_path(&root, "gadugi");
    let generated = generated_adapter_yaml();
    let committed = fs::read_to_string(&committed_path).unwrap();
    assert_eq!(
        committed,
        generated,
        "{} must be regenerated from the canonical starter-project scenario",
        committed_path.display()
    );
    assert_contains_all(
        "generated starter-project preflight adapter",
        generated,
        REQUIRED_ADAPTER_BOUNDARIES,
    );
    assert_contains_none(
        "generated starter-project preflight adapter",
        generated,
        INTERNAL_OR_OVERBROAD_LANGUAGE,
    );
    assert_no_doc_overclaims(
        GENERATED_ADAPTER_PATH,
        generated,
        &read_contract_overclaim_rules(&root),
    );
}

#[test]
fn documented_contract_defines_current_executable_doc_overclaim_check() {
    let root = repository_root();
    let text = read_repo_text(&root, CONTRACT_DOC_PATH);
    let rules = overclaim_rules_from_contract(&text);
    assert_contains_all(
        "starter-project/preflight readiness source contract",
        &text,
        REQUIRED_DOCUMENTED_CONTRACT_BOUNDARIES,
    );
    assert_contains_none_with_message(
        &text,
        PLANNED_EXTENSION_WORDING,
        &format!(
            "{CONTRACT_DOC_PATH} must describe the scoped documentation \
             overclaim check as current executable behavior, not planned future work"
        ),
    );
    assert_rules_match_contract(&rules, REQUIRED_DOCUMENTED_OVERCLAIM_RULES);
}

#[test]
fn scoped_starter_project_preflight_docs_do_not_overclaim_readiness_or_evidence() {
    let root = repository_root();
    let contract = read_repo_text(&root, CONTRACT_DOC_PATH);
    let text = read_repo_text(&root, EVIDENCE_DOC_PATH);
    assert_no_doc_overclaims(
        EVIDENCE_DOC_PATH,
        &text,
        &overclaim_rules_from_contract(&contract),
    );
}

#[test]
fn readiness_overclaim_detector_allows_negative_boundary_statements() {
    let rules = vec![
        OverclaimRule::new("PR ready", "starter-project preflight evidence recorded"),
        OverclaimRule::new("merge ready", "starter-project evidence boundary satisfied"),
        OverclaimRule::new(
            "production ready",
            "bounded preflight evidence available for review",
        ),
    ];
    let text = "\
starter-project preflight evidence is not PR ready. \
It is not merge ready. It is not production ready. \
It is not pull request readiness, mergeability, \
production suitability, complete lesson execution, user-like Alice UI coverage, \
save/reopen/export completion, grading, creative assessment, visible rendering \
correctness, or complete Alice coverage.";
    assert_no_doc_overclaims("docs/example.md", text, &rules);
}

#[test]
fn readiness_overclaim_detector_reports_actionable_failure_details() {
    let rules = vec![
        OverclaimRule::new("PR ready", "starter-project preflight evidence recorded"),
        OverclaimRule::new(
            "proves visible rendering correctness",
            "screenshot or window evidence is observation evidence only",
        ),
    ];
    let violations = doc_overclaims_in(
        "docs/example.md",
        "This starter-project preflight evidence is PR ready.\n\
         It proves visible rendering correctness.",
        &rules,
    );
    assert_eq!(violations.len(), 2);
    let details = format_overclaim_failures(&violations);
    assert!(details.contains("docs/example.md"));
    assert!(details.contains("PR ready"));
    assert!(details.contains("proves visible rendering correctness"));
    assert!(details.contains(CONTRACT_DOC_PATH));
    assert!(details.contains("starter-project preflight evidence recorded"));
    assert!(details.contains("screenshot or window evidence is observation evidence only"));
}

#[test]
fn overclaim_rules_from_contract_ignores_unrelated_markdown_tables() {
    let contract = "\
## GitHub metadata fields

| Field | Required value |
| --- | --- |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |

## Executable starter-project boundary check

| Prohibited phrase | Bounded replacement |
| --- | --- |
| `PR ready` | `starter-project preflight evidence recorded` |
| `merge ready` | `starter-project evidence boundary satisfied` |

| `unrelated` | `ignored` |
";
    let rules = overclaim_rules_from_contract(contract);
    assert_rules_match_contract(
        &rules,
        &[
            ("PR ready", "starter-project preflight evidence recorded"),
            ("merge ready", "starter-project evidence boundary satisfied"),
        ],
    );
}

#[test]
fn generated_adapter_readiness_report_preserves_silver_thread_contract() {
    let report = starter_project_readiness_report_segment(generated_adapter_yaml());
    assert_readiness_report_contract("generated starter-project readiness report", report);
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path(root: &Path, collection: &str) -> PathBuf {
    root.join("assets/scenarios")
        .join(collection)
        .join(format!("{SCENARIO_ID}.yaml"))
}

fn source_readiness_report_command() -> &'static str {
    source_scenario()
        .steps
        .iter()
        .find(|step| step.id == "record-run-observe-readiness-gaps")
        .expect("starter project preflight must record readiness report")
        .command
        .as_str()
}

fn source_scenario() -> &'static EatmeScenarioAsset {
    SOURCE_SCENARIO.get_or_init(|| serde_yaml::from_str(source_text()).unwrap())
}

fn source_text() -> &'static str {
    SOURCE_TEXT
        .get_or_init(|| {
            let root = repository_root();
            fs::read_to_string(scenario_path(&root, "eatme")).unwrap()
        })
        .as_str()
}

fn generated_adapter_yaml() -> &'static str {
    GENERATED_ADAPTER
        .get_or_init(|| {
            let root = repository_root();
            let source = scenario_path(&root, "eatme");
            generate_gadugi_adapter_yaml(&root, &source).unwrap()
        })
        .as_str()
}

fn starter_project_readiness_report_segment(text: &str) -> &str {
    let path = "starter-project-readiness-report.txt";
    let end = text
        .find(path)
        .expect("starter-project readiness report must be written");
    let start = text[..end]
        .rfind("printf")
        .expect("starter-project readiness report must be written by printf");
    &text[start..end]
}

fn assert_readiness_report_contract(label: &str, report: &str) {
    assert_contains_all(label, report, REQUIRED_READINESS_REPORT_FIELDS);
    assert_contains_all(label, report, REQUIRED_ABSENT_EVIDENCE_STATES);
    assert_contains_all(
        label,
        report,
        &[
            "silver thread",
            "minimal open/save/export path",
            "bundled starter project",
            "starter world",
            "starter program",
            "observable change evidence",
        ],
    );
    assert_contains_none(label, report, PROHIBITED_READINESS_REPORT_CLAIMS);
}

#[test]
fn evidence_doc_lists_overclaim_helpers_in_contract_update_block() {
    let root = repository_root();
    let text = read_repo_text(&root, EVIDENCE_DOC_PATH);
    let update_block: String = text
        .lines()
        .skip_while(|line| !line.contains("update these files together"))
        .take_while(|line| !line.starts_with("The Gadugi"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        update_block.contains("overclaim_test_helpers.rs"),
        "{EVIDENCE_DOC_PATH} 'update these files together' block must include \
         overclaim_test_helpers.rs since it is part of the boundary contract system"
    );
}

#[test]
fn source_scenario_shell_commands_use_fail_fast_mode() {
    let scenario = source_scenario();
    for step in scenario
        .steps
        .iter()
        .filter(|s| s.command.contains("bash -lc"))
    {
        assert!(
            step.command.contains("set -e"),
            "shell step '{}' must use 'set -e' for fail-fast error handling",
            step.id
        );
    }
}

#[test]
fn overclaim_detection_is_case_and_whitespace_insensitive() {
    let rules = vec![OverclaimRule::new(
        "PR ready",
        "starter-project preflight evidence recorded",
    )];
    let cases = &[
        "This is PR READY now",
        "This is pr   ready now",
        "This is  PR\tready  now",
    ];
    for text in cases {
        let violations = doc_overclaims_in("test.md", text, &rules);
        assert_eq!(
            violations.len(),
            1,
            "overclaim detection must catch '{text}' case-and-whitespace-insensitively"
        );
    }
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let normalized_text = normalize(text);
    let missing: Vec<_> = needles
        .iter()
        .filter(|n| !normalized_text.contains(&normalize(n)))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{label} is missing required bounded language: {missing:?}"
    );
}

fn assert_contains_none(label: &str, text: &str, needles: &[&str]) {
    assert_contains_none_with_message(
        text,
        needles,
        &format!("{label} contains internal or overbroad language"),
    );
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
