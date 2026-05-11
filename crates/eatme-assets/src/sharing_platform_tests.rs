use super::*;

// --- Input factory helpers ---

fn input_all_ready() -> SharingPlatformInput {
    SharingPlatformInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: true,
        deps_reason: "All required dependencies available".into(),
    }
}

fn input_blocked_assets() -> SharingPlatformInput {
    SharingPlatformInput {
        assets_valid: false,
        asset_reason: "Asset validation failed: 2 errors".into(),
        deps_available: true,
        deps_reason: "All required dependencies available".into(),
    }
}

fn input_blocked_deps() -> SharingPlatformInput {
    SharingPlatformInput {
        assets_valid: true,
        asset_reason: "All 93 scenario assets passed validation".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
    }
}

fn input_both_blocked() -> SharingPlatformInput {
    SharingPlatformInput {
        assets_valid: false,
        asset_reason: "Asset validation failed: 2 errors".into(),
        deps_available: false,
        deps_reason: "Missing required tools: Xvfb, wmctrl".into(),
    }
}

// --- Schema and structure tests ---

#[test]
fn schema_version_is_sharing_platform_v1() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.schema_version, "eatme.assets/sharing-platform/v1");
}

#[test]
fn lesson_is_building_a_scene_first_world() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.lesson, "building-a-scene-first-world");
}

#[test]
fn always_produces_six_entries() {
    for input in [
        input_all_ready(),
        input_blocked_assets(),
        input_blocked_deps(),
        input_both_blocked(),
    ] {
        let report = check_sharing_platform_readiness(input);
        assert_eq!(report.entries.len(), 6);
    }
}

#[test]
fn entry_names_in_order() {
    let report = check_sharing_platform_readiness(input_all_ready());
    let names: Vec<&str> = report.entries.iter().map(|e| e.feature.as_str()).collect();
    assert_eq!(
        names,
        [
            "validate-assets",
            "check-dependencies",
            "export-a3w",
            "file-sharing",
            "web-sharing",
            "classroom-deploy",
        ]
    );
}

// --- depends_on field tests ---

#[test]
fn depends_on_root_entries_have_empty_dependencies() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert!(report.entries[0].depends_on.is_empty(), "validate-assets");
    assert!(
        report.entries[1].depends_on.is_empty(),
        "check-dependencies"
    );
}

#[test]
fn depends_on_export_a3w_depends_on_both_preconditions() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(
        report.entries[2].depends_on,
        vec!["validate-assets", "check-dependencies"]
    );
}

#[test]
fn depends_on_file_sharing_depends_on_export_a3w() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.entries[3].depends_on, vec!["export-a3w"]);
}

#[test]
fn depends_on_platform_blocked_have_no_dependencies() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert!(report.entries[4].depends_on.is_empty(), "web-sharing");
    assert!(report.entries[5].depends_on.is_empty(), "classroom-deploy");
}

#[test]
fn depends_on_preserved_when_blocked() {
    let report = check_sharing_platform_readiness(input_both_blocked());
    assert_eq!(
        report.entries[2].depends_on,
        vec!["validate-assets", "check-dependencies"],
        "export-a3w depends_on should list both even when blocked"
    );
    assert_eq!(
        report.entries[3].depends_on,
        vec!["export-a3w"],
        "file-sharing depends_on should list export-a3w even when blocked"
    );
}

// --- All ready scenario ---

#[test]
fn all_ready_report_passes() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert!(
        report.passed,
        "report should pass when evaluable features are ready"
    );
}

#[test]
fn all_ready_preconditions_are_ready() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.entries[0].status, FeatureReadiness::Ready);
    assert_eq!(report.entries[1].status, FeatureReadiness::Ready);
}

#[test]
fn all_ready_export_a3w_is_ready() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.entries[2].status, FeatureReadiness::Ready);
    assert_eq!(
        report.entries[2].reason,
        "All preconditions met for .a3w export"
    );
}

#[test]
fn all_ready_file_sharing_is_ready() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.entries[3].status, FeatureReadiness::Ready);
    assert!(
        report.entries[3].reason.contains("export-a3w"),
        "file-sharing reason should reference export-a3w: {}",
        report.entries[3].reason
    );
}

#[test]
fn all_ready_reasons_propagate() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(
        report.entries[0].reason,
        "All 93 scenario assets passed validation"
    );
    assert_eq!(
        report.entries[1].reason,
        "All required dependencies available"
    );
}

// --- Blocked assets scenario ---

