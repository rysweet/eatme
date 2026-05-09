use std::fs;
use std::path::{Path, PathBuf};

const HANDOFF_DOC: &str = "docs/save-reopen-export-evidence-handoff.md";
const READINESS_DOC: &str = "docs/default-workflow-pr-readiness.md";
const INDEX_DOC: &str = "docs/index.md";
const PREFLIGHT_DOC: &str = "docs/starter-project-preflight-evidence.md";

const HANDOFF_SCENARIO_ID: &str = "instructor-student-save-reopen-export-evidence-handoff";

const HANDOFF_DOC_REQUIRED_SECTIONS: &[&str] = &[
    "# Save, reopen, and export evidence handoff",
    "## Quick start",
    "## What the scenario covers",
    "## Related documentation",
];

const HANDOFF_DOC_REQUIRED_CROSS_LINKS: &[&str] = &[
    "starter-project-preflight-evidence.md",
    "default-workflow-pr-readiness.md",
    "persona-assets.md",
    "gadugi-adapters.md",
    "generated-asset-consistency.md",
];

const READINESS_DOC_REQUIRED_RELATED_LINKS: &[&str] = &[
    "starter-project-preflight-evidence.md",
    "save-reopen-export-evidence-handoff.md",
];

const INDEX_EVIDENCE_TABLE_REQUIRED_ROWS: &[&str] = &[
    "real-alice-launch-smoke",
    "first-lessons-real-ui-actions",
    "instructor-student-save-reopen-export-evidence-handoff",
    "instructor-lesson-materials-remix",
];

#[test]
fn handoff_doc_has_required_sections_and_related_documentation_links() {
    let root = repository_root();
    let text = read_doc(&root, HANDOFF_DOC);

    assert_contains_all(
        "handoff doc required sections",
        &text,
        HANDOFF_DOC_REQUIRED_SECTIONS,
    );

    let related_section = extract_section(&text, "## Related documentation");
    assert!(
        !related_section.is_empty(),
        "{HANDOFF_DOC} must have a non-empty 'Related documentation' section"
    );
    assert_contains_all(
        "handoff doc cross-links",
        &related_section,
        HANDOFF_DOC_REQUIRED_CROSS_LINKS,
    );
}

#[test]
fn handoff_doc_cross_links_resolve_to_existing_files() {
    let root = repository_root();
    let text = read_doc(&root, HANDOFF_DOC);
    let broken = broken_markdown_links(&root, &text);
    assert!(
        broken.is_empty(),
        "{HANDOFF_DOC} has broken cross-links: {broken:?}"
    );
}

#[test]
fn readiness_doc_related_documentation_links_preflight_and_handoff() {
    let root = repository_root();
    let text = read_doc(&root, READINESS_DOC);

    let related_section = extract_section(&text, "## Related documentation");
    assert!(
        !related_section.is_empty(),
        "{READINESS_DOC} must have a non-empty 'Related documentation' section"
    );
    assert_contains_all(
        "readiness doc Related documentation links",
        &related_section,
        READINESS_DOC_REQUIRED_RELATED_LINKS,
    );
}

#[test]
fn readiness_doc_cross_links_resolve_to_existing_files() {
    let root = repository_root();
    let text = read_doc(&root, READINESS_DOC);
    let broken = broken_markdown_links(&root, &text);
    assert!(
        broken.is_empty(),
        "{READINESS_DOC} has broken cross-links: {broken:?}"
    );
}

#[test]
fn index_outside_in_evidence_table_includes_handoff_scenario() {
    let root = repository_root();
    let text = read_doc(&root, INDEX_DOC);

    let table_section = extract_section(&text, "## Outside-in evidence for Alice lesson scenarios");
    assert!(
        !table_section.is_empty(),
        "{INDEX_DOC} must have an 'Outside-in evidence' section"
    );

    for scenario_id in INDEX_EVIDENCE_TABLE_REQUIRED_ROWS {
        assert!(
            table_section.contains(scenario_id),
            "{INDEX_DOC} outside-in evidence table must include scenario `{scenario_id}`"
        );
    }

    assert!(
        table_section.contains(HANDOFF_SCENARIO_ID),
        "{INDEX_DOC} outside-in evidence table must include the handoff scenario `{HANDOFF_SCENARIO_ID}`"
    );
}

#[test]
fn index_audience_routes_link_handoff_and_preflight_docs() {
    let root = repository_root();
    let text = read_doc(&root, INDEX_DOC);

    let routes_section = extract_section(&text, "## Audience routes");
    assert!(
        !routes_section.is_empty(),
        "{INDEX_DOC} must have an 'Audience routes' section"
    );
    assert_contains_all(
        "index audience routes",
        &routes_section,
        &[
            "save-reopen-export-evidence-handoff.md",
            "starter-project-preflight-evidence.md",
            "default-workflow-pr-readiness.md",
        ],
    );
}

#[test]
fn preflight_doc_exists_and_is_linked_from_handoff_and_readiness() {
    let root = repository_root();
    assert!(
        root.join(PREFLIGHT_DOC).is_file(),
        "{PREFLIGHT_DOC} must exist as the preflight evidence doc"
    );

    let handoff_text = read_doc(&root, HANDOFF_DOC);
    let readiness_text = read_doc(&root, READINESS_DOC);

    assert!(
        handoff_text.contains("starter-project-preflight-evidence.md"),
        "{HANDOFF_DOC} must link to the preflight evidence doc"
    );
    assert!(
        readiness_text.contains("starter-project-preflight-evidence.md"),
        "{READINESS_DOC} must link to the preflight evidence doc"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_doc(root: &Path, relative_path: &str) -> String {
    let path = root.join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn extract_section(text: &str, heading: &str) -> String {
    let heading_level = heading.chars().take_while(|c| *c == '#').count();
    let section_prefix = "#".repeat(heading_level);

    let mut in_section = false;
    let mut lines = Vec::new();

    for line in text.lines() {
        if line.starts_with(heading) {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with(&format!("{section_prefix} "))
                || line.starts_with(&format!("{section_prefix}\t"))
            {
                break;
            }
            // Also break on same-level or higher headings
            let line_level = line.chars().take_while(|c| *c == '#').count();
            if line_level > 0 && line_level <= heading_level && line.contains(' ') {
                break;
            }
            lines.push(line);
        }
    }

    lines.join("\n")
}

fn broken_markdown_links(root: &Path, text: &str) -> Vec<String> {
    let mut broken = Vec::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(paren_start) = rest.find("](") {
            let after_paren = &rest[paren_start + 2..];
            if let Some(paren_end) = after_paren.find(')') {
                let link_target = &after_paren[..paren_end];
                // Only check relative .md links (not URLs, anchors, or code paths)
                if link_target.ends_with(".md")
                    && !link_target.starts_with("http")
                    && !link_target.contains("assets/")
                {
                    let resolved = root.join("docs").join(link_target);
                    if !resolved.is_file() {
                        broken.push(link_target.to_string());
                    }
                }
                rest = &after_paren[paren_end..];
            } else {
                break;
            }
        }
    }
    broken.sort();
    broken.dedup();
    broken
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    let missing: Vec<_> = needles
        .iter()
        .filter(|needle| !text.contains(**needle))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{label} is missing required content: {missing:?}"
    );
}
