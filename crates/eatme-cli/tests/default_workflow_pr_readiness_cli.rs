use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const HEAD_SHA: &str = "fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d";

#[test]
fn renders_merge_ready_with_exact_head_green_evidence_and_no_repository_changes() {
    let root = scratch_root("merge-ready-no-op");
    let evidence_path = write_evidence(&root, base_merge_ready_evidence());

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 0);
    let report = readiness_json(&output);
    assert_eq!(report["decision"], "MERGE_READY");
    assert_eq!(report["head_ref_oid"], HEAD_SHA);
    assert_eq!(report["local_head"], HEAD_SHA);
    assert_eq!(report["files_modified"], json!([]));
    assert!(
        report["no_op_justification"]
            .as_str()
            .unwrap()
            .contains(HEAD_SHA)
    );
    assert!(report["blockers"].as_array().unwrap().is_empty());
}

#[test]
fn head_alignment_gate_blocks_local_sha_mismatches_and_manual_merge_evidence() {
    let root = scratch_root("head-alignment-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["local"]["head"] = json!("1111111111111111111111111111111111111111");
    evidence["local"]["checkout_mode"] = json!("merge");
    evidence["local"]["manual_merge_performed"] = json!(true);
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_eq!(report["decision"], "NOT_MERGE_READY");
    assert_blocker(&report, "head_alignment", "head_mismatch");
    assert_blocker(&report, "head_alignment", "manual_merge_detected");
}

#[test]
fn pr_metadata_auditor_blocks_draft_dirty_closed_or_unknown_mergeability() {
    let root = scratch_root("metadata-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["pr"]["state"] = json!("CLOSED");
    evidence["pr"]["is_draft"] = json!(true);
    evidence["pr"]["mergeable"] = json!("UNKNOWN");
    evidence["pr"]["merge_state_status"] = json!("DIRTY");
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "pr_metadata", "pr_not_open");
    assert_blocker(&report, "pr_metadata", "pr_is_draft");
    assert_blocker(&report, "pr_metadata", "pr_not_mergeable");
    assert_blocker(&report, "pr_metadata", "merge_state_not_clean");
}

#[test]
fn github_actions_auditor_blocks_stale_pending_failing_skipped_or_missing_checks() {
    let root = scratch_root("actions-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["checks"] = json!([
        {
            "name": "quality-gates",
            "head_sha": HEAD_SHA,
            "status": "COMPLETED",
            "conclusion": "SUCCESS",
            "required": true
        },
        {
            "name": "docs",
            "head_sha": "2222222222222222222222222222222222222222",
            "status": "COMPLETED",
            "conclusion": "SUCCESS",
            "required": true
        },
        {
            "name": "assets",
            "head_sha": HEAD_SHA,
            "status": "IN_PROGRESS",
            "conclusion": null,
            "required": true
        },
        {
            "name": "scenario",
            "head_sha": HEAD_SHA,
            "status": "COMPLETED",
            "conclusion": "FAILURE",
            "required": true
        },
        {
            "name": "gadugi",
            "head_sha": HEAD_SHA,
            "status": "COMPLETED",
            "conclusion": "SKIPPED",
            "required": true
        }
    ]);
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "github_actions", "stale_check_sha");
    assert_blocker(&report, "github_actions", "check_not_complete");
    assert_blocker(&report, "github_actions", "check_not_successful");
    assert_blocker(&report, "github_actions", "required_check_skipped");
}

#[test]
fn runnable_evidence_runner_requires_all_commands_without_timeout_wrappers() {
    let root = scratch_root("runnable-evidence-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["commands"] = json!([
        command_evidence(
            "quality_gates",
            "timeout 600 TMPDIR=/tmp ./scripts/quality-gates.sh",
            0,
            true
        ),
        command_evidence(
            "assets_validate",
            "cargo run -q -p eatme-cli -- assets validate --json",
            0,
            false
        ),
        command_evidence(
            "gadugi_check",
            "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
            1,
            false
        )
    ]);
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "runnable_evidence", "timeout_wrapper_used");
    assert_blocker(&report, "runnable_evidence", "evidence_command_failed");
    assert_blocker(&report, "runnable_evidence", "missing_docs_build");
}

