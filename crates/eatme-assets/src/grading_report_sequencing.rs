//! Sequencing grading — covers doInOrder/doTogether procedure sequencing.

use eatme_core::ast::{SequenceBlock, SequenceKind};

pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

use crate::grading_report::{
    blocked_by_reason, build_preconditions, cascade_blocked, no_sequence_reason,
};

pub struct SequencingGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub sequence_blocks: Option<Vec<SequenceBlock>>,
}

pub fn grade_sequencing(input: SequencingGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("use-do-in-order", &["launch-smoke"]),
            cascade_blocked("use-do-together", &["launch-smoke"]),
            StepGrade {
                name: "combine-sequential-and-parallel-actions".into(),
                status: StepStatus::Blocked,
                reason: blocked_by_reason(
                    "combine-sequential-and-parallel-actions",
                    &["use-do-in-order", "use-do-together"],
                ),
                depends_on: vec!["use-do-in-order".into(), "use-do-together".into()],
            },
            cascade_blocked("save-project", &["combine-sequential-and-parallel-actions"]),
        ]
    } else {
        evaluate_sequencing_steps(&input.sequence_blocks)
    };

    let passed = !preconditions_blocked
        && interaction_steps
            .iter()
            .all(|step| step.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport::new(
        "eatme.assets/grading/v1",
        "procedure-sequencing-do-in-order-do-together",
        passed,
        steps,
    )
}

fn evaluate_sequencing_steps(sequence_blocks: &Option<Vec<SequenceBlock>>) -> Vec<StepGrade> {
    let Some(sequence_blocks) = sequence_blocks else {
        return vec![
            missing_sequence_step("use-do-in-order", &["launch-smoke"]),
            missing_sequence_step("use-do-together", &["launch-smoke"]),
            StepGrade {
                name: "combine-sequential-and-parallel-actions".into(),
                status: StepStatus::Blocked,
                reason: no_sequence_reason("combine-sequential-and-parallel-actions"),
                depends_on: vec!["use-do-in-order".into(), "use-do-together".into()],
            },
            missing_sequence_step("save-project", &["combine-sequential-and-parallel-actions"]),
        ];
    };

    let has_do_in_order = sequence_blocks
        .iter()
        .any(|block| matches!(block.kind, SequenceKind::DoInOrder));
    let has_do_together = sequence_blocks
        .iter()
        .any(|block| matches!(block.kind, SequenceKind::DoTogether));
    let used_both = has_do_in_order && has_do_together;

    vec![
        ready_or_blocked(
            "use-do-in-order",
            &["launch-smoke"],
            has_do_in_order,
            "doInOrder block found in student program",
            "No doInOrder block found in student program",
        ),
        ready_or_blocked(
            "use-do-together",
            &["launch-smoke"],
            has_do_together,
            "doTogether block found in student program",
            "No doTogether block found in student program",
        ),
        ready_or_blocked(
            "combine-sequential-and-parallel-actions",
            &["use-do-in-order", "use-do-together"],
            used_both,
            "Student used both doInOrder and doTogether",
            "Student must use both doInOrder and doTogether for full credit",
        ),
        ready_or_blocked(
            "save-project",
            &["combine-sequential-and-parallel-actions"],
            used_both,
            "Mixed sequencing constructs persist for grading",
            "Mixed sequencing constructs missing for save/reopen grading",
        ),
    ]
}

fn ready_or_blocked(
    name: &str,
    deps: &[&str],
    ready: bool,
    success_reason: &str,
    failure_reason: &str,
) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: if ready {
            StepStatus::Ready
        } else {
            StepStatus::Blocked
        },
        reason: if ready {
            format!(
                "{success_reason}. Keep this sequencing evidence saved for the next grading step."
            )
        } else {
            format!(
                "{failure_reason}. Update the student's sequence blocks, save the project, and rerun grading."
            )
        },
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}

fn missing_sequence_step(name: &str, deps: &[&str]) -> StepGrade {
    StepGrade {
        name: name.into(),
        status: StepStatus::Blocked,
        reason: no_sequence_reason(name),
        depends_on: deps.iter().map(|dep| (*dep).into()).collect(),
    }
}