#[test]
fn blocked_assets_report_fails() {
    let report = check_sharing_platform_readiness(input_blocked_assets());
    assert!(!report.passed, "report should fail when assets are invalid");
}

#[test]
fn blocked_assets_validate_assets_is_blocked() {
    let report = check_sharing_platform_readiness(input_blocked_assets());
    assert_eq!(report.entries[0].status, FeatureReadiness::Blocked);
    assert_eq!(
        report.entries[0].reason,
        "Asset validation failed: 2 errors"
    );
}

#[test]
fn blocked_assets_check_dependencies_is_ready() {
    let report = check_sharing_platform_readiness(input_blocked_assets());
    assert_eq!(report.entries[1].status, FeatureReadiness::Ready);
}

#[test]
fn blocked_assets_cascades_to_export_and_file_sharing() {
    let report = check_sharing_platform_readiness(input_blocked_assets());
    assert_eq!(report.entries[2].status, FeatureReadiness::Blocked);
    assert!(
        report.entries[2].reason.contains("validate-assets"),
        "export-a3w reason should mention the blocking step: {}",
        report.entries[2].reason
    );
    assert_eq!(report.entries[3].status, FeatureReadiness::Blocked);
    assert!(
        report.entries[3].reason.contains("export-a3w"),
        "file-sharing reason should mention export-a3w: {}",
        report.entries[3].reason
    );
}

// --- Blocked dependencies scenario ---

#[test]
fn blocked_deps_report_fails() {
    let report = check_sharing_platform_readiness(input_blocked_deps());
    assert!(
        !report.passed,
        "report should fail when deps are unavailable"
    );
}

#[test]
fn blocked_deps_validate_assets_is_ready() {
    let report = check_sharing_platform_readiness(input_blocked_deps());
    assert_eq!(report.entries[0].status, FeatureReadiness::Ready);
}

#[test]
fn blocked_deps_check_dependencies_is_blocked() {
    let report = check_sharing_platform_readiness(input_blocked_deps());
    assert_eq!(report.entries[1].status, FeatureReadiness::Blocked);
    assert_eq!(
        report.entries[1].reason,
        "Missing required tools: Xvfb, wmctrl"
    );
}

#[test]
fn blocked_deps_cascades_to_export_and_file_sharing() {
    let report = check_sharing_platform_readiness(input_blocked_deps());
    assert_eq!(report.entries[2].status, FeatureReadiness::Blocked);
    assert!(
        report.entries[2].reason.contains("check-dependencies"),
        "export-a3w reason should mention the blocking step: {}",
        report.entries[2].reason
    );
    assert_eq!(report.entries[3].status, FeatureReadiness::Blocked);
}

// --- Both blocked scenario ---

#[test]
fn both_blocked_report_fails() {
    let report = check_sharing_platform_readiness(input_both_blocked());
    assert!(!report.passed, "report should fail when both are blocked");
}

#[test]
fn both_blocked_preconditions_are_blocked() {
    let report = check_sharing_platform_readiness(input_both_blocked());
    assert_eq!(report.entries[0].status, FeatureReadiness::Blocked);
    assert_eq!(report.entries[1].status, FeatureReadiness::Blocked);
}

#[test]
fn both_blocked_export_a3w_mentions_both_blockers() {
    let report = check_sharing_platform_readiness(input_both_blocked());
    let reason = &report.entries[2].reason;
    assert!(
        reason.contains("validate-assets") && reason.contains("check-dependencies"),
        "export-a3w should mention both blocking steps: {reason}"
    );
}

#[test]
fn both_blocked_file_sharing_is_blocked() {
    let report = check_sharing_platform_readiness(input_both_blocked());
    assert_eq!(report.entries[3].status, FeatureReadiness::Blocked);
}

// --- Platform-blocked features are constant across all inputs ---

#[test]
fn platform_blocked_features_constant_across_all_inputs() {
    for input in [
        input_all_ready(),
        input_blocked_assets(),
        input_blocked_deps(),
        input_both_blocked(),
    ] {
        let report = check_sharing_platform_readiness(input);
        assert_eq!(
            report.entries[4].status,
            FeatureReadiness::PlatformBlocked,
            "web-sharing"
        );
        assert_eq!(
            report.entries[5].status,
            FeatureReadiness::PlatformBlocked,
            "classroom-deploy"
        );
    }
}

#[test]
fn platform_blocked_reasons_are_descriptive() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert!(
        report.entries[4].reason.to_lowercase().contains("web"),
        "web-sharing reason should mention web: {}",
        report.entries[4].reason
    );
    assert!(
        report.entries[5]
            .reason
            .to_lowercase()
            .contains("classroom"),
        "classroom-deploy reason should mention classroom: {}",
        report.entries[5].reason
    );
}