#[test]
fn quality_audit_recorder_requires_three_cycles_with_clean_final_cycle() {
    let root = scratch_root("quality-audit-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["audit_cycles"] = json!([
        audit_cycle(
            "cycle 1",
            "checks",
            "validated stale checks",
            "fixed PR body",
            true
        ),
        audit_cycle(
            "cycle 2",
            "docs",
            "validated docs impact",
            "blocked on missing evidence",
            false
        )
    ]);
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "quality_audit", "insufficient_audit_cycles");
    assert_blocker(&report, "quality_audit", "final_audit_cycle_not_clean");
}

#[test]
fn focused_diff_auditor_blocks_unrelated_churn_and_accidental_generated_artifacts() {
    let root = scratch_root("diff-scope-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["diff"] = json!({
        "files": [
            "docs/default-workflow-pr-readiness.md",
            "target/debug/build/stale-generated.rs",
            "assets/scenarios/gadugi/unrelated.yaml"
        ],
        "focused": false,
        "unrelated_churn": ["assets/scenarios/gadugi/unrelated.yaml"],
        "generated_artifacts": ["target/debug/build/stale-generated.rs"]
    });
    evidence["local"]["repository_changes"] = json!([
        "docs/default-workflow-pr-readiness.md",
        "target/debug/build/stale-generated.rs",
        "assets/scenarios/gadugi/unrelated.yaml"
    ]);
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "diff_scope", "diff_not_focused");
    assert_blocker(&report, "diff_scope", "unrelated_churn");
    assert_blocker(&report, "diff_scope", "generated_artifact_in_diff");
    assert_eq!(
        report["files_modified"],
        json!([
            "docs/default-workflow-pr-readiness.md",
            "target/debug/build/stale-generated.rs",
            "assets/scenarios/gadugi/unrelated.yaml"
        ])
    );
}

#[test]
fn docs_impact_auditor_requires_review_updated_or_ruled_out_docs_and_strict_build() {
    let root = scratch_root("docs-impact-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["docs"] = json!({
        "impact_reviewed": false,
        "updated_or_ruled_out": false,
        "strict_build_passed": false,
        "strict_build_command": "mkdocs build --strict"
    });
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "docs_impact", "docs_impact_not_reviewed");
    assert_blocker(
        &report,
        "docs_impact",
        "docs_update_not_proven_or_ruled_out",
    );
    assert_blocker(
        &report,
        "docs_impact",
        "docs_strict_build_missing_or_failed",
    );
}

#[test]
fn pr_description_auditor_requires_current_bounded_evidence_and_blocks_overclaims() {
    let root = scratch_root("pr-description-blockers");
    let mut evidence = base_merge_ready_evidence();
    evidence["pr_description_evidence"] = json!({
        "head_ref_oid": "3333333333333333333333333333333333333333",
        "mentions_green_actions": true,
        "mentions_runnable_qa": false,
        "mentions_docs_impact": false,
        "mentions_quality_audit_cycles": false,
        "unsupported_claims": [
            "full UI automation",
            "visible rendering correctness",
            "grading correctness"
        ]
    });
    let evidence_path = write_evidence(&root, evidence);

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 1);
    let report = readiness_json(&output);
    assert_blocker(&report, "pr_description", "stale_pr_description_head");
    assert_blocker(&report, "pr_description", "missing_runnable_qa_evidence");
    assert_blocker(&report, "pr_description", "missing_docs_impact_evidence");
    assert_blocker(&report, "pr_description", "missing_quality_audit_evidence");
    assert_blocker(&report, "pr_description", "unsupported_readiness_claim");
}

#[test]
fn malformed_evidence_returns_structured_input_error_instead_of_ready_decision() {
    let root = scratch_root("malformed-evidence");
    let evidence_path = root.join("evidence.json");
    fs::write(&evidence_path, "{not valid json").unwrap();

    let output = run_readiness(&evidence_path);

    assert_exit_code(&output, 2);
    let report = readiness_json(&output);
    assert_eq!(report["decision"], "NOT_MERGE_READY");
    assert_blocker(&report, "input", "malformed_evidence");
}

