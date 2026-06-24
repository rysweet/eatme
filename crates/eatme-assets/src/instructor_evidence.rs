use crate::schema::{EatmeScenarioAsset, EatmeScenarioResource};
use crate::validate_scenario_asset;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SCHEMA_VERSION: &str = "eatme.assets/instructor-agentic-evidence/v1";

#[derive(Debug, Serialize)]
pub struct InstructorAgenticEvidenceReport {
    pub schema_version: String,
    pub scenario_id: String,
    pub title: String,
    pub status: String,
    pub expected_outputs: Vec<String>,
    pub outputs: BTreeMap<String, InstructorAgenticEvidenceOutput>,
    pub source_basis: Vec<String>,
    pub does_not_claim: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InstructorAgenticEvidenceOutput {
    pub artifact_uri: String,
    pub summary: String,
    pub evidence: Vec<String>,
}

pub fn render_instructor_agentic_evidence(path: &Path) -> Result<InstructorAgenticEvidenceReport> {
    let validation =
        validate_scenario_asset(path).with_context(|| format!("validating {}", path.display()))?;
    if !validation.passed {
        bail!(
            "{} failed scenario validation: {}",
            path.display(),
            validation.errors.join("; ")
        );
    }

    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let scenario: EatmeScenarioAsset =
        serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    if scenario.kind != "instructor_agentic_flow" {
        bail!(
            "{} is kind {}; expected instructor_agentic_flow",
            scenario.id,
            scenario.kind
        );
    }

    let flow = scenario
        .agentic_flow
        .as_ref()
        .context("instructor_agentic_flow scenario must define agentic_flow")?;
    if flow.expected_outputs.is_empty() {
        bail!("{} must define agentic_flow.expected_outputs", scenario.id);
    }

    let source_basis = source_basis(&scenario.resource_basis);
    let mut outputs = BTreeMap::new();
    for output in &flow.expected_outputs {
        let artifact_uri = scenario
            .artifacts
            .get(output)
            .cloned()
            .with_context(|| format!("{} must define artifacts.{output}", scenario.id))?;
        outputs.insert(
            output.clone(),
            evidence_output(output, &artifact_uri, &scenario, &source_basis),
        );
    }

    Ok(InstructorAgenticEvidenceReport {
        schema_version: SCHEMA_VERSION.into(),
        scenario_id: scenario.id,
        title: scenario.title,
        status: "covered".into(),
        expected_outputs: flow.expected_outputs.clone(),
        outputs,
        source_basis,
        does_not_claim: vec![
            "desktop Alice launch success".into(),
            "native OpenGL driver diagnosis".into(),
            "full Alice UI automation".into(),
            "learner-world grading".into(),
        ],
    })
}

fn source_basis(resources: &[EatmeScenarioResource]) -> Vec<String> {
    resources
        .iter()
        .map(|resource| format!("{}: {}", resource.name.trim(), resource.use_note.trim()))
        .collect()
}

fn evidence_output(
    output: &str,
    artifact_uri: &str,
    scenario: &EatmeScenarioAsset,
    source_basis: &[String],
) -> InstructorAgenticEvidenceOutput {
    let basis = source_basis.join("; ");
    let probes = scenario.acceptance_probes.join("; ");
    let rubric = scenario
        .rubric
        .iter()
        .map(|criterion| format!("{}: {}", criterion.criterion, criterion.evidence.join("; ")))
        .collect::<Vec<_>>()
        .join("; ");

    let (summary, evidence) = if output.contains("checklist") {
        (
            format!("Instructor setup checklist for {}", scenario.title),
            vec![
                format!("Resource basis applied: {basis}"),
                "Verify classroom devices meet Alice desktop/laptop, graphics, Java/OpenGL, Linux, and Chromebook constraints before students depend on them.".into(),
                "Record observable readiness evidence: Alice can launch, a starter world can be created, and resource links are available to the class.".into(),
                format!("Acceptance probes covered: {probes}"),
            ],
        )
    } else if output.contains("fallback") {
        (
            format!("Fallback plan for {}", scenario.title),
            vec![
                "If Alice cannot launch because of install, graphics, driver, or device limits, treat it as an environment blocker rather than a learner mistake.".into(),
                "Preserve the same lesson concept with offline planning, paired observation, teacher-resource alternatives, or a known-good classroom machine.".into(),
                "Name blocker owner, escalation path, and retest signal before returning students to desktop work.".into(),
                format!("Rubric evidence covered: {rubric}"),
            ],
        )
    } else if output.contains("student") || output.contains("note") {
        (
            format!("Student-facing setup note for {}", scenario.title),
            vec![
                "If Alice setup or launch fails, report the device or install symptom; do not treat the failure as a bug in your world or code.".into(),
                "Work from the fallback activity or a shared working device while the instructor tracks the setup blocker.".into(),
                "Peer support is bounded to safe observation and reporting; students are not responsible for driver or installer repair.".into(),
            ],
        )
    } else {
        (
            format!("Instructor acceptance output for {}", scenario.title),
            vec![
                format!("Purpose: {}", scenario.purpose),
                format!("Acceptance probes covered: {probes}"),
                format!("Rubric evidence covered: {rubric}"),
            ],
        )
    };

    InstructorAgenticEvidenceOutput {
        artifact_uri: artifact_uri.into(),
        summary,
        evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::render_instructor_agentic_evidence;
    use std::path::{Path, PathBuf};

    #[test]
    fn classroom_setup_evidence_renders_checklist_fallback_and_student_note() {
        let report = render_instructor_agentic_evidence(&scenario_path()).unwrap();

        assert_eq!(report.scenario_id, "instructor-classroom-setup-readiness");
        assert_eq!(report.status, "covered");
        for output in ["setup_checklist", "fallback_plan", "student_setup_note"] {
            assert!(
                report.outputs.contains_key(output),
                "missing output evidence for {output}: {:?}",
                report.outputs.keys().collect::<Vec<_>>()
            );
        }
        let joined = serde_json::to_string(&report).unwrap();
        assert!(joined.contains("graphics"));
        assert!(joined.contains("environment blocker"));
        assert!(joined.contains("not responsible for driver or installer repair"));
        assert!(joined.contains("desktop Alice launch success"));
    }

    fn scenario_path() -> PathBuf {
        repository_root().join("assets/scenarios/eatme/instructor-classroom-setup-readiness.yaml")
    }

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
}
