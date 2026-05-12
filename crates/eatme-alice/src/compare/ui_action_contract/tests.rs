use super::*;

#[test]
fn requires_run_world_no_go_after_edit_proof() {
    let mut issues = Vec::new();

    inspect_ui_action_contract(
        "modernized",
        &contract_after_edit_without_run_no_go(),
        &mut issues,
    );

    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("passed run-world proof or a no-go precondition")),
        "issues should name missing run-world proof boundary: {issues:?}"
    );
}

#[test]
fn requires_save_project_no_go_after_run_world_proof() {
    let mut issues = Vec::new();

    inspect_ui_action_contract(
        "modernized",
        &contract_after_run_without_save_no_go(),
        &mut issues,
    );

    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("passed save-project proof or a no-go precondition")),
        "issues should name missing save-project proof boundary: {issues:?}"
    );
}

#[test]
fn rejects_save_project_candidate_with_validation_errors_as_unproven() {
    let mut issues = Vec::new();
    let mut contract = contract_after_run_without_save_no_go();
    contract["candidate_affordance_probes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "alice-side-project-save-command-hook",
            "action_id": "save-project",
            "status": "passed",
            "save_selector": "scene.eatmeFirstLessonStep",
            "candidate_hook_path": "/alice/tools/eatme-save-project",
            "saved_project_artifact": {"path": "project-save/saved-project.a3p", "size_bytes": 2},
            "save_artifact": {"path": "project-save/project-save.json", "size_bytes": 2},
            "validation_errors": ["save artifact did not validate"]
        }));

    inspect_ui_action_contract("modernized", &contract, &mut issues);

    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("passed save-project proof or a no-go precondition")),
        "save-project candidate with validation errors must not satisfy proof: {issues:?}"
    );
}

#[test]
fn accepts_recorded_failed_run_window_observation_after_shortcut_dispatch() {
    let mut issues = Vec::new();
    let mut contract = contract_after_edit_without_run_no_go();
    contract["executed_action_probes"][3]["status"] = serde_json::json!("failed");

    inspect_ui_action_contract("modernized", &contract, &mut issues);

    assert!(
        !issues
            .iter()
            .any(|issue| issue.contains("observe-run-window-after-shortcut probe")),
        "recorded failed observation should be accepted as an honest desktop-result boundary: {issues:?}"
    );
}

#[test]
fn rejects_missing_original_alice_action_evidence_with_structured_blocker() {
    let mut issues = Vec::new();
    let mut contract = contract_after_edit_without_run_no_go();
    remove_executed_action_probe(&mut contract, "dispatch-save-project-shortcut");

    inspect_ui_action_contract("original Alice", &contract, &mut issues);

    assert_issue_contains(&issues, "code=missing_real_action_evidence");
    assert_issue_contains(
        &issues,
        "action=original_alice.dispatch-save-project-shortcut",
    );
    assert_issue_contains(&issues, "reason=dispatch-save-project-shortcut-missing");
    assert_issue_contains(&issues, "automation scenarios");
    assert_issue_contains(&issues, "Original Alice");
    assert_issues_avoid_unsupported_claims(&issues);
}

#[test]
fn rejects_manifest_only_required_original_alice_action_evidence() {
    let mut issues = Vec::new();
    let mut contract = contract_after_edit_without_run_no_go();
    contract["required_actions"] = serde_json::json!([{
        "id": "verify-specific-alice-window",
        "required_evidence": "wmctrl or xwininfo output identifies the Alice main window"
    }]);

    inspect_ui_action_contract("original Alice", &contract, &mut issues);

    assert_issue_contains(&issues, "code=missing_real_action_evidence");
    assert_issue_contains(
        &issues,
        "action=original_alice.verify-specific-alice-window",
    );
    assert_issue_contains(&issues, "reason=required-action-probe-missing");
    assert_issue_contains(&issues, "automation scenarios");
    assert_issues_avoid_unsupported_claims(&issues);
}

