use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn run_first_lesson_readiness_cli_reports_incomplete_manifest_only_sequence() {
    let root = scratch_root("first-lesson-readiness-cli");
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

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-sequence",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    assert_eq!(
        report["schema_version"],
        "eatme.first-lesson-readiness-sequence/v1"
    );
    assert_eq!(report["passed"], false);
    assert_eq!(report["readiness_status"], "incomplete");
    assert_eq!(
        report["desktop_proof_contract"],
        report["readiness_report"]["desktop_proof_contract"]
    );
    assert_eq!(
        report["desktop_proof_contract"],
        serde_json::json!({
            "status": "skipped",
            "reason_code": "execute_not_requested",
            "detail": "execution was not requested; rerun with --execute on a machine with Alice desktop access to collect real desktop proof",
            "target_role": "modernized",
            "artifact": null
        })
    );
    assert_eq!(report["evidence_progress"]["total_required"], 10);
    assert!(
        report["evidence_progress"]["summary"]
            .as_str()
            .unwrap()
            .contains("required evidence items are present")
    );
    assert!(report["readiness_report"]["evidence_progress"]["items"].is_array());
    assert!(
        report["comparison_manifest_path"]
            .as_str()
            .unwrap()
            .ends_with("comparison-manifest.json")
    );
    let save_item = progress_item(&report, "save_project_proof_artifact");
    let select_item = progress_item(&report, "select_project_proof_artifact");
    assert_eq!(save_item["state"], "missing");
    assert_eq!(select_item["state"], "missing");
    assert_eq!(save_item["evidence"], "Save Project proof artifact");
    assert_eq!(select_item["evidence"], "Select Project proof artifact");
}

#[test]
fn run_first_lesson_readiness_cli_plain_text_lists_evidence_progress() {
    let root = scratch_root("first-lesson-readiness-cli-text");
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

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-sequence-text",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("First-lesson readiness: incomplete"));
    assert!(stdout.contains(
        "Desktop proof: skipped (execute_not_requested) - execution was not requested; rerun with --execute on a machine with Alice desktop access to collect real desktop proof"
    ));
    assert!(stdout.contains("Evidence progress:"));
    assert!(stdout.contains("required evidence items are present"));
    assert!(stdout.contains(
        "Required evidence file status (present/missing/invalid/blocked; present is artifact availability only, not proof of full UI automation):"
    ));
    assert!(
        stdout.contains("present: comparison-manifest.json with baseline and modernized targets")
    );
    assert!(stdout.contains("missing: launch evidence for each target"));
    assert!(stdout.contains("modernized desktop-run-pixel-observation.json status"));
    assert!(stdout.contains("missing: Save Project proof artifact"));
    assert!(stdout.contains("missing: Select Project proof artifact"));
    assert!(stdout.contains("Limits:"));
    assert!(stdout.contains("does not prove full Alice UI automation"));
    assert!(stdout.contains("does not prove visible rendering correctness"));
    assert!(stdout.contains("does not prove first-lesson completion"));
    assert!(stdout.contains("Still missing or blocked:"));
    assert_plain_output_avoids_project_proof_success_claims(&stdout);
}

#[test]
fn run_first_lesson_readiness_cli_plain_text_uses_scenario_blocker_language() {
    let root = scratch_root("first-lesson-readiness-cli-scenarios");
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

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-scenario-text",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("First-lesson automation scenario readiness: not ready"));
    assert!(stdout.contains("Blockers:"));
    assert!(stdout.contains("Select Project scenario evidence is missing."));
    assert!(stdout.contains("Procedure/edit scenario evidence is missing."));
    assert!(stdout.contains("Save scenario evidence is missing."));
    assert!(stdout.contains("Visible rendering scenario evidence is missing."));
    assert!(stdout.contains("Grading scenario evidence is missing."));
    assert!(stdout.contains("Creative assessment scenario evidence is missing."));
    assert!(stdout.contains("First-lesson completion scenario evidence is missing."));
    assert!(stdout.contains("automation scenarios"));
    assert_plain_output_avoids_boundary_jargon(&stdout);
    assert_plain_output_avoids_project_proof_success_claims(&stdout);
}

