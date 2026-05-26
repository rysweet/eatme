use super::RecoveryError;
use super::evidence::StructuredBlocker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QaCommand {
    CargoWorkspaceAllFeatures,
    AssetsValidateJson,
    GenerateGadugiCheckJson,
    MkdocsBuildStrict,
    QualityGatesWithTmpdir,
}

impl QaCommand {
    fn label(self) -> &'static str {
        match self {
            Self::CargoWorkspaceAllFeatures => "cargo test --workspace --all-features",
            Self::AssetsValidateJson => "cargo run -q -p eatme-cli -- assets validate --json",
            Self::GenerateGadugiCheckJson => {
                "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json"
            }
            Self::MkdocsBuildStrict => "mkdocs build --strict",
            Self::QualityGatesWithTmpdir => "TMPDIR=/tmp ./scripts/quality-gates.sh",
        }
    }
}

const REQUIRED_QA_COMMANDS: [QaCommand; 5] = [
    QaCommand::CargoWorkspaceAllFeatures,
    QaCommand::AssetsValidateJson,
    QaCommand::GenerateGadugiCheckJson,
    QaCommand::MkdocsBuildStrict,
    QaCommand::QualityGatesWithTmpdir,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QaOutcome {
    pub command: QaCommand,
    pub exit_code: i32,
    pub summary: String,
}

impl QaOutcome {
    pub fn passed(command: QaCommand) -> Self {
        Self {
            command,
            exit_code: 0,
            summary: "passed".into(),
        }
    }

    pub fn failed(command: QaCommand, exit_code: i32, summary: impl Into<String>) -> Self {
        Self {
            command,
            exit_code,
            summary: summary.into(),
        }
    }

    fn passed_status(&self) -> bool {
        self.exit_code == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QaReport {
    pub commands: Vec<QaOutcome>,
    pub passed: bool,
    pub blockers: Vec<StructuredBlocker>,
}

impl QaReport {
    pub fn includes(&self, command: QaCommand) -> bool {
        self.commands
            .iter()
            .any(|outcome| outcome.command == command)
    }
}

pub struct ScopedQaRunner;

impl ScopedQaRunner {
    pub fn summarize(outcomes: Vec<QaOutcome>) -> Result<QaReport, RecoveryError> {
        let mut seen_required = [false; REQUIRED_QA_COMMANDS.len()];
        let mut blockers = Vec::new();

        for outcome in &outcomes {
            seen_required[required_index(outcome.command)] = true;

            if !outcome.passed_status() {
                blockers.push(StructuredBlocker {
                    code: "scoped_qa_failed".into(),
                    status: "blocked".into(),
                    subject: outcome.command.label().into(),
                    reason: format!(
                        "QA command exited {}: {}",
                        outcome.exit_code, outcome.summary
                    ),
                    resolution: "Rerun and resolve the failing PR #199 recovery QA command.".into(),
                });
            }
        }

        for (index, command) in REQUIRED_QA_COMMANDS.iter().enumerate() {
            if !seen_required[index] {
                return Err(RecoveryError::new(
                    "scoped_qa_missing_command",
                    format!("missing required QA command: {}", command.label()),
                ));
            }
        }

        let passed = blockers.is_empty();

        Ok(QaReport {
            commands: outcomes,
            passed,
            blockers,
        })
    }
}

fn required_index(command: QaCommand) -> usize {
    match command {
        QaCommand::CargoWorkspaceAllFeatures => 0,
        QaCommand::AssetsValidateJson => 1,
        QaCommand::GenerateGadugiCheckJson => 2,
        QaCommand::MkdocsBuildStrict => 3,
        QaCommand::QualityGatesWithTmpdir => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::{QaCommand, QaOutcome, ScopedQaRunner};

    #[test]
    fn summarize_rejects_missing_required_commands() {
        let error = ScopedQaRunner::summarize(vec![
            QaOutcome::passed(QaCommand::CargoWorkspaceAllFeatures),
            QaOutcome::passed(QaCommand::AssetsValidateJson),
            QaOutcome::passed(QaCommand::GenerateGadugiCheckJson),
            QaOutcome::passed(QaCommand::MkdocsBuildStrict),
        ])
        .unwrap_err();

        assert_eq!(error.code(), "scoped_qa_missing_command");
        assert!(
            error
                .to_string()
                .contains("TMPDIR=/tmp ./scripts/quality-gates.sh")
        );
    }

    #[test]
    fn summarize_marks_all_required_passes_as_successful_and_includable() {
        let report = ScopedQaRunner::summarize(vec![
            QaOutcome::passed(QaCommand::CargoWorkspaceAllFeatures),
            QaOutcome::passed(QaCommand::AssetsValidateJson),
            QaOutcome::passed(QaCommand::GenerateGadugiCheckJson),
            QaOutcome::passed(QaCommand::MkdocsBuildStrict),
            QaOutcome::passed(QaCommand::QualityGatesWithTmpdir),
        ])
        .unwrap();

        assert!(report.passed);
        assert!(report.blockers.is_empty());
        assert!(report.includes(QaCommand::MkdocsBuildStrict));
        assert!(report.includes(QaCommand::CargoWorkspaceAllFeatures));
        assert_eq!(report.commands[0].summary, "passed");
    }

    #[test]
    fn summarize_collects_multiple_failures_as_blockers() {
        let report = ScopedQaRunner::summarize(vec![
            QaOutcome::failed(
                QaCommand::CargoWorkspaceAllFeatures,
                2,
                "workspace tests failed",
            ),
            QaOutcome::passed(QaCommand::AssetsValidateJson),
            QaOutcome::failed(
                QaCommand::GenerateGadugiCheckJson,
                3,
                "gadugi artifact drift",
            ),
            QaOutcome::passed(QaCommand::MkdocsBuildStrict),
            QaOutcome::passed(QaCommand::QualityGatesWithTmpdir),
        ])
        .unwrap();

        assert!(!report.passed);
        assert_eq!(report.blockers.len(), 2);
        assert!(report.blockers.iter().any(|blocker| {
            blocker
                .subject
                .contains("cargo test --workspace --all-features")
        }));
        assert!(
            report
                .blockers
                .iter()
                .any(|blocker| blocker.reason.contains("gadugi artifact drift"))
        );
    }
}
