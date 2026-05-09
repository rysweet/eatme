use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn launch_smoke_docs_match_ready_or_not_ready_status_contract() {
    let root = workspace_root();
    let launch_doc =
        fs::read_to_string(root.join("docs/real-alice-launch-smoke-readiness.md")).unwrap();
    let cli_usage = fs::read_to_string(root.join("docs/cli-usage.md")).unwrap();
    let smoke_doc = fs::read_to_string(root.join("docs/alice-lesson-smoke.md")).unwrap();
    let cli_launch_section = markdown_section(
        &cli_usage,
        "Check bounded launch-smoke readiness",
        "See [Real Alice Launch-Smoke Readiness]",
    );
    let smoke_launch_section = markdown_section(
        &smoke_doc,
        "`alice check-lesson-readiness` maps `real-alice-launch-smoke`",
        "The `first-lessons-real-ui-actions` scenario is different",
    );

    assert_no_blocked_status("launch-smoke readiness docs", &launch_doc);
    assert_no_blocked_status("CLI launch-smoke readiness docs", cli_launch_section);
    assert_no_blocked_status(
        "Alice smoke launch-smoke readiness docs",
        smoke_launch_section,
    );
}

fn assert_no_blocked_status(label: &str, text: &str) {
    assert!(
        !text.to_ascii_lowercase().contains("blocked"),
        "{label} must not advertise a blocked status: {text}"
    );
}

fn markdown_section<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = document
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker {start:?}"));
    let tail = &document[start_index..];
    let end_index = tail
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker {end:?}"));
    &tail[..end_index]
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
