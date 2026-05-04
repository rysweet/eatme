use std::path::Path;
use std::process::Command;

#[test]
fn assets_validate_rejects_bad_persona_reference_from_real_path() {
    let scenario_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/persona-root/scenarios/eatme/test_bad_persona.yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_eatme-cli"))
        .args(["assets", "validate", "--json", "--path"])
        .arg(&scenario_path)
        .output()
        .expect("run eatme-cli assets validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validation output is JSON");
    assert_eq!(report["passed"], false);

    let errors = report["errors"]
        .as_array()
        .expect("errors is an array")
        .iter()
        .filter_map(|error| error.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        errors.contains("missing instructor persona nonexistent-instructor"),
        "errors: {errors}"
    );
}
