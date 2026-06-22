use crate::generate_gadugi_adapter_yaml;
use crate::schema::EatmeScenarioAsset;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIO_ID: &str = "starter-project-open-save-export-preflight";
const SOURCE_SCENARIO_PATH: &str =
    "assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml";
const GENERATED_ADAPTER_PATH: &str =
    "assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml";
const CONTRACT_DOC_PATH: &str = "docs/default-workflow-pr-readiness.md";
const EVIDENCE_DOC_PATH: &str = "docs/starter-project-preflight-evidence.md";
const OVERCLAIM_RULE_TABLE_HEADER: &str = "| Prohibited phrase | Bounded replacement |";

const REQUIRED_SOURCE_BOUNDARIES: &[&str] = &[
    "plain automation scenario for instructors and students",
    "opened starter project",
    "small editable starter-world change",
    "attempt to run or observe",
    "LookingGlass REST evidence",
    "remaining classroom-readiness gaps",
    "not proof of visible rendering correctness",
    "not full UI automation",
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
    "separate LookingGlass save/reopen/export evidence",
    "not full UI automation",
    "not creative assessment",
    "not learner-world grading",
    "not complete Alice coverage",
    "not visible rendering correctness proof",
    "not first-lesson completion",
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
    assert_no_doc_overclaims(
        SOURCE_SCENARIO_PATH,
        &text,
        &read_contract_overclaim_rules(&root),
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
    assert_no_doc_overclaims(
        GENERATED_ADAPTER_PATH,
        &generated,
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
            "{CONTRACT_DOC_PATH} must describe the scoped documentation overclaim check as current executable behavior, not planned future work"
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
        "This starter-project preflight evidence is PR ready.\nIt proves visible rendering correctness.",
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

fn read_contract_overclaim_rules(root: &Path) -> Vec<OverclaimRule> {
    overclaim_rules_from_contract(&read_repo_text(root, CONTRACT_DOC_PATH))
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
        text,
        needles,
        &format!("{label} contains internal or overbroad language"),
    );
}

fn assert_contains_none_with_message(text: &str, needles: &[&str], message: &str) {
    let normalized_text = normalize(text).to_lowercase();
    let present = needles
        .iter()
        .filter(|needle| normalized_text.contains(&normalize(needle).to_lowercase()))
        .copied()
        .collect::<Vec<_>>();
    assert!(present.is_empty(), "{message}: {present:?}");
}

#[derive(Debug, PartialEq, Eq)]
struct OverclaimRule {
    phrase: String,
    normalized_phrase: String,
    bounded_replacement: String,
}

impl OverclaimRule {
    fn new(phrase: &str, bounded_replacement: &str) -> Self {
        Self {
            phrase: phrase.to_string(),
            normalized_phrase: normalize(phrase).to_lowercase(),
            bounded_replacement: bounded_replacement.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ReadinessOverclaim<'a> {
    file: &'static str,
    line_number: usize,
    phrase: &'a str,
    bounded_replacement: &'a str,
}

fn assert_no_doc_overclaims(file: &'static str, text: &str, rules: &[OverclaimRule]) {
    let violations = doc_overclaims_in(file, text, rules);
    assert!(
        violations.is_empty(),
        "{}",
        format_overclaim_failures(&violations)
    );
}

fn doc_overclaims_in<'a>(
    file: &'static str,
    text: &str,
    rules: &'a [OverclaimRule],
) -> Vec<ReadinessOverclaim<'a>> {
    text.lines()
        .enumerate()
        .flat_map(|(line_index, line)| {
            let normalized_line = normalize(line).to_lowercase();
            rules
                .iter()
                .filter(move |rule| line_overclaims(&normalized_line, rule))
                .map(move |rule| ReadinessOverclaim {
                    file,
                    line_number: line_index + 1,
                    phrase: &rule.phrase,
                    bounded_replacement: &rule.bounded_replacement,
                })
        })
        .collect()
}

fn line_overclaims(normalized_line: &str, rule: &OverclaimRule) -> bool {
    normalized_line
        .match_indices(&rule.normalized_phrase)
        .any(|(index, _)| !is_negated_boundary(&normalized_line[..index]))
}

fn is_negated_boundary(prefix: &str) -> bool {
    prefix.ends_with(" not ")
        || prefix.ends_with(" does not ")
        || prefix.ends_with(" do not ")
        || prefix.ends_with(" without ")
}

fn format_overclaim_failures(violations: &[ReadinessOverclaim<'_>]) -> String {
    violations
        .iter()
        .map(|violation| {
            format!(
                "{} overclaims starter-project/preflight readiness or evidence with prohibited phrase `{}` on line {}; source contract: {}; use bounded wording such as `{}`",
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

fn overclaim_rules_from_contract(text: &str) -> Vec<OverclaimRule> {
    let mut lines = text
        .lines()
        .skip_while(|line| line.trim() != OVERCLAIM_RULE_TABLE_HEADER);
    let Some(_) = lines.next() else {
        panic!("{CONTRACT_DOC_PATH} is missing table header `{OVERCLAIM_RULE_TABLE_HEADER}`");
    };

    let mut rules = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            break;
        }
        if is_markdown_table_separator(trimmed) {
            continue;
        }
        rules.push(parse_overclaim_rule(trimmed).unwrap_or_else(|| {
            panic!(
                "{CONTRACT_DOC_PATH} contains a malformed overclaim rule row after `{OVERCLAIM_RULE_TABLE_HEADER}`: {trimmed}"
            )
        }));
    }

    assert!(
        !rules.is_empty(),
        "{CONTRACT_DOC_PATH} table `{OVERCLAIM_RULE_TABLE_HEADER}` must define at least one overclaim rule"
    );
    rules
}

fn parse_overclaim_rule(line: &str) -> Option<OverclaimRule> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }

    let mut cells = trimmed.trim_matches('|').split('|').map(str::trim);
    let phrase = code_span_text(cells.next()?)?;
    let bounded_replacement = code_span_text(cells.next()?)?;
    if cells.next().is_some() {
        return None;
    }

    Some(OverclaimRule::new(phrase, bounded_replacement))
}

fn code_span_text(cell: &str) -> Option<&str> {
    cell.strip_prefix('`')?.strip_suffix('`')
}

fn is_markdown_table_separator(line: &str) -> bool {
    line.trim_matches('|')
        .split('|')
        .map(str::trim)
        .all(|cell| cell.chars().all(|ch| ch == '-'))
}

fn assert_rules_match_contract(rules: &[OverclaimRule], expected_rules: &[(&str, &str)]) {
    let documented_rules = rules
        .iter()
        .map(|rule| (rule.phrase.as_str(), rule.bounded_replacement.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        documented_rules, expected_rules,
        "{CONTRACT_DOC_PATH} must define exactly the documented starter-project/preflight overclaim rules"
    );
}

fn normalize(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for part in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(part);
    }
    normalized
}
