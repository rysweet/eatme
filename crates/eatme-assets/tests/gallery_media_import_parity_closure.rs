use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_text(relative_path: &str) -> String {
    let path = repository_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn read_yaml(relative_path: &str) -> Value {
    serde_yaml::from_str(&read_text(relative_path))
        .unwrap_or_else(|error| panic!("failed to parse {relative_path}: {error}"))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn string_at(value: &Value, path: &[&str]) -> String {
    value_at(value, path)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn strings_at(value: &Value, path: &[&str]) -> Vec<String> {
    value_at(value, path)
        .and_then(Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn matrix_row(scenario: &str) -> Value {
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    value_at(&matrix, &["rows"])
        .and_then(Value::as_sequence)
        .and_then(|rows| {
            rows.iter()
                .find(|row| string_at(row, &["scenario"]) == scenario)
                .cloned()
        })
        .unwrap_or_else(|| panic!("missing matrix row for {scenario}"))
}

fn matrix_rows() -> Vec<Value> {
    let matrix = read_yaml("assets/parity/rabbithole-lookingglass-journey-matrix.yaml");
    value_at(&matrix, &["rows"])
        .and_then(Value::as_sequence)
        .unwrap_or_else(|| panic!("matrix rows must be a YAML sequence"))
        .to_vec()
}

fn lookingglass_test_refs(row: &Value) -> Vec<String> {
    strings_at(row, &["closure", "required"])
        .join("\n")
        .split_whitespace()
        .filter_map(|token| token.strip_prefix("LookingGlass:test/"))
        .map(|path| {
            let trimmed = path.trim_end_matches(['.', ',', ';', ':', ')', ']']);
            format!("test/{trimmed}")
        })
        .collect()
}

#[test]
fn model_texture_import_checkpoint_is_covered_by_lookingglass_main_evidence() {
    let row = matrix_row("model-texture-import-checkpoint");
    let closure = strings_at(&row, &["closure", "required"]).join("\n");
    let expected_refs = [
        "LookingGlass:test/model-texture-import-checkpoint-closure.contract.test.ts",
        "LookingGlass:test/imported-project-assets-security.contract.test.ts",
        "LookingGlass:test/model-texture-camera-joint-export-workflow.contract.test.ts",
        "LookingGlass:test/imported-asset-project-io.test.ts",
    ];

    assert_eq!(
        string_at(&row, &["looking_glass", "status"]),
        "covered",
        "model/texture import is covered by cited LookingGlass main tests"
    );
    assert!(
        string_at(&row, &["looking_glass", "source_status"])
            .contains("LookingGlass main contains imported model"),
        "covered model/texture row must cite LookingGlass main evidence"
    );
    for expected_ref in expected_refs {
        assert!(
            closure.contains(expected_ref),
            "model-texture-import-checkpoint closure must cite {expected_ref}; closure was:\n{closure}"
        );
    }
}

#[test]
fn audio_gap_rows_stay_partial_and_use_bounded_metadata_language() {
    for scenario in [
        "media-audio-cue-storyboard",
        "audio-camera-and-export-sharecase",
    ] {
        let row = matrix_row(scenario);
        let row_text = serde_yaml::to_string(&row).expect("row serializes");

        assert_eq!(
            string_at(&row, &["looking_glass", "status"]),
            "partial",
            "{scenario} must stay partial until native audio/full audio authoring is proven"
        );
        assert!(
            row_text.contains("bounded")
                && row_text.contains("metadata")
                && row_text.contains("playback bridge"),
            "{scenario} must describe LookingGlass audio evidence as bounded metadata/playback-bridge support:\n{row_text}"
        );
        assert!(
            row_text.contains("does not claim native audio playback")
                && row_text.contains("does not claim native Web Share success"),
            "{scenario} must explicitly avoid native audio and native Web Share overclaims:\n{row_text}"
        );
        assert!(
            !row_text.contains("finished artifact package")
                && !row_text.contains("real/native audio playback")
                && !row_text.contains("full audio authoring"),
            "{scenario} row still contains overbroad audio/export wording:\n{row_text}"
        );
    }
}

#[test]
fn coverage_inventory_matches_bounded_gallery_media_boundaries() {
    let inventory = read_text("docs/eatme/alice-howto-coverage.md");
    let expectations = [
        (
            "media-audio-cue-storyboard",
            "Partial: bounded audio cue metadata and simulated playback bridge evidence only",
        ),
        (
            "audio-camera-and-export-sharecase",
            "Partial: camera/export/browser-download path proven; audio remains bounded metadata/playback bridge evidence",
        ),
        (
            "model-texture-import-checkpoint",
            "Covered: LookingGlass main contains imported model, texture, safe resource, export, and reopen persistence contract tests",
        ),
    ];

    for (scenario, expected_boundary) in expectations {
        let row = inventory
            .lines()
            .find(|line| line.contains(&format!("`{scenario}`")))
            .unwrap_or_else(|| panic!("coverage inventory missing {scenario}"));
        assert!(
            row.contains(expected_boundary),
            "{scenario} coverage inventory must carry the current LookingGlass boundary; row was:\n{row}"
        );
    }
    assert!(
        read_text("docs/tutorials/gallery-media-import-parity-walkthrough.md")
            .contains("covered model/texture import closure path"),
        "gallery/media walkthrough must describe model/texture import as covered once LookingGlass main evidence exists"
    );
    assert!(
        !inventory.contains("finished artifact package"),
        "coverage inventory must not overclaim finished artifact package support"
    );
}

#[test]
fn precise_lookingglass_closure_refs_are_run_by_the_row_command() {
    for row in matrix_rows() {
        let scenario = string_at(&row, &["scenario"]);
        let refs = lookingglass_test_refs(&row);
        if refs.is_empty() {
            continue;
        }

        let command = string_at(&row, &["looking_glass", "command"]);
        assert!(
            command.contains(r#"cd "${LOOKINGGLASS_HOME:?}""#),
            "{scenario} cites precise LookingGlass evidence, so looking_glass.command must cd through the portable LOOKINGGLASS_HOME guard; command was:\n{command}"
        );
        assert!(
            command.contains("npm test --"),
            "{scenario} cites precise LookingGlass evidence, so looking_glass.command must run the cited npm tests; command was:\n{command}"
        );
        for expected_ref in refs {
            assert!(
                command.contains(&expected_ref),
                "{scenario} closure cites LookingGlass:{expected_ref}, but looking_glass.command does not run it; command was:\n{command}"
            );
        }
    }
}

#[test]
fn unrelated_rows_do_not_carry_gallery_media_import_closure_commands() {
    for scenario in [
        "alice-objects-first-full-path",
        "first-lessons-real-ui-actions",
    ] {
        let row = matrix_row(scenario);
        let command = string_at(&row, &["looking_glass", "command"]);
        for misplaced_ref in [
            "test/project-audio-bounded-evidence.contract.test.ts",
            "test/project-export-share-fallback.contract.test.ts",
            "${LOOKINGGLASS_REPO",
        ] {
            assert!(
                !command.contains(misplaced_ref),
                "{scenario} looking_glass.command must not carry media/export LookingGlass closure commands; command was:\n{command}"
            );
        }
    }
}

#[test]
fn target_scenarios_include_precise_lookingglass_evidence_references() {
    let expectations = [
        (
            "model-texture-import-checkpoint",
            [
                "LookingGlass:test/model-texture-import-checkpoint-closure.contract.test.ts",
                "LookingGlass:test/imported-project-assets-security.contract.test.ts",
            ],
        ),
        (
            "media-audio-cue-storyboard",
            [
                "LookingGlass:test/project-audio-bounded-evidence.contract.test.ts",
                "bounded metadata/playback-bridge evidence",
            ],
        ),
        (
            "audio-camera-and-export-sharecase",
            [
                "LookingGlass:test/project-export-share-fallback.contract.test.ts",
                "does not claim native Web Share success",
            ],
        ),
    ];

    for (scenario, expected_refs) in expectations {
        let source = read_text(&format!("assets/scenarios/eatme/{scenario}.yaml"));
        for expected_ref in expected_refs {
            assert!(
                source.contains(expected_ref),
                "{scenario} source scenario must cite {expected_ref}"
            );
        }
    }

    for (scenario, expected_refs) in [
        (
            "media-audio-cue-storyboard",
            [
                "LookingGlass:test/project-audio-bounded-evidence.contract.test.ts",
                "bounded metadata/playback-bridge evidence",
            ],
        ),
        (
            "audio-camera-and-export-sharecase",
            [
                "LookingGlass:test/project-export-share-fallback.contract.test.ts",
                "does not claim native Web Share success",
            ],
        ),
    ] {
        let generated = read_text(&format!("assets/scenarios/gadugi/{scenario}.yaml"));
        for expected_ref in expected_refs {
            assert!(
                generated.contains(expected_ref),
                "{scenario} generated scenario mirror must cite {expected_ref}"
            );
        }
    }
}
