use crate::pr199_recovery::state::{Pr199QaReport, ReadinessBlocker};
use std::path::{Path, PathBuf};

pub const REQUIRED_QA_COMMANDS: [&str; 5] = [
    "cargo test --workspace --all-features",
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QaCommandProof {
    pub command: String,
    pub worktree: String,
    pub exit_code: i32,
}

impl QaCommandProof {
    pub fn passed_in_worktree(command: impl Into<String>, worktree: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            worktree: worktree.into(),
            exit_code: 0,
        }
    }

    pub fn failed_in_worktree(
        command: impl Into<String>,
        worktree: impl Into<String>,
        exit_code: i32,
    ) -> Self {
        Self {
            command: command.into(),
            worktree: worktree.into(),
            exit_code,
        }
    }
}

pub fn evaluate_qa(proofs: &[QaCommandProof]) -> (Pr199QaReport, Vec<ReadinessBlocker>) {
    let current_worktree = current_worktree();
    let mut blockers = Vec::new();
    let observed_commands = proofs
        .iter()
        .map(|proof| proof.command.clone())
        .collect::<Vec<_>>();

    for proof in proofs {
        if !REQUIRED_QA_COMMANDS.contains(&proof.command.as_str()) {
            blockers.push(ReadinessBlocker::new(
                "invalid_qa_command",
                qa_field_for_nearest_required_command(&proof.command),
                format!(
                    "{} is not one of the exact PR #199 recovery QA commands.",
                    proof.command
                ),
            ));
        }
    }

    for required in REQUIRED_QA_COMMANDS {
        match proofs.iter().find(|proof| proof.command == required) {
            Some(proof) if !same_worktree(&proof.worktree, &current_worktree) => {
                blockers.push(ReadinessBlocker::new(
                    "stale_qa_proof",
                    format!("qa.{required}"),
                    format!("{required} was not rerun in the current worktree."),
                ));
            }
            Some(proof) if proof.exit_code != 0 => {
                blockers.push(ReadinessBlocker::new(
                    "failed_qa_command",
                    format!("qa.{required}"),
                    format!("{required} failed with exit code {}.", proof.exit_code),
                ));
            }
            Some(_) => {}
            None => {
                blockers.push(ReadinessBlocker::new(
                    "missing_qa_proof",
                    format!("qa.{required}"),
                    format!("{required} has no worktree-local passing proof."),
                ));
            }
        }
    }

    if !all_required_commands_passed_in_worktree(proofs, &current_worktree) {
        blockers.push(ReadinessBlocker::new(
            "incomplete_qa_proof",
            "qa",
            "PR #199 recovery readiness requires the exact five scoped QA commands.",
        ));
    }

    let report = Pr199QaReport {
        required_commands: REQUIRED_QA_COMMANDS,
        observed_commands,
        required_commands_passed: blockers.is_empty(),
    };

    (report, blockers)
}

fn all_required_commands_passed_in_worktree(
    proofs: &[QaCommandProof],
    current_worktree: &Path,
) -> bool {
    REQUIRED_QA_COMMANDS.iter().all(|required| {
        proofs.iter().any(|proof| {
            proof.command == *required
                && proof.exit_code == 0
                && same_worktree(&proof.worktree, current_worktree)
        })
    })
}

fn qa_field_for_nearest_required_command(observed: &str) -> String {
    if observed == "./scripts/quality-gates.sh" {
        return "qa.TMPDIR=/tmp ./scripts/quality-gates.sh".to_string();
    }
    format!("qa.{observed}")
}

fn same_worktree(observed: &str, current_worktree: &Path) -> bool {
    normalize_path(observed) == normalize_path(current_worktree)
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf())
}

fn current_worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("eatme-core must live under crates/eatme-core")
        .to_path_buf()
}
