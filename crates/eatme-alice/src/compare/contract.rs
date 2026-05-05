use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ComparisonContract {
    pub schema_version: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub functionality_rules: Vec<String>,
    pub timing_rules: Vec<String>,
    pub non_claims: Vec<String>,
    pub next_capabilities: Vec<String>,
}

pub(super) fn comparison_contract() -> ComparisonContract {
    ComparisonContract {
        schema_version: "eatme.alice-comparison-contract/v1".into(),
        inputs: vec![
            "target registry defines baseline and modernized Alice targets".into(),
            "run id and scenario id identify the comparison case".into(),
            "execute mode requires resolved target homes for both roles".into(),
            "declared required paths must exist under each resolved Alice home before launch".into(),
        ],
        outputs: vec![
            "comparison manifest is written under runs/comparisons/<scenario-id>/<run-id>/".into(),
            "each target records status, duration, target metadata, and preparation details".into(),
            "executed targets attach a launch-smoke manifest artifact when launch smoke runs".into(),
            "scorecard records functionality, timing, durations, and faster target when measured".into(),
            "diff records target status, failure-category, and assertion differences".into(),
        ],
        functionality_rules: vec![
            "manifest-only mode records intent and reports functionality as not_measured".into(),
            "execute mode reports functionality as incomplete unless both targets pass launch smoke".into(),
            "matched means target statuses, failure categories, and normalized assertions match".into(),
            "different means target status, failure category, or a non-normalized assertion differs".into(),
            "passing display_responsive details normalize volatile X display identifiers".into(),
            "failed display_responsive assertions and pass/fail changes remain differences".into(),
        ],
        timing_rules: vec![
            "manifest-only mode reports timing as not_measured".into(),
            "timing is incomplete unless both targets pass launch smoke".into(),
            "target duration covers target preparation checks and launch-smoke execution".into(),
            "single runs are samples, not performance claims".into(),
            "speed claims require repeated same-machine samples for the same scenario and targets".into(),
        ],
        non_claims: vec![
            "does not automate full Alice lesson creation and consumption".into(),
            "does not perform creative assessment".into(),
            "does not grade student worlds".into(),
            "does not prove broad Alice compatibility beyond the selected scenario".into(),
            "does not prove modernization quality or coverage targets".into(),
        ],
        next_capabilities: vec![
            "instructor creates an assignment".into(),
            "student opens, changes, runs, saves, and shares an Alice project".into(),
            "comparison scenario covers object placement, procedures, events, playback, and save/load".into(),
            "telemetry identifies target setup, launch, action, save, and result collection spans".into(),
        ],
    }
}