fn contract_after_edit_without_run_no_go() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "eatme.ui-action-contract/v1",
        "status": "blocked",
        "blocking_reason": "world run remains blocked",
        "preflight_evidence": {
            "specific_alice_window_detected": true,
            "visual_evidence_captured": true,
            "log_captured": true
        },
        "executed_action_probes": [{
            "id": "activate-specific-alice-window",
            "status": "passed"
        }, {
            "id": "dispatch-save-project-shortcut",
            "status": "passed"
        }, {
            "id": "dispatch-run-world-shortcut",
            "status": "passed"
        }, {
            "id": "observe-run-window-after-shortcut",
            "status": "passed"
        }],
        "candidate_affordance_probes": [
            {
                "id": "alice-side-object-placement-command-hook",
                "action_id": "place-object",
                "status": "passed",
                "object_identifier": "alice-gallery://animals/bunny",
                "candidate_hook_path": "/alice/tools/eatme-place-object",
                "placement_artifact": {"path": "object-placement/placed-project.a3p", "size_bytes": 2},
                "scene_or_project_diff": {"path": "object-placement/scene.diff.json", "size_bytes": 2}
            },
            {
                "id": "alice-side-procedure-edit-command-hook",
                "action_id": "edit-procedure-or-code-block",
                "status": "passed",
                "procedure_selector": "scene.eatmeFirstLesson",
                "candidate_hook_path": "/alice/tools/eatme-edit-procedure",
                "edited_project_artifact": {"path": "procedure-edit/edited-project.a3p", "size_bytes": 2},
                "procedure_or_code_diff": {"path": "procedure-edit/procedure.diff.json", "size_bytes": 2}
            }
        ],
        "required_actions": []
    })
}

fn contract_after_run_without_save_no_go() -> serde_json::Value {
    let mut contract = contract_after_edit_without_run_no_go();
    contract["candidate_affordance_probes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "alice-side-world-run-command-hook",
            "action_id": "run-world",
            "status": "passed",
            "run_selector": "scene.eatmeFirstLessonStep",
            "candidate_hook_path": "/alice/tools/eatme-run-world",
            "run_artifact": {"path": "world-run/world-run.json", "size_bytes": 2},
            "runtime_or_log_evidence": {"path": "world-run/runtime.log", "size_bytes": 2}
        }));
    contract
}

fn remove_executed_action_probe(contract: &mut serde_json::Value, probe_id: &str) {
    contract["executed_action_probes"]
        .as_array_mut()
        .unwrap()
        .retain(|probe| probe.get("id").and_then(serde_json::Value::as_str) != Some(probe_id));
}

fn assert_issue_contains(issues: &[String], expected: &str) {
    assert!(
        issues.iter().any(|issue| issue.contains(expected)),
        "issues should contain {expected:?}: {issues:?}"
    );
}

#[test]
fn accepts_save_project_no_go_probe_after_run_world_proof() {
    let mut issues = Vec::new();
    let mut contract = contract_after_run_without_save_no_go();
    contract["action_precondition_probes"] = serde_json::json!([{
        "id": "project-save-precondition",
        "action_id": "save-project",
        "status": "blocked",
        "decision": "no_go",
        "missing_affordance": {
            "id": "deterministic-alice-project-save-affordance",
            "kind": "backend_or_ui_affordance",
            "required_capability": "Given an edited Alice project, save the project and return proof that the saved .a3p is readable",
            "missing_contract": "No Alice-side command at tools/eatme-save-project currently returns project-save proof",
            "next_implementation": "Add save-project command hook with named save control"
        },
        "preconditions": [
            {"id": "run-world", "passed": true},
            {"id": "deterministic-alice-project-save-affordance", "passed": false}
        ]
    }]);

    inspect_ui_action_contract("modernized", &contract, &mut issues);

    assert!(
        !issues
            .iter()
            .any(|issue| issue.contains("save-project proof or a no-go precondition")),
        "save-project no-go after run-world should satisfy the inspector: {issues:?}"
    );
}

fn assert_issues_avoid_unsupported_claims(issues: &[String]) {
    let joined = issues.join("\n").to_ascii_lowercase();
    for unsupported_claim in [
        "full alice ui automation",
        "grading",
        "creative assessment",
        "visible rendering correctness",
        "save completion",
        "first-lesson completion",
        "lesson completed",
    ] {
        assert!(
            !joined.contains(unsupported_claim),
            "issue text must not claim {unsupported_claim:?}: {issues:?}"
        );
    }
}
