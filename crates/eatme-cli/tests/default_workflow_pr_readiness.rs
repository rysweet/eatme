use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HEAD_SHA: &str = "5ab1cca881959b3aac063af7c5973e7f75c35c46";
const OLD_SHA: &str = "1111111111111111111111111111111111111111";

#[test]
fn finalize_pr_readiness_outputs_owner_free_no_op_for_clean_matching_current_head() {
    let root = scratch_root("clean-no-op");
    let evidence_path = root.join("evidence.json");
    fs::write(&evidence_path, clean_evidence_json(HEAD_SHA)).unwrap();

    let output = Command::new(eatme_bin())
        .args(["pr-readiness", "finalize", "--pr", "173", "--evidence"])
        .arg(&evidence_path)
        .args(["--json", "--dry-run"])
        .current_dir(workspace_root())
        .output()
        .unwrap();

    assert_exit_code(&output, 0);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("readiness output is JSON");
    assert_eq!(report["decision"], "MERGE_READY");
    assert_eq!(report["no_op"], true);
    assert_contains_json_text(&report, "No-op justification");
    assert_contains_json_text(&report, HEAD_SHA);
    assert_contains_json_text(&report, "classroom review handoff readiness");
    assert_contains_json_text(&report, "no repository edits or commits were required");
    assert_contains_json_text(&report, "does not claim");
    assert_contains_json_text(&report, "deployed sharing");
    assert_contains_json_text(&report, "production readiness");
    assert_contains_json_text(&report, "merge completion");
    assert_json_text_excludes(&report, "deployed sharing readiness");
}

#[test]
fn finalize_pr_readiness_rejects_final_head_drift_without_no_op_wording() {
    let root = scratch_root("head-drift");
    let evidence_path = root.join("evidence.json");
    fs::write(&evidence_path, clean_evidence_json(OLD_SHA)).unwrap();

    let output = Command::new(eatme_bin())
        .args(["pr-readiness", "finalize", "--pr", "173", "--evidence"])
        .arg(&evidence_path)
        .args(["--json", "--dry-run"])
        .current_dir(workspace_root())
        .output()
        .unwrap();

    assert_exit_code(&output, 1);
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("rejection output is JSON");
    assert_eq!(report["decision"], "NOT_MERGE_READY");
    assert_eq!(report["no_op"], false);
    assert_contains_json_text(&report, "final PR head");
    assert_contains_json_text(&report, OLD_SHA);
    assert_json_text_excludes(&report, "No-op justification");
}

fn clean_evidence_json(final_pr_head_sha: &str) -> String {
    format!(
        r#"{{
            "repository": "rysweet/eatme",
            "pr_number": 173,
            "head_ref_name": "wave6-deployed-sharing-gap-1778302300",
            "pr_head_sha": "{HEAD_SHA}",
            "state": "OPEN",
            "draft": false,
            "local_branch": "wave6-deployed-sharing-gap-1778302300",
            "local_head_sha": "{HEAD_SHA}",
            "final_pr_head_sha": "{final_pr_head_sha}",
            "worktree_clean": true,
            "merge_state_status": "CLEAN",
            "mergeable": "MERGEABLE",
            "checks": [
                {{
                    "name": "quality-gates",
                    "head_sha": "{HEAD_SHA}",
                    "conclusion": "SUCCESS",
                    "required": true,
                    "workflow_name": "CI",
                    "details_url": "https://github.com/rysweet/eatme/actions/runs/quality-gates"
                }},
                {{
                    "name": "mkdocs",
                    "head_sha": "{HEAD_SHA}",
                    "conclusion": "SUCCESS",
                    "required": true,
                    "workflow_name": "CI",
                    "details_url": "https://github.com/rysweet/eatme/actions/runs/mkdocs"
                }}
            ],
            "validated_gates": ["mkdocs build --strict"],
            "changed_files": ["docs/default-workflow-pr-readiness.md"],
            "claim_boundary": "classroom review handoff readiness only",
            "quality_audit_cycles": [
                {{
                    "seek": "scope and claim accuracy",
                    "validate": "reviewed readiness docs and PR metadata",
                    "fix": "no repository change required"
                }},
                {{
                    "seek": "canonical and generated asset consistency",
                    "validate": "GitHub checks current for head",
                    "fix": "no repository change required"
                }},
                {{
                    "seek": "gate completeness and final readiness",
                    "validate": "final PR head re-check matched",
                    "fix": "no repository change required"
                }}
            ]
        }}"#
    )
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
    let mut path = workspace_root().join("target/debug");
    path.push("eatme-cli");
    path
}

fn assert_exit_code(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_contains_json_text(value: &serde_json::Value, needle: &str) {
    let text = value.to_string();
    assert!(text.contains(needle), "{text} did not contain {needle}");
}

fn assert_json_text_excludes(value: &serde_json::Value, needle: &str) {
    let text = value.to_string();
    assert!(
        !text.contains(needle),
        "{text} unexpectedly contained {needle}"
    );
}