#[test]
fn run_first_lesson_readiness_cli_json_exposes_boundary_statuses_additively() {
    let root = scratch_root("first-lesson-readiness-cli-boundary-json");
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

    let output = Command::new(eatme_bin())
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "manifest-only-boundary-json",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    assert!(report["evidence_progress"]["items"].is_array());
    assert_eq!(
        report["evidence_boundaries"], report["readiness_report"]["evidence_boundaries"],
        "top-level sequence report must mirror readiness boundary statuses"
    );
    assert_boundary_ids(&report);
    for id in REQUIRED_BOUNDARY_IDS {
        let boundary = evidence_boundary(&report, id);
        assert_ne!(
            boundary["status"], "present",
            "manifest-only readiness cannot mark {id} as present: {boundary}"
        );
        assert_boundary_text_is_scenario_focused(boundary);
    }
}

#[test]
fn run_first_lesson_readiness_cli_reports_unsupported_environment_contract() {
    let root = scratch_root("first-lesson-readiness-cli-unsupported");
    let registry_path = root.join("targets.yaml");
    fs::write(
        &registry_path,
        r#"
schema_version: eatme.alice-comparison-targets/v1
targets:
  baseline:
    label: Baseline Alice
    description: Reference target.
    alice_home_env: EATME_TEST_BASELINE_HOME_NOT_SET
  modernized:
    label: Modernized Alice
    description: Candidate target.
    alice_home_env: EATME_TEST_MODERNIZED_HOME_NOT_SET
"#,
    )
    .unwrap();

    let output = Command::new(eatme_bin())
        .env("EATME_REAL_ALICE", "1")
        .args([
            "alice",
            "run-first-lesson-readiness",
            "--registry",
            registry_path.to_str().unwrap(),
            "--run-id",
            "unsupported-environment-contract",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--execute",
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    assert_eq!(
        report["desktop_proof_contract"],
        serde_json::json!({
            "status": "unsupported_environment",
            "reason_code": "alice_home_unresolved",
            "detail": "modernized target did not launch desktop Alice proof collection (alice_home_unresolved)",
            "target_role": "modernized",
            "artifact": null
        })
    );
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

fn progress_item<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["evidence_progress"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing evidence_progress item {id}"))
}

const REQUIRED_BOUNDARY_IDS: [&str; 7] = [
    "select_project",
    "procedure_edit",
    "save_project",
    "visible_rendering",
    "grading",
    "creative_assessment",
    "first_lesson_completion",
];

fn evidence_boundaries(report: &serde_json::Value) -> &[serde_json::Value] {
    report["evidence_boundaries"]
        .as_array()
        .unwrap_or_else(|| panic!("expected top-level evidence_boundaries[] in {report}"))
}

fn evidence_boundary<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    evidence_boundaries(report)
        .iter()
        .find(|boundary| boundary["id"] == id)
        .unwrap_or_else(|| panic!("missing evidence boundary {id} in {report}"))
}

fn assert_boundary_ids(report: &serde_json::Value) {
    let actual = evidence_boundaries(report)
        .iter()
        .map(|boundary| boundary["id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(actual, REQUIRED_BOUNDARY_IDS, "unexpected boundary order");
}

fn assert_boundary_text_is_scenario_focused(boundary: &serde_json::Value) {
    let label = boundary["label"].as_str().unwrap_or_default();
    let detail = boundary["detail"].as_str().unwrap_or_default();
    assert!(
        label.contains("scenario evidence"),
        "boundary label must use scenario-focused wording: {boundary}"
    );
    assert!(
        detail.contains("scenario") || detail.contains("automation scenarios"),
        "boundary detail must use scenario-focused wording: {boundary}"
    );
}

fn assert_plain_output_avoids_project_proof_success_claims(stdout: &str) {
    let stdout = stdout.to_ascii_lowercase();
    for forbidden in [
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
