use crate::schema::EatmeScenarioAsset;
use crate::validation::validate_scenario_asset;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct InstructorAgenticAcceptanceReport {
    pub schema_version: String,
    pub asset_path: String,
    pub scenario_id: String,
    pub prompt_source: String,
    pub passed: bool,
    pub output_evidence: BTreeMap<String, InstructorAgenticOutput>,
    pub probe_results: Vec<InstructorAgenticProbeResult>,
    pub boundaries: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InstructorAgenticOutput {
    pub title: String,
    pub evidence: String,
}

#[derive(Debug, Serialize)]
pub struct InstructorAgenticProbeResult {
    pub probe: String,
    pub passed: bool,
    pub evidence: String,
}

pub fn generate_instructor_agentic_acceptance_output(
    path: &Path,
) -> Result<InstructorAgenticAcceptanceReport> {
    let validation = validate_scenario_asset(path)?;
    if !validation.passed {
        bail!(
            "{} is not a valid scenario asset: {}",
            path.display(),
            validation.errors.join("; ")
        );
    }

    let scenario = read_eatme_scenario(path)?;
    if scenario.kind != "instructor_agentic_flow" {
        bail!(
            "{} is {}, not instructor_agentic_flow",
            scenario.id,
            scenario.kind
        );
    }

    let flow = scenario
        .agentic_flow
        .as_ref()
        .context("instructor_agentic_flow must define agentic_flow")?;
    if scenario.id != "setup-preflight-ready-to-create" {
        bail!(
            "{} does not have a registered runnable instructor agentic acceptance output writer",
            scenario.id
        );
    }

    let output_evidence = setup_preflight_output_evidence();
    let boundaries = setup_preflight_boundaries();
    let mut errors = Vec::new();
    for expected in &flow.expected_outputs {
        if !output_evidence.contains_key(expected) {
            errors.push(format!("missing expected output evidence {expected}"));
        }
    }

    let probe_results = scenario
        .acceptance_probes
        .iter()
        .map(|probe| {
            let evidence = setup_preflight_probe_evidence(probe);
            let passed = !evidence.is_empty();
            if !passed {
                errors.push(format!("unaddressed acceptance probe: {probe}"));
            }
            InstructorAgenticProbeResult {
                probe: probe.clone(),
                passed,
                evidence,
            }
        })
        .collect();

    Ok(InstructorAgenticAcceptanceReport {
        schema_version: "eatme.assets/instructor-agentic-acceptance-output/v1".into(),
        asset_path: path.display().to_string(),
        scenario_id: scenario.id,
        prompt_source: flow.prompt_source.clone(),
        passed: errors.is_empty(),
        output_evidence,
        probe_results,
        boundaries,
        errors,
    })
}

fn read_eatme_scenario(path: &Path) -> Result<EatmeScenarioAsset> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading eatme scenario asset {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("parsing eatme scenario YAML {}", path.display()))
}

fn setup_preflight_output_evidence() -> BTreeMap<String, InstructorAgenticOutput> {
    BTreeMap::from([
        (
            "setup_readiness_checklist".into(),
            InstructorAgenticOutput {
                title: "Setup readiness checklist".into(),
                evidence: [
                    "Confirm each device uses a supported Alice desktop package from the Alice download resources: Windows 64-bit, macOS, or a Linux .deb path; treat Chromebook use as a distinct case that requires Linux app access and school permission.",
                    "Confirm the Java dependency named by current Alice resources: Alice 3 source/build documentation cites Java 21, while packaged downloads may bundle the needed runtime; do not make students install Java separately unless the current download instructions require it.",
                    "Confirm graphics readiness before the first lesson: Alice depends on working OpenGL-capable graphics drivers for the 3D scene view, so red-screen or launch failures are environment blockers to route to driver/device support.",
                    "Confirm the student can install or launch under the account they will use in class; school-managed devices may need administrator approval before Alice, Java, Linux support, or graphics drivers can be changed.",
                ]
                .join("\n"),
            },
        ),
        (
            "student_self_check_card".into(),
            InstructorAgenticOutput {
                title: "Student self-check card".into(),
                evidence: [
                    "Try to launch Alice or start the installation using the same device and account you will use during class.",
                    "Write one sentence that starts, \"Right now in Alice I can...\" and names the action you can perform, such as opening Alice, reaching the welcome screen, or starting a new world.",
                    "If Alice does not launch, write one sentence that starts, \"My device blocker is...\" and names the specific environment problem, such as no install permission, missing Linux access on a Chromebook, Java/runtime trouble, or an OpenGL/graphics-driver error. This is an environment problem, not a student mistake.",
                ]
                .join("\n"),
            },
        ),
        (
            "fallback_path_guide".into(),
            InstructorAgenticOutput {
                title: "Fallback path guide".into(),
                evidence: [
                    "Pairing path: a student without a working device works beside a partner with a ready device, makes at least one creative decision aloud, and records what they would try first when their own device is ready.",
                    "Instructor-led demo path: the instructor drives Alice on a ready machine while blocked students choose objects, actions, or story beats and record expected visible outcomes.",
                    "Printed design-planning path: blocked students complete a storyboard or scene plan with object names, first action, and expected behavior so they can participate in the creative thinking portion without a working device.",
                    "Handoff note: collect every reported blocker, assign an owner or support path, and address each blocker before the first creation task begins; checklist items are based on current Alice download/resource requirements and must be rechecked when a new Alice version changes system requirements.",
                ]
                .join("\n"),
            },
        ),
    ])
}

fn setup_preflight_boundaries() -> Vec<String> {
    vec![
        "not full user interface automation".into(),
        "not automated creative assessment".into(),
        "not learner-world grading".into(),
        "not complete Alice coverage".into(),
        "not real Alice startup proof".into(),
    ]
}

fn setup_preflight_probe_evidence(probe: &str) -> String {
    let lower = probe.to_ascii_lowercase();
    if lower.contains("os-level")
        && lower.contains("java")
        && lower.contains("opengl")
        && lower.contains("chromebook")
    {
        return "setup_readiness_checklist names Windows 64-bit, macOS, Linux .deb, Java 21, OpenGL-capable graphics drivers, install permission, and Chromebook Linux access.".into();
    }
    if lower.contains("one thing they can do now") {
        return "student_self_check_card prompts: \"Right now in Alice I can...\"".into();
    }
    if lower.contains("one specific blocker") && lower.contains("environment problem") {
        return "student_self_check_card prompts: \"My device blocker is...\" and states this is an environment problem, not a student mistake.".into();
    }
    if lower.contains("no-install") && lower.contains("pairing") {
        return "fallback_path_guide provides pairing, instructor-led demo, and printed design-planning no-install paths.".into();
    }
    if lower.contains("address every reported blocker") {
        return "fallback_path_guide handoff note tells the instructor to collect every blocker and address each one before the first creation task.".into();
    }
    if lower.contains("alice download page") || lower.contains("new alice versions") {
        return "fallback_path_guide handoff note says requirements come from current Alice download/resource requirements and must be rechecked for new Alice versions.".into();
    }
    if lower.contains("not full user interface automation")
        && lower.contains("not automated creative assessment")
        && lower.contains("not learner-world grading")
        && lower.contains("not complete alice coverage")
    {
        return setup_preflight_boundaries().join("; ");
    }
    String::new()
}
