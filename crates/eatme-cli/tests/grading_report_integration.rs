use eatme_assets::{GradingReport, StepStatus};
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn grading_report_cli_emits_json_contract_even_when_interactive_steps_remain() {
    let output = Command::new(eatme_bin())
        .args(["assets", "grading-report", "--json", "--path"])
        .arg(workspace_root())
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets grading-report --json");

    assert_eq!(
        output.status.code(),
        Some(1),
        "unexpected status {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: GradingReport =
        serde_json::from_slice(&output.stdout).expect("grading report should be JSON");

    assert_eq!(report.schema_version, "eatme.assets/grading/v1");
    assert_eq!(report.lesson, "building-a-scene-first-world");
    assert!(!report.passed);
    assert_eq!(report.steps.len(), 6);
    assert_eq!(report.steps[0].name, "validate-assets");
    assert_eq!(report.steps[1].name, "check-dependencies");
    assert_eq!(report.steps[2].name, "launch-smoke");
    assert!(
        report.steps[3..]
            .iter()
            .all(|step| matches!(step.status, StepStatus::NotYetTested | StepStatus::Blocked))
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("not all steps ready"));
}

#[test]
fn grading_report_cli_matches_validate_and_dependency_preconditions() {
    let validate = Command::new(eatme_bin())
        .args(["assets", "validate", "--json"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets validate --json");
    assert_eq!(validate.status.code(), Some(0));
    let validate_json: serde_json::Value =
        serde_json::from_slice(&validate.stdout).expect("asset validation report is JSON");

    let deps = Command::new(eatme_bin())
        .args(["deps", "check", "--json"])
        .current_dir(workspace_root())
        .output()
        .expect("run eatme deps check --json");
    assert_eq!(deps.status.code(), Some(0));
    let deps_json: serde_json::Value =
        serde_json::from_slice(&deps.stdout).expect("dependency report is JSON");

    let grading = Command::new(eatme_bin())
        .args(["assets", "grading-report", "--json", "--path"])
        .arg(workspace_root())
        .current_dir(workspace_root())
        .output()
        .expect("run eatme assets grading-report --json --path");
    let report: GradingReport =
        serde_json::from_slice(&grading.stdout).expect("grading report should be JSON");

    let assets_valid = validate_json["passed"]
        .as_bool()
        .expect("validate passed bool");
    let deps_valid = deps_json["all_required_available"]
        .as_bool()
        .expect("dependency readiness bool");

    assert_eq!(
        report.steps[0].status,
        if assets_valid {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        }
    );
    assert_eq!(
        report.steps[1].status,
        if deps_valid {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        }
    );
    assert_eq!(
        report.steps[2].status,
        if assets_valid && deps_valid {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        }
    );
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
