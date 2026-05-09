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
        let required = [
            QaCommand::CargoWorkspaceAllFeatures,
            QaCommand::AssetsValidateJson,
            QaCommand::GenerateGadugiCheckJson,
            QaCommand::MkdocsBuildStrict,
            QaCommand::QualityGatesWithTmpdir,
        ];
        for command in required {
            if !outcomes.iter().any(|outcome| outcome.command == command) {
                return Err(RecoveryError::new(
                    "scoped_qa_missing_command",
                    format!("missing required QA command: {}", command.label()),
                ));
            }
        }

        let blockers = outcomes
            .iter()
            .filter(|outcome| !outcome.passed_status())
            .map(|outcome| StructuredBlocker {
                code: "scoped_qa_failed".into(),
                status: "blocked".into(),
                subject: outcome.command.label().into(),
                reason: format!(
                    "QA command exited {}: {}",
                    outcome.exit_code, outcome.summary
                ),
                resolution: "Rerun and resolve the failing PR #199 recovery QA command.".into(),
            })
            .collect::<Vec<_>>();
        let passed = blockers.is_empty();

        Ok(QaReport {
            commands: outcomes,
            passed,
            blockers,
        })
    }
}
