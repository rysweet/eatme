use crate::render_instructor_agentic_output;
use std::path::Path;

#[test]
fn student_launch_handoff_renders_runnable_output_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = render_instructor_agentic_output(
        &root.join("assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml"),
    )
    .unwrap();

    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(report.id, "instructor-student-launch-evidence-handoff");
    assert!(report.outputs.iter().any(|output| {
        output.name == "real_alice_evidence_handoff_card"
            && output.evidence_inputs.contains(&"manifest".to_string())
            && output
                .evidence_inputs
                .contains(&"ui-action-contract.json".to_string())
    }));
    assert!(report.outputs.iter().any(|output| {
        output.name == "instructor_readiness_note"
            && output.body.contains("environment")
            && output.body.contains("student")
    }));
    assert!(report.outputs.iter().any(|output| {
        output.name == "student_action_prompt"
            && output
                .student_evidence
                .contains(&"one Alice action".to_string())
            && output
                .student_evidence
                .contains(&"visible run result".to_string())
            && output
                .student_evidence
                .contains(&"one next revision".to_string())
    }));
    assert!(
        report
            .acceptance_probe_results
            .iter()
            .all(|probe| probe.covered)
    );
}
