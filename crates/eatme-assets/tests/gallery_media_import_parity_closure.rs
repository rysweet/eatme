use std::fs;

fn read_text(path: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let full_path = root.join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn coverage_inventory_matches_bounded_gallery_media_boundaries() {
    let inventory = read_text("docs/eatme/alice-howto-coverage.md");
    let expectations = [
        (
            "media-audio-cue-storyboard",
            "Partial: bounded audio cue metadata and simulated playback bridge evidence only; covered metadata/playback bridge evidence; missing native audio playback and full authoring evidence",
        ),
        (
            "audio-camera-and-export-sharecase",
            "Partial: camera/export/browser-download path proven; audio remains bounded metadata/playback bridge evidence; covered camera/export/download path; missing native audio playback and native Web Share evidence",
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
        !inventory.contains("finished artifact package"),
        "coverage inventory must not overclaim finished artifact package support"
    );
}
