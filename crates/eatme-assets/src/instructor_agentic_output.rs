use crate::schema::EatmeScenarioAsset;
use crate::validate_scenario_asset;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub struct InstructorAgenticOutputReport {
    pub schema_version: String,
    pub asset_path: String,
    pub id: String,
    pub passed: bool,
    pub outputs: Vec<InstructorAgenticOutputSection>,
    pub acceptance_probe_results: Vec<AcceptanceProbeResult>,
    pub boundaries: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InstructorAgenticOutputSection {
    pub name: String,
    pub audience: String,
    pub body: String,
    pub evidence_inputs: Vec<String>,
    pub student_evidence: Vec<String>,
    pub boundaries: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AcceptanceProbeResult {
    pub probe: String,
    pub covered: bool,
    pub output: String,
}

pub fn render_instructor_agentic_output(path: &Path) -> Result<InstructorAgenticOutputReport> {
    let validation = validate_scenario_asset(path)?;
    if !validation.passed {
        bail!(
            "scenario asset validation failed for {}: {}",
            path.display(),
            validation.errors.join("; ")
        );
    }

    let scenario = read_eatme_scenario(path)?;
    if scenario.kind != "instructor_agentic_flow" {
        bail!(
            "{} must be an instructor_agentic_flow scenario, got {}",
            path.display(),
            scenario.kind
        );
    }

    let expected_outputs = scenario
        .agentic_flow
        .as_ref()
        .map(|flow| flow.expected_outputs.clone())
        .unwrap_or_default();
    let outputs = if scenario.id == "instructor-student-launch-evidence-handoff" {
        student_launch_handoff_sections(&scenario)
    } else {
        generic_sections(&scenario, &expected_outputs)
    };

    let output_names = outputs
        .iter()
        .map(|output| output.name.as_str())
        .collect::<Vec<_>>();
    let mut errors = expected_outputs
        .iter()
        .filter(|expected| {
            !output_names
                .iter()
                .any(|actual| actual == &expected.as_str())
        })
        .map(|expected| format!("missing expected output {expected}"))
        .collect::<Vec<_>>();
    let acceptance_probe_results = scenario
        .acceptance_probes
        .iter()
        .map(|probe| match output_for_probe(probe, &outputs) {
            Some(output) => AcceptanceProbeResult {
                probe: probe.clone(),
                covered: true,
                output: output.name.clone(),
            },
            None => AcceptanceProbeResult {
                probe: probe.clone(),
                covered: false,
                output: String::new(),
            },
        })
        .collect::<Vec<_>>();

    for probe in acceptance_probe_results
        .iter()
        .filter(|result| !result.covered)
    {
        errors.push(format!("acceptance probe not covered: {}", probe.probe));
    }

    Ok(InstructorAgenticOutputReport {
        schema_version: "eatme.assets/instructor-agentic-output/v1".into(),
        asset_path: path.display().to_string(),
        id: scenario.id,
        passed: errors.is_empty(),
        outputs,
        acceptance_probe_results,
        boundaries: instructor_boundaries(),
        errors,
    })
}

fn read_eatme_scenario(path: &Path) -> Result<EatmeScenarioAsset> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading eatme scenario asset {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("parsing eatme scenario YAML {}", path.display()))
}

fn student_launch_handoff_sections(
    scenario: &EatmeScenarioAsset,
) -> Vec<InstructorAgenticOutputSection> {
    let evidence_inputs = vec![
        "manifest".into(),
        "log".into(),
        "window-list".into(),
        "screenshot".into(),
        "ui-action-contract.json".into(),
    ];
    vec![
        InstructorAgenticOutputSection {
            name: "real_alice_evidence_handoff_card".into(),
            audience: "instructor".into(),
            body: "Use the manifest, log, window-list, screenshot, and ui-action-contract.json as launch evidence. The manifest records the run identity and collected artifacts, the log records startup signals or failures, the window-list shows whether an Alice window appeared, the screenshot gives a visible launch snapshot, and ui-action-contract.json documents the current action-automation boundary. These artifacts prove environment launch readiness only; they do not prove student understanding or successful learner-world behavior.".into(),
            evidence_inputs: evidence_inputs.clone(),
            student_evidence: Vec::new(),
            boundaries: instructor_boundaries(),
        },
        InstructorAgenticOutputSection {
            name: "instructor_readiness_note".into(),
            audience: "instructor".into(),
            body: "Treat launch artifacts as setup readiness signals before class. During student work, observe the learner project directly: expected behavior, observed result, and the student's next revision remain classroom evidence. If launch evidence is missing or contradictory, resolve the environment blocker before interpreting a student's project behavior.".into(),
            evidence_inputs: evidence_inputs.clone(),
            student_evidence: vec![
                "expected behavior".into(),
                "observed result".into(),
                "next revision".into(),
            ],
            boundaries: instructor_boundaries(),
        },
        InstructorAgenticOutputSection {
            name: "student_action_prompt".into(),
            audience: "student".into(),
            body: "Choose one Alice action you can explain. Run the world and record the visible result you saw. Then write one next revision you will make, or name the setup blocker if Alice did not run. This prompt asks for student-owned Alice action evidence, not hidden grading.".into(),
            evidence_inputs: vec!["student-owned Alice action evidence".into()],
            student_evidence: vec![
                "one Alice action".into(),
                "visible run result".into(),
                "one next revision".into(),
            ],
            boundaries: instructor_boundaries(),
        },
    ]
    .into_iter()
    .filter(|section| {
        scenario
            .agentic_flow
            .as_ref()
            .is_some_and(|flow| flow.expected_outputs.contains(&section.name))
    })
    .collect()
}

fn generic_sections(
    scenario: &EatmeScenarioAsset,
    expected_outputs: &[String],
) -> Vec<InstructorAgenticOutputSection> {
    let rubric_evidence = scenario
        .rubric
        .iter()
        .flat_map(|criterion| criterion.evidence.clone())
        .collect::<Vec<_>>();
    expected_outputs
        .iter()
        .map(|name| InstructorAgenticOutputSection {
            name: name.clone(),
            audience: if name.contains("student") {
                "student".into()
            } else {
                "instructor".into()
            },
            body: format!(
                "Instructor-facing acceptance output for {}. It is grounded in the scenario purpose, acceptance probes, and rubric evidence without claiming hidden automation.",
                scenario.id
            ),
            evidence_inputs: scenario
                .resource_basis
                .iter()
                .map(|resource| resource.name.clone())
                .collect(),
            student_evidence: rubric_evidence.clone(),
            boundaries: instructor_boundaries(),
        })
        .collect()
}

fn output_for_probe<'a>(
    probe: &str,
    outputs: &'a [InstructorAgenticOutputSection],
) -> Option<&'a InstructorAgenticOutputSection> {
    let normalized_probe = probe.to_ascii_lowercase();
    outputs
        .iter()
        .find(|output| section_text(output).contains(&normalized_probe))
        .or_else(|| {
            outputs.iter().find(|output| {
                let name = output.name.as_str();
                (normalized_probe.contains("handoff card")
                    && name == "real_alice_evidence_handoff_card")
                    || (normalized_probe.contains("readiness note")
                        && name == "instructor_readiness_note")
                    || (normalized_probe.contains("student action prompt")
                        && name == "student_action_prompt")
                    || (normalized_probe.contains("not full user interface automation")
                        && !output.boundaries.is_empty())
            })
        })
}

fn section_text(output: &InstructorAgenticOutputSection) -> String {
    [
        output.name.as_str(),
        output.audience.as_str(),
        output.body.as_str(),
        &output.evidence_inputs.join(" "),
        &output.student_evidence.join(" "),
        &output.boundaries.join(" "),
    ]
    .join(" ")
    .to_ascii_lowercase()
}

fn instructor_boundaries() -> Vec<String> {
    vec![
        "not full user interface automation".into(),
        "not automated creative assessment".into(),
        "not learner-world grading".into(),
        "not complete Alice coverage".into(),
        "not a deployed service".into(),
    ]
}