fn base_merge_ready_evidence() -> Value {
    json!({
        "schema_version": "eatme.default-workflow-pr-readiness-evidence/v1",
        "pr": {
            "number": 203,
            "state": "OPEN",
            "is_draft": false,
            "mergeable": "MERGEABLE",
            "merge_state_status": "CLEAN",
            "head_ref_oid": HEAD_SHA,
            "head_ref_name": "feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul",
            "body": "Exact head fe0fcb4c5d4c73fa4022f774857d75ebb2624c6d has green Actions, runnable QA, docs impact, and three quality-audit cycles."
        },
        "local": {
            "head": HEAD_SHA,
            "checkout_mode": "detached",
            "manual_merge_performed": false,
            "repository_changes": []
        },
        "checks": [
            {
                "name": "quality-gates",
                "head_sha": HEAD_SHA,
                "status": "COMPLETED",
                "conclusion": "SUCCESS",
                "required": true
            },
            {
                "name": "docs",
                "head_sha": HEAD_SHA,
                "status": "COMPLETED",
                "conclusion": "SUCCESS",
                "required": true
            },
            {
                "name": "manual real Alice launch smoke",
                "head_sha": HEAD_SHA,
                "status": "COMPLETED",
                "conclusion": "SKIPPED",
                "required": false
            }
        ],
        "commands": [
            command_evidence(
                "quality_gates",
                "TMPDIR=/tmp ./scripts/quality-gates.sh",
                0,
                false
            ),
            command_evidence(
                "assets_validate",
                "cargo run -q -p eatme-cli -- assets validate --json",
                0,
                false
            ),
            command_evidence(
                "gadugi_check",
                "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
                0,
                false
            ),
            command_evidence("docs_build", "mkdocs build --strict", 0, false)
        ],
        "audit_cycles": [
            audit_cycle("cycle 1", "metadata", "validated exact head", "no fix required", true),
            audit_cycle("cycle 2", "runnable evidence", "validated commands", "no fix required", true),
            audit_cycle("cycle 3", "final", "validated no remaining blockers", "no fix required", true)
        ],
        "diff": {
            "files": ["docs/default-workflow-pr-readiness.md"],
            "focused": true,
            "unrelated_churn": [],
            "generated_artifacts": []
        },
        "docs": {
            "impact_reviewed": true,
            "updated_or_ruled_out": true,
            "strict_build_passed": true,
            "strict_build_command": "mkdocs build --strict"
        },
        "pr_description_evidence": {
            "head_ref_oid": HEAD_SHA,
            "mentions_green_actions": true,
            "mentions_runnable_qa": true,
            "mentions_docs_impact": true,
            "mentions_quality_audit_cycles": true,
            "unsupported_claims": []
        }
    })
}

fn command_evidence(
    id: &str,
    command: &str,
    exit_status: i32,
    used_timeout_wrapper: bool,
) -> Value {
    json!({
        "id": id,
        "command": command,
        "exit_status": exit_status,
        "used_timeout_wrapper": used_timeout_wrapper
    })
}

fn audit_cycle(name: &str, seek: &str, validate: &str, fix: &str, clean: bool) -> Value {
    json!({
        "name": name,
        "seek": seek,
        "validate": validate,
        "fix": fix,
        "clean": clean
    })
}

fn run_readiness(evidence_path: &Path) -> Output {
    Command::new(eatme_bin())
        .args([
            "default-workflow",
            "pr-readiness",
            "--evidence",
            evidence_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap()
}

fn write_evidence(root: &Path, evidence: Value) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    let evidence_path = root.join("evidence.json");
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    evidence_path
}

fn readiness_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "expected readiness JSON on stdout: {error}; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_blocker(report: &Value, gate: &str, code: &str) {
    let blockers = report["blockers"]
        .as_array()
        .unwrap_or_else(|| panic!("expected blockers[] in report: {report}"));
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker["gate"] == gate && blocker["code"] == code),
        "missing blocker {gate}/{code}: {report}"
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

fn assert_exit_code(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "unexpected status {:?}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
