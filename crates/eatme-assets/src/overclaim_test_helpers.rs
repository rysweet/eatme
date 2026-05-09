//! Shared overclaim detection helpers for boundary contract tests.

use std::fs;
use std::path::Path;

pub const CONTRACT_DOC_PATH: &str = "docs/default-workflow-pr-readiness.md";
pub const EVIDENCE_DOC_PATH: &str = "docs/starter-project-preflight-evidence.md";

const OVERCLAIM_RULE_TABLE_HEADER: &str = "| Prohibited phrase | Bounded replacement |";

pub fn read_repo_text(root: &Path, repo_relative_path: &str) -> String {
    fs::read_to_string(root.join(repo_relative_path))
        .unwrap_or_else(|e| panic!("failed to read {repo_relative_path}: {e}"))
}

pub fn read_contract_overclaim_rules(root: &Path) -> Vec<OverclaimRule> {
    overclaim_rules_from_contract(&read_repo_text(root, CONTRACT_DOC_PATH))
}

pub fn assert_no_doc_overclaims(file: &'static str, text: &str, rules: &[OverclaimRule]) {
    let violations = doc_overclaims_in(file, text, rules);
    assert!(
        violations.is_empty(),
        "{}",
        format_overclaim_failures(&violations)
    );
}

pub fn doc_overclaims_in<'a>(
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

pub fn overclaim_rules_from_contract(text: &str) -> Vec<OverclaimRule> {
    let mut lines = text
        .lines()
        .skip_while(|line| line.trim() != OVERCLAIM_RULE_TABLE_HEADER);
    let Some(_) = lines.next() else {
        panic!(
            "{CONTRACT_DOC_PATH} is missing table header \
             `{OVERCLAIM_RULE_TABLE_HEADER}`"
        );
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
                "{CONTRACT_DOC_PATH} contains a malformed overclaim rule row \
                 after `{OVERCLAIM_RULE_TABLE_HEADER}`: {trimmed}"
            )
        }));
    }
    assert!(
        !rules.is_empty(),
        "{CONTRACT_DOC_PATH} table `{OVERCLAIM_RULE_TABLE_HEADER}` \
         must define at least one overclaim rule"
    );
    rules
}

pub fn assert_rules_match_contract(rules: &[OverclaimRule], expected: &[(&str, &str)]) {
    let documented: Vec<_> = rules
        .iter()
        .map(|r| (r.phrase.as_str(), r.bounded_replacement.as_str()))
        .collect();
    assert_eq!(
        documented, expected,
        "{CONTRACT_DOC_PATH} must define exactly the documented \
         starter-project/preflight overclaim rules"
    );
}

pub fn assert_contains_none_with_message(text: &str, needles: &[&str], message: &str) {
    let normalized_text = normalize(text).to_lowercase();
    let present: Vec<_> = needles
        .iter()
        .filter(|n| normalized_text.contains(&normalize(n).to_lowercase()))
        .copied()
        .collect();
    assert!(present.is_empty(), "{message}: {present:?}");
}

#[derive(Debug, PartialEq, Eq)]
pub struct OverclaimRule {
    pub phrase: String,
    pub normalized_phrase: String,
    pub bounded_replacement: String,
}

impl OverclaimRule {
    pub fn new(phrase: &str, bounded_replacement: &str) -> Self {
        Self {
            phrase: phrase.to_string(),
            normalized_phrase: normalize(phrase).to_lowercase(),
            bounded_replacement: bounded_replacement.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReadinessOverclaim<'a> {
    pub file: &'static str,
    pub line_number: usize,
    pub phrase: &'a str,
    pub bounded_replacement: &'a str,
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

pub fn format_overclaim_failures(violations: &[ReadinessOverclaim<'_>]) -> String {
    violations
        .iter()
        .map(|v| {
            format!(
                "{} overclaims starter-project/preflight readiness or evidence \
                 with prohibited phrase `{}` on line {}; source contract: {}; \
                 use bounded wording such as `{}`",
                v.file, v.phrase, v.line_number, CONTRACT_DOC_PATH, v.bounded_replacement
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_overclaim_rule(line: &str) -> Option<OverclaimRule> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let mut cells = trimmed.trim_matches('|').split('|').map(str::trim);
    let phrase = cells.next()?.strip_prefix('`')?.strip_suffix('`')?;
    let replacement = cells.next()?.strip_prefix('`')?.strip_suffix('`')?;
    if cells.next().is_some() {
        return None;
    }
    Some(OverclaimRule::new(phrase, replacement))
}

fn is_markdown_table_separator(line: &str) -> bool {
    line.trim_matches('|')
        .split('|')
        .map(str::trim)
        .all(|cell| cell.chars().all(|ch| ch == '-'))
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_consecutive_whitespace_to_single_space() {
        assert_eq!(normalize("hello   world"), "hello world");
        assert_eq!(
            normalize("  leading  and trailing  "),
            "leading and trailing"
        );
        assert_eq!(normalize("tab\there"), "tab here");
        assert_eq!(normalize("single"), "single");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn overclaim_rule_new_stores_normalized_lowercase_phrase() {
        let rule = OverclaimRule::new("PR   Ready", "bounded wording");
        assert_eq!(rule.phrase, "PR   Ready");
        assert_eq!(rule.normalized_phrase, "pr ready");
        assert_eq!(rule.bounded_replacement, "bounded wording");
    }
}
