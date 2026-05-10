use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn run_first_lesson_readiness_cli_plain_text_uses_user_facing_readiness_sections() {
    let root = scratch_root("first-lesson-readiness-cli-report-sections");
    let registry_path = write_registry(&root);

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-report-sections",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("First-lesson/grading gap report: not ready"));
    assert!(stdout.contains(
        "Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only."
    ));
    assert!(stdout.contains("Shown:"));
    assert!(stdout.contains("Not yet shown:"));
    assert!(stdout.contains("- Save option/action evidence is not yet shown."));
    assert!(stdout.contains("- Grading is not yet shown."));
    assert!(stdout.contains("- Creative assessment is not yet shown."));
    assert!(stdout.contains("- First-lesson completion is not yet shown."));
    assert!(stdout.contains("Unproven:"));
    assert!(stdout.contains("- Full Alice UI automation is not proven."));
    assert!(stdout.contains("- Grading is not proven."));
    assert!(stdout.contains("- Creative assessment is not proven."));
    assert!(stdout.contains("- Visible rendering correctness is not proven."));
    assert!(stdout.contains("- Save completion is not proven."));
    assert!(stdout.contains("- First-lesson completion is not proven."));
    for legacy_heading in [
        "Evidence progress:",
        "Blockers:",
        "Still missing or blocked:",
    ] {
        assert!(
            !stdout.contains(legacy_heading),
            "plain output should use user-facing readiness sections instead of {legacy_heading:?}: {stdout}"
        );
    }
    assert_plain_output_avoids_boundary_jargon(&stdout);
    assert_plain_output_avoids_project_proof_success_claims(&stdout);
}

#[test]
fn run_first_lesson_readiness_cli_json_exposes_user_facing_readiness_buckets() {
    let root = scratch_root("first-lesson-readiness-cli-user-facing-json");
    let registry_path = write_registry(&root);

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-user-facing-json",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    assert_readiness_bucket_is_array(&report, "shown_evidence");
    assert_readiness_bucket_is_array(&report, "not_yet_shown");
    assert_readiness_bucket_is_array(&report, "unproven_claims");
    assert!(
        report.get("desktop_next_action").is_none(),
        "absent desktop next-action evidence must omit desktop_next_action: {report}"
    );
    assert_eq!(
        report["shown_evidence"], report["readiness_report"]["shown_evidence"],
        "sequence report should mirror readiness shown_evidence"
    );
    assert_eq!(
        report["not_yet_shown"], report["readiness_report"]["not_yet_shown"],
        "sequence report should mirror readiness not_yet_shown"
    );
    assert_eq!(
        report["unproven_claims"], report["readiness_report"]["unproven_claims"],
        "sequence report should mirror readiness unproven_claims"
    );

    let save = readiness_bucket_item(&report, "not_yet_shown", "save_project");
    assert_eq!(
        save["summary"],
        "Save option/action evidence is not yet shown."
    );
    assert_user_facing_not_yet_shown_summary(save);
    let completion = readiness_bucket_item(&report, "not_yet_shown", "first_lesson_completion");
    assert_eq!(
        completion["summary"],
        "First-lesson completion is not yet shown."
    );
    assert_user_facing_not_yet_shown_summary(completion);
    assert_unproven_claims(&report);
    assert_readiness_json_avoids_unsupported_success_claims(&report);
}

fn write_registry(root: &Path) -> PathBuf {
    let registry_path = root.join("targets.yaml");
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home: ./alice-baseline
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home: ./alice-modernized
"#,
    )
    .unwrap();
    registry_path
}

fn scratch_root(name: &str) -> PathBuf {
    let root = workspace_root()
        .join("target/eatme-cli-integration-tests")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn eatme_bin() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_eatme-cli") {
        return PathBuf::from(path);
    }
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("eatme-cli")
}

fn assert_exit_code(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "unexpected status {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_readiness_bucket_is_array(report: &serde_json::Value, field: &str) {
    assert!(
        report[field].as_array().is_some(),
        "expected top-level {field}[] in readiness JSON: {report}"
    );
}

fn readiness_bucket_item<'a>(
    report: &'a serde_json::Value,
    field: &str,
    id: &str,
) -> &'a serde_json::Value {
    report[field]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level {field}[] in readiness JSON: {report}"))
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing {field} item {id} in readiness JSON: {report}"))
}

fn assert_user_facing_not_yet_shown_summary(item: &serde_json::Value) {
    let summary = item["summary"].as_str().unwrap_or_default();
    assert!(
        summary.contains("not yet shown"),
        "not_yet_shown summary must use plain user-facing wording: {item}"
    );
    let summary_lower = summary.to_ascii_lowercase();
    for forbidden in [
        "blocker",
        "blocked",
        "invalid",
        "missing",
        "proof artifact",
        "ui-action-contract",
        "desktop-run-pixel",
        "desktop-first-lesson-next-action",
        "no_go",
    ] {
        assert!(
            !summary_lower.contains(forbidden),
            "not_yet_shown summary leaked internal wording {forbidden:?}: {item}"
        );
    }
}

fn assert_unproven_claims(report: &serde_json::Value) {
    let claims = report["unproven_claims"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level unproven_claims[]: {report}"))
        .iter()
        .map(|claim| claim.as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        claims,
        [
            "Full Alice UI automation is not proven.",
            "Grading is not proven.",
            "Creative assessment is not proven.",
            "Visible rendering correctness is not proven.",
            "Save completion is not proven.",
            "First-lesson completion is not proven.",
        ],
        "unproven_claims must be the canonical user-facing non-claims"
    );
}

fn assert_plain_output_avoids_boundary_jargon(stdout: &str) {
    for forbidden in [
        "proof artifact",
        "ui-action-contract",
        "desktop-run-pixel",
        "desktop first-lesson next-action",
        "action_id",
        "no_go",
        "RabbitHole",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "plain scenario output leaked implementation detail {forbidden:?}: {stdout}"
        );
    }
}

fn assert_plain_output_avoids_project_proof_success_claims(stdout: &str) {
    let stdout = stdout.to_ascii_lowercase();
    for forbidden in [
        "save completion evidence",
        "save completed",
        "ui automation succeeded",
        "automation passed",
        "lesson completed",
        "grading occurred",
        "creative assessment passed",
        "creative quality assessed",
        "save project succeeded",
        "select project succeeded",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "plain output must not claim {forbidden:?}"
        );
    }
}

fn assert_readiness_json_avoids_unsupported_success_claims(report: &serde_json::Value) {
    let text = serde_json::to_string(report).unwrap().to_ascii_lowercase();
    for forbidden in [
        "save completion evidence",
        "save completed",
        "save project succeeded",
        "lesson completed",
        "ui automation succeeded",
        "grading occurred",
        "creative assessment passed",
        "creative quality assessed",
    ] {
        assert!(
            !text.contains(forbidden),
            "CLI readiness JSON must not claim {forbidden:?}: {text}"
        );
    }
}
