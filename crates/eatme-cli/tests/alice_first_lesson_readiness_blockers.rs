use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn run_first_lesson_readiness_cli_json_exposes_original_alice_action_blockers() {
    let root = scratch_root("first-lesson-readiness-cli-action-blockers-json");
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
            "manifest-only-action-blockers-json",
            "--runs-dir",
            root.join("runs").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sequence report is JSON");
    let baseline = readiness_target_evidence(&report, "baseline");
    let blockers = baseline["blockers"]
        .as_array()
        .unwrap_or_else(|| panic!("baseline target should expose blockers: {baseline}"));
    let blocker = blockers
        .iter()
        .find(|blocker| {
            blocker["code"] == "missing_real_action_evidence" && blocker["action"] == "save-project"
        })
        .unwrap_or_else(|| panic!("missing save-project blocker: {blockers:?}"));
    assert_eq!(
        blocker,
        &serde_json::json!({
            "code": "missing_real_action_evidence",
            "action": "save-project",
            "reason": "Required original Alice action evidence is missing from automation scenarios."
        })
    );
    assert_safe_blocker_reason(blocker["reason"].as_str().unwrap_or_default());
    let modernized = readiness_target_evidence(&report, "modernized");
    assert_eq!(
        modernized["blockers"],
        serde_json::json!([]),
        "only original Alice evidence should receive original-action blockers"
    );
    assert_limitations_preserve_non_claims(&report);
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

fn readiness_target_evidence<'a>(
    report: &'a serde_json::Value,
    role: &str,
) -> &'a serde_json::Value {
    report["readiness_report"]["target_evidence"]
        .as_array()
        .unwrap_or_else(|| {
            panic!("report should expose readiness_report.target_evidence[]: {report}")
        })
        .iter()
        .find(|target| target["role"] == role)
        .unwrap_or_else(|| panic!("missing readiness target evidence {role}: {report}"))
}

fn assert_safe_blocker_reason(reason: &str) {
    for forbidden in [
        "ui-action-contract.json",
        "full UI automation",
        "grading",
        "creative assessment",
        "visible rendering correctness",
        "Save completion",
        "first-lesson completion",
    ] {
        assert!(
            !reason.contains(forbidden),
            "blocker reason must not claim or expose {forbidden:?}: {reason}"
        );
    }
    assert!(
        reason.contains("automation scenarios"),
        "blocker reason must name automation scenarios: {reason}"
    );
}

fn assert_limitations_preserve_non_claims(report: &serde_json::Value) {
    let limitations = report["limitations"]
        .as_array()
        .unwrap_or_else(|| panic!("report should expose limitations[]: {report}"))
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    for expected in [
        "does not prove full Alice UI automation",
        "does not perform creative assessment",
        "does not grade student worlds",
        "does not prove visible rendering correctness",
        "does not prove first-lesson completion",
    ] {
        assert!(
            limitations.contains(&expected),
            "missing explicit non-claim {expected:?}: {limitations:?}"
        );
    }
}
