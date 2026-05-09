use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn silver_thread_gap_names_shown_and_not_yet_shown_scenario_evidence() {
    let section = silver_thread_gap_section();

    assert!(
        section.contains("Scenario evidence shown:"),
        "Silver-thread run/observe gap must name what scenario evidence shows:\n{section}"
    );
    assert!(
        section.contains("What remains not yet shown:"),
        "Silver-thread run/observe gap must name what remains not yet shown:\n{section}"
    );
    assert!(
        section.contains("Next automation scenario evidence needed:"),
        "Silver-thread run/observe gap must name the next bounded automation scenario evidence:\n{section}"
    );
}

#[test]
fn silver_thread_gap_uses_plain_scenario_terms_without_runtime_output_claims() {
    let section = silver_thread_gap_section();

    for required in [
        "automation scenario",
        "scenario evidence",
        "user-facing Run-window state",
        "gap is still open",
    ] {
        assert!(
            section.contains(required),
            "Silver-thread run/observe gap must include plain user-facing term {required:?}:\n{section}"
        );
    }

    for forbidden in [
        "CLI renders a Silver-thread",
        "report renders a Silver-thread",
        "rendered Silver-thread section",
    ] {
        assert!(
            !section.contains(forbidden),
            "Silver-thread run/observe gap must not claim a distinct runtime-rendered section: {forbidden}"
        );
    }
}

#[test]
fn silver_thread_gap_keeps_completion_and_assessment_claims_unproven() {
    let section = silver_thread_gap_section();

    for forbidden in [
        "full world execution is shown",
        "world fully ran",
        "visible rendering is correct",
        "grading result is shown",
        "creative assessment result is shown",
        "Save completed",
        "full Save completion",
        "first lesson is complete",
        "first-lesson completion is shown",
    ] {
        assert!(
            !section.contains(forbidden),
            "Silver-thread run/observe gap must not make unsupported claim: {forbidden}"
        );
    }

    assert!(
        section.contains("without treating it as"),
        "Silver-thread run/observe gap must explicitly keep unsupported completion claims out:\n{section}"
    );
}

fn silver_thread_gap_section() -> String {
    let document = fs::read_to_string(readiness_doc_path()).unwrap();
    section_between(
        &document,
        "### Silver-thread run/observe gap",
        "| Result | Meaning | What to do |",
    )
    .to_string()
}

fn readiness_doc_path() -> PathBuf {
    repository_root().join("docs/first-lesson-evidence-readiness.md")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn section_between<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = document
        .find(start)
        .unwrap_or_else(|| panic!("missing section heading {start:?}"));
    let after_start = &document[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing section boundary {end:?}"));
    &after_start[..end_index]
}