// --- Pass logic ---

#[test]
fn passed_true_only_when_export_and_file_sharing_ready() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert!(report.passed);
    assert_eq!(report.entries[2].status, FeatureReadiness::Ready);
    assert_eq!(report.entries[3].status, FeatureReadiness::Ready);
}

#[test]
fn passed_false_when_any_precondition_blocked() {
    assert!(!check_sharing_platform_readiness(input_blocked_assets()).passed);
    assert!(!check_sharing_platform_readiness(input_blocked_deps()).passed);
    assert!(!check_sharing_platform_readiness(input_both_blocked()).passed);
}

#[test]
fn passed_ignores_platform_blocked_features() {
    let report = check_sharing_platform_readiness(input_all_ready());
    assert_eq!(report.entries[4].status, FeatureReadiness::PlatformBlocked);
    assert_eq!(report.entries[5].status, FeatureReadiness::PlatformBlocked);
    assert!(
        report.passed,
        "platform-blocked features should not affect passed"
    );
}

// --- JSON serialization ---

#[test]
fn feature_readiness_serializes_as_lowercase() {
    assert_eq!(
        serde_json::to_string(&FeatureReadiness::Ready).unwrap(),
        "\"ready\""
    );
    assert_eq!(
        serde_json::to_string(&FeatureReadiness::Blocked).unwrap(),
        "\"blocked\""
    );
    assert_eq!(
        serde_json::to_string(&FeatureReadiness::PlatformBlocked).unwrap(),
        "\"platform-blocked\""
    );
}

#[test]
fn report_serializes_to_expected_json_shape() {
    let report = check_sharing_platform_readiness(input_all_ready());
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["schema_version"], "eatme.assets/sharing-platform/v1");
    assert_eq!(json["lesson"], "building-a-scene-first-world");
    assert!(json["passed"].as_bool().unwrap());
    assert!(json["entries"].is_array());
    assert_eq!(json["entries"].as_array().unwrap().len(), 6);

    let first = &json["entries"][0];
    assert!(
        first.get("feature").is_some(),
        "entry should have 'feature' field"
    );
    assert!(
        first.get("status").is_some(),
        "entry should have 'status' field"
    );
    assert!(
        first.get("depends_on").is_some(),
        "entry should have 'depends_on' field"
    );
    assert!(
        first.get("reason").is_some(),
        "entry should have 'reason' field"
    );
}

#[test]
fn json_ready_scenario_entries() {
    let report = check_sharing_platform_readiness(input_all_ready());
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["entries"][0]["feature"], "validate-assets");
    assert_eq!(json["entries"][0]["status"], "ready");
    assert_eq!(
        json["entries"][0]["depends_on"].as_array().unwrap().len(),
        0
    );
    assert_eq!(json["entries"][1]["feature"], "check-dependencies");
    assert_eq!(json["entries"][1]["status"], "ready");
    assert_eq!(json["entries"][2]["feature"], "export-a3w");
    assert_eq!(json["entries"][2]["status"], "ready");
    assert_eq!(json["entries"][2]["depends_on"][0], "validate-assets");
    assert_eq!(json["entries"][2]["depends_on"][1], "check-dependencies");
}

#[test]
fn json_platform_blocked_entries() {
    let report = check_sharing_platform_readiness(input_all_ready());
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert_eq!(json["entries"][4]["feature"], "web-sharing");
    assert_eq!(json["entries"][4]["status"], "platform-blocked");
    assert_eq!(
        json["entries"][4]["depends_on"].as_array().unwrap().len(),
        0
    );
    assert_eq!(json["entries"][5]["feature"], "classroom-deploy");
    assert_eq!(json["entries"][5]["status"], "platform-blocked");
}

#[test]
fn json_blocked_scenario_shows_blocked_status() {
    let report = check_sharing_platform_readiness(input_blocked_assets());
    let json: serde_json::Value = serde_json::to_value(&report).unwrap();

    assert!(!json["passed"].as_bool().unwrap());
    assert_eq!(json["entries"][0]["status"], "blocked");
    assert_eq!(json["entries"][2]["status"], "blocked");
    assert_eq!(json["entries"][3]["status"], "blocked");
    assert_eq!(json["entries"][4]["status"], "platform-blocked");
    assert_eq!(json["entries"][5]["status"], "platform-blocked");
}
